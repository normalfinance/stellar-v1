use core::cmp::max;

use crate::errors::LongShortPairError;
use crate::events::{Events, LongShortPairEvents};
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol, Vec};
use utils::constant::{FUNDING_RATE_BUFFER_I128, ONE_HOUR_I128, PRICE_PRECISION, TWENTY_FOUR_HOUR};
use utils::math::oracle::calculate_new_twap;
use utils::math::safe_math::{PrecisionMath, SafeMath};

use crate::storage::{
    get_collateral_percent_long, get_cumulative_funding_index_long,
    get_cumulative_funding_index_short, get_funding_period, get_last_24h_avg_funding_rate,
    get_last_funding_rate_ts, get_pool, get_sanitize_clamp_denominator,
    put_user_funding_checkpoint, set_cumulative_funding_index_long,
    set_cumulative_funding_index_short, set_last_24h_avg_funding_rate, set_last_funding_rate,
    set_last_funding_rate_ts,
};

#[derive(Clone)]
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

pub fn update_funding_rate(e: &Env, funding_paused: bool, current_time: u64) -> bool {
    // Pause funding if oracle is invalid or if mark/oracle spread is too divergent
    let block_funding_rate_update = false;
    // oracle::block_operation(
    //     market,
    //     oracle_map.get_price_data(&market.oracle_id())?,
    //     guard_rails,
    //     reserve_price,
    //     slot
    // );

    let last_funding_rate_ts = get_last_funding_rate_ts(e);
    let funding_period = get_funding_period(e);

    let time_until_next_update =
        on_the_hour_update(e, current_time, last_funding_rate_ts, funding_period);

    let valid_funding_update =
        !funding_paused && !block_funding_rate_update && time_until_next_update == 0;

    if valid_funding_update {
        let funding_rate = 0;

        // get oracle price
        let sanitize_clamp_denominator = get_sanitize_clamp_denominator(e);

        // update twap

        let period_adjustment = (24_i128)
            .safe_mul(e, ONE_HOUR_I128)
            .safe_div(e, max(ONE_HOUR_I128, funding_period as i128));
        // funding period = 1 hour, window = 1 day
        // low periodicity => quickly updating/settled funding rates => lower funding rate payment per interval

        // ...

        match e.try_invoke_contract::<Vec<u128>, soroban_sdk::Error>(
            &get_pool(e),
            &Symbol::new(e, "get_reserves"),
            Vec::from_array(e, []),
        ) {
            Ok(Err(_)) | Err(_) => {
                panic_with_error!(e, LongShortPairError::FailedToGetPoolReserves);
            }
            Ok(Ok(reserves)) => {
                let (reserve_long, reserve_short) =
                    (reserves.get(0).unwrap(), reserves.get(1).unwrap());

                let pool_long_ratio = reserve_long.safe_fixed_div_round(
                    e,
                    reserve_long.safe_add(e, reserve_short),
                    PRICE_PRECISION,
                );

                let expected_long_ratio = get_collateral_percent_long(e);
                let ratio_spread =
                    (pool_long_ratio as i128).safe_sub(e, expected_long_ratio as i128);

                // clamp ratio divergence
                let max_ratio_spread: i128 = 2_000_000; // max +/-0.2 spread; TODO: make dynamic based on contract tier for funding rate calculation

                let clamped_ratio_spread = ratio_spread.clamp(-max_ratio_spread, max_ratio_spread);

                let funding_rate = clamped_ratio_spread
                    .safe_mul(e, FUNDING_RATE_BUFFER_I128)
                    .safe_div(e, period_adjustment) as i64;

                set_last_funding_rate(e, &funding_rate);

                set_cumulative_funding_index_long(
                    e,
                    &get_cumulative_funding_index_long(e).safe_sub(e, funding_rate),
                );
                set_cumulative_funding_index_short(
                    e,
                    &get_cumulative_funding_index_short(e).safe_add(e, funding_rate),
                );

                let last_24h_avg_funding_rate = get_last_24h_avg_funding_rate(e);
                set_last_24h_avg_funding_rate(
                    e,
                    &calculate_new_twap(
                        e,
                        funding_rate,
                        current_time as i64,
                        last_24h_avg_funding_rate,
                        last_funding_rate_ts as i64,
                        TWENTY_FOUR_HOUR as i64,
                    ),
                );
                set_last_funding_rate_ts(e, &current_time);

                Events::new(&e).funding_rate_record(current_time, funding_rate);
            }
        };
    } else {
        return false;
    }

    true
}

pub fn on_the_hour_update(e: &Env, now: u64, last_update_ts: u64, update_period: u64) -> u64 {
    let time_since_last_update = now.safe_sub(e, last_update_ts);

    // round next update time to be available on the hour
    let mut next_update_wait = update_period;
    if update_period > 1 {
        let last_update_delay = last_update_ts.rem_euclid(update_period);
        if last_update_delay != 0 {
            let max_delay_for_next_period = update_period.safe_div(e, 3);

            let two_funding_periods = update_period.safe_mul(e, 2);

            if last_update_delay > max_delay_for_next_period {
                // too late for on the hour next period, delay to following period
                next_update_wait = two_funding_periods.safe_sub(e, last_update_delay);
            } else {
                // allow update on the hour
                next_update_wait = update_period.safe_sub(e, last_update_delay);
            }

            if next_update_wait > two_funding_periods {
                next_update_wait = next_update_wait.safe_sub(e, update_period);
            }
        }
    }

    let time_remaining_until_update = next_update_wait.safe_sub(e, time_since_last_update).max(0);

    time_remaining_until_update
}
