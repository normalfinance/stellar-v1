use soroban_sdk::{panic_with_error, Address, Env};
use types::pair::{PairAmountsWithUSDC, Side};
use utils::constant::{ONE_U128, PRICE_PRECISION};
use utils::math::safe_math::{PrecisionMath, SafeMath};

use crate::errors::TreasuryError;

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
        usdc: usdc_value,
        long: long_value,
        short: short_value,
    }
}

pub fn nav(e: &Env, balances: &PairAmountsWithUSDC, prices: &PairAmountsWithUSDC) -> u128 {
    let values = values(e, balances, prices);

    values
        .long
        .safe_add(e, values.short)
        .safe_add(e, values.usdc)
}

pub fn nav_with_values(e: &Env, values: &PairAmountsWithUSDC) -> u128 {
    values
        .long
        .safe_add(e, values.short)
        .safe_add(e, values.usdc)
}

pub fn pairs_to_nav(
    e: &Env,
    pairs_amount: u128,
    collateral_per_pair: u128,
    prices: &PairAmountsWithUSDC,
) -> u128 {
    let balances = PairAmountsWithUSDC {
        long: pairs_amount,
        short: pairs_amount,
        usdc: pairs_amount.safe_fixed_mul_floor(&e, collateral_per_pair, PRICE_PRECISION),
    };

    nav(e, &balances, prices)
}

/// Mint LP shares based on value contributed (NAV), not raw pair count.
///
/// Invariants:
/// - If shares exist, nav_before must be > 0 (otherwise shares are unbacked / insolvent)
/// - First deposit bootstraps 1:1 shares to deposit value (in NAV units)
pub fn nav_to_shares(
    e: &Env,
    pair: &Address,
    nav_before: u128, // total treasury NAV before deposit (quote units, PRICE_PRECISION-scaled)
    deposit_nav: u128, // depositor's contribution value (quote units, PRICE_PRECISION-scaled)
    total_shares_before: u128, // total LP shares before deposit (same precision convention as NAV shares)
) -> u128 {
    if deposit_nav == 0 {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    // If shares exist but nav is 0, someone drained value / accounting is broken.
    // This prevents "donation deposits" that mint 0 shares.
    if total_shares_before > 0 && nav_before == 0 {
        panic_with_error!(e, TreasuryError::InsufficientInventory);
    }

    // First deposit: bootstrap shares at 1:1 with deposited NAV
    // Require empty treasury to avoid implicit donations / weird seeding.
    if total_shares_before == 0 {
        let balances = crate::storage::get_balances(&e, pair);
        if balances.long != 0 || balances.short != 0 || balances.usdc != 0 {
            panic_with_error!(e, TreasuryError::InvalidBalance);
        }

        // 1 share == 1 NAV unit (in your fixed precision)
        return deposit_nav;
    }

    // Defensive: if shares exist, nav must exist.
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

/// Compute how much NAV value (quote units) a user is entitled to when burning shares.
///
/// nav_total: total treasury NAV at withdrawal time (quote units, PRICE_PRECISION-scaled)
/// returns: nav_out in same units
pub fn shares_to_nav(e: &Env, nav_total: u128, total_shares: u128, shares_in: u128) -> u128 {
    if shares_in == 0 {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    if total_shares == 0 {
        panic_with_error!(e, TreasuryError::InvalidBalance);
    }

    // If shares exist, nav_total should not be 0 (otherwise insolvency / accounting break)
    if nav_total == 0 {
        panic_with_error!(e, TreasuryError::InsufficientInventory);
    }

    // nav_out = floor(shares_in * nav_total / total_shares)
    let num = shares_in.safe_fixed_mul_floor(e, nav_total, PRICE_PRECISION);
    let nav_out = num.safe_fixed_div_floor(e, total_shares, PRICE_PRECISION);

    if nav_out == 0 {
        panic_with_error!(e, TreasuryError::WithdrawTooSmall);
    }

    nav_out
}

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

    // out = floor(balance * shares_in / total_shares)
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

pub fn validate_lp_withdrawal(e: &Env, side: Side, price: u128, toxic_threshold: u128) {
    // Near upper bound: short is toxic
    if side == Side::Short && price >= toxic_threshold {
        panic_with_error!(e, TreasuryError::ToxicSideNotAccepted);
    }

    // Near lower bound: long is toxic
    if side == Side::Long && price <= ONE_U128.saturating_sub(price) {
        panic_with_error!(e, TreasuryError::ToxicSideNotAccepted);
    }
}

#[cfg(test)]
mod tests {
    use crate::Treasury;

    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup_test_contract(e: &Env) -> (Address, Address) {
        let admin = Address::generate(e);
        let treasury_contract = e.register(Treasury, (&admin,));

        (treasury_contract, admin)
    }

    fn complete_test_setup(e: &Env) -> (Address, Address) {
        let (treasury_contract, admin) = setup_test_contract(e);

        (treasury_contract, admin)
    }

    fn balances(long: u128, short: u128, usdc: u128) -> PairAmountsWithUSDC {
        PairAmountsWithUSDC {
            long: long,
            short: short,
            usdc: usdc,
        }
    }
}
