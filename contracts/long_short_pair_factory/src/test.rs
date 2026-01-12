#![cfg(test)]
extern crate std;

use crate::testutils;
use crate::testutils::Setup;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Symbol};

#[test]
fn test_deploy_contract() {
    let setup = Setup::default();
    let admin = setup.admin.clone();

    let pair_calculator = Address::generate(&setup.env);
    let oracle = Address::generate(&setup.env);
    let collateral_token = Address::generate(&setup.env);

    let long_token = Address::generate(&setup.env);
    let short_token = Address::generate(&setup.env);

    // Deploy pair
    let pair_address = setup
        .factory
        .deploy_pair_contract(&admin, &Symbol::new(&setup.env, "Solana"));

    let pair_client = testutils::long_short_pair::Client::new(&setup.env, &pair_address);

    let params = testutils::long_short_pair::PairParams {
        admin: admin.clone(),
        asset: Symbol::new(&setup.env, "Solana"),

        collateral_token,
        calculator,
        collateral_per_pair: 100_0000000,

        oracle,

        long_token,
        short_token,

        lower_bound: 0_0000000,
        upper_bound: 200_0000000,
    };
    pair_client.initialize(&params);

    let pair_tokens = pair_client.get_tokens();
    assert_eq!(pair_tokens.len(), 2);
}
