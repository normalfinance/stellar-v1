use core::cmp::max;

use crate::errors::LongShortPairError;
use crate::events::{Events, LongShortPairEvents};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{contracttype, log, panic_with_error, Address, Env, IntoVal, Symbol, Vec};
use utils::constant::{
    FEE_MULTIPLIER, FUNDING_RATE_BUFFER, FUNDING_RATE_BUFFER_I128, ONE_HOUR_I128, ONE_HOUR_I64,
    PERCENTAGE_PRECISION_U64, PERCENT_MULTIPLIER, PERCENT_MULTIPLIER_I128, PERCENT_MULTIPLIER_I64,
    PERCENT_MULTIPLIER_U64, PRICE_PRECISION, PRICE_PRECISION_I128, PRICE_PRECISION_I64,
    TWENTY_FOUR_HOUR,
};
use utils::math::safe_math::{PrecisionMath, SafeMath};

use crate::storage::{
    get_collateral_per_pair, get_collateral_percent_long, get_cumulative_funding_index_long,
    get_cumulative_funding_index_short, get_funding_clamp, get_funding_period,
    get_is_killed_update_funding, get_last_24h_avg_funding_rate, get_last_funding_rate_ts,
    get_last_update_ts, get_max_ratio_percent_divergence, get_pool_long, get_pool_plane,
    get_pool_short, put_user_funding_checkpoint, set_cumulative_funding_index_long,
    set_cumulative_funding_index_short, set_last_24h_avg_funding_rate, set_last_funding_rate,
    set_last_funding_rate_ts,
};

/**
 *
 * Funding Types
 *
 */

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub struct FundingCheckpoint {
    pub account: Address,
    pub long_index: i64,
    pub short_index: i64,
    pub long_balance: u128,
    pub short_balance: u128,
}

impl FundingCheckpoint {
    pub fn new(account: Address) -> Self {
        FundingCheckpoint {
            account,
            long_index: 0,
            short_index: 0,
            long_balance: 0,
            short_balance: 0,
        }
    }

    pub fn save(&mut self, e: &Env) {
        put_user_funding_checkpoint(e, &self.account, &self);
    }

    pub fn mint(&mut self, e: &Env, tokens_to_mint: u128) {
        self.long_index = get_cumulative_funding_index_long(e);
        self.short_index = get_cumulative_funding_index_short(e);
        self.long_balance = self.long_balance.safe_add(e, tokens_to_mint);
        self.short_balance = self.short_balance.safe_add(e, tokens_to_mint);
        self.save(&e);
    }

    pub fn redeem(&mut self, e: &Env, tokens_to_redeem: u128) {
        self.long_balance = self.long_balance.safe_sub(e, tokens_to_redeem);
        self.short_balance = self.short_balance.safe_sub(e, tokens_to_redeem);
        self.long_index = get_cumulative_funding_index_long(e);
        self.short_index = get_cumulative_funding_index_short(e);
        self.save(&e);
    }

    pub fn net_funding_delta(&self, e: &Env) -> i64 {
        let long_delta = get_cumulative_funding_index_long(e).safe_sub(e, self.long_index);
        let short_delta = get_cumulative_funding_index_short(e).safe_sub(e, self.short_index);

        short_delta.safe_sub(e, long_delta)
    }
}

/**
 *
 * Funding Rate helpers
 *
 */

pub fn validate_funding_rate_update(e: &Env, funding_paused: bool, current_time: u64) {
    // TODO: Pause funding if oracle is invalid?

    let last_funding_rate_ts = get_last_funding_rate_ts(e);
    let funding_period = get_funding_period(e);
    log!(&e, "last_funding_rate_ts", last_funding_rate_ts);
    log!(&e, "funding_period", funding_period);

    let time_until_next_update =
        on_the_hour_update(e, current_time, last_funding_rate_ts, funding_period);
    log!(&e, "time_until_next_update", time_until_next_update);

    let valid_funding_update = !funding_paused && time_until_next_update == 0;

    if !valid_funding_update {
        panic_with_error!(e, LongShortPairError::FundingWasNotUpdated);
    }
}

/**
 * Fetches the Long Pool and Short Pool reserves from the Pool Plane.
 * If any reserve is zero, throws an error. Calculates each pool's price and returns both prices.
 */
