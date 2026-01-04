#![allow(dead_code)]
#![cfg(test)]
extern crate std;
use crate::testutils::long_short_pair_calculator::LinearLongShortPairParameters;
use crate::testutils::normal_oracle::OracleSource;
use crate::LongShortPairClient;

use sep_40_oracle::testutils::{Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol, Vec};
use soroban_sdk::{IntoVal, String};
use std::vec;
use token_pair::token_contract::{Client as PairTokenClient, WASM};
use types::oracle::OraclePriceData;
use types::pair::PairParams;
use utils::test_utils::jump;

pub mod long_short_pair_calculator {
    soroban_sdk::contractimport!(file = "../../wasm/long_short_pair_calculator.wasm");
}

pub fn create_pair_calculator_contract<'a>(e: &Env) -> long_short_pair_calculator::Client<'a> {
    long_short_pair_calculator::Client::new(e, &e.register(long_short_pair_calculator::WASM, ()))
}

pub mod normal_oracle {
    soroban_sdk::contractimport!(file = "../../wasm/normal_oracle.wasm");
}

pub fn create_normal_oracle_contract<'a>(e: &Env) -> normal_oracle::Client<'a> {
    normal_oracle::Client::new(e, &e.register(normal_oracle::WASM, ()))
}

pub fn create_pair_contract<'a>(e: &Env) -> LongShortPairClient<'a> {
    LongShortPairClient::new(e, &e.register(crate::LongShortPair {}, ()))
}

pub(crate) struct TestConfig {
    pub(crate) users_count: u32,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig { users_count: 3 }
    }
}

pub(crate) struct Setup<'a> {
    pub(crate) env: Env,

    // Tokens
    pub(crate) token_long: PairTokenClient<'a>,
    // pub(crate) token_long_admin_client: SorobanTokenAdminClient<'a>,
    pub(crate) token_short: PairTokenClient<'a>,
    // pub(crate) token_short_admin_client: SorobanTokenAdminClient<'a>,
    pub(crate) token_usdc: SorobanTokenClient<'a>,
    pub(crate) token_usdc_admin_client: SorobanTokenAdminClient<'a>,

    // Contracts
    pub(crate) pair: LongShortPairClient<'a>,
    pub(crate) pair_calculator: long_short_pair_calculator::Client<'a>,
    pub(crate) oracle: normal_oracle::Client<'a>,

    // Oracle
    pub(crate) reflector_addr: Address,
    pub(crate) reflector_client: MockPriceOracleClient<'a>,

    // Addresses
    pub(crate) users: vec::Vec<Address>,
    pub(crate) admin: Address,
    pub(crate) emergency_admin: Address,
    pub(crate) pause_admin: Address,

    // Other
    pub(crate) pool_init_args: Vec<u128>,
    pub(crate) pool_type: Symbol,
    pub(crate) solana: Symbol,
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

    // Create users, token1, token2, and more...
    pub(crate) fn setup(config: &TestConfig) -> Self {
        let e: Env = Env::default();
        e.mock_all_auths();
        e.cost_estimate().budget().reset_unlimited();

        let start_time = 1767285451; // e.ledger().timestamp();
        jump(&e, start_time);

        // Addresses
        let users = Self::generate_random_users(&e, config.users_count);
        let admin = users[0].clone();
        let emergency_admin = Address::generate(&e);
        let pause_admin = Address::generate(&e);

        // Collateral Token
        let token_usdc = create_token_contract(&e, &admin);
        let token_usdc_admin_client = get_token_admin_client(&e, &token_usdc.address.clone());

        // Setup Oracle
        let sol_symbol = Symbol::new(&e, "SOL");
        let solana = MockAsset::Other(sol_symbol.clone());

        let (reflector_addr, reflector_client) = setup_reflector_price_feed_oracle(
            &e,
            &admin,
            &MockAsset::Other(Symbol::new(&e, "USD")),
            &Vec::from_array(
                &e,
                [solana.clone(), MockAsset::Other(Symbol::new(&e, "USDC"))],
            ),
            14,
            300,
        );

        let solana_price = 100_00000000000000;
        let usdc_price = 1_00000000000000;
        let prices_initial: Vec<i128> = Vec::from_array(&e, [solana_price, usdc_price]);
        reflector_client.set_price(&prices_initial, &start_time);

        // verify price data can be fetched
        let result_1 = reflector_client.lastprice(&solana.clone()).unwrap();
        assert_eq!(result_1.price, prices_initial.get_unchecked(0));

        // Create Normal Oracle
        let oracle = create_normal_oracle_contract(&e);
        oracle.initialize(
            &admin,
            &sol_symbol.clone(),
            &OracleSource::Reflector,
            &reflector_addr,
        );

        // Setup Calculator
        let pair_calculator = create_pair_calculator_contract(&e);

        // Setup Long Short Pair
        let pair = create_pair_contract(&e);

        // Initialize Pair
        pair.initialize(
            &(PairParams {
                admin: admin.clone(),
                privileged_addrs: (emergency_admin.clone(), pause_admin.clone()),
                asset: sol_symbol.clone(),
                collateral_token: token_usdc.address.clone(),
                pair_token_wasm_hash: install_pair_token_wasm(&e),
                oracle: oracle.address.clone(),
                pair_calculator: pair_calculator.address.clone(),
                collateral_per_pair: 100_0000000,
                lower_bound: 50_0000000,
                upper_bound: 150_0000000,
            }),
        );

        let pair_tokens = pair.get_tokens();
        let token_long = PairTokenClient::new(&e, &pair_tokens.get(0).unwrap());
        let token_short = PairTokenClient::new(&e, &pair_tokens.get(1).unwrap());

        // Ensure calculator boundaries were updated during initialization
        assert_eq!(
            pair_calculator.get_params(&pair.address),
            LinearLongShortPairParameters {
                lower_bound: 50_0000000,
                upper_bound: 150_0000000,
            }
        );

        Self {
            env: e,

            // Tokens
            token_long,
            // token_long_admin_client,
            token_short,
            // token_short_admin_client,
            token_usdc,
            token_usdc_admin_client,

            // Contracts
            pair_calculator,
            pair,
            oracle,

            // Oracle
            reflector_addr,
            reflector_client,

            // Addresses
            admin,
            users,
            emergency_admin,
            pause_admin,

            // Other
            pool_type,
            pool_init_args,
            solana: sol_symbol,
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

pub fn setup_reflector_price_feed_oracle<'a>(
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

pub fn install_pair_token_wasm(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(WASM)
}
