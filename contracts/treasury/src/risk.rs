use soroban_sdk::{panic_with_error, Address, Env};
use types::pair::Side;
use utils::constant::{ONE_U128, PRICE_PRECISION};
use utils::math::safe_math::PrecisionMath;

use crate::errors::TreasuryError;

/// Computes the minimum USDC token balance ("USDC floor") required to back
/// `floor_fraction` of total NAV.
///
/// All values use `PRICE_PRECISION` (1e7) fixed-point.
///
/// # Arguments
/// - `nav_total`: total treasury NAV (quote units, 1e7 precision)
/// - `floor_fraction`: fraction of NAV to keep as USDC (0..=1e7)
/// - `usdc_price`: quote price of 1 USDC (normally == 1e7)
///
/// # Returns
/// Minimum USDC token amount that must remain in the treasury.
///
/// # Panics
/// - If `floor_fraction > PRICE_PRECISION`
/// - If `usdc_price == 0`
pub fn get_usdc_floor(e: &Env, nav_total: u128, floor_fraction: u128, usdc_price: u128) -> u128 {
    // Sanity checks
    if floor_fraction > PRICE_PRECISION {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }
    if usdc_price == 0 {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    // floor_nav = NAV * floor_fraction
    // still in quote units (1e7 precision)
    let floor_nav = nav_total.safe_fixed_mul_floor(e, floor_fraction, PRICE_PRECISION);

    // Convert NAV value into token amount:
    //
    // floor_amount = floor_nav / usdc_price
    //
    // Both are 1e7 precision quote units, so divide directly.
    let floor_amount = floor_nav.safe_fixed_div_floor(e, usdc_price, PRICE_PRECISION);

    floor_amount
}

/// Validates that a USDC-out trade will not reduce the treasury USDC balance below the configured floor.
///
/// The floor is expressed as a fraction of total NAV:
/// `floor_nav = floor_fraction * nav_total`
/// and converted into a USDC token amount:
/// `usdc_floor_amount = floor_nav / usdc_price`
///
/// # Arguments
/// - `usdc_balance`: current treasury USDC token balance
/// - `usdc_out`: USDC tokens that would be paid out by the trade
/// - `nav_total`: current total treasury NAV (quote units, PRICE_PRECISION-scaled)
/// - `usdc_price`: quote price per 1 USDC token (PRICE_PRECISION-scaled, usually == PRICE_PRECISION)
///
/// # Panics
/// - `TreasuryError::InvalidInput` if `usdc_price == 0` or `usdc_out > usdc_balance`
/// - `TreasuryError::CannotPassFloor` if the resulting USDC balance would be below the floor
pub fn validate_usdc_floor(
    e: &Env,
    usdc_balance: u128,
    usdc_out: u128,
    nav_total: u128,
    usdc_price: u128,
) {
    if usdc_price == 0 {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    // Disallow underflow; this should be a hard revert (trying to pay more USDC than exists).
    if usdc_out > usdc_balance {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    let floor_fraction = crate::storage::get_usdc_floor_fraction(e);
    if floor_fraction > PRICE_PRECISION {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    let usdc_floor = get_usdc_floor(e, nav_total, floor_fraction, usdc_price);

    // Remaining USDC after payout
    let remaining = usdc_balance - usdc_out;

    // Panic if the trade would violate the USDC floor
    if remaining < usdc_floor {
        panic_with_error!(e, TreasuryError::CannotPassFloor);
    }
}

pub fn block_toxic_trades(e: &Env, pair: &Address, side: Side, price: u128) {
    let risk_params = crate::storage::get_risk_parameters(e, pair);

    // Near upper bound: short is toxic
    if side == Side::Short && price >= risk_params.toxic_threshold {
        panic_with_error!(e, TreasuryError::ToxicSideNotAccepted);
    }

    // Near lower bound: long is toxic
    if side == Side::Long && price <= ONE_U128.saturating_sub(risk_params.toxic_threshold) {
        panic_with_error!(e, TreasuryError::ToxicSideNotAccepted);
    }
}

#[cfg(test)]
mod risk_validation_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::{storage::TreasuryRiskParameters, Treasury};
    use types::pair::Side;
    use utils::constant::{ONE_U128, PRICE_PRECISION};

    fn env() -> Env {
        Env::default()
    }

    fn setup_test_contract(e: &Env) -> (Address, Address) {
        let admin = Address::generate(e);
        let treasury_contract = e.register(Treasury, (&admin,));

        (treasury_contract, admin)
    }

    // ------------------------------------------------------------
    // get_usdc_floor
    // ------------------------------------------------------------

    #[test]
    fn get_usdc_floor_zero_nav_is_zero() {
        let e = env();
        let out = get_usdc_floor(&e, 0, 5_000_000, PRICE_PRECISION);
        assert_eq!(out, 0);
    }

    #[test]
    fn get_usdc_floor_zero_fraction_is_zero() {
        let e = env();
        let nav = 1_000_0000u128 * 1_000; // 1000.0
        let out = get_usdc_floor(&e, nav, 0, PRICE_PRECISION);
        assert_eq!(out, 0);
    }

    #[test]
    fn get_usdc_floor_one_to_one_when_usdc_price_is_one() {
        let e = env();

        // nav_total is in quote units (1e7). Use exact numbers for clean asserts.
        let nav_total = 1_000_0000u128 * 1_000; // 1000.0
        let floor_fraction = 2_500_000; // 0.25
        let usdc_price = PRICE_PRECISION; // 1.0

        // floor_nav = 1000 * 0.25 = 250
        let floor = get_usdc_floor(&e, nav_total, floor_fraction, usdc_price);
        assert_eq!(floor, 1_000_0000u128 * 250);
    }

    #[test]
    fn get_usdc_floor_increases_when_usdc_depegs_down() {
        let e = env();

        let nav_total = 1_000_0000u128 * 1_000; // 1000
        let floor_fraction = 5_000_000; // 0.5

        let usdc_price_par = PRICE_PRECISION; // 1.0
        let usdc_price_depeg = 9_000_000; // 0.9

        let floor_par = get_usdc_floor(&e, nav_total, floor_fraction, usdc_price_par);
        let floor_depeg = get_usdc_floor(&e, nav_total, floor_fraction, usdc_price_depeg);

        // if price is lower, you need more tokens to meet same value floor
        assert!(floor_depeg > floor_par);
    }

    #[test]
    fn get_usdc_floor_decreases_when_usdc_premium() {
        let e = env();

        let nav_total = 1_000_0000u128 * 1_000; // 1000
        let floor_fraction = 5_000_000; // 0.5

        let usdc_price_par = PRICE_PRECISION; // 1.0
        let usdc_price_premium = 11_000_000; // 1.1

        let floor_par = get_usdc_floor(&e, nav_total, floor_fraction, usdc_price_par);
        let floor_premium = get_usdc_floor(&e, nav_total, floor_fraction, usdc_price_premium);

        assert!(floor_premium < floor_par);
    }

    #[test]
    #[should_panic]
    fn get_usdc_floor_panics_if_fraction_gt_one() {
        let e = env();
        let _ = get_usdc_floor(&e, 1_000_0000, PRICE_PRECISION + 1, PRICE_PRECISION);
    }

    #[test]
    #[should_panic]
    fn get_usdc_floor_panics_if_usdc_price_zero() {
        let e = env();
        let _ = get_usdc_floor(&e, 1_000_0000, 1, 0);
    }

    // ------------------------------------------------------------
    // validate_usdc_floor
    //
    // NOTE: validate_usdc_floor reads `crate::storage::get_usdc_floor_fraction(e)`.
    // These tests assume you have a corresponding setter available in test builds.
    // If your storage module uses a DataKey, replace the `set_...` helper below
    // with the appropriate direct write.
    // ------------------------------------------------------------

    fn set_usdc_floor_fraction(e: &Env, treasury: &Address, frac: u128) {
        e.as_contract(treasury, || {
            crate::storage::set_usdc_floor_fraction(&e, &frac);
        });
    }

    #[test]
    fn validate_usdc_floor_allows_trade_when_remaining_equals_floor() {
        let e = env();
        let (treasury, _) = setup_test_contract(&e);

        set_usdc_floor_fraction(&e, &treasury, 5_000_000); // 0.5

        let nav_total = 1_000_0000u128 * 1_000; // 1000
        let usdc_price = PRICE_PRECISION; // 1.0

        // floor = 500
        let floor = get_usdc_floor(&e, nav_total, 5_000_000, usdc_price);
        assert_eq!(floor, 1_000_0000u128 * 500);

        // If remaining == floor, should pass
        let usdc_balance = 1_000_0000u128 * 800;
        let usdc_out = usdc_balance - floor;

        validate_usdc_floor(&e, usdc_balance, usdc_out, nav_total, usdc_price);
    }

    #[test]
    #[should_panic]
    fn validate_usdc_floor_panics_when_remaining_below_floor() {
        let e = env();
        let (treasury, _) = setup_test_contract(&e);

        set_usdc_floor_fraction(&e, &treasury, 5_000_000); // 0.5

        let nav_total = 1_000_0000u128 * 1_000; // 1000
        let usdc_price = PRICE_PRECISION; // 1.0
        let floor = get_usdc_floor(&e, nav_total, 5_000_000, usdc_price);

        let usdc_balance = 1_000_0000u128 * 800;
        // Make remaining < floor by 1
        let usdc_out = usdc_balance - floor + 1;

        let _ = validate_usdc_floor(&e, usdc_balance, usdc_out, nav_total, usdc_price);
    }

    #[test]
    #[should_panic]
    fn validate_usdc_floor_panics_if_usdc_out_gt_balance() {
        let e = env();
        let (treasury, _) = setup_test_contract(&e);

        set_usdc_floor_fraction(&e, &treasury, 1_000_000); // 0.1

        let _ = validate_usdc_floor(&e, 100, 101, 1_000_0000, PRICE_PRECISION);
    }

    #[test]
    #[should_panic]
    fn validate_usdc_floor_panics_if_usdc_price_zero() {
        let e = env();
        let (treasury, _) = setup_test_contract(&e);

        set_usdc_floor_fraction(&e, &treasury, 1_000_000); // 0.1

        let _ = validate_usdc_floor(&e, 100, 1, 1_000_0000, 0);
    }

    #[test]
    #[should_panic]
    fn validate_usdc_floor_panics_if_floor_fraction_invalid() {
        let e = env();
        let (treasury, _) = setup_test_contract(&e);

        set_usdc_floor_fraction(&e, &treasury, PRICE_PRECISION + 1);

        let _ = validate_usdc_floor(&e, 1_000_0000, 1, 1_000_0000, PRICE_PRECISION);
    }

    // ------------------------------------------------------------
    // block_toxic_trades
    //
    // NOTE: block_toxic_trades reads `crate::storage::get_risk_parameters(e, pair)`.
    // These tests assume you have a setter for risk parameters in test builds.
    // Also: your "near lower bound" logic currently looks wrong:
    //   `price <= ONE_U128.saturating_sub(price)`
    // It should likely be `price <= risk_params.lower_toxic_threshold`
    // or `price <= (ONE_U128 - risk_params.toxic_threshold)` depending on how you store thresholds.
    // The tests below validate the "upper bound short toxic" path as written,
    // and include an explicit test that will likely FAIL for the long side,
    // which is a useful red flag.
    // ------------------------------------------------------------

    fn set_risk_params_toxic_threshold(e: &Env, pair: &Address, toxic_threshold: u128) {
        // Replace with your real setter or a cfg(test) helper.
        e.as_contract(pair, || {
            crate::storage::set_risk_parameters(
                &e,
                pair,
                &(TreasuryRiskParameters { toxic_threshold }),
            );
        });
    }

    #[test]
    fn block_toxic_trades_allows_short_below_threshold() {
        let e = env();
        let pair = Address::generate(&e);

        set_risk_params_toxic_threshold(&e, &pair, 9_000_000);

        // price below threshold => ok
        block_toxic_trades(&e, &pair, Side::Short, 8_999_999);
    }

    #[test]
    #[should_panic]
    fn block_toxic_trades_panics_short_at_or_above_threshold() {
        let e = env();
        let pair = Address::generate(&e);

        set_risk_params_toxic_threshold(&e, &pair, 9_000_000);

        // price >= threshold => short side blocked
        block_toxic_trades(&e, &pair, Side::Short, 9_000_000);
    }

    #[test]
    fn block_toxic_trades_allows_long_above_lower_threshold() {
        let e = env();
        let pair = Address::generate(&e);

        // upper threshold = 0.90
        set_risk_params_toxic_threshold(&e, &pair, 9_000_000);

        // mirrored lower threshold = 0.10
        let lower = ONE_U128 - 9_000_000;

        // price just above lower threshold => OK
        block_toxic_trades(&e, &pair, Side::Long, lower + 1);
    }

    #[test]
    #[should_panic]
    fn block_toxic_trades_panics_long_at_or_below_lower_threshold() {
        let e = env();
        let pair = Address::generate(&e);

        set_risk_params_toxic_threshold(&e, &pair, 9_000_000);

        let lower = ONE_U128 - 9_000_000;

        // price exactly at lower threshold => panic
        block_toxic_trades(&e, &pair, Side::Long, lower);
    }
}