pub fn fetch_pool_prices(e: &Env) -> (u128, u128) {
    let pools = Vec::from_array(e, [get_pool_long(e), get_pool_short(e)]);

    match e.try_invoke_contract::<Vec<(Symbol, Vec<u128>, Vec<u128>)>, soroban_sdk::Error>(
        &get_pool_plane(e),
        &Symbol::new(e, "get"),
        Vec::from_array(e, [pools.into_val(e)]),
    ) {
        Ok(Err(_)) | Err(_) => {
            panic_with_error!(e, LongShortPairError::FailedToGetPoolReserves);
        }
        Ok(Ok(pools_info)) => {
            let (_, _, pool_long_reserves) = pools_info.get(0).unwrap();
            let (_, _, pool_short_reserves) = pools_info.get(1).unwrap();

            let pool_long_reserve_a = pool_long_reserves.get(0).unwrap_or(0);
            let pool_long_reserve_b = pool_long_reserves.get(1).unwrap_or(0);

            let pool_short_reserve_a = pool_short_reserves.get(0).unwrap_or(0);
            let pool_short_reserve_b = pool_short_reserves.get(1).unwrap_or(0);

            // If there is empty liquidity, do not update the funding rate
            if pool_long_reserve_a == 0
                || pool_long_reserve_b == 0
                || pool_short_reserve_a == 0
                || pool_short_reserve_b == 0
            {
                panic_with_error!(e, LongShortPairError::FundingRateRequiresPoolLiquidity);
            }

            let pool_long_price =
                pool_long_reserve_b.safe_fixed_div_round(e, pool_long_reserve_a, PRICE_PRECISION);
            let pool_short_price =
                pool_short_reserve_b.safe_fixed_div_round(e, pool_short_reserve_a, PRICE_PRECISION);

            return (pool_long_price, pool_short_price);
        }
    }
}

pub fn calculate_funding_rate(
    e: &Env,
    long_pool_price: u128,
    short_pool_price: u128,
    collateral_percent_long: u64,
    period_adjustment: i64,
) -> (i64, bool) {
    if collateral_percent_long > PERCENT_MULTIPLIER_U64 {
        panic_with_error!(e, LongShortPairError::InvalidCalculatorValue);
    }

    // Globals
    let collateral_per_pair = get_collateral_per_pair(e);
    let collateral_percent_short = PERCENT_MULTIPLIER_U64.safe_sub(e, collateral_percent_long);

    // Long
    let long_pool_ideal_price = (collateral_per_pair
        * (PERCENT_MULTIPLIER - (collateral_percent_long as u128)))
        / PERCENT_MULTIPLIER;
    log!(e, "long_pool_ideal_price", long_pool_ideal_price);

    let delta_long_price = (long_pool_price as i128).safe_sub(e, long_pool_ideal_price as i128);
    log!(e, "delta_long_price", delta_long_price);

    let delta_long_pct = delta_long_price
        .fixed_div_floor(long_pool_ideal_price as i128, PERCENT_MULTIPLIER_I128)
        .unwrap() as i64;
    log!(e, "delta_long_pct", delta_long_pct);

    // Short Pool
    let short_pool_ideal_price = (collateral_per_pair
        * (PERCENT_MULTIPLIER - (collateral_percent_short as u128)))
        / PERCENT_MULTIPLIER;
    log!(e, "short_pool_ideal_price", short_pool_ideal_price);

    let delta_short_price = (short_pool_price as i128).safe_sub(e, short_pool_ideal_price as i128);
    log!(e, "delta_short_price", delta_short_price);

    let delta_short_pct = delta_short_price
        .fixed_div_floor(short_pool_ideal_price as i128, PERCENT_MULTIPLIER_I128)
        .unwrap() as i64;
    log!(e, "delta_short_pct", delta_short_pct);

    let requires_funding = do_deltas_require_funding(delta_long_pct, delta_short_pct);
    log!(e, "requires_funding", requires_funding);

    // Market sentiment
    let imbalance = delta_long_pct.safe_sub(e, delta_short_pct);
    // let total_delta = long_pool_price_delta.safe_add(e, short_pool_price_delta);
    log!(e, "imbalance", imbalance);

    // FIXME: should we still validate the price delta before assuming?
    if !requires_funding {
        return (0_i64, requires_funding);
    }

    // TODO: validate_price_delta(e, total_delta, current_time);

    // clamp the imbalance
    let max_imbalance = get_funding_clamp(e);

    let clamped_imbalance = (imbalance as i128).clamp(-max_imbalance, max_imbalance) as i64;
    log!(e, "clamped_imbalance", clamped_imbalance);

    log!(e, "period_adjustment", period_adjustment);
    let funding_rate = clamped_imbalance
        .safe_mul(e, FUNDING_RATE_BUFFER as i64)
        .safe_div(e, PERCENT_MULTIPLIER_I64)
        .safe_div(e, period_adjustment);
    log!(e, "funding_rate", funding_rate);

    (funding_rate, requires_funding)
}

// If the price deltas are opposite, we can be certain arbitrage will NOT naturally fix them.
// Therefore, the funding rate must be applied, otherwise, we set it to zero.
pub fn do_deltas_require_funding(delta_long: i64, delta_short: i64) -> bool {
    if (delta_long > 0 && delta_short < 0) || (delta_long < 0 && delta_short > 0) {
        return true;
    } else {
        return false;
    }
}

// pub fn validate_price_delta(e: &Env, price_delta: i128, current_time: u64) {
//     // FIXME: Pause funding if delta is too divergent
//     let price_delta_pct = calculate_price_delta_pct(
//         e,
//         price_delta as i64,
//         expected_price
//     );

//     let block_funding_rate_update = block_operation(e, price_delta as i64, current_time);

//     if block_funding_rate_update {
//         // return false;
//         panic_with_error!(e, LongShortPairError::AlreadyInitialized)
//     }
// }

