#![allow(dead_code)]
#![cfg(test)]
extern crate std;

use crate::LongShortPairFactoryClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};
use std::vec;

pub mod long_short_pair {
    soroban_sdk::contractimport!(file = "../../wasm/long_short_pair.wasm");
}

pub fn install_pair_hash(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(long_short_pair::WASM)
}

pub(crate) struct TestConfig {
    pub(crate) users_count: u32,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig { users_count: 2 }
    }
}

pub(crate) struct Setup<'a> {
    pub(crate) env: Env,
    pub(crate) users: vec::Vec<Address>,

    pub(crate) factory: LongShortPairFactoryClient<'a>,

    pub(crate) admin: Address,
    pub(crate) emergency_admin: Address,
    pub(crate) rewards_admin: Address,
    pub(crate) operations_admin: Address,
    pub(crate) pause_admin: Address,
    pub(crate) emergency_pause_admin: Address,
    pub(crate) system_fee_admin: Address,
}

impl Default for Setup<'_> {
    // Create setup from default config
    fn default() -> Self {
        let default_config = TestConfig::default();
        Self::new_with_config(&default_config)
    }
}

impl Setup<'_> {
    // Create setup from config
    pub(crate) fn new_with_config(config: &TestConfig) -> Self {
        let setup = Self::setup(config);
        setup
    }

    pub(crate) fn setup(config: &TestConfig) -> Self {
        let e: Env = Env::default();
        e.mock_all_auths();
        e.cost_estimate().budget().reset_unlimited();

        let users = Self::generate_random_users(&e, config.users_count);
        let admin = users[0].clone();
        let emergency_admin = Address::generate(&e);
        let rewards_admin = Address::generate(&e);
        let operations_admin = Address::generate(&e);
        let pause_admin = Address::generate(&e);
        let emergency_pause_admin = Address::generate(&e);
        let system_fee_admin = Address::generate(&e);

        let pair_hash = install_pair_hash(&e);

        let factory = create_factory_contract(
            &e,
            &admin,
            &emergency_admin,
            &rewards_admin,
            &operations_admin,
            &system_fee_admin,
            &pair_hash,
        );

        Self {
            env: e,
            users,
            factory,
            admin,
            emergency_admin,
            rewards_admin,
            operations_admin,
            pause_admin,
            emergency_pause_admin,
            system_fee_admin,
        }
    }

    pub(crate) fn generate_random_users(e: &Env, users_count: u32) -> vec::Vec<Address> {
        let mut users = vec![];
        for _c in 0..users_count {
            users.push(Address::generate(e));
        }
        users
    }
}

pub fn create_factory_contract<'a>(
    e: &Env,
    admin: &Address,
    emergency_admin: &Address,
    rewards_admin: &Address,
    operations_admin: &Address,
    system_fee_admin: &Address,
    pair_contract_wasm: &BytesN<32>,
) -> LongShortPairFactoryClient<'a> {
    let factory = LongShortPairFactoryClient::new(
        e,
        &e.register(
            crate::LongShortPairFactory {},
            (
                admin,
                emergency_admin,
                rewards_admin,
                operations_admin,
                system_fee_admin,
                pair_contract_wasm,
            ),
        ),
    );
    factory
}
