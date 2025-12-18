#![allow(dead_code)]
#![cfg(test)]
extern crate std;
use crate::LongShortPairClient;
use access_control::constants::ADMIN_ACTIONS_DELAY;

use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol, Vec};
use std::vec;
use utils::test_utils::jump;

mod long_short_pair_calculator {
    soroban_sdk::contractimport!(file = "../../wasm/long_short_pair_calculator.wasm");
}

pub fn create_pair_calculator_contract<'a>(e: &Env) -> long_short_pair_calculator::Client<'a> {
    long_short_pair_calculator::Client::new(e, &e.register(long_short_pair_calculator::WASM, ()))
}

pub mod normal_oracle {
    soroban_sdk::contractimport!(file = "../../wasm/normal_oracle.wasm");
}

pub fn create_normal_oracle_contract<'a>(e: &Env) -> normal_oracle::Client<'a> {
    let factory = normal_oracle::Client::new(e, &e.register(normal_oracle::WASM, ()));
    factory
}

pub(crate) struct TestConfig {
    pub(crate) users_count: u32,
    pub(crate) mint_to_user: i128,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            users_count: 2,
            mint_to_user: 1000,
        }
    }
}

pub(crate) struct Setup<'a> {
    pub(crate) env: Env,

    // Tokens
    pub(crate) token_long: SorobanTokenClient<'a>,
    pub(crate) token_long_admin_client: SorobanTokenAdminClient<'a>,
    pub(crate) token_short: SorobanTokenClient<'a>,
    pub(crate) token_short_admin_client: SorobanTokenAdminClient<'a>,
    pub(crate) token_collateral: SorobanTokenClient<'a>,
    pub(crate) token_collateral_admin_client: SorobanTokenAdminClient<'a>,

    // Contracts
    pub(crate) pair: LongShortPairClient<'a>,
    pub(crate) pair_calculator: long_short_pair_calculator::Client<'a>,
    pub(crate) normal_oracle: normal_oracle::Client<'a>,

    // Oracle
    pub(crate) oracle_addr: Address,
    pub(crate) oracle_client: MockPriceOracleClient<'a>,

    // Addresses
    pub(crate) users: vec::Vec<Address>,
    pub(crate) admin: Address,
    pub(crate) emergency_admin: Address,
    pub(crate) operations_admin: Address,
    pub(crate) pause_admin: Address,
    pub(crate) emergency_pause_admin: Address,
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
        setup.mint_tokens_for_users(config.mint_to_user);
        setup
    }

    // Create users, token1, token2, and more...
    pub(crate) fn setup(config: &TestConfig) -> Self {
        let e: Env = Env::default();
        e.mock_all_auths();
        e.cost_estimate().budget().reset_unlimited();

        // Addresses
        let users = Self::generate_random_users(&e, config.users_count);
        let admin = users[0].clone();
        let emergency_admin = Address::generate(&e);
        let operations_admin = Address::generate(&e);
        let pause_admin = Address::generate(&e);
        let emergency_pause_admin = Address::generate(&e);

        // Tokens
        let token_long = create_token_contract(&e, &admin);
        let token_short = create_token_contract(&e, &admin);
        let token_collateral = create_token_contract(&e, &admin);

        let token_long_admin_client = get_token_admin_client(&e, &token_long.address.clone());
        let token_short_admin_client = get_token_admin_client(&e, &token_short.address.clone());
        let token_collateral_admin_client =
            get_token_admin_client(&e, &token_collateral.address.clone());

        // Oracle
        let sol_symbol = Symbol::new(&e, "SOL");
        let usdc_symbol = Symbol::new(&e, "USDC");
        let usd_sybmol = Symbol::new(&e, "USD");

        let sol_asset = MockAsset::Other(sol_symbol.clone());
        let usdc_asset = MockAsset::Other(usdc_symbol.clone());
        let usd_asset = MockAsset::Other(usd_sybmol);

        let (oracle_addr, oracle_client) = setup_price_feed_oracle(
            &e,
            &admin,
            &usd_asset,
            &Vec::from_array(&e, [sol_asset.clone(), usdc_asset.clone()]),
            14,
            300,
        );

        let prices_1: Vec<i128> = Vec::from_array(&e, [230_00000000000000, 1_00000000000000]);
        oracle_client.set_price(&prices_1, &start_time);

        let normal_oracle = create_normal_oracle_contract(&e);

        // Calculator
        let pair_calculator = create_pair_calculator_contract(&e);

        Self {
            env: e,

            // Tokens
            token_long,
            token_long_admin_client,
            token_short,
            token_short_admin_client,
            token_collateral,
            token_collateral_admin_client,

            // Contracts
            pair_calculator,
            pair,
            normal_oracle,

            // Oracle
            oracle_addr,
            oracle_client,

            // Addresses
            admin,
            users,
            emergency_admin,
            operations_admin,
            pause_admin,
            emergency_pause_admin,
        }
    }

    pub(crate) fn generate_random_users(e: &Env, users_count: u32) -> vec::Vec<Address> {
        let mut users = vec![];
        for _c in 0..users_count {
            users.push(Address::generate(e));
        }
        users
    }

    pub(crate) fn mint_tokens_for_users(&self, amount: i128) {
        for user in self.users.iter() {
            self.token_collateral_admin_client.mint(user, &amount);
            assert_eq!(self.token_collateral.balance(user), amount.clone());
        }
    }
}

pub(crate) fn create_token_contract<'a>(e: &Env, admin: &Address) -> SorobanTokenClient<'a> {
    SorobanTokenClient::new(
        e,
        &e.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

pub(crate) fn get_token_admin_client<'a>(
    e: &Env,
    address: &Address,
) -> SorobanTokenAdminClient<'a> {
    SorobanTokenAdminClient::new(e, address)
}

pub fn install_token_wasm(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(WASM)
}

pub fn setup_price_feed_oracle<'a>(
    env: &Env,
    admin: &Address,
    base: &MockAsset,
    assets: &Vec<MockAsset>,
    decimals: u32,
    resolution: u32,
) -> (Address, MockPriceOracleClient<'a>) {
    let oracle_addr = env.register(MockPriceOracleWASM, ());
    let oracle_client = MockPriceOracleClient::new(env, &oracle_addr);
    oracle_client.set_data(admin, base, assets, &decimals, &resolution);
    (oracle_addr, oracle_client)
}
