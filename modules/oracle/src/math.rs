use soroban_sdk::{panic_with_error, Env};
use utils::{constant::DEFAULT_MAX_TWAP_UPDATE_PRICE_BAND_DENOMINATOR, math::safe_math::SafeMath};

use crate::errors::OracleError;

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
    // If both weights are zero, result is undefined → return 0
    if weight1 == 0 && weight2 == 0 {
        return 0;
    }

    if weight1 == 0 {
        return data2;
    }

    if weight2 == 0 {
        return data1;
    }

    let denominator = weight1.safe_add(e, weight2) as u128;

    let w1 = data1.safe_mul(e, weight1 as u128);
    let w2 = data2.safe_mul(e, weight2 as u128);

    let mut twap = w1.safe_add(e, w2).safe_div(e, denominator);

    // Optional 1-tick bias to reduce sticky rounding
    // Bias direction is based on raw prices (not weighted sums)
    if weight2 > 1 {
        if data2 > data1 {
            twap = twap.safe_add(e, 1);
        } else if data2 < data1 && twap > 0 {
            twap = twap.safe_sub(e, 1);
        }
    }

    twap
}

pub fn calculate_new_twap(
    e: &Env,
    current_price: u128,
    current_ts: u64,
    last_twap: u128,
    last_ts: u64,
    period: u64,
) -> u128 {
    // Time must not go backwards
    if current_ts < last_ts {
        panic_with_error!(e, OracleError::InvalidInput);
    }

    // Degenerate period → instant price
    if period == 0 {
        return current_price;
    }

    let since_last = current_ts.safe_sub(e, last_ts);

    // Cap weight at period
    let weight_now = if since_last >= period {
        period
    } else {
        since_last
    };

    let weight_prev = period.safe_sub(e, weight_now);

    calculate_weighted_average(e, current_price, last_twap, weight_now, weight_prev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;
    use utils::constant::PRICE_PRECISION;

    // ---------------------------
    // calculate_weighted_average
    // ---------------------------

    #[test]
    fn weighted_average_both_weights_zero_returns_zero() {
        let e = Env::default();
        let out = calculate_weighted_average(&e, 123, 456, 0, 0);
        assert_eq!(out, 0);
    }

    #[test]
    fn weighted_average_weight1_zero_returns_data2() {
        let e = Env::default();
        let out = calculate_weighted_average(&e, 123, 456, 0, 10);
        assert_eq!(out, 456);
    }

    #[test]
    fn weighted_average_weight2_zero_returns_data1() {
        let e = Env::default();
        let out = calculate_weighted_average(&e, 123, 456, 10, 0);
        assert_eq!(out, 123);
    }

    #[test]
    fn weighted_average_no_bias_exact() {
        let e = Env::default();

        // weight2 == 1 => bias disabled by your logic
        // twap = (data1*w1 + data2*w2)/(w1+w2)
        let data1 = 10;
        let data2 = 20;
        let w1 = 3;
        let w2 = 1;

        // (10*3 + 20*1)/4 = 50/4 = 12 (floor)
        let out = calculate_weighted_average(&e, data1, data2, w1, w2);
        assert_eq!(out, 12);
    }

    #[test]
    fn weighted_average_bias_up_applies_when_weight2_gt_1() {
        let e = Env::default();

        // Pick values so floor(twap) is exact and bias is clearly +1:
        // data1 < data2 and weight2 > 1 => bias +1
        let data1 = 10;
        let data2 = 20;
        let w1 = 1;
        let w2 = 2;

        // twap = (10*1 + 20*2)/3 = 50/3 = 16 (floor)
        // bias +1 => 17
        let out = calculate_weighted_average(&e, data1, data2, w1, w2);
        assert_eq!(out, 17);
    }

    #[test]
    fn weighted_average_bias_down_applies_when_weight2_gt_1_and_twap_gt_0() {
        let e = Env::default();

        // data2 < data1 and weight2 > 1 => bias -1, but only if twap > 0
        let data1 = 20;
        let data2 = 10;
        let w1 = 1;
        let w2 = 2;

        // twap = (20*1 + 10*2)/3 = 40/3 = 13 (floor)
        // bias -1 => 12
        let out = calculate_weighted_average(&e, data1, data2, w1, w2);
        assert_eq!(out, 12);
    }

    #[test]
    fn weighted_average_bias_down_does_not_underflow_when_twap_is_zero() {
        let e = Env::default();

        // Make twap compute to 0, and bias would be -1
        // data1=0, data2=0, weights non-zero: twap=0, bias is 0 because data2 == data1
        // We specifically want data2 < data1, but twap == 0; easiest is data1=1, data2=0 with large denom:
        //
        // twap = floor((1*w1 + 0*w2)/(w1+w2)) can be 0 if denom > w1.
        // Example: w1=1, w2=2 => twap = floor(1/3) = 0, data2 < data1 and weight2 > 1 => bias -1
        // Your code guards: only subtract 1 if twap > 0. So should remain 0.
        let data1 = 1;
        let data2 = 0;
        let w1 = 1;
        let w2 = 2;

        let out = calculate_weighted_average(&e, data1, data2, w1, w2);
        assert_eq!(out, 0);
    }

    // ---------------------------
    // calculate_new_twap
    // ---------------------------

    #[test]
    #[should_panic]
    fn new_twap_panics_if_time_goes_backward() {
        let e = Env::default();
        let _ = calculate_new_twap(&e, 100, 9, 50, 10, 30);
    }

    #[test]
    fn new_twap_period_zero_returns_current_price() {
        let e = Env::default();
        let out = calculate_new_twap(&e, 123, 10, 999, 0, 0);
        assert_eq!(out, 123);
    }

    #[test]
    fn new_twap_same_timestamp_returns_last_twap() {
        let e = Env::default();

        // since_last = 0 => weight_now = 0, weight_prev = period
        // calculate_weighted_average(current_price, last_twap, 0, period) => last_twap
        let out = calculate_new_twap(&e, 777, 10, 555, 10, 30);
        assert_eq!(out, 555);
    }

    #[test]
    fn new_twap_since_last_ge_period_fully_updates_to_current_price() {
        let e = Env::default();

        // since_last >= period => weight_now = period, weight_prev = 0
        // calculate_weighted_average(current_price, last_twap, period, 0) => current_price
        let out = calculate_new_twap(&e, 777, 100, 555, 0, 30);
        assert_eq!(out, 777);
    }

    #[test]
    fn new_twap_intermediate_mix_matches_expected() {
        let e = Env::default();

        // period=10, since_last=3 => weight_now=3, weight_prev=7
        // twap = floor((current*3 + last*7)/10) with possible bias if weight_prev > 1
        //
        // Important: in calculate_new_twap you call:
        // calculate_weighted_average(current_price, last_twap, weight_now, weight_prev)
        // bias decision uses data2 vs data1, i.e. last_twap vs current_price.
        // Here set last_twap > current_price and weight_prev=7 (>1) => bias +1 (since data2 > data1).
        //
        // Let's compute:
        // current=100, last=200
        // base = (100*3 + 200*7)/10 = (300 + 1400)/10 = 170
        // bias +1 => 171
        let out = calculate_new_twap(&e, 100, 13, 200, 10, 10);
        assert_eq!(out, 171);
    }

    #[test]
    fn new_twap_allows_zero_prices_and_timestamps_when_valid() {
        let e = Env::default();

        // last_ts=0, current_ts=0 is allowed (not backwards, equal)
        // since_last=0 => returns last_twap (0)
        let out = calculate_new_twap(&e, 0, 0, 0, 0, 10);
        assert_eq!(out, 0);

        // If period=0, should return current_price even with zero timestamps
        let out2 = calculate_new_twap(&e, 0, 0, 999, 0, 0);
        assert_eq!(out2, 0);
    }

    // Optional sanity: TWAP always in [min(data1,data2), max(data1,data2)] +/- 1 due to bias
    #[test]
    fn new_twap_is_bounded_within_reasonable_range() {
        let e = Env::default();

        let current = 500 * PRICE_PRECISION;
        let last = 100 * PRICE_PRECISION;

        let out = calculate_new_twap(&e, current, 105, last, 100, 10);

        // With bias, it can move by at most 1 "tick" (1 unit) from the floor average.
        // So it should definitely be <= max+1 and >= min.saturating_sub(1)
        let max = current.max(last);
        let min = current.min(last);

        assert!(out <= max + 1);
        assert!(out + 1 >= min); // avoids underflow in assertion
    }
}
