#![allow(dead_code)]
#![cfg(test)]
extern crate std;
use crate::contracts;
use crate::contracts::long_short_pair::PairParams;
use crate::contracts::long_short_pair_factory::CreatorParams;
use crate::contracts::normal_oracle::OracleSource;
use sep_40_oracle::testutils::{Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::xdr::Asset;
use soroban_sdk::{Address, BytesN, Env, IntoVal, Symbol, Vec};

pub(crate) struct Setup<'a> {
    pub(crate) env: Env,

    // Addresses
    pub(crate) admin: Address,
    pub(crate) operator: Address,
    pub(crate) emergency_admin: Address,

    // Oracle
    pub(crate) oracle: Address,
    pub(crate) oracle_client: MockPriceOracleClient<'a>,

    // Contracts
    pub(crate) router: contracts::router::Client<'a>,
    pub(crate) pair_calculator: contracts::long_short_pair_calculator::Client<'a>,
    pub(crate) pair_factory: contracts::long_short_pair_factory::Client<'a>,

    // Tokens
    pub(crate) usdc_token: Address,
    pub(crate) reward_token: Address,
}

impl Default for Setup<'_> {
    fn default() -> Self {
        Self::setup()
    }
}

impl Setup<'_> {
    pub(crate) fn setup() -> Self {
        let e: Env = Env::default();
        e.mock_all_auths();
        e.cost_estimate().budget().reset_unlimited();

        let admin = Address::generate(&e);
        let operator = Address::generate(&e);
        let emergency_admin = Address::generate(&e);

        // Setup tokens
        let usdc_token = create_token_contract(&e, &admin);
        let reward_token = create_token_contract(&e, &admin);

        // init oracle
        let asset = Symbol::new(&e, "SOL");
        let usdc_symbol = Symbol::new(&e, "USDC");
        let usd_sybmol = Symbol::new(&e, "USD");

        let (oracle, oracle_client) = setup_price_feed_oracle(
            &e,
            &admin,
            &MockAsset::Other(usdc_symbol.clone()),
            &Vec::from_array(
                &e,
                [
                    MockAsset::Other(sol_symbol.clone()).clone(),
                    usdc_asset.clone(),
                ],
            ),
            14,
            300,
        );
        let normal_oracle = deploy_normal_oracle_contract(
            e.clone(),
            &admin,
            &asset,
            &OracleSource::Reflector,
            &oracle,
        );

        // init pair factory
        let pair_hash = e
            .deployer()
            .upload_contract_wasm(contracts::long_short_pair::WASM);
        let pair_calculator = deploy_pair_calculator_contract(e.clone());
        let pair_factory = deploy_pair_factory_contract(
            e.clone(),
            &admin,
            &emergency_admin,
            &token_factory.address,
            pair_hash,
        );

        // init swap router
        let pool_hash = e
            .deployer()
            .upload_contract_wasm(contracts::constant_product_pool::WASM);
        let token_hash = e
            .deployer()
            .upload_contract_wasm(contracts::token_pool::WASM);
        let plane = deploy_plane_contract(&e);

        let router = deploy_liqpool_router_contract(e.clone());
        router.init_admin(&admin);
        router.init_config_storage(
            &admin,
            &deploy_config_storage(&e, &admin, &emergency_admin).address,
        );
        router.set_rewards_gauge_hash(
            &admin,
            &e.deployer()
                .upload_contract_wasm(contracts::rewards_gauge::WASM),
        );
        router.set_pool_hash(&admin, &pool_hash);
        router.set_stableswap_pool_hash(
            &admin,
            &e.deployer()
                .upload_contract_wasm(contracts::stableswap_pool::WASM),
        );
        router.set_synthetic_pool_hash(
            &admin,
            &e.deployer()
                .upload_contract_wasm(contracts::synthetic_pool::WASM),
        );
        router.set_token_hash(&admin, &token_hash);
        router.set_reward_token(&admin, &reward_token.address);
        router.set_pools_plane(&admin, &plane.address);
        router.configure_init_pool_payment(
            &admin,
            &reward_token.address,
            &10_0000000,
            &1_0000000,
            &router.address,
        );
        router.set_protocol_fee_fraction(&admin, &5000);

        Self {
            env: e,
            admin,
            operator,
            emergency_admin,

            oracle,
            oracle_client,

            router,
            pair_calculator,
            pair_factory,
            token_factory,

            usdc_token: usdc_token.address,
            reward_token: reward_token.address,
        }
    }

    pub(crate) fn deploy_long_short_pair(&self) -> (contracts::long_short_pair::Client, Address) {
        // get_token_admin_client(&self.env, &self.reward_token).mint(&self.admin, &10_0000000);

        let params = CreatorParams {
            admin: self.admin,
            oracle: self.oracle,
            calculator: self.pair_calculator.address,
            collateral_per_pair: 100_0000000, // $100
            collateral_token: self.usdc_token,
            pair_name: Symbol::new(&self.env, "Test Pair"),
            serialized_long_asset: Asset::CreditAlphanum4(("TEST", "TEST")).into_val(&self.env),
            // serialized_short_asset:
        };

        let pair_address = self.pair_factory.deploy_lsp_contract(&self.admin, &params);
        (
            contracts::long_short_pair::Client::new(&self.env, &pair_address),
            pair_address,
        )
    }

    pub(crate) fn deploy_standard_pool(
        &self,
        token_a: &Address,
        token_b: &Address,
        fee_fraction: u32,
    ) -> (contracts::constant_product_pool::Client, BytesN<32>) {
        get_token_admin_client(&self.env, &self.reward_token).mint(&self.admin, &10_0000000);
        let (pool_hash, pool_address) = self.router.init_standard_pool(
            &self.admin,
            &Vec::from_array(&self.env, [token_a.clone(), token_b.clone()]),
            &fee_fraction,
        );
        (
            contracts::constant_product_pool::Client::new(&self.env, &pool_address),
            pool_hash,
        )
    }

    pub(crate) fn deploy_stableswap_pool(
        &self,
        token_a: &Address,
        token_b: &Address,
        fee_fraction: u32,
    ) -> (contracts::stableswap_pool::Client, BytesN<32>) {
        get_token_admin_client(&self.env, &self.reward_token).mint(&self.admin, &1_0000000);
        let (pool_hash, pool_address) = self.router.init_stableswap_pool(
            &self.admin,
            &Vec::from_array(&self.env, [token_a.clone(), token_b.clone()]),
            &fee_fraction,
        );
        (
            contracts::stableswap_pool::Client::new(&self.env, &pool_address),
            pool_hash,
        )
    }

    pub(crate) fn deploy_synthetic_pool(
        &self,
        token_a: &Address,
        token_b: &Address,
        fee_fraction: u32,
    ) -> (contracts::constant_product_pool::Client, BytesN<32>) {
        get_token_admin_client(&self.env, &self.reward_token).mint(&self.admin, &10_0000000);
        let (pool_hash, pool_address) = self.router.init_standard_pool(
            &self.admin,
            &Vec::from_array(&self.env, [token_a.clone(), token_b.clone()]),
            &fee_fraction,
        );
        (
            contracts::constant_product_pool::Client::new(&self.env, &pool_address),
            pool_hash,
        )
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

fn deploy_liqpool_router_contract<'a>(e: Env) -> contracts::router::Client<'a> {
    contracts::router::Client::new(&e, &e.register(contracts::router::WASM, ()))
}

fn deploy_plane_contract<'a>(e: &Env) -> contracts::pool_plane::Client {
    contracts::pool_plane::Client::new(e, &e.register(contracts::pool_plane::WASM, ()))
}

