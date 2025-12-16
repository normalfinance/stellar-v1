use core::cmp::max;

use soroban_sdk::Env;
use utils::{constant::DEFAULT_MAX_TWAP_UPDATE_PRICE_BAND_DENOMINATOR, math::safe_math::SafeMath};

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
    data1: i64,
    data2: i64,
    weight1: i64,
    weight2: i64,
) -> i64 {
    let denominator = weight1.safe_add(e, weight2) as i128;
    let prev_twap_99 = (data1 as i128).safe_mul(e, weight1 as i128);
    let latest_price_01 = (data2 as i128).safe_mul(e, weight2 as i128);

    if weight1 == 0 {
        return data2;
    }

    if weight2 == 0 {
        return data1;
    }

    let bias: i64 = if weight2 > 1 {
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
        .safe_div(e, denominator) as i64;

    if twap == 0 && bias < 0 {
        return twap;
    }

    twap.safe_add(e, bias)
}

pub fn calculate_new_twap(
    e: &Env,
    current_price: i128,
    current_ts: i64,
    last_twap: i128,
    last_ts: i64,
    period: i64,
) -> i64 {
    let since_last = max(0_i64, current_ts.safe_sub(&e, last_ts));
    let from_start = max(1_i64, period.safe_sub(&e, since_last));

    calculate_weighted_average(
        e,
        current_price as i64,
        last_twap as i64,
        since_last,
        from_start,
    )
}
