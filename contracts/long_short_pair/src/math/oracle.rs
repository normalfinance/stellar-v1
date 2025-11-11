use core::cmp::max;

use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{panic_with_error, Env, Symbol};
use utils::{
    constant::{
        DEFAULT_MAX_TWAP_UPDATE_PRICE_BAND_DENOMINATOR, FIVE_MINUTE, PERCENTAGE_PRECISION,
        PERCENTAGE_PRECISION_U64, PRICE_PRECISION,
    },
    math::safe_math::{PrecisionMath, SafeConversion, SafeMath},
    temporal::Delay,
};

use crate::{
    errors::LongShortPairError,
    state::oracle::{HistoricalOracleData, OraclePriceData, OracleValidity},
    storage::{
        get_historical_oracle_data, get_oracle, get_oracle_guard_rails, put_historical_oracle_data,
    },
};

// Fetches the latest oracle price and timestamp for a given asset.
//
// Wraps the `PriceFeedClient` to retrieve the last published price and calculates
// the delay since publication based on the current timestamp.
//
// # Arguments
// * `e` - Soroban environment reference.
// * `oracle` - Address of the price oracle contract.
// * `asset` - Address of the asset being queried.
// * `now` - Current timestamp.
//
// # Returns
// - `OraclePriceData` containing the price and delay since last update.
pub fn get_oracle_price(e: &Env, asset: &Symbol, now: u64) -> OraclePriceData {
    assert!(now > 0, "now timestamp must be positive");

    let oracle_addr = get_oracle(e);
    let oracle_client = PriceFeedClient::new(e, &oracle_addr);
    let oracle_asset = Asset::Other(asset.clone());

    let oracle_price: u128;
    let published_ts: u64;

    let oracle_price_data = oracle_client.lastprice(&oracle_asset).unwrap();

    oracle_price = (oracle_price_data.price as u128).safe_div(&e, PRICE_PRECISION);
    published_ts = oracle_price_data.timestamp;

    let oracle_delay = Delay::from_timestamp_diff_expect(
        now,
        published_ts,
        "Oracle published timestamp exceeds allowed clock drift tolerance",
    );

    OraclePriceData {
        price: oracle_price,
        delay: oracle_delay,
    }
}

// Updates the time-weighted average price (TWAP) for a given asset using a new oracle price.
//
// The new price is first sanitized to prevent manipulation, then incorporated into the TWAP
// using a weighted rolling average. The result is stored as updated historical oracle data.
//
// # Arguments
// * `e` - Soroban environment reference.
// * `historical_oracle_data` - The previously recorded oracle data.
// * `oracle_price_data` - The newly observed price and timestamp.
// * `sanitize_clamp_denominator` - Clamp denominator for price sanitization.
// * `now` - Current timestamp.
pub fn update_twap(
    e: &Env,
    asset: &Symbol,
    historical_oracle_data: &HistoricalOracleData,
    oracle_price_data: &OraclePriceData,
    sanitize_clamp_denominator: u64,
    now: u64,
) {
    let capped_oracle_update_price = sanitize_new_price(
        e,
        oracle_price_data.price,
        historical_oracle_data.last_price_twap,
        sanitize_clamp_denominator,
    );

    let oracle_price_twap = calculate_new_twap(
        e,
        capped_oracle_update_price,
        now,
        historical_oracle_data.last_price_twap,
        historical_oracle_data.last_update_ts,
        FIVE_MINUTE as u64,
    );

    put_historical_oracle_data(
        e,
        asset,
        &(HistoricalOracleData {
            last_price_twap: oracle_price_twap,
            last_price: oracle_price_data.price,
            last_update_ts: now,
        }),
    );
}

