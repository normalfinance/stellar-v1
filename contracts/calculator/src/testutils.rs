#![allow(dead_code)]
#![cfg(test)]
extern crate std;
use crate::CalculatorClient;

use soroban_sdk::{testutils::Address as _, Address, Env};
use std::vec;

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
    pub(crate) calculator: CalculatorClient<'a>,
    pub(crate) admin: Address,
    pub(crate) pair: Address,
}

impl Default for Setup<'_> {
    // Create setup from default config and mint tokens for all users & set rewards config
    fn default() -> Self {
        let default_config = TestConfig::default();
        Self::new_with_config(&default_config)
    }
}

impl Setup<'_> {
    // Create setup from config and mint tokens for all users
    pub(crate) fn new_with_config(config: &TestConfig) -> Self {
        let setup = Self::setup(config);
        setup
    }

    // Create users, token1, token2, reward token, lp token
    //
    // Mint reward token (1_000_000_0000000) & approve for liquidity_pool token
    pub(crate) fn setup(config: &TestConfig) -> Self {
        let e: Env = Env::default();
        e.mock_all_auths();
        e.cost_estimate().budget().reset_unlimited();

        let users = Self::generate_random_users(&e, config.users_count);
        let admin = users[0].clone();
        let pair = Address::generate(&e);

        let calculator = create_calculator_contract(&e);

        Self {
            env: e,
            users,
            calculator,
            admin,
            pair,
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

pub fn create_calculator_contract<'a>(e: &Env) -> CalculatorClient<'a> {
    let calculator = CalculatorClient::new(e, &e.register(crate::Calculator {}, ()));
    calculator
}
