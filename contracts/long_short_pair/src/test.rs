#![cfg(test)]
extern crate std;

use crate::funding::FundingCheckpoint;
use crate::testutils::{
    create_token_contract, get_token_admin_client, install_token_wasm, Setup, TestConfig,
};
use access_control::constants::ADMIN_ACTIONS_DELAY;
use core::cmp::min;
use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation, Events};
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{
    symbol_short, testutils::Address as _, vec, Address, Env, Error, IntoVal, Map, Symbol, Val, Vec,
};
use types::pair::{LinearLongShortPairParameters, PairParams};
use utils::constant::ONE_HOUR;
use utils::test_utils::{install_dummy_wasm, jump};

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn initialize_already_initialized() {
    let setup = Setup::default();

    let params = PairParams {
        admin: setup.admin,
        privileged_addrs: (setup.emergency_admin, setup.pause_admin),
        tokens: Vec::from_array(
            &setup.env,
            [
                setup.token_long.address.clone(),
                setup.token_short.address.clone(),
            ],
        ),
        oracle: setup.oracle.address,
        pool_plane: Address::generate(&setup.env),
        pair_calculator: setup.pair_calculator.address,
        lower_bound: 50_000000,
        upper_bound: 300_0000000,
    };

    setup.pair.initialize(&params);
}

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_mint_invalid_amount() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );

    setup.pair.mint(&setup.users[1], &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #210)")]
fn test_mint_without_pools() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            ..TestConfig::default()
        }),
    );

    setup.pair.mint(&setup.users[1], &100_0000000);
}

#[test]
fn test_mint() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 100_0000000;

    // Set pools
    let pool_long = Address::generate(&setup.env);
    let pool_short = Address::generate(&setup.env);
    setup.pair.set_pools(&setup.admin, &pool_long, &pool_short);

    // Mint tokens
    let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
    assert_eq!(minted_tokens, tokens_to_mint);

    // Collateral transferred from user1 to pair
    let collateral_info = setup.pair.get_collateral_info();
    let expected_collateral = tokens_to_mint * collateral_info.collateral_per_pair;
    assert_eq!(
        setup.token_usdc.balance(&user1),
        i128::MAX - (expected_collateral as i128)
    );
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

#[test]
fn test_transfer_long_token_updates_funding_checkpoint() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let user2 = setup.users[2].clone();
    let tokens_to_mint = 100_0000000;
    let tokens_to_transfer = 50_000000;

    // Set pools
    let pool_long = Address::generate(&setup.env);
    let pool_short = Address::generate(&setup.env);
    setup.pair.set_pools(&setup.admin, &pool_long, &pool_short);

    // Mint tokens
    let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
    assert_eq!(minted_tokens, tokens_to_mint);

    // Transfer tokens
    setup
        .token_long
        .transfer(&user1, &user2, &(tokens_to_transfer as i128));
    assert_eq!(
        setup.token_long.balance(&user1) as u128,
        tokens_to_mint - tokens_to_transfer
    );
    assert_eq!(setup.token_long.balance(&user2) as u128, tokens_to_transfer);

    // Funding checkpoint updated for sending user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user1),
        FundingCheckpoint {
            account: user1,
            long_index: 0,
            short_index: 0,
            long_balance: tokens_to_mint - tokens_to_transfer,
            short_balance: tokens_to_mint,
        }
    );

    // And receiving user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user2),
        FundingCheckpoint {
            account: user2,
            long_index: 0,
            short_index: 0,
            long_balance: tokens_to_transfer,
            short_balance: 0,
        }
    );
}

