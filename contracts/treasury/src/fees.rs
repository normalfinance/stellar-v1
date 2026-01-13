use soroban_sdk::{panic_with_error, Env};
use types::pair::{PairAmountsWithUSDC, Side};
use utils::constant::{MAX_BOUND_POWER, ONE_U128, ONE_YEAR, PRICE_PRECISION};
use utils::math::safe_math::{PrecisionMath, SafeMath};

use crate::errors::TreasuryError;
use crate::storage::TreasuryFeeConfig;

/// Apply a fee to an input amount, taking the fee from the input amount.
///
/// The fee is expressed in `PRICE_PRECISION` (typically 1e7), where:
/// - `fee = 0` means 0%
/// - `fee = PRICE_PRECISION` means 100% (net becomes 0)
///
/// ## Formula
/// `net_in = amount_in * (1 - fee)`
///
/// ## Panics
/// - `TreasuryError::InvalidInput` if `amount_in == 0`
/// - `TreasuryError::InvalidInput` if `fee > PRICE_PRECISION`
pub fn apply_fee_to_input(e: &Env, amount_in: u128, fee: u128) -> u128 {
    if amount_in == 0 || fee > PRICE_PRECISION {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    amount_in
        .safe_mul(e, PRICE_PRECISION.safe_sub(e, fee))
        .safe_div(e, PRICE_PRECISION)
}

/// Computes the taker fee (half-spread) for a trade in `PRICE_PRECISION` (1e7) units.
///
/// This function:
/// 1) Computes the treasury NAV and the value of long/short inventory
/// 2) Derives `skew_badness` and `bound_proximity`
/// 3) Computes a non-negative fee via [`fee`]
///
/// The returned fee is always `>= 0` and clamped to `[0, PRICE_PRECISION]`.
///
/// ## Panics
/// - Propagates panics from `crate::lp::values` / `crate::lp::nav_with_values`
/// - Propagates panics from conversion helpers if misconfigured inputs exist
pub fn calculate_fee(
    e: &Env,
    side: Side,
    fee_config: &TreasuryFeeConfig,
    balances: &PairAmountsWithUSDC,
    prices: &PairAmountsWithUSDC,
    collateral_percent_long: u128,
) -> u128 {
    let values = crate::lp::values(e, balances, prices);
    let nav = crate::lp::nav_with_values(e, &values);

    calculate_fee_from_values(
        e,
        side,
        fee_config,
        values.long,
        values.short,
        nav,
        collateral_percent_long,
    )
}
/// Computes the taker fee (half-spread) given already-computed inventory values and NAV.
///
/// This is split out to make the fee logic unit-testable without needing to mock `crate::lp`.
///
/// All values are expected to be in quote-value terms and `PRICE_PRECISION` compatible.
///
/// Returned fee is always `>= 0` and clamped to `[0, PRICE_PRECISION]`.
pub fn calculate_fee_from_values(
    e: &Env,
    side: Side,
    fee_config: &TreasuryFeeConfig,
    v_long: u128,
    v_short: u128,
    nav: u128,
    collateral_percent_long: u128,
) -> u128 {
    let s_bad = skew_badness(e, v_long, v_short, nav);
    let b_prox = bound_proximity(collateral_percent_long, side == Side::Short);

    // Treat `implied_volatility` as an annualized realized-variance-like quantity in 1e7 precision,
    // then convert to "per-second" rate by dividing by seconds/year.
    let rv_per_sec = fee_config.implied_volatility.safe_div(e, ONE_YEAR); // u128

    if fee_config.bound_power > MAX_BOUND_POWER {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    fee(
        e,
        fee_config.taker_base_fee,
        rv_per_sec,
        fee_config.reaction_time_secs,
        s_bad,
        b_prox,
        fee_config.coefficient_a,
        fee_config.coefficient_c,
        fee_config.coefficient_d,
        fee_config.bound_power, // u32
    )
}

fn clamp_u128(x: u128, lo: u128, hi: u128) -> u128 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

/// Computes a directionless inventory skew severity in 1e7 precision.
///
/// - `0`      => perfectly balanced (by value)
/// - `1e7`    => maximally skewed (one side dominates NAV)
///
/// ## Definition
/// `skew_badness = |V_long - V_short| / NAV`, clamped to `[0, 1]`.
///
/// ## Notes
/// - Uses quote-value terms, not raw token quantities
/// - If `nav == 0`, returns `0` (cannot form a meaningful ratio)
pub fn skew_badness(e: &Env, v_long: u128, v_short: u128, nav: u128) -> u128 {
    if nav == 0 {
        return 0;
    }

    let diff = if v_long >= v_short {
        v_long.safe_sub(e, v_short)
    } else {
        v_short.safe_sub(e, v_long)
    };

    // diff / nav in 1e7 precision
    let num = diff.safe_fixed_mul_floor(e, ONE_U128, ONE_U128);
    let frac = num.safe_fixed_div_floor(e, nav, ONE_U128);

    clamp_u128(frac, 0, ONE_U128)
}

/// For spread/cap logic, returns proximity to the toxic bound in 1e7 precision.
///
/// This function is designed to reuse an already-computed `collateral_percent_long`
/// (which is equivalent to the band progress `t` for a linear payoff curve).
///
/// - If `toxic_is_short == true` (short is toxic near UPPER bound): `bound_proximity = t`
/// - If `toxic_is_short == false` (long is toxic near LOWER bound): `bound_proximity = 1 - t`
///
/// where `t = collateral_percent_long ∈ [0, 1]`.
pub fn bound_proximity(collateral_percent_long: u128, toxic_is_short: bool) -> u128 {
    let t = collateral_percent_long.min(ONE_U128); // defensive clamp
    if toxic_is_short {
        t
    } else {
        ONE_U128.saturating_sub(t)
    }
}

/// Computes a non-negative taker fee (half-spread) in 1e7 precision.
///
/// ## Inputs (all unsigned)
/// - `base_fee`: baseline half-spread (1e7 precision)
/// - `rv_per_sec`: annualized realized-variance-like rate per second (1e7 precision)
/// - `reaction_time_secs`: assumed reaction horizon Δt in seconds
/// - `skew_badness`: ∈ [0, 1e7]
/// - `bound_proximity`: ∈ [0, 1e7]
/// - `a, c, d`: dimensionless coefficients (1e7 precision)
/// - `bound_pow`: exponent for bound term (e.g. 3–5)
///
/// ## Formula
/// `fee = base_fee + a*m + c*s^2 + d*b^p` (each scaled by 1e7),
/// where:
/// - `m = sqrt(rv_per_sec * reaction_time_secs)` (risk over Δt)
/// - `s = skew_badness / 1e7`
/// - `b = bound_proximity / 1e7`
///
/// Returned fee is clamped to `[0, PRICE_PRECISION]`.
///
/// ## Panics
/// Does not panic for non-negative inputs, but if callers pass values outside expected
/// ranges, the result may saturate/clamp more often.
pub fn fee(
    e: &Env,
    base_fee: u128,
    rv_per_sec: u128,
    reaction_time_secs: u128,
    skew_badness: u128,
    bound_proximity: u128,
    a: u128,
    c: u128,
    d: u128,
    bound_pow: u32,
) -> u128 {
    // ----------------------------
    // 1) Expected move term: m = sqrt(rv_per_sec * Δt)
    // ----------------------------
    //
    // rv_per_sec is "annualized units per second" (1e7 precision).
    // Multiply by seconds to get "annualized units" over the horizon (still 1e7 precision).
    let rv_dt = rv_per_sec.saturating_mul(reaction_time_secs);

    // m should be in 1e7 precision, so take sqrt(rv_dt * 1e7)
    let m = isqrt_u128_strict(e, rv_dt.saturating_mul(ONE_U128));

    // move_term = a * m / 1e7
    let move_term = a.saturating_mul(m).saturating_div(ONE_U128);

    // ----------------------------
    // 2) Skew penalty: c * s^2
    // ----------------------------
    let s = skew_badness.min(ONE_U128);
    let s2 = s.saturating_mul(s).saturating_div(ONE_U128); // still 1e7
    let skew_term = c.saturating_mul(s2).saturating_div(ONE_U128);

    // ----------------------------
    // 3) Bound toxicity: d * b^p
    // ----------------------------
    let b = bound_proximity.min(ONE_U128);
    let b_pow = pow_fixed_1e7(b, bound_pow);
    let bound_term = d.saturating_mul(b_pow).saturating_div(ONE_U128);

    // ----------------------------
    // Final fee (non-negative), clamped
    // ----------------------------
    let raw = base_fee
        .saturating_add(move_term)
        .saturating_add(skew_term)
        .saturating_add(bound_term);

    // Fee must be in [0, 1] ideally; clamp defensively.
    clamp_u128(raw, 0, PRICE_PRECISION)
}

/// Integer square root for `u128`.
///
/// Returns `floor(sqrt(x))`.
///
/// Deterministic and safe for Soroban (no floats).
/// Panics if internal additions overflow (only possible at extreme inputs).
pub fn isqrt_u128_strict(e: &Env, x: u128) -> u128 {
    if x == 0 {
        return 0;
    }

    let mut z = x.safe_add(e, 1).safe_div(e, 2);
    let mut y = x;

    while z < y {
        y = z;
        let t = x.safe_div(e, z);
        z = t.safe_add(e, z).safe_div(e, 2);
    }

    y
}

/// Computes `base^exp` where `base` is in 1e7 fixed-point and the result is also 1e7 fixed-point.
/// Uses exponentiation-by-squaring: O(log exp).
///
/// Semantics:
/// - pow_fixed(base, 0) = 1.0
/// - base is clamped to [0, 1.0] if you pass it through `.min(ONE_U128)` beforehand.
///
/// Safe: saturating arithmetic, deterministic, no floats.
pub fn pow_fixed_1e7(mut base: u128, mut exp: u32) -> u128 {
    // result = 1.0
    let mut result: u128 = ONE_U128;

    while exp > 0 {
        if (exp & 1) == 1 {
            // result *= base
            result = result.saturating_mul(base).saturating_div(ONE_U128);
        }
        exp >>= 1;
        if exp > 0 {
            // base *= base (only if we still need it)
            base = base.saturating_mul(base).saturating_div(ONE_U128);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    fn env() -> Env {
        Env::default()
    }

    #[test]
    fn test_pow_fixed_1e7() {
        assert_eq!(pow_fixed_1e7(10_000_000, 0), 10_000_000); // 1^0 = 1
        assert_eq!(pow_fixed_1e7(10_000_000, 5), 10_000_000); // 1^p = 1

        assert_eq!(pow_fixed_1e7(0, 0), 10_000_000); // 0^0 -> 1 by convention here
        assert_eq!(pow_fixed_1e7(0, 5), 0); // 0^p = 0

        // 0.5^2 = 0.25
        let half = 5_000_000u128;
        assert_eq!(pow_fixed_1e7(half, 2), 2_500_000);

        // 0.9^3 ~= 0.729
        let nine_tenths = 9_000_000u128;
        assert_eq!(pow_fixed_1e7(nine_tenths, 3), 7_290_000);
    }

    // ----------------------------
    // apply_fee_to_input() tests
    // ----------------------------

    #[test]
    fn apply_fee_to_input_zero_fee_no_change() {
        let e = Env::default();
        let amt = 1_234_567u128;
        assert_eq!(apply_fee_to_input(&e, amt, 0), amt);
    }

    #[test]
    #[should_panic]
    fn apply_fee_to_input_over_max_panics() {
        let e = Env::default();
        let amt = 1_234_567u128;
        assert_eq!(apply_fee_to_input(&e, amt, PRICE_PRECISION + 1), 0);
    }

    #[test]
    fn apply_fee_to_input_matches_expected_formula() {
        let e = Env::default();
        // fee = 1% => net = amt * 0.99
        let amt = 1_000_000u128;
        let fee = PRICE_PRECISION / 100; // 1%
        let expected = (amt * (PRICE_PRECISION - fee)) / PRICE_PRECISION;
        assert_eq!(apply_fee_to_input(&e, amt, fee), expected);
        assert!(apply_fee_to_input(&e, amt, fee) < amt);
    }

    // --- apply_fee_to_input -------------------------------------------------

    #[test]
    fn test_apply_fee_to_input_zero_fee() {
        let e = env();
        let out = apply_fee_to_input(&e, 1_000_000, 0);
        assert_eq!(out, 1_000_000);
    }

    #[test]
    fn test_apply_fee_to_input_full_fee() {
        let e = env();
        let out = apply_fee_to_input(&e, 1_000_000, PRICE_PRECISION);
        assert_eq!(out, 0);
    }

    #[test]
    fn test_apply_fee_to_input_half_fee() {
        let e = env();
        // 50% fee => 500000 out
        let out = apply_fee_to_input(&e, 1_000_000, PRICE_PRECISION / 2);
        assert_eq!(out, 500_000);
    }

    #[test]
    #[should_panic]
    fn test_apply_fee_to_input_amount_zero_panics() {
        let e = env();
        let _ = apply_fee_to_input(&e, 0, 0);
    }

    #[test]
    #[should_panic]
    fn test_apply_fee_to_input_fee_too_large_panics() {
        let e = env();
        let _ = apply_fee_to_input(&e, 1, PRICE_PRECISION + 1);
    }

    // --- skew_badness -------------------------------------------------------

    #[test]
    fn test_skew_badness_nav_zero_is_zero() {
        let e = env();
        assert_eq!(skew_badness(&e, 10, 0, 0), 0);
    }

    #[test]
    fn test_skew_badness_balanced_is_zero() {
        let e = env();
        let nav = 1_000_000;
        let v = 400_000;
        assert_eq!(skew_badness(&e, v, v, nav), 0);
    }

    #[test]
    fn test_skew_badness_simple_ratio() {
        let e = env();
        // v_long=700, v_short=300, nav=1000 => diff=400 => 0.4 => 4_000_000
        let s = skew_badness(&e, 700, 300, 1000);
        assert_eq!(s, 4_000_000);
    }

    #[test]
    fn test_skew_badness_clamps_to_one() {
        let e = env();
        // diff > nav should clamp to 1.0
        let s = skew_badness(&e, 2_000, 0, 1_000);
        assert_eq!(s, ONE_U128);
    }

    // --- bound_proximity ----------------------------------------------------

    #[test]
    fn test_bound_proximity_toxic_short_is_t() {
        let t = 7_500_000;
        assert_eq!(bound_proximity(t, true), t);
    }

    #[test]
    fn test_bound_proximity_toxic_long_is_one_minus_t() {
        let t = 7_500_000;
        assert_eq!(bound_proximity(t, false), ONE_U128 - t);
    }

    #[test]
    fn test_bound_proximity_clamps_input() {
        // if t > 1.0, clamp to 1.0
        assert_eq!(bound_proximity(ONE_U128 + 123, true), ONE_U128);
        assert_eq!(bound_proximity(ONE_U128 + 123, false), 0);
    }

    // --- isqrt_u128 ---------------------------------------------------------

    #[test]
    fn test_isqrt_u128_basic() {
        let e = env();
        assert_eq!(isqrt_u128_strict(&e, 0), 0);
        assert_eq!(isqrt_u128_strict(&e, 1), 1);
        assert_eq!(isqrt_u128_strict(&e, 4), 2);
        assert_eq!(isqrt_u128_strict(&e, 9), 3);
        assert_eq!(isqrt_u128_strict(&e, 15), 3);
        assert_eq!(isqrt_u128_strict(&e, 16), 4);
    }

    #[test]
    fn test_isqrt_u128_large_square() {
        let e = env();
        let x = 123_456u128;
        let sq = x * x;
        assert_eq!(isqrt_u128_strict(&e, sq), x);
    }

    // --- fee ----------------------------------------------------------------

    #[test]
    fn test_fee_non_negative_and_at_least_base() {
        let e = env();
        let base = 30_000;
        let out = fee(
            &e, base, 0, // rv_per_sec
            0, // reaction_time
            0, // skew
            0, // bound
            0, 0, 0, 4,
        );
        assert_eq!(out, base);
    }

    #[test]
    fn test_fee_increases_with_skew() {
        let e = env();
        let base = 0;
        let f0 = fee(&e, base, 0, 0, 0, 0, 0, 2_000_0000, 0, 2);
        let f1 = fee(&e, base, 0, 0, 5_000_000, 0, 0, 2_000_0000, 0, 2);
        assert!(f1 > f0);
    }

    #[test]
    fn test_fee_increases_with_bound_proximity() {
        let e = env();
        let base = 0;
        let f0 = fee(&e, base, 0, 0, 0, 0, 0, 0, 10_000_0000, 4);
        let f1 = fee(&e, base, 0, 0, 0, 9_000_000, 0, 0, 10_000_0000, 4);
        assert!(f1 > f0);
    }

    #[test]
    fn test_fee_clamps_to_price_precision() {
        let e = env();
        // Make it huge
        let f = fee(
            &e,
            PRICE_PRECISION,
            PRICE_PRECISION,
            1_000_000,
            ONE_U128,
            ONE_U128,
            PRICE_PRECISION,
            PRICE_PRECISION,
            PRICE_PRECISION,
            6,
        );
        assert_eq!(f, PRICE_PRECISION);
    }

    // --- calculate_fee_from_values -----------------------------------------

    #[test]
    fn test_calculate_fee_from_values_runs_and_is_clamped() {
        let e = env();

        // Minimal dummy config; coefficients intentionally large to exercise clamping.
        let cfg = TreasuryFeeConfig {
            taker_base_fee: 30_000,
            maker_base_fee: 30_000,
            implied_volatility: 5_000_000, // 0.5 in 1e7 "variance-like" terms (example)
            reaction_time_secs: 600,       // 10 minutes
            coefficient_a: 1_000_0000,
            coefficient_c: 2_000_0000,
            coefficient_d: 10_000_0000,
            bound_power: 4,
            // If your real struct has more fields, fill them here or use `..Default::default()`.
        };

        let v_long = 600;
        let v_short = 200;
        let nav = 1000;

        let fee_out = calculate_fee_from_values(
            &e,
            Side::Short, // toxic_is_short => bound_proximity = t
            &cfg,
            v_long,
            v_short,
            nav,
            9_500_000, // near upper
        );

        assert!(fee_out <= PRICE_PRECISION);
        // Should be at least base fee
        assert!(fee_out >= cfg.taker_base_fee);
    }
}
