#![allow(dead_code)]
#![cfg(test)]
extern crate std;
use crate::LongShortPairClient;
use access_control::constants::ADMIN_ACTIONS_DELAY;

use sep_40_oracle::testutils::{Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol, Vec};
use std::vec;
use utils::test_utils::jump;

pub(crate) struct TestConfig {
    pub(crate) users_count: u32,
    pub(crate) mint_to_user: i128,
    pub(crate) liq_pool_fee: u32,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            users_count: 2,
            mint_to_user: 1000,
            liq_pool_fee: 30,
        }
    }
}

pub(crate) struct Setup<'a> {
    pub(crate) env: Env,
    pub(crate) users: vec::Vec<Address>,
    pub(crate) token_long: SorobanTokenClient<'a>,
    pub(crate) token_long_admin_client: SorobanTokenAdminClient<'a>,
    pub(crate) token_short: SorobanTokenClient<'a>,
    pub(crate) token_short_admin_client: SorobanTokenAdminClient<'a>,
    pub(crate) token_collateral: SorobanTokenClient<'a>,
    pub(crate) token_collateral_admin_client: SorobanTokenAdminClient<'a>,

    pub(crate) token_factory: PoolClient<'a>,
    pub(crate) calculator: CalculatorClient<'a>,
    pub(crate) pair: LongShortPairClient<'a>,

    pub(crate) oracle_addr: Address,
    pub(crate) oracle_client: MockPriceOracleClient<'a>,

    pub(crate) admin: Address,
    pub(crate) emergency_admin: Address,
    pub(crate) rewards_admin: Address,
    pub(crate) operations_admin: Address,
    pub(crate) pause_admin: Address,
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

    // Create users, token1, token2, reward token, lp token
    //
    // Mint reward token (1_000_000_0000000) & approve for liquidity_pool token
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
        let system_fee_admin = Address::generate(&e);
        let emergency_pause_admin = Address::generate(&e);

        let token_long = create_token_contract(&e, &admin);
        let token_short = create_token_contract(&e, &admin);
        let token_collateral = create_token_contract(&e, &admin);

        let token_long_admin_client = get_token_admin_client(&e, &token_long.address.clone());
        let token_short_admin_client = get_token_admin_client(&e, &token_short.address.clone());
        let token_collateral_admin_client =
            get_token_admin_client(&e, &token_collateral.address.clone());

        // Setup oracle
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

        let calculator = Address::generate(&e);

        let pair = create_pair_contract(
            &e,
            &admin,
            &Vec::from_array(
                &e,
                [
                    token1.address.clone(),
                    token2.address.clone(),
                    token2.address.clone(),
                ],
            ),
            &oracle_addr,
            &calculator,
        );

        Self {
            env: e,
            users,
            token_long,
            token_long_admin_client,
            token_short,
            token_short_admin_client,
            token_collateral,
            token_collateral_admin_client,
            token_factory,
            calculator,
            pair,
            oracle_addr,
            oracle_client,
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

    pub(crate) fn mint_tokens_for_users(&self, amount: i128) {
        for user in self.users.iter() {
            self.token1_admin_client.mint(user, &amount);
            assert_eq!(self.token1.balance(user), amount.clone());

            self.token2_admin_client.mint(user, &amount);
            assert_eq!(self.token2.balance(user), amount.clone());
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

pub(crate) fn create_plane_contract<'a>(e: &Env) -> PoolPlaneClient<'a> {
    PoolPlaneClient::new(e, &e.register(pool_plane::WASM, ()))
}

pub fn create_pair_contract<'a>(
    e: &Env,
    admin: &Address,
    tokens: &Vec<Address>,
    oracle: &Address,
    calculator: &Address,
) -> LongShortPairClient<'a> {
    let pair = LongShortPairClient::new(e, &e.register(crate::LongShortPair {}, ()));
    pair.initialize(
        &admin,
        &(
            admin.clone(),
            admin.clone(),
            admin.clone(),
            admin.clone(),
            Vec::from_array(e, [admin.clone()]),
            admin.clone(),
        ),
        tokens,
        oracle,
        calculator,
    );
    pair
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
