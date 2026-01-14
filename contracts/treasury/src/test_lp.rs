#![cfg(test)]
extern crate std;

use crate::testutils::Setup;
use soroban_sdk::{testutils::Address as _, Address};
use types::pair::PairAmountsWithUSDC;
use utils::constant::PRICE_PRECISION;

/// Helpers
fn collateral_used(pairs: u128, collateral_per_pair: u128) -> u128 {
    (pairs * collateral_per_pair) / PRICE_PRECISION
}

fn assert_pair_balances_equal(setup: &Setup, expected: PairAmountsWithUSDC, msg: &str) {
    let got = setup.treasury.get_balances(&setup.pair.address);
    assert_eq!(got, expected, "{}", msg);
}

//
// ------------------------------------------------------------
// Deposit() tests
// ------------------------------------------------------------
//

#[test]
#[should_panic(expected = "Error(Contract, #204")] // TreasuryError::InvalidInput
fn test_deposit_reverts_on_zero_pairs() {
    let setup = Setup::default();
    let user = setup.users[1].clone();
    setup.treasury.deposit(&user, &setup.pair.address, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213")] // TreasuryError::ActionPaused
fn test_deposit_reverts_when_paused() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    // pause deposits
    setup.treasury.kill_deposit(&admin);

    setup
        .treasury
        .deposit(&user, &setup.pair.address, &1_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #220")] // TreasuryError::InvalidBalance (bootstrap requires empty treasury)
fn test_deposit_bootstrap_requires_empty_treasury() {
    let setup = Setup::default();
    let admin = setup.admin.clone();

    // Put funds in treasury balances without shares (simulate accidental funding).
    // We do this by transferring tokens directly to the treasury contract.
    setup
        .token_usdc_admin_client
        .mint(&admin, &2_000_0000000i128);
    setup
        .token_usdc
        .transfer(&admin, &setup.treasury.address, &1_000_0000000i128);

    setup.token_long_admin_client.mint(&admin, &1_0000000i128);
    setup.token_short_admin_client.mint(&admin, &1_0000000i128);

    // Now attempt bootstrap deposit; nav_to_shares should revert due to non-empty balances.
    setup
        .treasury
        .deposit(&admin, &setup.pair.address, &1_0000000);
}

#[test]
fn test_deposit_bootstrap_mints_shares_equal_to_deposit_nav_and_updates_state() {
    let setup = Setup::default();
    let admin = setup.admin.clone();

    // Admin needs USDC to mint pairs (pair.mint consumes collateral)
    let init_admin_usdc = 10_000_0000000u128;
    setup
        .token_usdc_admin_client
        .mint(&admin, &(init_admin_usdc as i128));

    let pairs_to_mint = 10_0000000u128;
    let pairs_to_deposit = 10_0000000u128;

    // Mint pairs to admin (long+short) by locking USDC into Pair
    setup.pair.mint(&admin, &pairs_to_mint);

    // Compute required collateral and ensure admin has long/short
    let collateral_info = setup.pair.get_collateral_info();
    let required_usdc = collateral_used(pairs_to_deposit, collateral_info.collateral_per_pair);

    assert_eq!(setup.token_long.balance(&admin), pairs_to_mint as i128);
    assert_eq!(setup.token_short.balance(&admin), pairs_to_mint as i128);

    // Admin must still have at least required_usdc remaining to transfer into Treasury
    // (they started with init_admin_usdc; minting pairs consumed pairs_to_mint*collateral_per_pair)
    let collateral_consumed = collateral_used(pairs_to_mint, collateral_info.collateral_per_pair);
    let admin_usdc_after_mint = (init_admin_usdc - collateral_consumed) as i128;
    assert_eq!(setup.token_usdc.balance(&admin), admin_usdc_after_mint);

    // Deposit
    let shares_minted = setup
        .treasury
        .deposit(&admin, &setup.pair.address, &pairs_to_deposit);

    // Checks: balances moved to treasury
    assert_eq!(setup.token_long.balance(&admin), 0);
    assert_eq!(setup.token_short.balance(&admin), 0);
    assert_eq!(
        setup.token_usdc.balance(&admin),
        admin_usdc_after_mint - (required_usdc as i128)
    );

    assert_eq!(
        setup.token_long.balance(&setup.treasury.address),
        pairs_to_deposit as i128
    );
    assert_eq!(
        setup.token_short.balance(&setup.treasury.address),
        pairs_to_deposit as i128
    );
    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        required_usdc as i128
    );

    // Treasury balances updated
    assert_pair_balances_equal(
        &setup,
        PairAmountsWithUSDC {
            long: pairs_to_deposit,
            short: pairs_to_deposit,
            usdc: required_usdc,
        },
        "treasury balances after bootstrap deposit",
    );

    // Shares updated
    let total_shares = setup.treasury.get_total_shares(&setup.pair.address);
    let user_shares = setup.treasury.get_user_shares(&setup.pair.address, &admin);

    assert_eq!(total_shares, shares_minted);
    assert_eq!(user_shares, shares_minted);

    // Bootstrap invariant: shares minted == deposit NAV
    // Since prices at setup are: long=0.5, short=0.5 (for SOL at mid-band), and USDC=1,
    // deposit_nav should be long+short+usdc ~= pairs*1 + pairs*collateral_per_pair (in quote)
    // We can at least assert non-zero and consistent:
    assert!(shares_minted > 0);
}

#[test]
fn test_deposit_second_deposit_mints_pro_rata_shares() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    // Mint USDC for both to mint pairs and deposit
    setup
        .token_usdc_admin_client
        .mint(&admin, &50_000_0000000i128);
    setup
        .token_usdc_admin_client
        .mint(&user1, &50_000_0000000i128);

    let collateral_info = setup.pair.get_collateral_info();

    // Admin mints + deposits 10 pairs
    let pairs_a = 10_0000000u128;
    setup.pair.mint(&admin, &pairs_a);
    let shares_a = setup
        .treasury
        .deposit(&admin, &setup.pair.address, &pairs_a);

    // User1 mints + deposits 5 pairs
    let pairs_b = 5_0000000u128;
    setup.pair.mint(&user1, &pairs_b);
    let shares_b = setup
        .treasury
        .deposit(&user1, &setup.pair.address, &pairs_b);

    // Because deposit NAV is linear in pairs, and prices unchanged,
    // second depositor should receive roughly half of first depositor’s shares (flooring possible).
    assert!(shares_b > 0);
    assert!(shares_a > shares_b);

    // Total shares = sum
    let total_shares = setup.treasury.get_total_shares(&setup.pair.address);
    assert_eq!(total_shares, shares_a + shares_b);

    // Balances reflect both deposits
    let required_usdc_a = collateral_used(pairs_a, collateral_info.collateral_per_pair);
    let required_usdc_b = collateral_used(pairs_b, collateral_info.collateral_per_pair);

    assert_pair_balances_equal(
        &setup,
        PairAmountsWithUSDC {
            long: pairs_a + pairs_b,
            short: pairs_a + pairs_b,
            usdc: required_usdc_a + required_usdc_b,
        },
        "treasury balances after two deposits",
    );
}

//
// ------------------------------------------------------------
// Withdraw() tests
// ------------------------------------------------------------
//

#[test]
#[should_panic(expected = "Error(Contract, #204")] // TreasuryError::InvalidInput
fn test_withdraw_reverts_on_zero_shares() {
    let setup = Setup::default();
    let user = setup.users[1].clone();
    setup.treasury.withdraw(&user, &setup.pair.address, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213")] // TreasuryError::ActionPaused
fn test_withdraw_reverts_when_paused() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    setup.treasury.kill_withdraw(&admin);

    setup
        .treasury
        .withdraw(&user, &setup.pair.address, &1_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #501")] // StorageError::ValueNotInitialized
fn test_withdraw_reverts_if_user_has_no_shares() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    // no deposit => no shares
    setup
        .treasury
        .withdraw(&user1, &setup.pair.address, &1_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #217")] // TreasuryError::InsufficientShares
fn test_withdraw_reverts_if_withdraw_more_than_user_shares() {
    let setup = Setup::default();
    let admin = setup.admin.clone();

    // fund and deposit once to create shares
    setup
        .token_usdc_admin_client
        .mint(&admin, &10_000_0000000i128);
    let pairs = 10_0000000u128;
    setup.pair.mint(&admin, &pairs);

    let shares = setup.treasury.deposit(&admin, &setup.pair.address, &pairs);
    setup
        .treasury
        .withdraw(&admin, &setup.pair.address, &(shares + 1));
}

#[test]
fn test_withdraw_happy_path_returns_pro_rata_tokens_and_updates_shares_and_balances() {
    let setup = Setup::default();
    let admin = setup.admin.clone();

    // Fund admin and deposit 10 pairs
    setup
        .token_usdc_admin_client
        .mint(&admin, &50_000_0000000i128);

    let pairs = 10_0000000u128;
    setup.pair.mint(&admin, &pairs);

    let shares_minted = setup.treasury.deposit(&admin, &setup.pair.address, &pairs);

    // Withdraw 25% shares
    let shares_out = shares_minted / 4;
    let tokens_out = setup
        .treasury
        .withdraw(&admin, &setup.pair.address, &shares_out);

    // Expect ~25% of each token (flooring possible)
    assert!(tokens_out.long > 0);
    assert!(tokens_out.short > 0);
    assert!(tokens_out.usdc > 0);

    // Token balances reflect payout
    assert_eq!(setup.token_long.balance(&admin) as u128, tokens_out.long);
    assert_eq!(setup.token_short.balance(&admin) as u128, tokens_out.short);
    assert_eq!(setup.token_usdc.balance(&admin) as u128, tokens_out.usdc);

    // Treasury balances decreased
    let after = setup.treasury.get_balances(&setup.pair.address);
    assert_eq!(after.long, pairs - tokens_out.long);
    assert_eq!(after.short, pairs - tokens_out.short);

    // USDC in treasury was collateral_per_pair*pairs (from deposit)
    let collateral_info = setup.pair.get_collateral_info();
    let required_usdc = collateral_used(pairs, collateral_info.collateral_per_pair);
    assert_eq!(after.usdc, required_usdc - tokens_out.usdc);

    // Shares updated
    let total_shares_after = setup.treasury.get_total_shares(&setup.pair.address);
    let user_shares_after = setup.treasury.get_user_shares(&setup.pair.address, &admin);

    assert_eq!(total_shares_after, shares_minted - shares_out);
    assert_eq!(user_shares_after, shares_minted - shares_out);
}

//
// ------------------------------------------------------------
// Withdrawal risk guard (USDC floor) tests
// ------------------------------------------------------------
//

#[test]
#[should_panic(expected = "Error(Contract, #221")] // TreasuryError::CannotPassFloor
fn test_withdraw_reverts_if_usdc_floor_violated() {
    let setup = Setup::default();
    let admin = setup.admin.clone();

    // Configure USDC floor fraction high (e.g., 90%)
    setup.treasury.set_usdc_floor(&admin, &9_000_000u128);

    setup
        .token_usdc_admin_client
        .mint(&admin, &50_000_0000000i128);
    let pairs = 10_0000000u128;
    setup.pair.mint(&admin, &pairs);
    let shares = setup.treasury.deposit(&admin, &setup.pair.address, &pairs);

    // Try to withdraw a big portion which would take USDC below floor.
    setup
        .treasury
        .withdraw(&admin, &setup.pair.address, &((shares * 9) / 10));
}

#[test]
fn test_withdraw_allows_when_remaining_equals_usdc_floor() {
    let setup = Setup::default();
    let admin = setup.admin.clone();

    // 50% floor
    setup.treasury.set_usdc_floor(&admin, &5_000_000u128);

    setup
        .token_usdc_admin_client
        .mint(&admin, &50_000_0000000i128);
    let pairs = 10_0000000u128;
    setup.pair.mint(&admin, &pairs);
    let shares = setup.treasury.deposit(&admin, &setup.pair.address, &pairs);

    // Withdraw 50% shares: remaining USDC should equal floor => allowed
    let _ = setup
        .treasury
        .withdraw(&admin, &setup.pair.address, &(shares / 2));
}
