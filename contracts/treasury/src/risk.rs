use soroban_sdk::{log, panic_with_error, Address, Env};
use types::pair::{PairAmountsWithUSDC, Side};
use utils::constant::{ONE_U128, PRICE_PRECISION};
use utils::math::safe_math::{PrecisionMath, SafeMath};

use crate::errors::TreasuryError;
use crate::storage::TreasuryRiskParameters;

/* ========================================================================
 * USDC floor
 * ======================================================================== */

/// Computes the minimum USDC token balance ("USDC floor") required to back
/// a configured fraction of total treasury NAV.
///
/// All monetary values are expressed in `PRICE_PRECISION` fixed-point (typically 1e7),
/// meaning `PRICE_PRECISION == 1.0`.
///
/// This is used to prevent the Treasury from paying out so much USDC that it becomes
/// unable to meet withdrawals, settle, or otherwise remain liquid.
///
/// ## Definitions
/// - `nav_total`: Total NAV of the Treasury in **quote units**, scaled by `PRICE_PRECISION`.
/// - `floor_fraction`: Fraction of NAV that must remain as USDC, in `[0, PRICE_PRECISION]`.
/// - `usdc_price`: Quote price of 1 USDC token, scaled by `PRICE_PRECISION`
///   (normally `PRICE_PRECISION`, but may deviate if you model depegs).
///
/// ## Formula
/// ```text
/// floor_nav   = nav_total * floor_fraction
/// usdc_floor  = floor_nav / usdc_price
/// ```
///
/// Because `nav_total` and `usdc_price` are both `PRICE_PRECISION`-scaled quote units,
/// the division yields a **token amount** (in USDC smallest units).
///
/// ## Panics
/// - [`TreasuryError::InvalidInput`] if `floor_fraction > PRICE_PRECISION`
/// - [`TreasuryError::InvalidInput`] if `usdc_price == 0`
pub fn get_usdc_floor(e: &Env, nav_total: u128, floor_fraction: u128, usdc_price: u128) -> u128 {
    if floor_fraction > PRICE_PRECISION || usdc_price == 0 {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    // floor_nav = NAV * floor_fraction (quote units)
    let floor_nav = nav_total.safe_fixed_mul_floor(e, floor_fraction, PRICE_PRECISION);

    // Convert quote value into USDC token amount (token units)
    floor_nav.safe_fixed_div_floor(e, usdc_price, PRICE_PRECISION)
}

/// Internal helper: validate a USDC floor using an explicitly provided fraction.
///
/// This lets tests exercise the full floor logic without depending on storage access.
/// The public [`validate_usdc_floor`] reads the fraction from storage then calls this.
///
/// ## Panics
/// - [`TreasuryError::InvalidInput`] if:
///   - `usdc_price == 0`
///   - `usdc_out > usdc_balance`
///   - `floor_fraction > PRICE_PRECISION`
/// - [`TreasuryError::CannotPassFloor`] if `usdc_balance - usdc_out < usdc_floor`
fn validate_usdc_floor_with_fraction(
    e: &Env,
    floor_fraction: u128,
    usdc_balance: u128,
    usdc_out: u128,
    nav_total: u128,
    usdc_price: u128,
) {
    if usdc_price == 0 || floor_fraction > PRICE_PRECISION || usdc_out > usdc_balance {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    let usdc_floor = get_usdc_floor(e, nav_total, floor_fraction, usdc_price);
    let remaining = usdc_balance.safe_sub(e, usdc_out);

    if remaining < usdc_floor {
        panic_with_error!(e, TreasuryError::CannotPassFloor);
    }
}

/// Validates that a USDC-out trade will not reduce the Treasury’s USDC balance below the
/// configured floor fraction of NAV.
///
/// The floor is stored in contract storage as a fraction of NAV (in `PRICE_PRECISION` units).
/// This function:
/// 1) reads `floor_fraction` from storage
/// 2) converts that to a USDC token amount via [`get_usdc_floor`]
/// 3) rejects a trade if the post-trade USDC balance would be below the floor.
///
/// ## Panics
/// - [`TreasuryError::InvalidInput`] if:
///   - `usdc_price == 0`
///   - `usdc_out > usdc_balance`
///   - stored `floor_fraction > PRICE_PRECISION`
/// - [`TreasuryError::CannotPassFloor`] if the resulting USDC balance would be below the floor
pub fn validate_usdc_floor(
    e: &Env,
    usdc_balance: u128,
    usdc_out: u128,
    nav_total: u128,
    usdc_price: u128,
) {
    let floor_fraction = crate::storage::get_usdc_floor_fraction(e);
    validate_usdc_floor_with_fraction(
        e,
        floor_fraction,
        usdc_balance,
        usdc_out,
        nav_total,
        usdc_price,
    );
}

/* ========================================================================
 * Toxic-side guard
 * ======================================================================== */

/// Internal helper: block toxic trades using explicitly provided risk parameters.
///
/// The public [`block_toxic_trades`] reads parameters from storage then calls this.
/// This helper exists to make the logic unit-testable without storage wiring.
///
/// ## Toxicity rules (normalized price space)
/// - Near the upper bound: **SHORT** is toxic when `price >= toxic_threshold`
/// - Near the lower bound: **LONG** is toxic when `price <= 1 - toxic_threshold`
///
/// `price` and `toxic_threshold` are expected to be scaled to `ONE_U128`
/// (typically `PRICE_PRECISION` / 1.0).
///
/// ## Panics
/// - [`TreasuryError::ToxicSideNotAccepted`] when the trade is into the toxic side.
fn block_toxic_trades_with_params(
    e: &Env,
    side: Side,
    price: u128,
    risk_params: &TreasuryRiskParameters,
) {
    // Near upper bound: short is toxic
    if side == Side::Short && price >= risk_params.toxic_threshold {
        panic_with_error!(e, TreasuryError::ToxicSideNotAccepted);
    }

    // Near lower bound: long is toxic
    // mirrored threshold: lower = 1 - toxic_threshold
    let lower = ONE_U128.saturating_sub(risk_params.toxic_threshold);
    if side == Side::Long && price <= lower {
        panic_with_error!(e, TreasuryError::ToxicSideNotAccepted);
    }
}

/// Blocks trades on a side that has become **toxic** due to proximity to settlement bounds.
///
/// This is a *terminal-value* guard: when price is near an extreme, one side of the
/// long–short pair is approaching near-zero settlement value. Allowing the Treasury to
/// accumulate that side creates asymmetric loss (the Treasury buys something that is
/// close to worthless at settlement).
///
/// - Near the **upper bound**: `SHORT` becomes toxic (settles toward 0).
/// - Near the **lower bound**: `LONG` becomes toxic (settles toward 0).
///
/// `risk_params.toxic_threshold` defines how close to the bound we reject trades, in the
/// same normalized price space as `price`.
///
/// ## Panics
/// - [`TreasuryError::ToxicSideNotAccepted`] if the side is toxic at the given `price`.
pub fn block_toxic_trades(e: &Env, pair: &Address, side: Side, price: u128) {
    let risk_params = crate::storage::get_risk_parameters(e, pair);
    block_toxic_trades_with_params(e, side, price, &risk_params);
}

/* ========================================================================
 * Net inventory imbalance (token-count based)
 * ======================================================================== */

/// Computes the Treasury’s signed net inventory imbalance in **token units**:
///
/// ```text
/// net = long_tokens - short_tokens
/// ```
///
/// - `net > 0` means the Treasury is net LONG (holds more long tokens than short)
/// - `net < 0` means the Treasury is net SHORT
///
/// This is intentionally **token-count based** (not value-based) so it is stable under
/// oracle price moves and only changes due to trade flow and inventory operations.
pub fn net_imbalance_tokens(e: &Env, balances: &PairAmountsWithUSDC) -> i128 {
    let long = balances.long as i128;
    let short = balances.short as i128;
    long.safe_sub(e, short)
}

/// Absolute value of a signed imbalance (token units).
pub fn abs_imbalance_tokens(net: i128) -> u128 {
    if net >= 0 {
        net as u128
    } else {
        -net as u128
    }
}

/// Validates that a post-trade state does not exceed the configured hard net-imbalance limit.
///
/// The hard limit is enforced in **token units**:
/// ```text
/// abs(long_tokens - short_tokens) <= max_net_imbalance_tokens
/// ```
///
/// If `max_net_imbalance_tokens == 0`, the hard limit is treated as disabled.
///
/// ## Returns
/// `(abs_before, abs_after, abs_delta, increases)` where:
/// - `abs_before`  = `abs(net_before)`
/// - `abs_after`   = `abs(net_after)`
/// - `abs_delta`   = `abs(abs_after - abs_before)`
/// - `increases`   = `abs_after > abs_before`
///
/// The `(abs_delta, increases)` pair is convenient for fee logic that adds a surcharge when
/// a trade **worsens** imbalance, and gives a rebate when it **improves** imbalance.
///
/// ## Panics
/// - [`TreasuryError::InvalidBalance`] if the hard limit is enabled and would be exceeded.
pub fn validate_net_imbalance_after(
    e: &Env,
    risk_params: &TreasuryRiskParameters,
    balances_before: &PairAmountsWithUSDC,
    balances_after: &PairAmountsWithUSDC,
) -> (u128, u128, u128, bool) {
    let net_before = net_imbalance_tokens(e, balances_before);
    let net_after = net_imbalance_tokens(e, balances_after);

    let abs_before = abs_imbalance_tokens(net_before);
    let abs_after = abs_imbalance_tokens(net_after);

    if risk_params.max_net_imbalance_tokens != 0 && abs_after > risk_params.max_net_imbalance_tokens
    {
        panic_with_error!(e, TreasuryError::InvalidBalance);
    }

    let increases = abs_after > abs_before;
    let abs_delta = if increases {
        abs_after.safe_sub(e, abs_before)
    } else {
        abs_before.safe_sub(e, abs_after)
    };

    (abs_before, abs_after, abs_delta, increases)
}
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;
    use utils::constant::{ONE_U128, PRICE_PRECISION};

    fn e() -> Env {
        Env::default()
    }

    fn risk_params(max_net: u128, toxic_threshold: u128) -> TreasuryRiskParameters {
        TreasuryRiskParameters {
            toxic_threshold,
            max_net_imbalance_tokens: max_net,
            imbalance_fee_max: 0, // not used in risk.rs
        }
    }

    fn bals(long: u128, short: u128, usdc: u128) -> PairAmountsWithUSDC {
        PairAmountsWithUSDC { long, short, usdc }
    }

    // ============================================================
    // get_usdc_floor
    // ============================================================

    #[test]
    fn usdc_floor_zero_nav_or_zero_fraction_is_zero() {
        let env = e();
        assert_eq!(get_usdc_floor(&env, 0, 5_000_000, PRICE_PRECISION), 0);
        assert_eq!(
            get_usdc_floor(&env, 1_000_0000 * 1000, 0, PRICE_PRECISION),
            0
        );
    }

    #[test]
    fn usdc_floor_matches_expected_at_par() {
        let env = e();
        let nav_total = 1_000_0000u128 * 1_000; // 1000.0
        let floor_fraction = 2_500_000; // 25%

        let floor = get_usdc_floor(&env, nav_total, floor_fraction, PRICE_PRECISION);
        assert_eq!(floor, 1_000_0000u128 * 250);
    }

    #[test]
    fn usdc_floor_monotonic_in_usdc_price() {
        let env = e();
        let nav_total = 1_000_0000u128 * 1_000;
        let floor_fraction = 5_000_000; // 50%

        let floor_par = get_usdc_floor(&env, nav_total, floor_fraction, PRICE_PRECISION);
        let floor_depeg = get_usdc_floor(&env, nav_total, floor_fraction, 9_000_000);
        let floor_premium = get_usdc_floor(&env, nav_total, floor_fraction, 11_000_000);

        assert!(floor_depeg > floor_par);
        assert!(floor_premium < floor_par);
    }

    #[test]
    #[should_panic]
    fn usdc_floor_panics_on_fraction_gt_one() {
        let env = e();
        let _ = get_usdc_floor(&env, 1_000_0000, PRICE_PRECISION + 1, PRICE_PRECISION);
    }

    #[test]
    #[should_panic]
    fn usdc_floor_panics_on_zero_price() {
        let env = e();
        let _ = get_usdc_floor(&env, 1_000_0000, 1, 0);
    }

    // ============================================================
    // validate_usdc_floor_with_fraction
    // ============================================================

    #[test]
    fn validate_usdc_floor_allows_remaining_equal_to_floor() {
        let env = e();
        let nav_total = 1_000_0000u128 * 1_000;
        let floor_fraction = 5_000_000;
        let usdc_price = PRICE_PRECISION;

        let floor = get_usdc_floor(&env, nav_total, floor_fraction, usdc_price);
        let usdc_balance = floor + 100;
        let usdc_out = 100;

        validate_usdc_floor_with_fraction(
            &env,
            floor_fraction,
            usdc_balance,
            usdc_out,
            nav_total,
            usdc_price,
        );
    }

    #[test]
    #[should_panic]
    fn validate_usdc_floor_panics_when_remaining_below_floor() {
        let env = e();
        let nav_total = 1_000_0000u128 * 1_000;
        let floor_fraction = 5_000_000;
        let usdc_price = PRICE_PRECISION;

        let floor = get_usdc_floor(&env, nav_total, floor_fraction, usdc_price);
        let usdc_balance = floor + 100;
        let usdc_out = 101;

        validate_usdc_floor_with_fraction(
            &env,
            floor_fraction,
            usdc_balance,
            usdc_out,
            nav_total,
            usdc_price,
        );
    }

    #[test]
    #[should_panic]
    fn validate_usdc_floor_panics_when_usdc_out_exceeds_balance() {
        let env = e();
        validate_usdc_floor_with_fraction(&env, 1_000_000, 100, 101, 1_000_0000, PRICE_PRECISION);
    }

    #[test]
    #[should_panic]
    fn validate_usdc_floor_panics_when_fraction_invalid() {
        let env = e();
        validate_usdc_floor_with_fraction(
            &env,
            PRICE_PRECISION + 1,
            100,
            1,
            1_000_0000,
            PRICE_PRECISION,
        );
    }

    #[test]
    #[should_panic]
    fn validate_usdc_floor_panics_when_usdc_price_zero() {
        let env = e();
        validate_usdc_floor_with_fraction(&env, 1_000_000, 100, 1, 1_000_0000, 0);
    }

    // ============================================================
    // block_toxic_trades_with_params
    // ============================================================

    #[test]
    fn toxic_guard_allows_short_below_threshold() {
        let env = e();
        let rp = risk_params(0, 9_000_000);

        block_toxic_trades_with_params(&env, Side::Short, 8_999_999, &rp);
    }

    #[test]
    #[should_panic]
    fn toxic_guard_blocks_short_at_threshold() {
        let env = e();
        let rp = risk_params(0, 9_000_000);

        block_toxic_trades_with_params(&env, Side::Short, 9_000_000, &rp);
    }

    #[test]
    fn toxic_guard_allows_long_above_lower_threshold() {
        let env = e();
        let rp = risk_params(0, 9_000_000);
        let lower = ONE_U128.saturating_sub(rp.toxic_threshold);

        block_toxic_trades_with_params(&env, Side::Long, lower + 1, &rp);
    }

    #[test]
    #[should_panic]
    fn toxic_guard_blocks_long_at_lower_threshold() {
        let env = e();
        let rp = risk_params(0, 9_000_000);
        let lower = ONE_U128.saturating_sub(rp.toxic_threshold);

        block_toxic_trades_with_params(&env, Side::Long, lower, &rp);
    }

    #[test]
    #[should_panic]
    fn toxic_guard_blocks_long_below_lower_threshold() {
        let env = e();
        let rp = risk_params(0, 9_000_000);
        let lower = ONE_U128.saturating_sub(rp.toxic_threshold);

        block_toxic_trades_with_params(&env, Side::Long, lower - 1, &rp);
    }

    #[test]
    #[should_panic]
    fn toxic_threshold_zero_blocks_short_always() {
        let env = e();
        let rp = risk_params(0, 0);

        block_toxic_trades_with_params(&env, Side::Short, 0, &rp);
    }

    #[test]
    fn toxic_threshold_one_allows_most_prices() {
        let env = e();
        let rp = risk_params(0, ONE_U128);

        block_toxic_trades_with_params(&env, Side::Short, ONE_U128 - 1, &rp);
        block_toxic_trades_with_params(&env, Side::Long, 1, &rp);
    }

    #[test]
    #[should_panic]
    fn toxic_threshold_one_blocks_short_at_upper_bound() {
        let env = e();
        let rp = risk_params(0, ONE_U128);

        block_toxic_trades_with_params(&env, Side::Short, ONE_U128, &rp);
    }

    #[test]
    #[should_panic]
    fn toxic_threshold_one_blocks_long_at_lower_bound() {
        let env = e();
        let rp = risk_params(0, ONE_U128);

        block_toxic_trades_with_params(&env, Side::Long, 0, &rp);
    }

    // ============================================================
    // net imbalance
    // ============================================================

    #[test]
    fn net_imbalance_sign_and_abs() {
        let env = e();

        let b1 = bals(10, 7, 0);
        let net1 = net_imbalance_tokens(&env, &b1);
        assert_eq!(net1, 3);
        assert_eq!(abs_imbalance_tokens(net1), 3);

        let b2 = bals(5, 9, 0);
        let net2 = net_imbalance_tokens(&env, &b2);
        assert_eq!(net2, -4);
        assert_eq!(abs_imbalance_tokens(net2), 4);
    }

    #[test]
    fn validate_net_imbalance_disabled_limit_returns_delta() {
        let env = e();
        let rp = risk_params(0, 0);

        let before = bals(100, 100, 0);
        let after = bals(90, 100, 0); // net = -10

        let (abs_b, abs_a, delta, inc) = validate_net_imbalance_after(&env, &rp, &before, &after);

        assert_eq!(abs_b, 0);
        assert_eq!(abs_a, 10);
        assert_eq!(delta, 10);
        assert!(inc);
    }

    #[test]
    fn validate_net_imbalance_reports_decrease_correctly() {
        let env = e();
        let rp = risk_params(1000, 0);

        let before = bals(150, 100, 0); // abs = 50
        let after = bals(120, 100, 0); // abs = 20

        let (abs_b, abs_a, delta, inc) = validate_net_imbalance_after(&env, &rp, &before, &after);

        assert_eq!(abs_b, 50);
        assert_eq!(abs_a, 20);
        assert_eq!(delta, 30);
        assert!(!inc);
    }

    #[test]
    #[should_panic]
    fn validate_net_imbalance_panics_when_exceeds_hard_limit() {
        let env = e();
        let rp = risk_params(10, 0);

        let before = bals(100, 100, 0);
        let after = bals(100, 120, 0); // abs = 20 > 10

        validate_net_imbalance_after(&env, &rp, &before, &after);
    }
}
