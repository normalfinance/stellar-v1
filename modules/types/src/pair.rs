use soroban_sdk::{contracttype, Address, Vec};

#[contracttype]
pub struct PairParams {
    pub admin: Address,
    pub privileged_addrs: (Address, Address, Address, Address, Vec<Address>, Address),
    pub tokens: Vec<Address>,
    pub oracle: Address,
    pub pool: Address,
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
