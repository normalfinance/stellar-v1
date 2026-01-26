#![allow(dead_code)]
#![cfg(test)]
extern crate std;

use crate::NormalOracleClient;
use sep_40_oracle::testutils::{Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol, Vec};
use std::vec;
use types::oracle::OracleSource;
use utils::test_utils::jump;

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
    pub(crate) normal_oracle: NormalOracleClient<'a>,
    pub(crate) reflector_addr: Address,
    pub(crate) reflector_client: MockPriceOracleClient<'a>,
    pub(crate) admin: Address,
    pub(crate) initial_asset_price: i128,
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

        let start_time = 1767285451; // e.ledger().timestamp();
        jump(&e, start_time);

        let users = Self::generate_random_users(&e, config.users_count);
        let admin = users[0].clone();

        // Setup oracle
        let asset_symbol = Symbol::new(&e, "SOL");
        let asset = MockAsset::Other(asset_symbol.clone());

        let (reflector_addr, reflector_client) = setup_price_feed_oracle(
            &e,
            &admin,
            &MockAsset::Other(Symbol::new(&e, "USD")),
            &Vec::from_array(&e, [asset.clone()]),
            14,
            300,
        );

        let initial_asset_price = 230_00000000000000;
        let prices: Vec<i128> = Vec::from_array(&e, [initial_asset_price, 1_00000000000000]);
        reflector_client.set_price(&prices, &start_time);

        // verify price data can be fetched
        let result_1 = reflector_client.lastprice(&asset.clone()).unwrap();
        assert_eq!(result_1.price, initial_asset_price);

        let normal_oracle = create_normal_oracle_contract(
            &e,
            &admin,
            &asset_symbol,
            &OracleSource::Reflector,
            &reflector_addr,
        );

        Self {
            env: e,
            users,
            normal_oracle,
            reflector_addr,
            reflector_client,
            admin,
            initial_asset_price,
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
    admin: &Address,
    asset: &Symbol,
    oracle_source: &OracleSource,
    oracle_addr: &Address,
) -> NormalOracleClient<'a> {
    let normal_oracle = NormalOracleClient::new(
        e,
        &e.register(
            crate::NormalOracle {},
            (admin, asset, oracle_source.clone(), oracle_addr),
        ),
    );
    normal_oracle
}

pub fn setup_price_feed_oracle<'a>(
    env: &Env,
    admin: &Address,
    base: &MockAsset,
    assets: &Vec<MockAsset>,
    decimals: u32,
    resolution: u32,
) -> (Address, MockPriceOracleClient<'a>) {
    let reflector_addr = env.register(MockPriceOracleWASM, ());
    let reflector_client = MockPriceOracleClient::new(env, &reflector_addr);
    reflector_client.set_data(admin, base, assets, &decimals, &resolution);
    (reflector_addr, reflector_client)
}
