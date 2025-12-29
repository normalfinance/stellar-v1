#![allow(dead_code)]
#![cfg(test)]
extern crate std;
use crate::LongShortPairClient;

use access_control::constants::ADMIN_ACTIONS_DELAY;
use sep_40_oracle::testutils::{Asset as MockAsset, MockPriceOracleClient};
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol, Vec};
use std::vec;
use types::pair::{LinearLongShortPairParameters, PairParams};
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

pub mod plane {
    soroban_sdk::contractimport!(file = "../../wasm/liquidity_pool_plane.wasm");
}

pub(crate) fn create_plane_contract<'a>(e: &Env) -> plane::Client<'a> {
    plane::Client::new(e, &e.register(plane::WASM, ()))
}

pub mod pool {
    soroban_sdk::contractimport!(file = "../../wasm/liquidity_pool.wasm");
}

pub(crate) fn create_pool_contract<'a>(e: &Env) -> pool::Client<'a> {
    pool::Client::new(e, &e.register(pool::WASM, ()))
}

pub mod token_pool {
    soroban_sdk::contractimport!(file = "../../wasm/token_pool.wasm");
}

pub fn create_pair_contract<'a>(e: &Env) -> LongShortPairClient<'a> {
    let pair = LongShortPairClient::new(e, &e.register(normal_oracle::WASM, ()));
    pair
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
    pub(crate) token_usdc: SorobanTokenClient<'a>,
    pub(crate) token_usdc_admin_client: SorobanTokenAdminClient<'a>,

    // Contracts
    pub(crate) pair: LongShortPairClient<'a>,
    pub(crate) pair_calculator: long_short_pair_calculator::Client<'a>,
    pub(crate) oracle: normal_oracle::Client<'a>,
    pub(crate) plane: plane::Client<'a>,
    pub(crate) pool_long: pool::Client<'a>,
    pub(crate) pool_short: pool::Client<'a>,

    // Oracle
    pub(crate) reflector_addr: Address,
    pub(crate) reflector_client: MockPriceOracleClient<'a>,

    // Addresses
    pub(crate) users: vec::Vec<Address>,
    pub(crate) admin: Address,
    pub(crate) emergency_admin: Address,
    pub(crate) pause_admin: Address,

    // Addresses - Pools
    pub(crate) rewards_admin: Address,
    pub(crate) operations_admin: Address,
    pub(crate) emergency_pause_admin: Address,
    pub(crate) system_fee_admin: Address,
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

        let start_time = 0;

        // Addresses
        let users = Self::generate_random_users(&e, config.users_count);
        let admin = users[0].clone();
        let emergency_admin = Address::generate(&e);
        let pause_admin = Address::generate(&e);

        let emergency_admin = Address::generate(&e);
        let rewards_admin = Address::generate(&e);
        let operations_admin = Address::generate(&e);
        let pause_admin = Address::generate(&e);
        let system_fee_admin = Address::generate(&e);
        let emergency_pause_admin = Address::generate(&e);

        // Tokens
        let token_long = create_token_contract(&e, &admin);
        let token_short = create_token_contract(&e, &admin);
        let token_usdc = create_token_contract(&e, &admin);

        let token_long_admin_client = get_token_admin_client(&e, &token_long.address.clone());
        let token_short_admin_client = get_token_admin_client(&e, &token_short.address.clone());
        let token_usdc_admin_client = get_token_admin_client(&e, &token_usdc.address.clone());

        // Pools
        let plane = create_plane_contract(&e);

        let config_storage = deploy_config_storage(&e, &admin, &emergency_admin);
        let router = Address::generate(&e);

        // Long Pool
        let pool_long = create_liqpool_contract(
            &e,
            &admin,
            &router,
            &install_token_wasm(&e),
            &Vec::from_array(&e, [token_long.address.clone(), token_usdc.address.clone()]),
            &tok.address,
            30,
            &plane.address,
            &config_storage.address,
        );

        pool_long.set_privileged_addrs(
            &admin,
            &rewards_admin,
            &operations_admin,
            &pause_admin,
            &Vec::from_array(&e, [emergency_pause_admin.clone()]),
            &system_fee_admin,
        );

        let emergency_admin = Address::generate(&e);
        pool_long.commit_transfer_ownership(
            &admin,
            &Symbol::new(&e, "EmergencyAdmin"),
            &emergency_admin,
        );
        jump(&e, ADMIN_ACTIONS_DELAY + 1); // delay is mandatory since emergency admin was set during initialization
        pool_long.apply_transfer_ownership(&admin, &Symbol::new(&e, "EmergencyAdmin"));

        pool_long.set_protocol_fee_fraction(&admin, &5000);

        let token_pool_long = token_pool::Client::new(&e, &pool_long.share_id());

        // Short Pool
        let pool_short = create_liqpool_contract(
            &e,
            &admin,
            &router,
            &install_token_wasm(&e),
            &Vec::from_array(
                &e,
                [token_short.address.clone(), token_usdc.address.clone()],
            ),
            &token_usdc.address,
            30,
            &plane.address,
            &config_storage.address,
        );

        pool_short.set_privileged_addrs(
            &admin,
            &rewards_admin,
            &operations_admin,
            &pause_admin,
            &Vec::from_array(&e, [emergency_pause_admin.clone()]),
            &system_fee_admin,
        );

        let emergency_admin = Address::generate(&e);
        pool_short.commit_transfer_ownership(
            &admin,
            &Symbol::new(&e, "EmergencyAdmin"),
            &emergency_admin,
        );
        jump(&e, ADMIN_ACTIONS_DELAY + 1); // delay is mandatory since emergency admin was set during initialization
        pool_short.apply_transfer_ownership(&admin, &Symbol::new(&e, "EmergencyAdmin"));

        pool_short.set_protocol_fee_fraction(&admin, &5000);

        let token_pool_short = token_pool::Client::new(&e, &pool_short.share_id());

        // Add liquidity to pool
        let amount_to_deposit = 100_0000000;
        let desired_amounts = Vec::from_array(&e, [amount_to_deposit, amount_to_deposit]);

        // Add liquidity to pools
        pool_long.deposit(&user1, &desired_amounts, &0);
        pool_short.deposit(&user1, &desired_amounts, &0);

        // Setup Oracle
        let sol_symbol = Symbol::new(&e, "SOL");
        let usdc_symbol = Symbol::new(&e, "USDC");

        let sol_asset = MockAsset::Other(sol_symbol.clone());
        let usdc_asset = MockAsset::Other(usdc_symbol.clone());
        let usd_asset = MockAsset::Other(Symbol::new(&e, "USD"));

        let (reflector_addr, reflector_client) = setup_reflector_price_feed_oracle(
            &e,
            &admin,
            &usd_asset,
            &Vec::from_array(&e, [sol_asset.clone(), usdc_asset.clone()]),
            14,
            300,
        );

        let solana_price = 100_00000000000000;
        let usdc_price = 1_00000000000000;
        let prices_initial: Vec<i128> = Vec::from_array(&e, [solana_price, usdc_price]);
        reflector_client.set_price(&prices_initial, &start_time);

        let oracle = create_normal_oracle_contract(&e);

        // Setup Calculator
        let pair_calculator = create_pair_calculator_contract(&e);

        //Setup Long Short Pair
        let pair = create_pair_contract(&e);

        // Add pools to the Pair
        let pools: Vec<Address> = Vec::from_array(&e, [pool_long.address, pool_short.address]);
        pair.set_pools(&admin, &pools);

        assert_eq!(
            pair_calculator.get_parameters(&e, &pair.address),
            LinearLongShortPairParameters {
                upper_bound: 0,
                lower_bound: 0,
            }
        );

        // Initialize Pair
        pair.initialize(
            &(PairParams {
                admin: admin,
                privileged_addrs: (emergency_admin, pause_admin),
                tokens: Vec::from_array(
                    &e,
                    [token_long.address.clone(), token_short.address.clone()],
                ),
                oracle: oracle.address,
                pool_plane: Address::generate(&e),
                pair_calculator: pair_calculator.address,
                lower_bound: 50_000000,
                upper_bound: 300_0000000,
            }),
        );

        // Ensure calculator boundaries were updated during initialization
        assert_eq!(
            pair_calculator.get_parameters(&e, &pair.address),
            LinearLongShortPairParameters {
                upper_bound: 50_000000,
                lower_bound: 300_0000000,
            }
        );

        Self {
            env: e,

            // Tokens
            token_long,
            token_long_admin_client,
            token_short,
            token_short_admin_client,
            token_usdc,
            token_usdc_admin_client,

            // Contracts
            pair_calculator,
            pair,
            oracle,
            plane,
            pool_long,
            pool_short,

            // Oracle
            reflector_addr,
            reflector_client,

            // Addresses
            admin,
            users,
            emergency_admin,
            pause_admin,
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

pub fn create_liqpool_contract<'a>(
    e: &Env,
    admin: &Address,
    router: &Address,
    token_wasm_hash: &BytesN<32>,
    tokens: &Vec<Address>,
    reward_token: &Address,
    fee_fraction: u32,
    plane: &Address,
    config_storage: &Address,
) -> pool::Client<'a> {
    let liqpool = pool::Client::new(e, &e.register(pool::WASM, ()));
    liqpool.initialize_all(
        &admin,
        &(
            admin.clone(),
            admin.clone(),
            admin.clone(),
            admin.clone(),
            Vec::from_array(e, [admin.clone()]),
            admin.clone(),
        ),
        router,
        token_wasm_hash,
        tokens,
        &(
            fee_fraction,
            5000, // 50% protocol fee fraction
        ),
        reward_token,
        plane,
        config_storage,
    );
    liqpool
}
