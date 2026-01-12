#![allow(dead_code)]
#![cfg(test)]
extern crate std;
use crate::contracts::long_short_pair::PairParams;
use crate::contracts::normal_oracle::OracleSource;
use crate::contracts::{self, long_short_pair};
use sep_40_oracle::testutils::{Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

pub fn create_treasury_contract<'a>(e: &Env) -> contracts::treasury::Client<'a> {
    contracts::treasury::Client::new(e, &e.register(contracts::treasury::WASM, ()))
}

pub fn create_pair_calculator_contract<'a>(
    e: &Env,
) -> contracts::long_short_pair_calculator::Client<'a> {
    contracts::long_short_pair_calculator::Client::new(
        e,
        &e.register(contracts::long_short_pair_calculator::WASM, ()),
    )
}

pub(crate) struct Setup<'a> {
    pub(crate) env: Env,

    // Addresses
    pub(crate) admin: Address,

    // Oracle
    pub(crate) oracle: Address,
    pub(crate) oracle_client: MockPriceOracleClient<'a>,

    // Contracts
    pub(crate) treasury: contracts::treasury::Client<'a>,
    pub(crate) pair_calculator: contracts::long_short_pair_calculator::Client<'a>,
    pub(crate) pair_factory: contracts::long_short_pair_factory::Client<'a>,

    // Tokens
    pub(crate) usdc_token: Address,
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
                [MockAsset::Other(asset.clone()).clone(), usdc_asset.clone()],
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

        // Setup Treasury
        let treasury = create_treasury_contract(&e);

        // Setup Pair Calculator
        let pair_calculator = create_pair_calculator_contract(&e);

        // init pair factory
        let pair_hash = e
            .deployer()
            .upload_contract_wasm(contracts::long_short_pair::WASM);

        let pair_factory = deploy_pair_factory_contract(e.clone(), &admin, pair_hash);

        // Deploy a Pair
        let pair_address = pair_factory.deploy_pair_contract(&pair_params);

        let pair_client = long_short_pair::Client::new(&e, &pair_address);

        let pair_params = long_short_pair::PairParams {
            admin,
            collateral_per_pair: 100_0000000,
            collateral_token: usdc_token.address,
            oracle: normal_oracle.address,
            pair_calculator: pair_calculator.address,
            asset: Symbol::new(&e, "Testing"),
            long_token: admin,
            short_token: admin,

            lower_bound: 0_0000000,
            upper_bound: 200_0000000,
        };
        pair_client.initialize(&params);

        let pair_tokens = pair_client.get_tokens();

        // Add it to the Treasury
        treasury.add_pair(
            &admin,
            &pair_address,
            &usdc_token.address,
            pair_tokens.get(0),
            pair_tokens.get(1),
        );

        Self {
            env: e,
            admin,

            // Oracle
            oracle,
            oracle_client,

            // Contracts
            treasury,
            pair_calculator,
            pair_factory,

            // Tokens
            usdc_token: usdc_token.address,
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

// extrea

fn deploy_pair_factory_contract<'a>(
    e: Env,
    admin: &Address,
    pair_wasm: BytesN<32>,
) -> contracts::long_short_pair_factory::Client<'a> {
    contracts::long_short_pair_factory::Client::new(
        &e,
        &e.register(contracts::long_short_pair_factory::WASM, (admin, pair_wasm)),
    )
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
