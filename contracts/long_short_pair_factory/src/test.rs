#![cfg(test)]
extern crate std;

use crate::testutils;
use crate::testutils::long_short_pair::PairTokens;
use crate::testutils::Setup;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Symbol, Vec};
use types::pair::PairParams;

#[test]
fn test_deploy_contract() {
    let setup = Setup::default();
    let admin = setup.admin.clone();

    let calculator = Address::generate(&setup.env);
    let oracle = Address::generate(&setup.env);
    let collateral_token = Address::generate(&setup.env);

    let long_token = Address::generate(&setup.env);
    let short_token = Address::generate(&setup.env);

    // Deploy pair
    let params = PairParams {
        admin: admin.clone(),
        emergency_admin: setup.emergency_admin.clone(),
        pause_admin: setup.pause_admin.clone(),
        emergency_pause_admins: Vec::from_array(&setup.env, [setup.emergency_pause_admin.clone()]),
        operations_admin: setup.operations_admin.clone(),
        rewards_admin: setup.rewards_admin.clone(),
        system_fee_admin: setup.system_fee_admin.clone(),

        asset: Symbol::new(&setup.env, "Solana"),

        collateral_token: collateral_token.clone(),
        calculator,
        collateral_per_pair: 100_0000000,

        oracle,

        long_token: long_token.clone(),
        short_token: short_token.clone(),

        lower_bound: 0_0000000,
        upper_bound: 200_0000000,
    };

    let pair_address = setup.factory.deploy_pair_contract(&admin, &params);

    let pair_client = testutils::long_short_pair::Client::new(&setup.env, &pair_address);

    let pair_tokens = pair_client.get_tokens();
    assert_eq!(
        pair_tokens,
        PairTokens {
            long: long_token,
            short: short_token,
            collateral: collateral_token,
        }
    );
}