#[test]
fn test_transfer_long_tokens_after_funding_updates_funding_checkpoint() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let user2 = setup.users[2].clone();
    let tokens_to_mint = 100_0000000;
    let tokens_to_transfer = 50_000000;

    // Set pools
    let pool_long = Address::generate(&setup.env);
    let pool_short = Address::generate(&setup.env);
    setup.pair.set_pools(&setup.admin, &pool_long, &pool_short);

    // Mint tokens
    let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
    assert_eq!(minted_tokens, tokens_to_mint);

    // Update funding
    setup.pair.update_funding_rate(&setup.admin);
    let funding_info = setup.pair.get_funding_info();

    // Transfer tokens
    setup
        .token_long
        .transfer(&user1, &user2, &(tokens_to_transfer as i128));
    assert_eq!(
        setup.token_long.balance(&user1) as u128,
        tokens_to_mint - tokens_to_transfer
    );
    assert_eq!(setup.token_long.balance(&user2) as u128, tokens_to_transfer);

    // Funding checkpoint updated for sending user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user1),
        FundingCheckpoint {
            account: user1,
            long_index: funding_info.last_funding_rate,
            short_index: -funding_info.last_funding_rate,
            long_balance: tokens_to_mint - tokens_to_transfer,
            short_balance: tokens_to_mint,
        }
    );

    // And receiving user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user2),
        FundingCheckpoint {
            account: user2,
            long_index: funding_info.last_funding_rate,
            short_index: -funding_info.last_funding_rate,
            long_balance: tokens_to_transfer,
            short_balance: 0,
        }
    );

    // Update funding
    setup.pair.update_funding_rate(&setup.admin);

    // Che
}

#[test]
fn test_transfer_short_token_updates_funding_checkpoint() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let user2 = setup.users[2].clone();
    let tokens_to_mint = 100_0000000;
    let tokens_to_transfer = 50_000000;

    // Set pools
    let pool_long = Address::generate(&setup.env);
    let pool_short = Address::generate(&setup.env);
    setup.pair.set_pools(&setup.admin, &pool_long, &pool_short);

    // Mint tokens
    let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
    assert_eq!(minted_tokens, tokens_to_mint);

    // Transfer tokens
    setup
        .token_short
        .transfer(&user1, &user2, &(tokens_to_transfer as i128));
    assert_eq!(
        setup.token_short.balance(&user1) as u128,
        tokens_to_mint - tokens_to_transfer
    );
    assert_eq!(
        setup.token_short.balance(&user2) as u128,
        tokens_to_transfer
    );

    // Funding checkpoint updated for sending user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user1),
        FundingCheckpoint {
            account: user1,
            long_index: 0,
            short_index: 0,
            long_balance: tokens_to_mint,
            short_balance: tokens_to_mint - tokens_to_transfer,
        }
    );

    // And receiving user
    assert_eq!(
        setup.pair.get_user_funding_checkpoint(&user2),
        FundingCheckpoint {
            account: user2,
            long_index: 0,
            short_index: 0,
            long_balance: 0,
            short_balance: tokens_to_transfer,
        }
    );
}

/**
 * Test cases:
 * - mint twice
 * -
 */

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_redeem_invalid_amount() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 100_0000000;
    let tokens_to_redeem = 100_0000000;

    // Set pools
    let pool_long = Address::generate(&setup.env);
    let pool_short = Address::generate(&setup.env);
    setup.pair.set_pools(&setup.admin, &pool_long, &pool_short);

    // Mint tokens
    let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
    assert_eq!(minted_tokens, tokens_to_mint);

    // Redeem
    setup.pair.redeem(&user1, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #210)")]
fn test_redeem_invalid_without_pools() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 100_0000000;
    let tokens_to_redeem = 100_0000000;

    // Mint tokens
    // let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
    // assert_eq!(minted_tokens, tokens_to_mint);

    setup.pair.redeem(&user1, &tokens_to_redeem);
}

#[test]
fn test_redeem_full() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 100_0000000;
    let tokens_to_redeem = 100_0000000;

    // Set pools
    let pool_long = Address::generate(&setup.env);
    let pool_short = Address::generate(&setup.env);
    setup.pair.set_pools(&setup.admin, &pool_long, &pool_short);

    // Mint tokens
    let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
    assert_eq!(minted_tokens, tokens_to_mint);

    let redeemed_tokens = setup.pair.redeem(&user1, &tokens_to_redeem);
    assert_eq!(redeemed_tokens, tokens_to_redeem);

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

    // Collateral token went down by collateral per paif
    let collateral_info = setup.pair.get_collateral_info();
    assert_eq!(setup.token_usdc.balance(&user1), i128::MAX);
    assert_eq!(setup.token_usdc.balance(&setup.pair.address), 0);
}

