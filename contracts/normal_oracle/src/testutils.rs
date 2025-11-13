#![allow(dead_code)]
#![cfg(test)]
extern crate std;

use crate::NormalOracleClient;
use sep_40_oracle::testutils::{ Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM };
use soroban_sdk::{ testutils::Address as _, Address, Env, Symbol, Vec };
use std::vec;
use utils::test_utils::jump;

pub(crate) struct TestConfig {
    pub(crate) users_count: u32,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            users_count: 2,
        }
    }
}

pub(crate) struct Setup<'a> {
    pub(crate) env: Env,
    pub(crate) users: vec::Vec<Address>,
    pub(crate) normal_oracle: NormalOracleClient<'a>,
    pub(crate) oracle_addr: Address,
    pub(crate) oracle_client: MockPriceOracleClient<'a>,
    pub(crate) admin: Address,
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

        // Setup oracle
        let sol_symbol = Symbol::new(&e, "SOL");
        let usd_sybmol = Symbol::new(&e, "USD");

        let sol_asset = MockAsset::Other(sol_symbol.clone());
        let usd_asset = MockAsset::Other(usd_sybmol);

        let (oracle_addr, oracle_client) = setup_price_feed_oracle(
            &e,
            &admin,
            &usd_asset,
            &Vec::from_array(&e, [sol_asset.clone()]),
            14,
            300
        );

        let prices_1: Vec<i128> = Vec::from_array(&e, [230_00000000000000, 1_00000000000000]);
        oracle_client.set_price(&prices_1, &start_time);

        let calculator = Address::generate(&e);

        let normal_oracle = create_normal_oracle_contract(&e, &sol_symbol, &oracle_addr);

        Self {
            env: e,
            users,
            normal_oracle,
            oracle_addr,
            oracle_client,
            admin,
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

pub fn create_normal_oracle_contract<'a>(
    e: &Env,
    asset: &Symbol,
    oracle: &Address
) -> NormalOracleClient<'a> {
    let normal_oracle = NormalOracleClient::new(
        e,
        &e.register(crate::NormalOracle {}, (asset, oracle))
    );
    normal_oracle
}

pub fn setup_price_feed_oracle<'a>(
    env: &Env,
    admin: &Address,
    base: &MockAsset,
    assets: &Vec<MockAsset>,
    decimals: u32,
    resolution: u32
) -> (Address, MockPriceOracleClient<'a>) {
    let oracle_addr = env.register(MockPriceOracleWASM, ());
    let oracle_client = MockPriceOracleClient::new(env, &oracle_addr);
    oracle_client.set_data(admin, base, assets, &decimals, &resolution);
    (oracle_addr, oracle_client)
}
