#![cfg(test)]
extern crate std;

use crate::testutils::Setup;

use soroban_sdk::{testutils::Address as _, Address};
// use utils::test_utils::jump;

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn initialize_already_initialized() {
    let setup = Setup::default();

    setup.treasury.initialize(&setup.admin);
}

/**
 *
 * Deposit
 *
 */

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_deposit_invalid_amount() {
    let setup = Setup::default();
    let pair = Address::generate(&setup.env);

    setup.treasury.deposit(&setup.users[1], &pair, &0, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_deposit_long_amount_doesnt_match_short_amount() {
    let setup = Setup::default();
    let pair = Address::generate(&setup.env);

    setup.treasury.deposit(&setup.users[1], &pair, &1, &2, &1);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn test_deposit_killed() {
    let setup = Setup::default();
    let admin = &setup.admin;
    let pair = Address::generate(&setup.env);

    setup.treasury.kill_deposit(admin);

    setup.treasury.deposit(&setup.users[1], &pair, &1, &1, &1);
}

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn test_deposit_fails_if_not_supported() {
    let setup = Setup::default();
    let unsupported_pair = Address::generate(&setup.env);

    setup
        .treasury
        .deposit(&setup.users[1], &unsupported_pair, &1, &1, &1);
}

// TODO: pair - get_collateral_info() tests

#[test]
fn test_deposit() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let tokens_to_deposit = 1_0000000;

    // Mint initial tokens to user
    setup.token_long_admin_client.mint(&user1, &1_0000000);
    assert_eq!(setup.token_long.balance(&user1) as u128, 1_0000000);

    setup.token_short_admin_client.mint(&user1, &1_0000000);
    assert_eq!(setup.token_short.balance(&user1) as u128, 1_0000000);

    setup.token_usdc_admin_client.mint(&user1, &1_0000000);
    assert_eq!(setup.token_usdc.balance(&user1) as u128, 1_0000000);

    // Setup
    let total_shares_before = setup.treasury.get_total_shares(&setup.pair.address);
    assert_eq!(total_shares_before, 0);
    let user_shares_before = setup.treasury.get_user_shares(&setup.pair.address, &user1);
    assert_eq!(user_shares_before, 0);

    // Deposit
    let new_shares = setup.treasury.deposit(
        &user1,
        &setup.pair.address,
        &tokens_to_deposit,
        &tokens_to_deposit,
        &tokens_to_deposit,
    );

    // Assertions

    // [ ] all 3 tokens moved to treasury
    assert_eq!(
        setup.token_long.balance(&setup.treasury.address) as u128,
        tokens_to_deposit
    );
    assert_eq!(
        setup.token_short.balance(&setup.treasury.address) as u128,
        tokens_to_deposit
    );
    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address) as u128,
        tokens_to_deposit
    );

    // [ ] Total shares and user shares incremented
    let total_shares = setup.treasury.get_total_shares(&setup.pair.address);
    assert_eq!(total_shares, total_shares_before + new_shares);

    let user_shares = setup.treasury.get_user_shares(&setup.pair.address, &user1);
    assert_eq!(user_shares, user_shares_before + new_shares);

    // [ ] tvl increases
}

// #[test]
// fn test_deposit_increases_total_shares_correct_after_first() {}

// #[test]
// #[should_panic(expected = "Error(Contract, #204)")]
// fn test_withdraw_invalid_amount() {
//     let setup = Setup::default();
//     let user1 = setup.users[1].clone();

//     setup.treasury.withdraw(&user1, &0);
// }

// #[test]
// fn test_withdraw() {
//     let setup = Setup::default();
//     let user1 = setup.users[1].clone();
//     let tokens_to_mint = 1_0000000;
//     let tokens_to_withdraw = 1_0000000;

//     // Mint collateral to user
//     setup.token_usdc_admin_client.mint(&user1, &100_0000000);
//     assert_eq!(setup.token_usdc.balance(&user1) as u128, 100_0000000);

//     // Mint tokens
//     setup.treasury.mint(&user1, &tokens_to_mint);

//     // Then, redeem
//     let redeemed_tokens = setup.treasury.redeem(&user1, &tokens_to_redeem);

//     let collateral_info = setup.treasury.get_collateral_info();
//     let expected_collateral =
//         (tokens_to_redeem * collateral_info.collateral_per_pair) / PRICE_PRECISION;
//     assert_eq!(redeemed_tokens, expected_collateral);

//     // TODO: Ensure price is updated

//     // Long and Short tokens are burned from the user
//     assert_eq!(setup.token_long.balance(&user1) as u128, 0);
//     assert_eq!(setup.token_short.balance(&user1) as u128, 0);

//     // Collateral token went down by collateral per pair
//     let collateral_info = setup.treasury.get_collateral_info();
//     assert_eq!(setup.token_usdc.balance(&user1), collateral_info.collateral_per_pair as i128);
//     assert_eq!(setup.token_usdc.balance(&setup.treasury.address), 0);
// }

// /**
//  * Trade
//  */
// #[test]
// fn test_buy_long() {
//     let setup = Setup::default();

//     let collateral_info_before = setup.treasury.get_collateral_info();
//     // defaults
//     assert_eq!(collateral_info_before, CollateralInfo {
//         collateral_per_pair: 100_0000000,
//         collateral_percent_long: 5000,
//     });

//     // Update the oracle price
//     jump(&setup.env, ONE_MINUTE as u64);

//     let new_prices: Vec<i128> = Vec::from_array(&setup.env, [110_00000000000000, 1_00000000000000]);
//     setup.reflector_client.set_price(&new_prices, &setup.env.ledger().timestamp());

//     // Call pair
//     setup.treasury.sync_collateral_percent_long();

//     assert_eq!(setup.treasury.get_collateral_info(), CollateralInfo {
//         collateral_per_pair: 100_0000000, // unchanged,
//         collateral_percent_long: 6000,
//     });
// }
