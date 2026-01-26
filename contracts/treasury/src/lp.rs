use soroban_sdk::{panic_with_error, Address, Env};
use types::pair::PairAmountsWithUSDC;
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::{PrecisionMath, SafeMath};

use crate::errors::TreasuryError;

/// Converts token balances into quote-value balances using per-token prices.
///
/// All values are computed using `PRICE_PRECISION` fixed-point math:
/// `value = floor(amount * price / PRICE_PRECISION)`.
///
/// # Arguments
/// - `balances`: token amounts held
/// - `prices`: quote-per-token prices (1e7 precision)
///
/// # Returns
/// A `PairAmountsWithUSDC` whose fields are quote-values (still 1e7 precision).
pub fn values(
    e: &Env,
    balances: &PairAmountsWithUSDC,
    prices: &PairAmountsWithUSDC,
) -> PairAmountsWithUSDC {
    let long_value = balances
        .long
        .safe_fixed_mul_floor(e, prices.long, PRICE_PRECISION);
    let short_value = balances
        .short
        .safe_fixed_mul_floor(e, prices.short, PRICE_PRECISION);
    let usdc_value = balances
        .usdc
        .safe_fixed_mul_floor(e, prices.usdc, PRICE_PRECISION);

    PairAmountsWithUSDC {
        long: long_value,
        short: short_value,
        usdc: usdc_value,
    }
}

/// Computes NAV (total quote value) from token balances + prices.
///
/// NAV is:
/// `NAV = long_value + short_value + usdc_value`.
///
/// # Arguments
/// - `balances`: token amounts held
/// - `prices`: quote-per-token prices (1e7 precision)
///
/// # Returns
/// Total NAV in quote units (1e7 precision).
pub fn nav(e: &Env, balances: &PairAmountsWithUSDC, prices: &PairAmountsWithUSDC) -> u128 {
    let values = values(e, balances, prices);

    values
        .long
        .safe_add(e, values.short)
        .safe_add(e, values.usdc)
}

/// Computes NAV from already computed quote-values.
///
/// # Arguments
/// - `values`: quote-values for each token leg (1e7 precision)
///
/// # Returns
/// Total NAV in quote units (1e7 precision).
pub fn nav_with_values(e: &Env, values: &PairAmountsWithUSDC) -> u128 {
    values
        .long
        .safe_add(e, values.short)
        .safe_add(e, values.usdc)
}

/// Converts a "pairs + collateral" deposit shape into NAV.
///
/// Deposit shape:
/// - `pairs_amount` long
/// - `pairs_amount` short
/// - `pairs_amount * collateral_per_pair` USDC
///
/// # Arguments
/// - `pairs_amount`: number of long+short pairs (token units)
/// - `collateral_per_pair`: USDC collateral per pair (token units, 1e7 precision if you treat USDC as quote)
/// - `prices`: quote-per-token prices (1e7 precision)
///
/// # Returns
/// Deposit NAV in quote units (1e7 precision).
pub fn pairs_to_nav(
    e: &Env,
    pairs_amount: u128,
    collateral_per_pair: u128,
    prices: &PairAmountsWithUSDC,
) -> u128 {
    let balances = PairAmountsWithUSDC {
        long: pairs_amount,
        short: pairs_amount,
        usdc: pairs_amount.safe_fixed_mul_floor(e, collateral_per_pair, PRICE_PRECISION),
    };

    nav(e, &balances, prices)
}

