use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env, Vec};
use utils::bump::{bump_instance, bump_persistent};
use utils::{
    errors::storage_errors::StorageError, generate_instance_storage_getter,
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenFactory,

    LongShortPairContractWASM, // wasm of the Long Short Pair contract

    ContractSequence(Address),

    // LSP registry storage
    DeployedLSPs(Address), // manager -> Vec<Address>
    AllDeployedLSPs,       // global registry -> Vec<Address>

    IsKilledCreate,
}

generate_instance_storage_getter_and_setter!(token_factory, DataKey::TokenFactory, Address);

generate_instance_storage_getter_and_setter!(
    lsp_contract_wasm,
    DataKey::LongShortPairContractWASM,
    BytesN<32>
);

// paused ops
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_create,
    DataKey::IsKilledCreate,
    bool,
    false
);

pub(crate) fn get_contract_sequence(env: &Env, manager: Address) -> u32 {
    let key = DataKey::ContractSequence(manager);
    match env.storage().persistent().get(&key) {
        Some(sequence) => {
            bump_persistent(env, &key);
            sequence
        }
        None => 0,
    }
}

pub(crate) fn set_contract_sequence(env: &Env, manager: Address, sequence: u32) {
    let key = DataKey::ContractSequence(manager);
    env.storage().persistent().set(&key, &sequence);
    bump_persistent(env, &key);
}

// Index registry functions
pub fn add_deployed_pair(env: &Env, manager: &Address, index_address: &Address) {
    // Add to manager's list
    let manager_key: DataKey = DataKey::DeployedLSPs(manager.clone());
    let mut manager_pairs: Vec<Address> = match env.storage().persistent().get(&manager_key) {
        Some(pairs) => pairs,
        None => Vec::new(env),
    };
    manager_pairs.push_back(index_address.clone());
    env.storage().persistent().set(&manager_key, &manager_pairs);
    bump_persistent(env, &manager_key);

    // Add to global list
    let global_key = DataKey::AllDeployedLSPs;
    let mut all_pairs: Vec<Address> = match env.storage().persistent().get(&global_key) {
        Some(pairs) => pairs,
        None => Vec::new(env),
    };
    all_pairs.push_back(index_address.clone());
    env.storage().persistent().set(&global_key, &all_pairs);
    bump_persistent(env, &global_key);
}

pub fn get_deployed_pairs(env: &Env, manager: &Address) -> Vec<Address> {
    let key = DataKey::DeployedLSPs(manager.clone());
    match env.storage().persistent().get(&key) {
        Some(pairs) => {
            bump_persistent(env, &key);
            pairs
        }
        None => Vec::new(env),
    }
}

pub fn get_all_deployed_pairs(env: &Env) -> Vec<Address> {
    let key = DataKey::AllDeployedLSPs;
    match env.storage().persistent().get(&key) {
        Some(pairs) => {
            bump_persistent(env, &key);
            pairs
        }
        None => Vec::new(env),
    }
}
