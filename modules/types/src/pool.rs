use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec};

use crate::pair::Direction;

#[contracttype]
pub struct PoolParams {
    pub admin: Address,
    pub privileged_addrs: (Address, Address, Address, Address, Vec<Address>, Address),
    pub router: Address,
    pub long_short_pair: Address,
    pub lp_token_wasm_hash: BytesN<32>,
    pub tokens: Vec<Address>,
    pub fees_config: (u32, u32),
    pub assets_config: (Symbol, Symbol),
    pub direction: Direction,
}