pub fn update_funding_info(e: &Env, requires_funding: bool, funding_rate: i64, current_time: u64) {
    set_last_funding_rate(e, &funding_rate);

    // Set both cumulative funding indexes to zero if th
    let new_funding_index_long = if requires_funding {
        get_cumulative_funding_index_long(e).safe_sub(e, funding_rate)
    } else {
        0
    };
    set_cumulative_funding_index_long(e, &new_funding_index_long);

    let new_funding_index_short = if requires_funding {
        get_cumulative_funding_index_short(e).safe_add(e, funding_rate)
    } else {
        0
    };
    set_cumulative_funding_index_short(e, &new_funding_index_short);

    let last_24h_avg_funding_rate = get_last_24h_avg_funding_rate(e);
    let last_funding_rate_ts = get_last_funding_rate_ts(e);
    set_last_24h_avg_funding_rate(
        e,
        &oracle::math::calculate_new_twap(
            e,
            funding_rate as i128,
            current_time as i64,
            last_24h_avg_funding_rate as i128,
            last_funding_rate_ts as i64,
            TWENTY_FOUR_HOUR as i64,
        ),
    );
    set_last_funding_rate_ts(e, &current_time);
}

/**
 *
 * Core Funding Methods
 *
 */

pub fn update_funding_rate(e: &Env, funding_period: u64, funding_paused: bool, current_time: u64) {
    validate_funding_rate_update(e, funding_paused, current_time);

    // funding period = 1 hour, window = 1 day
    // low periodicity => quickly updating/settled funding rates => lower funding rate payment per interval
    let period_adjustment = (24_i64)
        .safe_mul(e, ONE_HOUR_I64)
        .safe_div(e, max(ONE_HOUR_I64, funding_period as i64));

    // Compute the
    let (long_pool_price, short_pool_price) = fetch_pool_prices(e);
    log!(e, "long_pool_price", long_pool_price);
    log!(e, "short_pool_price", short_pool_price);

    let collateral_percent_long = get_collateral_percent_long(e);
    let (funding_rate, requires_funding) = calculate_funding_rate(
        e,
        long_pool_price,
        short_pool_price,
        collateral_percent_long,
        period_adjustment,
    );

    update_funding_info(e, requires_funding, funding_rate, current_time);

    Events::new(&e).funding_rate_record(current_time, funding_rate);
}

/**
 *
 * Helpers
 *
 */

pub fn calculate_price_delta_pct(e: &Env, spread: i64, other_price: u64) -> i64 {
    // price_spread_pct
    spread
        .safe_mul(e, PRICE_PRECISION_I64)
        .safe_div(e, other_price as i64)
}

pub fn is_price_delta_too_divergent(e: &Env, ratio_spread_pct: i64) -> bool {
    let max_divergence = get_max_ratio_percent_divergence(e).max(PERCENTAGE_PRECISION_U64 / 10);

    ratio_spread_pct.unsigned_abs() > max_divergence
}

pub fn block_operation(e: &Env, price_delta_pct: i64, now: u64) -> bool {
    let is_spread_pct_too_divergent: bool = is_price_delta_too_divergent(e, price_delta_pct);

    let seconds_since_update = now.saturating_sub(get_last_update_ts(e));

    let funding_paused = get_is_killed_update_funding(e);

    // block if pair hasnt been updated since over half the funding period (assuming slot ~= 500ms)
    let block = seconds_since_update > get_funding_period(e)
        || is_spread_pct_too_divergent
        || funding_paused;

    block
}

pub fn on_the_hour_update(e: &Env, now: u64, last_update_ts: u64, update_period: u64) -> u64 {
    let time_since_last_update = now.safe_sub(e, last_update_ts);
    log!(&e, "time_since_last_update", time_since_last_update);

    // round next update time to be available on the hour
    let mut next_update_wait = update_period;
    if update_period > 1 {
        let last_update_delay = last_update_ts.rem_euclid(update_period);
        log!(&e, "last_update_delay", last_update_delay);
        if last_update_delay != 0 {
            let max_delay_for_next_period = update_period.safe_div(e, 3);

            let two_funding_periods = update_period.safe_mul(e, 2);

            if last_update_delay > max_delay_for_next_period {
                // too late for on the hour next period, delay to following period
                next_update_wait = two_funding_periods.safe_sub(e, last_update_delay);
                log!(&e, "a", next_update_wait);
            } else {
                // allow update on the hour
                next_update_wait = update_period.safe_sub(e, last_update_delay);
                log!(&e, "b", next_update_wait);
            }

            if next_update_wait > two_funding_periods {
                next_update_wait = next_update_wait.safe_sub(e, update_period);
                log!(&e, "c", next_update_wait);
            }
        }
    }

    let time_remaining_until_update = (next_update_wait as i64)
        .safe_sub(e, time_since_last_update as i64)
        .max(0);
    log!(
        &e,
        "time_remaining_until_update",
        time_remaining_until_update
    );

    time_remaining_until_update as u64
}