#[test]
fn test_redeem_partial() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 100_0000000;
    let tokens_to_redeem = 50_0000000;

    // Set pools
    let pool_long = Address::generate(&setup.env);
    let pool_short = Address::generate(&setup.env);
    let pools: Vec<Address> = Vec::from_array(&setup.env, [pool_long, pool_short]);

    setup.pair.set_pools(&setup.admin, &pools);

    // Mint tokens
    let minted_tokens = setup.pair.mint(&user1, &tokens_to_mint);
    assert_eq!(minted_tokens, tokens_to_mint);

    let redeemed_tokens = setup.pair.redeem(&user1, &tokens_to_redeem);
    assert_eq!(redeemed_tokens, tokens_to_redeem);

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
            long_balance: tokens_to_mint - tokens_to_redeem,
            short_balance: tokens_to_mint - tokens_to_redeem,
        }
    );

    // Collateral token went down by collateral per paif
    let collateral_info = setup.pair.get_collateral_info();
    assert_eq!(setup.token_usdc.balance(&user1), i128::MAX);
    assert_eq!(setup.token_usdc.balance(&setup.pair.address), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #207)")]
fn test_update_oracle_price_bad_contract() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let bad_calculator = Address::generate(&setup.env);

    setup.pair.set_calculator(&setup.admin, &bad_calculator);

    setup.pair.update_oracle_price();
}

#[test]
fn test_update_oracle_price() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );

    let collateral_info_before = setup.pair.get_collateral_info();
    assert_eq!(collateral_info_before.collateral_percent_long, 5000); // 50%

    // Update oracle price
    let new_prices: Vec<i128> = Vec::from_array(&setup.env, [110_00000000000000, 1_00000000000000]);
    setup
        .oracle_client
        .set_price(&new_prices, &setup.env.ledger().timestamp());

    // Call pair
    setup.pair.update_oracle_price();

    assert_eq!(
        setup.pair.get_collateral_info().collateral_percent_long,
        6000
    ); // 60%
}

// Funding Period

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_update_funding_period_invalid() {
    let setup = Setup::default();
    setup.pair.update_funding_period(&setup.admin, &0);
}

#[test]
fn test_update_funding_period() {
    let setup = Setup::default();
    let new_funding_period = 1000000;

    setup
        .pair
        .update_funding_period(&setup.admin, &new_funding_period);

    let funding_info = setup.pair.get_funding_info();
    assert_eq!(funding_info.funding_period, new_funding_period);
}

// Funding Rate

#[test]
fn test_update_funding_rate() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let amount_to_deposit = 100_0000000;
    let desired_amounts = Vec::from_array(&setup.env, [amount_to_deposit, amount_to_deposit]);

    // Add liquidity to pools
    setup.pool_long.deposit(&user1, &desired_amounts, &0);
    setup.pool_short.deposit(&user1, &desired_amounts, &0);

    let new_funding_period = 1000000;

    let funding_info_before = setup.pair.get_funding_info();

    setup
        .pair
        .update_funding_period(&setup.admin, &new_funding_period);

    assert_eq!(
        setup.pair.get_funding_info().funding_period,
        new_funding_period
    );
}

#[test]
fn test_update_funding_rate_again_after_time() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[1].clone();
    let amount_to_deposit = 100_0000000;
    let desired_amounts = Vec::from_array(&setup.env, [amount_to_deposit, amount_to_deposit]);

    // Add liquidity to pools
    setup.pool_long.deposit(&user1, &desired_amounts, &0);
    setup.pool_short.deposit(&user1, &desired_amounts, &0);

    setup.pair.update_funding_rate(&setup.admin);

    jump(&setup.env, ONE_HOUR * 8);

    setup.pair.update_funding_rate(&setup.admin);

    // assert_eq!()
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_update_funding_rate_while_paused() {
    let setup = Setup::default();
    setup.pair.kill_update_funding(&setup.admin);

    setup.pair.update_funding_rate(&setup.admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_update_funding_rate_too_early() {
    let setup = Setup::default();

    setup.pair.update_funding_rate(&setup.admin);

    jump(&setup.env, 100);

    setup.pair.update_funding_rate(&setup.admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #210)")]
fn test_update_funding_rate_without_pools() {
    let setup = Setup::default();

    // Set pools
    let pool_long = Address::generate(&setup.env);
    let pool_short = Address::generate(&setup.env);
    setup.pair.set_pools(&setup.admin, &pool_long, &pool_short);

    setup.pair.update_funding_rate(&setup.admin);
}