// Classifies the current oracle price data as valid, stale, or invalid.
//
// Uses three core checks:
// - Price is positive
// - Price is not too volatile relative to last TWAP
// - Price is not too old (stale) for use in pools
//
// # Arguments
// * `e` - Soroban environment reference.
// * `last_oracle_twap` - Previous TWAP value.
// * `oracle_price_data` - Current oracle price and timestamp.
//
// # Returns
// - `OracleValidity` enum indicating the health of the oracle data.
pub fn oracle_validity(
    e: &Env,
    last_oracle_twap: u128,
    oracle_price_data: &OraclePriceData,
) -> OracleValidity {
    let OraclePriceData {
        price: oracle_price,
        delay: oracle_delay,
    } = *oracle_price_data;

    let oracle_guard_rails = get_oracle_guard_rails(e);

    // NonPositive
    let is_oracle_price_nonpositive = oracle_price <= 0;

    // Volatility
    // if Δprice <= 0.80 or 1.20 <= Δprice → too volatile
    let lower_bound =
        PERCENTAGE_PRECISION_U64.safe_sub(e, oracle_guard_rails.validity.too_volatile_ratio);

    let upper_bound = oracle_guard_rails
        .validity
        .too_volatile_ratio
        .safe_add(e, PERCENTAGE_PRECISION_U64);

    // Use round-to-nearest for volatility calculation (fair assessment)
    let price_delta = oracle_price
        .safe_fixed_div_round(e, last_oracle_twap, PERCENTAGE_PRECISION)
        .safe_to_u64(e);

    let is_oracle_price_too_volatile = price_delta <= lower_bound || upper_bound <= price_delta;

    // StaleForPool
    let is_stale_for_pool = oracle_delay
        .as_seconds()
        .ge(&oracle_guard_rails.validity.seconds_before_stale_for_pool);

    let oracle_validity = if is_oracle_price_nonpositive {
        OracleValidity::NonPositive
    } else if is_oracle_price_too_volatile {
        OracleValidity::TooVolatile
    } else if is_stale_for_pool {
        OracleValidity::StaleForPool
    } else {
        OracleValidity::Valid
    };

    oracle_validity
}

pub fn get_oracle_price_with_validity(
    e: &Env,
    asset: &Symbol,
    current_time: u64,
) -> HistoricalOracleData {
    let oracle_price_data = get_oracle_price(&e, &asset, current_time);

    let historical_oracle_data = get_historical_oracle_data(e, asset);

    let oracle_validity = oracle_validity(
        &e,
        historical_oracle_data.last_price_twap,
        &oracle_price_data,
    );

    if oracle_validity != OracleValidity::Valid {
        panic_with_error!(e, LongShortPairError::InvalidOracle);
    }

    update_twap(
        e,
        asset,
        &historical_oracle_data,
        &oracle_price_data,
        1,
        current_time,
    );

    get_historical_oracle_data(e, asset)
}

