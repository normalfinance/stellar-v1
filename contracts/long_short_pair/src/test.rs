#![cfg(test)]
extern crate std;

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
fn initialize_initialize() {
    let setup = Setup::default();

    assert_eq!(
        setup
            .pair_calculator
            .get_parameters(&setup.env, &setup.pair.address),
        LinearLongShortPairParameters {
            upper_bound: 0,
            lower_bound: 0,
        }
    );

    let params = PairParams {
        admin: setup.admin,
        privileged_addrs: (setup.admin, setup.admin, setup.admin),
        tokens: Vec::from_array(
            &setup.env,
            [
                setup.token_long.address.clone(),
                setup.token_short.address.clone(),
            ],
        ),
        oracle: setup.normal_oracle.address,
        pool: Address::generate(&setup.env),
        pair_calculator: setup.pair_calculator.address,
        lower_bound: 50_000000,
        upper_bound: 300_0000000,
    };

    setup.pair.initialize(&params);

    // assert calc values
    assert_eq!(
        setup
            .pair_calculator
            .get_parameters(&setup.env, &setup.pair.address),
        LinearLongShortPairParameters {
            upper_bound: 50_000000,
            lower_bound: 300_0000000,
        }
    );

    //
}

#[test]
#[should_panic(expected = "Error(Contract, #201)")]
fn initialize_already_initialized() {
    let setup = Setup::default();

    let params = PairParams {
        admin: setup.admin,
        privileged_addrs: (setup.admin, setup.admin, setup.admin),
        tokens: Vec::from_array(
            &setup.env,
            [
                setup.token_long.address.clone(),
                setup.token_short.address.clone(),
            ],
        ),
        oracle: setup.normal_oracle.address,
        pool: Address::generate(&setup.env),
        pair_calculator: setup.pair_calculator.address,
        lower_bound: 50_000000,
        upper_bound: 300_0000000,
    };

    setup.pair.initialize(&params);
}

#[test]
fn test_mint() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
    let user1 = setup.users[0].clone();
    let tokens_to_create = 100_0000000;

    let created_tokens = setup.pair.mint(&user1, &tokens_to_create);

    let collateral_info = setup.pair.get_collateral_info();

    // user collateral token went down by collateral per paif
    assert_eq!(
        setup.token_collateral.balance(&user1),
        i128::MAX - (collateral_info.collateral_per_pair as i128)
    );

    assert_eq!(
        setup.token_collateral.balance(&setup.pair.address),
        collateral_info.collateral_per_pair as i128
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
    let created_tokens = setup.pair.mint(&user1, &tokens_to_create);

    let collateral_info = setup.pair.get_collateral_info();

    // user collateral token went down by collateral per paif
    assert_eq!(
        setup.token_collateral.balance(&user1),
        i128::MAX - (collateral_info.collateral_per_pair as i128)
    );

    assert_eq!(
        setup.token_collateral.balance(&setup.pair.address),
        collateral_info.collateral_per_pair as i128
    );

    // Long and short tokens are minted to the user
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_create);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_create);

    // Response equals request amount
    assert_eq!(created_tokens, tokens_to_create);
}

#[test]
fn test_update_oracle_price() {
    let setup = Setup::new_with_config(
        &(TestConfig {
            mint_to_user: i128::MAX,
            ..TestConfig::default()
        }),
    );
}

// Funding Period

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

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_update_funding_period_invalid() {
    let setup = Setup::default();
    setup.pair.update_funding_period(&setup.admin, &0);
}

// Funding Rate

#[test]
fn test_update_funding_rate() {
    let setup = Setup::default();
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
    let setup = Setup::default();

    setup.pair.update_funding_rate(&setup.admin);

    jump(&setup.env, ONE_HOUR * 8);

    setup.pair.update_funding_rate(&setup.admin);
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
