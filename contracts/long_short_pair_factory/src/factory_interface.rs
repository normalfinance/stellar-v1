use soroban_sdk::{Address, BytesN, Env, Map, Symbol, Vec};
use types::pair::PairParams;

use crate::contract::FactoryConfig;

pub trait LongShortPairFactoryTrait {
    fn deploy_pair_contract(e: Env, admin: Address, params: PairParams) -> Address;
}

pub trait AdminInterface {
    // Set privileged addresses
    fn set_privileged_addrs(
        e: Env,
        admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        pause_admin: Address,
        emergency_pause_admins: Vec<Address>,
    );

    // Get map of privileged roles
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>>;

    fn get_factory_config(e: Env) -> FactoryConfig;

    fn get_pair_contract_wasm(e: Env) -> BytesN<32>;

    fn get_all_deployed_pairs(e: Env) -> Vec<Address>;

    fn get_total_pair_count(e: Env) -> u32;

    fn get_pair_by_asset(e: Env, asset: Symbol) -> Address;

    fn set_pair_contract_wasm(e: Env, admin: Address, pair_contract_wasm: BytesN<32>);

    // Stop pair creation instantly
    fn kill_create(e: Env, admin: Address);

    // Resume pair creation
    fn unkill_create(e: Env, admin: Address);

    // Get killswitch status
    fn get_is_killed_create(e: Env) -> bool;
}