/// Sanitizes a new oracle price update by clamping it within a band around the TWAP.
///
/// This function guards against abrupt oracle spikes by limiting `new_price` to
/// a maximum delta from `last_price_twap`. If `sanitize_clamp_denominator` is
/// non-zero, the allowed price band is:
///
/// ```text
/// band = last_price_twap / sanitize_clamp_denominator
/// ```
///
/// The sanitized price is then:
///
/// ```text
/// if abs(new_price - last_price_twap) > band:
///     // clamp toward the TWAP edge
///     capped = last_price_twap ± band
/// else:
///     capped = new_price
/// ```
///
/// If `sanitize_clamp_denominator == 0`, no clamping is applied and `new_price` is returned.
/// If `last_price_twap == 0`, normalization isn’t attempted and `new_price` is returned as-is.
///
/// # Arguments
///
/// * `e` — Soroban [`Env`] for safe math helpers (e.g., `safe_add`, `safe_sub`, `safe_div`).
/// * `new_price` — The latest oracle price reading (u128, must be > 0).
/// * `last_price_twap` — The previous time-weighted average price used as an anchor (u128).
/// * `sanitize_clamp_denominator` — Denominator controlling the allowed deviation.
///   - If 0, disables clamping.
///   - If non-zero, the max deviation is `last_price_twap / sanitize_clamp_denominator`.
///   - If omitted/0, a default (`DEFAULT_MAX_TWAP_UPDATE_PRICE_BAND_DENOMINATOR`) is used.
///
/// # Returns
///
/// * `u128` — The sanitized price, clamped to the TWAP band if applicable.
///
/// # Panics
///
/// * Panics if `new_price == 0`.
/// * Asserts that `last_price_twap` is non-negative (redundant for `u128`, but retained for clarity).
///
/// # Notes
///
/// * Uses `safe_*` helpers to avoid overflow/underflow on intermediate arithmetic.
/// * When clamping downward, if `band >= last_price_twap`, the function returns `0`
///   to avoid underflow and represent a floor at zero.
///
/// # Examples
///
/// ```rust
/// // Allow at most ±1% move from TWAP (denominator 100)
/// let twap = 100_000u128;
/// let new = 105_000u128; // +5%
/// let clamped = sanitize_new_price(&env, new, twap, 100);
/// assert_eq!(clamped, 101_000); // TWAP + 1% band
/// ```
pub fn sanitize_new_price(
    e: &Env,
    new_price: u128,
    last_price_twap: u128,
    sanitize_clamp_denominator: u64,
) -> u128 {
    assert!(new_price > 0, "new_price must be positive");
    assert!(last_price_twap >= 0, "last_price_twap must be non-negative");

    // when/if twap is 0, dont try to normalize new_price
    if last_price_twap == 0 {
        return new_price;
    }

    let (new_price_spread, price_is_increasing) = if new_price >= last_price_twap {
        (new_price.safe_sub(e, last_price_twap), true)
    } else {
        (last_price_twap.safe_sub(e, new_price), false)
    };

    // cap new oracle update to 100/MAX_TWAP_UPDATE_PRICE_BAND_DENOMINATOR% delta from twap
    let sanitize_clamp_denominator = if sanitize_clamp_denominator != 0 {
        sanitize_clamp_denominator
    } else {
        DEFAULT_MAX_TWAP_UPDATE_PRICE_BAND_DENOMINATOR
    };

    if sanitize_clamp_denominator == 0 {
        // no need to use price band check
        return new_price;
    }

    let price_twap_price_band = last_price_twap.safe_div(e, sanitize_clamp_denominator as u128);

    let capped_update_price = if new_price_spread > price_twap_price_band {
        if price_is_increasing {
            last_price_twap.safe_add(e, price_twap_price_band)
        } else {
            if price_twap_price_band >= last_price_twap {
                0
            } else {
                last_price_twap.safe_sub(e, price_twap_price_band)
            }
        }
    } else {
        new_price
    };

    capped_update_price
}

pub fn calculate_weighted_average(
    e: &Env,
    data1: u128,
    data2: u128,
    weight1: u64,
    weight2: u64,
) -> u128 {
    let denominator = weight1.safe_add(e, weight2) as u128;
    let prev_twap_99 = data1.safe_mul(e, weight1 as u128);
    let latest_price_01 = data2.safe_mul(e, weight2 as u128);

    if weight1 == 0 {
        return data2;
    }

    if weight2 == 0 {
        return data1;
    }

    let bias: i128 = if weight2 > 1 {
        if latest_price_01 < prev_twap_99 {
            -1
        } else if latest_price_01 > prev_twap_99 {
            1
        } else {
            0
        }
    } else {
        0
    };

    let twap = prev_twap_99
        .safe_add(e, latest_price_01)
        .safe_div(e, denominator);

    if twap == 0 && bias < 0 {
        return twap;
    }

    (twap as i128).safe_add(e, bias) as u128
}

pub fn calculate_new_twap(
    e: &Env,
    current_price: u128,
    current_ts: u64,
    last_twap: u128,
    last_ts: u64,
    period: u64,
) -> u128 {
    let since_last = max(0_u64, current_ts.safe_sub(&e, last_ts));
    let from_start = max(1_u64, period.safe_sub(&e, since_last));

    calculate_weighted_average(e, current_price, last_twap, since_last, from_start)
}