/// Converts a NAV deposit into LP shares to mint (NAV-based shares).
///
/// # Invariants
/// - `deposit_nav > 0`
/// - If `total_shares_before > 0`, then `nav_before > 0`
/// - First deposit: requires empty treasury balances for the `pair`, mints 1:1 (`shares = deposit_nav`)
///
/// # Arguments
/// - `pair`: pair address used to check the treasury is empty on bootstrap
/// - `nav_before`: total treasury NAV before deposit (quote units, 1e7 precision)
/// - `deposit_nav`: depositor's NAV contribution (quote units, 1e7 precision)
/// - `total_shares_before`: total shares outstanding before deposit
///
/// # Returns
/// Shares to mint (same precision convention as NAV-shares).
///
/// # Panics
/// - `InvalidInput` if `deposit_nav == 0`
/// - `InsufficientInventory` if shares exist but `nav_before == 0`
/// - `InvalidBalance` if bootstrap deposit but treasury already has balances for `pair`
/// - `InvalidBalance` if shares exist and `nav_before == 0` (defensive)
/// - `DepositTooSmall` if minted shares would be 0 due to rounding
pub fn nav_to_shares(
    e: &Env,
    pair: &Address,
    nav_before: u128,
    deposit_nav: u128,
    total_shares_before: u128,
) -> u128 {
    if deposit_nav == 0 {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    if total_shares_before > 0 && nav_before == 0 {
        panic_with_error!(e, TreasuryError::InsufficientInventory);
    }

    if total_shares_before == 0 {
        let balances = crate::storage::get_balances(e, pair);
        if balances.long != 0 || balances.short != 0 || balances.usdc != 0 {
            panic_with_error!(e, TreasuryError::InvalidBalance);
        }
        return deposit_nav;
    }

    if nav_before == 0 {
        panic_with_error!(e, TreasuryError::InvalidBalance);
    }

    // shares_to_mint = floor(deposit_nav * total_shares_before / nav_before)
    let numerator = deposit_nav.safe_fixed_mul_floor(e, total_shares_before, PRICE_PRECISION);
    let shares_to_mint = numerator.safe_fixed_div_floor(e, nav_before, PRICE_PRECISION);

    if shares_to_mint == 0 {
        panic_with_error!(e, TreasuryError::DepositTooSmall);
    }

    shares_to_mint
}

/// Converts shares burned into NAV entitlement.
///
/// # Arguments
/// - `nav_total`: total treasury NAV at withdrawal time (quote units, 1e7 precision)
/// - `total_shares`: total shares outstanding
/// - `shares_in`: shares being burned
///
/// # Returns
/// `nav_out = floor(shares_in * nav_total / total_shares)` (quote units, 1e7 precision)
///
/// # Panics
/// - `InvalidInput` if `shares_in == 0`
/// - `InvalidBalance` if `total_shares == 0`
/// - `InsufficientInventory` if `nav_total == 0` while shares exist
/// - `WithdrawTooSmall` if `nav_out == 0` due to rounding
pub fn shares_to_nav(e: &Env, nav_total: u128, total_shares: u128, shares_in: u128) -> u128 {
    if shares_in == 0 {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }
    if total_shares == 0 {
        panic_with_error!(e, TreasuryError::InvalidBalance);
    }
    if nav_total == 0 {
        panic_with_error!(e, TreasuryError::InsufficientInventory);
    }

    let num = shares_in.safe_fixed_mul_floor(e, nav_total, PRICE_PRECISION);
    let nav_out = num.safe_fixed_div_floor(e, total_shares, PRICE_PRECISION);

    if nav_out == 0 {
        panic_with_error!(e, TreasuryError::WithdrawTooSmall);
    }

    nav_out
}

/// Converts shares burned into *transferable token amounts* using pro-rata inventory slicing.
///
/// This is the Uniswap-style payout policy:
/// each LP owns a fraction of each held token.
///
/// # Arguments
/// - `balances`: current treasury token balances
/// - `total_shares`: total shares outstanding
/// - `shares_in`: shares being burned
///
/// # Returns
/// (long_out, short_out, usdc_out) where each is:
/// `floor(balance * shares_in / total_shares)`.
///
/// # Panics
/// - `InvalidInput` if `shares_in == 0`
/// - `InvalidBalance` if `total_shares == 0`
/// - `WithdrawTooSmall` if all outputs are 0 due to rounding
pub fn shares_to_token_amounts(
    e: &Env,
    balances: &PairAmountsWithUSDC,
    total_shares: u128,
    shares_in: u128,
) -> PairAmountsWithUSDC {
    if shares_in == 0 {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }
    if total_shares == 0 {
        panic_with_error!(e, TreasuryError::InvalidBalance);
    }

    let long_out = balances
        .long
        .safe_fixed_mul_floor(e, shares_in, PRICE_PRECISION)
        .safe_fixed_div_floor(e, total_shares, PRICE_PRECISION);

    let short_out = balances
        .short
        .safe_fixed_mul_floor(e, shares_in, PRICE_PRECISION)
        .safe_fixed_div_floor(e, total_shares, PRICE_PRECISION);

    let usdc_out = balances
        .usdc
        .safe_fixed_mul_floor(e, shares_in, PRICE_PRECISION)
        .safe_fixed_div_floor(e, total_shares, PRICE_PRECISION);

    if long_out == 0 && short_out == 0 && usdc_out == 0 {
        panic_with_error!(e, TreasuryError::WithdrawTooSmall);
    }

    PairAmountsWithUSDC {
        long: long_out,
        short: short_out,
        usdc: usdc_out,
    }
}

/// Validates an LP withdrawal (NAV-based shares) against balance and risk constraints.
///
/// This function enforces:
/// 1) shares are valid: `shares_in > 0` and `shares_in <= total_shares`
/// 2) treasury is solvent: if shares exist, NAV must be non-zero
/// 3) payout amounts are feasible: cannot transfer more tokens than the treasury holds
/// 4) optional USDC floor: remaining USDC must be >= floor (fraction of NAV)
///
/// # Arguments
/// - `balances`: current treasury balances
/// - `prices`: current prices (1e7 precision) used to compute NAV for floor
/// - `total_shares`: total shares outstanding
/// - `shares_in`: shares to burn
///
/// # Returns
/// Token amounts to transfer to the withdrawer (in token units).
///
/// # Panics
/// - `InvalidInput` if `shares_in == 0` or `shares_in > total_shares`
/// - `InvalidBalance` if `total_shares == 0`
/// - `InsufficientInventory` if computed NAV is 0 while shares exist
/// - `CannotPassFloor` if remaining USDC would violate configured floor
pub fn validate_lp_withdrawal(
    e: &Env,
    balances: &PairAmountsWithUSDC,
    prices: &PairAmountsWithUSDC,
    total_shares: u128,
    shares_in: u128,
) -> PairAmountsWithUSDC {
    if shares_in == 0 {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }
    if total_shares == 0 {
        panic_with_error!(e, TreasuryError::InvalidBalance);
    }
    if shares_in > total_shares {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    // Compute NAV for solvency checks and (optionally) the USDC floor.
    let nav_total = nav(e, balances, prices);
    if nav_total == 0 {
        // If shares exist, NAV==0 implies insolvency/accounting break.
        panic_with_error!(e, TreasuryError::InsufficientInventory);
    }

    // Compute pro-rata amounts out.
    let out = shares_to_token_amounts(e, balances, total_shares, shares_in);

    // Feasibility: never allow transferring more than exists (defensive).
    if out.long > balances.long || out.short > balances.short || out.usdc > balances.usdc {
        panic_with_error!(e, TreasuryError::InvalidBalance);
    }

    // USDC floor check (if configured).
    // Expects storage value in [0, 1e7]. If you don't want a floor for LP withdrawals,
    // set floor_fraction = 0.
    if !crate::storage::get_is_killed_withdraw_floor(e) {
        let floor_fraction = crate::storage::get_usdc_floor_fraction(e);
        if floor_fraction > PRICE_PRECISION {
            panic_with_error!(e, TreasuryError::InvalidInput);
        }

        if floor_fraction > 0 {
            let usdc_price = prices.usdc;
            if usdc_price == 0 {
                panic_with_error!(e, TreasuryError::InvalidInput);
            }

            // floor_nav = floor_fraction * nav_total
            let floor_nav = nav_total.safe_fixed_mul_floor(e, floor_fraction, PRICE_PRECISION);
            // usdc_floor_amount = floor_nav / usdc_price
            let usdc_floor_amount = floor_nav.safe_fixed_div_floor(e, usdc_price, PRICE_PRECISION);

            let remaining_usdc = balances.usdc.safe_sub(e, out.usdc);
            if remaining_usdc < usdc_floor_amount {
                panic_with_error!(e, TreasuryError::CannotPassFloor);
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use crate::Treasury;

    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn env() -> Env {
        Env::default()
    }

    fn setup_test_contract(e: &Env) -> (Address, Address) {
        let admin = Address::generate(e);
        let treasury_contract = e.register(Treasury, (&admin, &admin));

        (treasury_contract, admin)
    }

    fn balances(long: u128, short: u128, usdc: u128) -> PairAmountsWithUSDC {
        PairAmountsWithUSDC {
            long: long,
            short: short,
            usdc: usdc,
        }
    }

    fn set_floor_fraction(e: &Env, treasury: &Address, frac: u128) {
        e.as_contract(treasury, || {
            crate::storage::set_usdc_floor_fraction(&e, &frac);
        });
    }

    // -------------------------
    // values / nav / nav_with_values
    // -------------------------

    #[test]
    fn test_values_and_nav_match_expected() {
        let e = env();
        // amounts: 2,3,4
        let bals = balances(2_000_0000, 3_000_0000, 4_000_0000);
        // prices: 5.0, 1.0, 2.5
        let prices = balances(5_000_0000, 10_000_000, 2_500_0000);

        let v = values(&e, &bals, &prices);
        assert_eq!(v.long, 10_000_0000);
        assert_eq!(v.short, 3_000_0000);
        assert_eq!(v.usdc, 10_000_0000);

        let n = nav(&e, &bals, &prices);
        assert_eq!(n, 23_000_0000);

        let n2 = nav_with_values(&e, &v);
        assert_eq!(n2, n);
    }

    #[test]
    fn test_pairs_to_nav_deposit_shape() {
        let e = env();
        let prices = balances(10_000_000, 10_000_000, 10_000_000); // all 1.0

        // 10 pairs, collateral_per_pair = 50 USDC
        let pairs = 10_000_0000u128; // 10.0
        let cpp = 50_000_0000u128; // 50.0

        // balances would be: long=10, short=10, usdc=500
        // nav = 520
        let n = pairs_to_nav(&e, pairs, cpp, &prices);
        assert_eq!(n, 520_000_0000u128);
    }

    // -------------------------
    // nav_to_shares
    // -------------------------

    #[test]
    fn test_nav_to_shares_non_bootstrap_proportional() {
        let e = env();
        let pair = Address::generate(&e);

        // If nav_before=1000 and total_shares=100, deposit_nav=50 => mint=5
        let nav_before = 1_000_0000u128 * 1000;
        let total_shares_before = 1_000_0000u128 * 100;
        let deposit_nav = 1_000_0000u128 * 50;

        let minted = nav_to_shares(&e, &pair, nav_before, deposit_nav, total_shares_before);
        assert_eq!(minted, 1_000_0000u128 * 5);
    }

    #[test]
    #[should_panic]
    fn test_nav_to_shares_panics_on_zero_deposit() {
        let e = env();
        let pair = Address::generate(&e);
        let _ = nav_to_shares(&e, &pair, 1_000_0000, 0, 1_000_0000);
    }

    #[test]
    #[should_panic]
    fn test_nav_to_shares_panics_if_shares_exist_but_nav_zero() {
        let e = env();
        let pair = Address::generate(&e);
        let _ = nav_to_shares(&e, &pair, 0, 1_000_0000, 1_000_0000);
    }

    #[test]
    #[should_panic]
    fn test_nav_to_shares_panics_on_dust_mint() {
        let e = env();
        let pair = Address::generate(&e);

        let nav_before = 1_000_0000u128 * 1_000_000;
        let total_shares_before = 1_000_0000u128 * 1_000;
        let deposit_nav = 1; // tiny

        let _ = nav_to_shares(&e, &pair, nav_before, deposit_nav, total_shares_before);
    }

    // -------------------------
    // shares_to_nav
    // -------------------------

    #[test]
    fn test_shares_to_nav_pro_rata() {
        let e = env();
        // nav_total=1000, shares=200, burn=50 => 250
        let nav_total = 1_000_0000u128 * 1000;
        let total_shares = 1_000_0000u128 * 200;
        let shares_in = 1_000_0000u128 * 50;

        let out = shares_to_nav(&e, nav_total, total_shares, shares_in);
        assert_eq!(out, 1_000_0000u128 * 250);
    }

    #[test]
    #[should_panic]
    fn test_shares_to_nav_panics_on_zero_shares_in() {
        let e = env();
        let _ = shares_to_nav(&e, 1_000_0000, 1_000_0000, 0);
    }

    // -------------------------
    // shares_to_token_amounts
    // -------------------------

    #[test]
    fn test_shares_to_token_amounts_pro_rata() {
        let e = env();
        // balances: 100 long, 200 short, 300 usdc
        let bals = balances(100_000_0000, 200_000_0000, 300_000_0000);
        // total_shares=1000, burn=250 => 25%
        let total_shares = 1_000_0000u128 * 1000;
        let shares_in = 1_000_0000u128 * 250;

        let out = shares_to_token_amounts(&e, &bals, total_shares, shares_in);
        assert_eq!(out.long, 25_000_0000);
        assert_eq!(out.short, 50_000_0000);
        assert_eq!(out.usdc, 75_000_0000);
    }

    // -------------------------
    // validate_lp_withdrawal
    // -------------------------

    #[test]
    fn test_validate_lp_withdrawal_passes_when_no_floor() {
        let e = env();
        let (treasury, _) = setup_test_contract(&e);
        set_floor_fraction(&e, &treasury, 0);

        let bals = balances(100_000_0000, 100_000_0000, 100_000_0000);
        let prices = balances(10_000_000, 10_000_000, 10_000_000); // all 1.0

        let total_shares = 1_000_0000u128 * 1000;
        let shares_in = 1_000_0000u128 * 100;

        e.as_contract(&treasury, || {
            let out = validate_lp_withdrawal(&e, &bals, &prices, total_shares, shares_in);
            assert!(out.long > 0 || out.short > 0 || out.usdc > 0);
        });
    }

    #[test]
    fn test_validate_lp_withdrawal_allows_remaining_equal_floor() {
        let e = env();
        let (treasury, _) = setup_test_contract(&e);
        // Floor: 50% NAV must remain as USDC
        set_floor_fraction(&e, &treasury, 5_000_000);

        // Treasury has 1000 USDC and no other assets, NAV=1000
        let bals = balances(0, 0, 1_000_0000u128 * 1000);
        let prices = balances(10_000_000, 10_000_000, 10_000_000); // usdc price = 1.0

        let total_shares = 1_000_0000u128 * 100;
        // Burn 50% shares => tries to withdraw 500 USDC, leaving 500 which equals the floor.
        let shares_in = 1_000_0000u128 * 50;

        e.as_contract(&treasury, || {
            let out = validate_lp_withdrawal(&e, &bals, &prices, total_shares, shares_in);
            assert_eq!(out.usdc, 1_000_0000u128 * 500);
        });
    }

    #[test]
    #[should_panic]
    fn test_validate_lp_withdrawal_panics_if_floor_violated() {
        let e = env();
        let (treasury, _) = setup_test_contract(&e);
        set_floor_fraction(&e, &treasury, 5_000_000); // 50%

        // Treasury has 1000 USDC, NAV=1000
        let bals = balances(0, 0, 1_000_0000u128 * 1000);
        let prices = balances(10_000_000, 10_000_000, 10_000_000);

        let total_shares = 1_000_0000u128 * 100;
        // Burn 60% shares => out=600, remaining=400 < floor(500) => should panic.
        let shares_in = 1_000_0000u128 * 60;

        let _ = validate_lp_withdrawal(&e, &bals, &prices, total_shares, shares_in);
    }

    #[test]
    #[should_panic]
    fn test_validate_lp_withdrawal_panics_if_shares_in_gt_total() {
        let e = env();
        let (treasury, _) = setup_test_contract(&e);
        set_floor_fraction(&e, &treasury, 0);

        let bals = balances(1_000_0000, 1_000_0000, 1_000_0000);
        let prices = balances(10_000_000, 10_000_000, 10_000_000);

        let total_shares = 1_000_0000u128 * 100;
        let shares_in = total_shares + 1;

        let _ = validate_lp_withdrawal(&e, &bals, &prices, total_shares, shares_in);
    }
}
