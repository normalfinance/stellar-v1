use soroban_sdk::{Address, BytesN, Env, Map, Symbol, Vec};

use crate::contract::{CreatorParams, FactoryConfig};

pub trait LongShortPairFactoryTrait {
    fn deploy_lsp_contract(e: Env, params: CreatorParams) -> Address;
}

pub trait AdminInterface {
    fn get_factory_config(e: Env) -> FactoryConfig;

    fn get_lsp_contract_wasm(e: Env) -> BytesN<32>;

    fn get_deployed_pairs(e: Env, operator: Address) -> Vec<Address>;

    fn get_all_deployed_pairs(e: Env) -> Vec<Address>;

    fn get_pair_count(e: Env, operator: Address) -> u32;

    fn get_total_pair_count(e: Env) -> u32;

    fn set_lsp_contract_wasm(e: Env, admin: Address, lsp_contract_wasm: BytesN<32>);

    // Set privileged addresses
    fn set_privileged_addrs(
        e: Env,
        admin: Address,
        pause_admin: Address,
        emergency_pause_admin: Address,
    );

    // Get map of privileged roles
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>>;

    // Stop LSP creation instantly
    fn kill_create(e: Env, admin: Address);

    // Resume LSP creation
    fn unkill_create(e: Env, admin: Address);

    // Get killswitch status
    fn get_is_killed_create(e: Env) -> bool;
}
