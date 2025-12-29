use soroban_sdk::{contracttype, Address, Vec};

#[contracttype]
pub struct PairParams {
    pub admin: Address,
    pub privileged_addrs: (Address, Address),
    pub tokens: Vec<Address>,
    pub oracle: Address,
    pub pool_plane: Address,
    pub pair_calculator: Address,
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
    pub upper_bound: u128,
    pub lower_bound: u128,
}
