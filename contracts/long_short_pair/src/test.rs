#![cfg(test)]
extern crate std;

use crate::funding::FundingCheckpoint;
use crate::storage::CollateralInfo;
use crate::testutils::{install_pair_token_wasm, Setup};

use soroban_sdk::{testutils::Address as _, Address, Vec};
use types::pair::PairParams;
use utils::constant::{ONE_MINUTE, PRICE_PRECISION};
use utils::test_utils::jump;

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn initialize_already_initialized() {
    let setup = Setup::default();

    let params = PairParams {
        admin: setup.admin,
        privileged_addrs: (setup.emergency_admin, setup.pause_admin),
        asset: setup.solana,
        collateral_token: setup.token_usdc.address,
        pair_token_wasm_hash: install_pair_token_wasm(&setup.env),
        oracle: setup.oracle.address,
        pair_calculator: setup.pair_calculator.address,
        collateral_per_pair: 100_0000000,
        lower_bound: 50_000000,
        upper_bound: 300_0000000,
    };

    setup.pair.initialize(&params);
}

/**
 *
 * Mint
 *
 */

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_mint_invalid_amount() {
    let setup = Setup::default();

    setup.pair.mint(&setup.users[1], &0);
}

#[test]
fn test_mint() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 1_0000000;

    setup.token_usdc_admin_client.mint(&user1, &100_0000000);
    assert_eq!(setup.token_usdc.balance(&user1) as u128, 100_0000000);

    // Mint tokens
    let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
    assert_eq!(minted_tokens, tokens_to_mint);

    // Collateral transferred from user1 to pair
    let collateral_info = setup.pair.get_collateral_info();
    let expected_collateral =
        (tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;
    assert_eq!(setup.token_usdc.balance(&user1), 0);
    assert_eq!(
        setup.token_usdc.balance(&setup.pair.address),
        expected_collateral as i128
    );

    // Long and Short tokens are minted to the user
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_mint);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_mint);

    // Funding checkpoint
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user1),
        FundingCheckpoint {
            account: user1,
            long_index: 0,
            short_index: 0,
            long_balance: tokens_to_mint,
            short_balance: tokens_to_mint,
        }
    );
}

/**
 *
 * Redeem
 *
 */

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_redeem_invalid_amount() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    setup.pair.redeem(&user1, &0);
}

#[test]
fn test_redeem() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 1_0000000;
    let tokens_to_redeem = 1_0000000;

    // Mint collateral to user
    setup.token_usdc_admin_client.mint(&user1, &100_0000000);
    assert_eq!(setup.token_usdc.balance(&user1) as u128, 100_0000000);

    // Mint tokens
    setup.pair.mint(&user1, &tokens_to_mint);

    // Then, redeem
    let redeemed_tokens = setup.pair.redeem(&user1, &tokens_to_redeem);

    let collateral_info = setup.pair.get_collateral_info();
    let expected_collateral =
        (tokens_to_redeem * collateral_info.collateral_per_pair) / PRICE_PRECISION;
    assert_eq!(redeemed_tokens, expected_collateral);

    // TODO: Ensure price is updated

    // Long and Short tokens are burned from the user
    assert_eq!(setup.token_long.balance(&user1) as u128, 0);
    assert_eq!(setup.token_short.balance(&user1) as u128, 0);

    // Funding checkpoint
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user1),
        FundingCheckpoint {
            account: user1.clone(),
            long_index: 0,
            short_index: 0,
            long_balance: 0,
            short_balance: 0,
        }
    );

    // Collateral token went down by collateral per pair
    let collateral_info = setup.pair.get_collateral_info();
    assert_eq!(
        setup.token_usdc.balance(&user1),
        collateral_info.collateral_per_pair as i128
    );
    assert_eq!(setup.token_usdc.balance(&setup.pair.address), 0);
}

// #[test]
// fn test_redeem_partial() {
//     let setup = Setup::new_with_config(
//         &(TestConfig {
//             mint_to_user: i128::MAX,
//             ..TestConfig::default()
//         })
//     );
//     let user1 = setup.users[1].clone();
//     let tokens_to_mint = 100_0000000;
//     let tokens_to_redeem = 50_0000000;

//     // Mint tokens
//     let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
//     assert_eq!(minted_tokens, tokens_to_mint);

//     let redeemed_tokens = setup.pair.redeem(&user1, &tokens_to_redeem);
//     assert_eq!(redeemed_tokens, tokens_to_redeem);

//     // Long and Short tokens are burned from the user
//     assert_eq!(setup.token_long.balance(&user1) as u128, 0);
//     assert_eq!(setup.token_short.balance(&user1) as u128, 0);

//     // Funding checkpoint
//     assert_eq!(setup.pair.get_user_funding_checkpoint(&user1), FundingCheckpoint {
//         account: user1.clone(),
//         long_index: 0,
//         short_index: 0,
//         long_balance: tokens_to_mint - tokens_to_redeem,
//         short_balance: tokens_to_mint - tokens_to_redeem,
//     });

//     // Collateral token went down by collateral per paif
//     let collateral_info = setup.pair.get_collateral_info();
//     assert_eq!(setup.token_usdc.balance(&user1), i128::MAX);
//     assert_eq!(setup.token_usdc.balance(&setup.pair.address), 0);
// }

#[test]
#[should_panic(expected = "Error(Contract, #207)")]
fn test_sync_collateral_percent_long_bad_contract() {
    let setup = Setup::default();
    let bad_calculator = Address::generate(&setup.env);

    setup.pair.set_calculator(&setup.admin, &bad_calculator);

    // Update the oracle price
    jump(&setup.env, ONE_MINUTE as u64);

    let new_prices: Vec<i128> = Vec::from_array(&setup.env, [110_00000000000000, 1_00000000000000]);
    setup
        .reflector_client
        .set_price(&new_prices, &setup.env.ledger().timestamp());

    // Call pair
    setup.pair.sync_collateral_percent_long();
}

#[test]
fn test_sync_collateral_percent_long() {
    let setup = Setup::default();

    let collateral_info_before = setup.pair.get_collateral_info();
    // defaults
    assert_eq!(
        collateral_info_before,
        CollateralInfo {
            collateral_per_pair: 100_0000000,
            collateral_percent_long: 5000,
        }
    );

    // Update the oracle price
    jump(&setup.env, ONE_MINUTE as u64);

    let new_prices: Vec<i128> = Vec::from_array(&setup.env, [110_00000000000000, 1_00000000000000]);
    setup
        .reflector_client
        .set_price(&new_prices, &setup.env.ledger().timestamp());

    // Call pair
    setup.pair.sync_collateral_percent_long();

    assert_eq!(
        setup.pair.get_collateral_info(),
        CollateralInfo {
            collateral_per_pair: 100_0000000, // unchanged,
            collateral_percent_long: 6000,
        }
    );
}