fn deploy_config_storage<'a>(
    e: &Env,
    admin: &Address,
    emergency_admin: &Address,
) -> contracts::config_storage::Client<'a> {
    contracts::config_storage::Client::new(
        e,
        &e.register(
            contracts::config_storage::WASM,
            contracts::config_storage::Args::__constructor(admin, emergency_admin),
        ),
    )
}

// extrea

fn deploy_pair_factory_contract<'a>(
    e: Env,
    admin: &Address,
    emergency_admin: &Address,
    token_factory: &Address,
    pair_wasm: BytesN<32>,
) -> contracts::long_short_pair_factory::Client<'a> {
    contracts::long_short_pair_factory::Client::new(
        &e,
        &e.register(
            contracts::long_short_pair_factory::WASM,
            (admin, emergency_admin, token_factory, pair_wasm),
        ),
    )
}
//  admin: Address,
//         emergency_admin: Address,
//         token_factory: Address,
//         lsp_contract_wasm: BytesN<32>,

fn deploy_token_factory_contract<'a>(e: Env) -> contracts::token_factory::Client<'a> {
    contracts::token_factory::Client::new(&e, &e.register(contracts::token_factory::WASM, ()))
}

fn deploy_normal_oracle_contract<'a>(
    e: Env,
    admin: &Address,
    asset: &Symbol,
    source: &OracleSource,
    addr: &Address,
) -> contracts::normal_oracle::Client<'a> {
    contracts::normal_oracle::Client::new(
        &e,
        &e.register(contracts::normal_oracle::WASM, (admin, asset, source, addr)),
    )
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

fn deploy_pair_calculator_contract<'a>(e: &Env) -> contracts::long_short_pair_calculator::Client {
    contracts::long_short_pair_calculator::Client::new(
        e,
        &e.register(contracts::long_short_pair_calculator::WASM, ()),
    )
}
