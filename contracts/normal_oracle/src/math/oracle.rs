use oracle::{
    errors::OracleError,
    math::{calculate_new_twap, sanitize_new_price},
    state::{HistoricalOracleData, OraclePriceData, OracleValidity},
};
use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{panic_with_error, Address, Env, Symbol};
use types::oracle::OracleSource;
use utils::{
    constant::{FIVE_MINUTE, PERCENTAGE_PRECISION, PERCENTAGE_PRECISION_U64, PRICE_PRECISION},
    math::safe_math::{PrecisionMath, SafeConversion, SafeMath},
    temporal::Delay,
};

use crate::storage::{
    get_historical_data, get_oracle, get_oracle_source, get_seconds_before_stale,
    get_too_volatile_ratio, put_historical_data,
};

pub fn get_reflector_oracle_price(
    e: &Env,
    oracle_addr: &Address,
    asset: &Symbol,
    now: u64,
) -> OraclePriceData {
    let oracle_client = PriceFeedClient::new(e, oracle_addr);
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
    let oracle_source = get_oracle_source(e);

    match oracle_source {
        OracleSource::Reflector => get_reflector_oracle_price(e, &oracle_addr, asset, now),
    }
}

// Updates the time-weighted average price (TWAP) for a given asset using a new oracle price.
//
// The new price is first sanitized to prevent manipulation, then incorporated into the TWAP
// using a weighted rolling average. The result is stored as updated historical oracle data.
//
// # Arguments
// * `e` - Soroban environment reference.
// * `historical_data` - The previously recorded oracle data.
// * `oracle_price_data` - The newly observed price and timestamp.
// * `sanitize_clamp_denominator` - Clamp denominator for price sanitization.
// * `now` - Current timestamp.
pub fn update_twap(
    e: &Env,
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
        capped_oracle_update_price as i128,
        now as i64,
        historical_oracle_data.last_price_twap as i128,
        historical_oracle_data.last_update_ts as i64,
        FIVE_MINUTE as i64,
    );

    put_historical_data(
        e,
        &(HistoricalOracleData {
            last_price_twap: oracle_price_twap as u128,
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

    // Get guard rails
    let too_volatile_ratio = get_too_volatile_ratio(e);
    let seconds_before_stale = get_seconds_before_stale(e);

    // NonPositive
    let is_oracle_price_nonpositive = oracle_price <= 0;

    // Volatility
    // if Δprice <= 0.80 or 1.20 <= Δprice → too volatile
    let lower_bound = PERCENTAGE_PRECISION_U64.safe_sub(e, too_volatile_ratio);

    let upper_bound = too_volatile_ratio.safe_add(e, PERCENTAGE_PRECISION_U64);

    // Use round-to-nearest for volatility calculation (fair assessment)
    let price_delta = oracle_price
        .safe_fixed_div_round(e, last_oracle_twap, PERCENTAGE_PRECISION)
        .safe_to_u64(e);

    let is_oracle_price_too_volatile = price_delta <= lower_bound || upper_bound <= price_delta;

    // StaleForPool
    let is_stale_for_pool = oracle_delay.as_seconds().ge(&seconds_before_stale);

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

    let historical_oracle_data = get_historical_data(e);

    let oracle_validity = oracle_validity(
        &e,
        historical_oracle_data.last_price_twap,
        &oracle_price_data,
    );

    if oracle_validity != OracleValidity::Valid {
        panic_with_error!(e, OracleError::OracleInvalid);
    }

    update_twap(
        e,
        &historical_oracle_data,
        &oracle_price_data,
        1,
        current_time,
    );

    get_historical_data(e)
}
