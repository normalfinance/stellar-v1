use oracle::{
    errors::OracleError,
    math::{calculate_new_twap, sanitize_new_price},
    state::{HistoricalOracleData, OracleValidity},
};
use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{panic_with_error, Address, Env, Symbol};
use types::oracle::OraclePriceData;
use utils::{
    constant::{FIVE_MINUTE, PERCENTAGE_PRECISION, PERCENTAGE_PRECISION_U64, PRICE_PRECISION},
    math::safe_math::{PrecisionMath, SafeConversion, SafeMath},
    temporal::Delay,
};

use crate::{
    errors::NormalOracleError,
    storage::{get_seconds_before_stale, get_too_volatile_ratio, put_historical_data},
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

    match oracle_client.try_lastprice(&oracle_asset) {
        Ok(Err(_)) | Err(_) => {
            panic_with_error!(e, NormalOracleError::FailedToGetOraclePrice);
        }
        Ok(Ok(result)) => {
            let oracle_price_data = result.unwrap();

            if oracle_price_data.price < 0 {
                panic_with_error!(e, OracleError::OracleNonPositive);
            }

            oracle_price = oracle_price_data
                .price
                .safe_to_u128(e)
                .safe_div(&e, PRICE_PRECISION);

            published_ts = oracle_price_data.timestamp;

            let oracle_delay = Delay::from_timestamp_diff_expect(e, now, published_ts);

            OraclePriceData {
                price: oracle_price,
                delay: oracle_delay,
            }
        }
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
    sanitize_clamp_denominator: u128,
    now: u64,
) -> HistoricalOracleData {
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

    let new_historical_oracle_data = HistoricalOracleData {
        last_price_twap: oracle_price_twap.safe_to_u128(e),
        last_price: oracle_price_data.price,
        last_update_ts: now,
        last_delay_ts: oracle_price_data.delay.as_seconds(),
    };
    put_historical_data(e, &new_historical_oracle_data);

    new_historical_oracle_data
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
    oracle_price_data: &OraclePriceData,
    last_oracle_twap: u128,
) -> OracleValidity {
    let OraclePriceData {
        price: oracle_price,
        delay: oracle_delay,
    } = *oracle_price_data;

    // Guard rails
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

    let is_price_too_volatile = price_delta <= lower_bound || upper_bound <= price_delta;

    // StaleForPair
    let is_stale = oracle_delay.as_seconds().ge(&seconds_before_stale);

    let oracle_validity = if is_oracle_price_nonpositive {
        OracleValidity::NonPositive
    } else if is_price_too_volatile {
        OracleValidity::TooVolatile
    } else if is_stale {
        OracleValidity::StaleForPair
    } else {
        OracleValidity::Valid
    };

    oracle_validity
}
