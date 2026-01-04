use soroban_sdk::{contracttype, Address, Bytes, BytesN, Symbol, Vec};

#[contracttype]
pub struct PairParams {
    pub admin: Address,
    pub privileged_addrs: (Address, Address),
    pub asset: Symbol,
    pub collateral_token: Address,
    pub pair_token_wasm_hash: BytesN<32>,
    pub oracle: Address,
    pub pair_calculator: Address,
    pub collateral_per_pair: u128,
    pub lower_bound: u128,
    pub upper_bound: u128,
}

#[contracttype]
#[derive(Clone)]
pub enum Direction {
    Long,
    Short,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearLongShortPairParameters {
    pub lower_bound: u128,
    pub upper_bound: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairStatus {
    Active,
    Settlement,
    Inactive,
}
