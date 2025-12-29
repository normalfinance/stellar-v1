#![cfg(test)]
extern crate std;

use crate::testutils::{create_token_contract, get_token_admin_client, Setup};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::TokenClient;
use soroban_sdk::{vec, Address, Vec};

/**
 * Tests to write
 * -
 *
 */

#[test]
fn test_integration() {
    let setup = Setup::default();

    // create tokens
    let mut tokens = std::vec![
        create_token_contract(&setup.env, &setup.admin).address,
        create_token_contract(&setup.env, &setup.admin).address,
        create_token_contract(&setup.env, &setup.admin).address
    ];
    tokens.sort();
    let usdc = TokenClient::new(&setup.env, &tokens[0]);
    let long = TokenClient::new(&setup.env, &tokens[1]);
    let short = TokenClient::new(&setup.env, &tokens[2]);

    let usdc_admin = get_token_admin_client(&setup.env, &usdc.address);
    let long_admin = get_token_admin_client(&setup.env, &long.address);
    let short_admin = get_token_admin_client(&setup.env, &short.address);

    // setup long short pair
    let (pair, pair_address) = setup.deploy_long_short_pair();

    // deploy pools
    let (long_pool, long_pool_hash) = setup.deploy_synthetic_pool(&long.address, &usdc.address, 30);
    let (short_pool, short_pool_hash) =
        setup.deploy_synthetic_pool(&short.address, &usdc.address, 30);

    usdc_admin.mint(&setup.admin, &1_000_0000000);

    // mint long/short
    pair.mint(user, &10_0000000);

    // add liquidity
    long_pool.deposit(
        &setup.admin,
        &Vec::from_array(&setup.env, [344_000_0000000, 100_000_0000000]),
        &0,
    );
    short_pool.deposit(
        &setup.admin,
        &Vec::from_array(&setup.env, [344_000_0000000, 100_000_0000000]),
        &0,
    );

    // swaps

    setup.router.swap(
        user, tokens, token_in, token_out, pool_index, in_amount, out_min,
    );

    setup.router.swap(
        user, tokens, token_in, token_out, pool_index, in_amount, out_min,
    );
}
