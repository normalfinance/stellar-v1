#![cfg(test)]
extern crate std;

use crate::testutils::{
    create_plane_contract, create_token_contract, get_token_admin_client, install_token_wasm,
    Setup, TestConfig,
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
use utils::test_utils::{install_dummy_wasm, jump};

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn initialize_already_initialized() {
    let setup = Setup::default();

    let users = Setup::generate_random_users(&setup.env, 3);
    let token1 = create_token_contract(&setup.env, &users[1]);
    let token2 = create_token_contract(&setup.env, &users[2]);

    setup.liq_pool.initialize(
        &users[0],
        &(
            users[0].clone(),
            users[0].clone(),
            users[0].clone(),
            users[0].clone(),
            Vec::from_array(&setup.env, [users[0].clone()]),
            users[0].clone(),
        ),
        &users[0],
        &setup.oracle_addr,
        &install_token_wasm(&setup.env),
        &Vec::from_array(&setup.env, [token1.address.clone(), token2.address.clone()]),
        &(10_u32, 5000_u32),
        &(setup.sol_symbol, setup.usdc_symbol),
    );
}

#[test]
fn test_create() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[0].clone();
    let tokens_to_create = 100_0000000;

    let created_tokens = setup.pair.create(&user1, &tokens_to_create);

    let collateral_per_pair = setup.pair.get_();

    // user collateral token went down by collateral per paif
    assert_eq!(
        setup.token_collateral.balance(&user1),
        i128::MAX - collateral_per_pair
    );

    assert_eq!(
        setup.token_collateral.balance(&setup.pair.address),
        collateral_per_pair
    );

    // Long and short tokens are minted to the user
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_create);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_create);

    // Response equals request amount
    assert_eq!(created_tokens, tokens_to_create);
}

#[test]
fn test_redeem() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[0].clone();
    let tokens_to_create = 100_0000000;

    // TODO:
    let created_tokens = setup.pair.create(&user1, &tokens_to_create);

    let collateral_per_pair = setup.pair.get_();

    // user collateral token went down by collateral per paif
    assert_eq!(
        setup.token_collateral.balance(&user1),
        i128::MAX - collateral_per_pair
    );

    assert_eq!(
        setup.token_collateral.balance(&setup.pair.address),
        collateral_per_pair
    );

    // Long and short tokens are minted to the user
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_create);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_create);

    // Response equals request amount
    assert_eq!(created_tokens, tokens_to_create);
}

#[test]
fn test_settle() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[0].clone();
    let tokens_to_create = 100_0000000;

    let created_tokens = setup.pair.create(&user1, &tokens_to_create);

    // TODO:
    let collateral_per_pair = setup.pair.get_();

    // user collateral token went down by collateral per paif
    assert_eq!(
        setup.token_collateral.balance(&user1),
        i128::MAX - collateral_per_pair
    );

    assert_eq!(
        setup.token_collateral.balance(&setup.pair.address),
        collateral_per_pair
    );

    // Long and short tokens are minted to the user
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_create);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_create);

    // Response equals request amount
    assert_eq!(created_tokens, tokens_to_create);
}
