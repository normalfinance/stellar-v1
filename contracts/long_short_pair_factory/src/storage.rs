use paste::paste;
use soroban_sdk::{contracterror, contracttype, panic_with_error, Address, BytesN, Env, Vec};
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
    PairContractWASM,      // wasm of the Long Short Pair contract
    AssetPair(BytesN<32>), // asset > pair address
    AllDeployedPairs,      // vec(Address)
    // paused ops
    IsKilledCreate,
}

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum FactoryError {
    PairAlreadyExists = 401,
    PairNotFound = 404,
}

pub fn get_pair(e: &Env, salt: BytesN<32>) -> Address {
    let key = DataKey::AssetPair(salt);
    match e.storage().persistent().get(&key) {
        Some(address) => {
            bump_persistent(e, &key);
            address
        }
        None => panic_with_error!(&e, FactoryError::PairNotFound),
    }
}

pub fn put_pair(e: &Env, salt: BytesN<32>, pair: &Address) {
    let key = DataKey::AssetPair(salt);
    e.storage().persistent().set(&key, pair);
    bump_persistent(e, &key);
}

generate_instance_storage_getter_and_setter!(
    pair_contract_wasm,
    DataKey::PairContractWASM,
    BytesN<32>
);

// paused ops
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_create,
    DataKey::IsKilledCreate,
    bool,
    false
);

// TODO: https://app.almanax.ai/scan/13ca3512-fbc7-4909-929a-53855e07d7af/findings/897d84fc-0d49-4569-afc8-7704bd534a5c
pub fn add_deployed_pair(env: &Env, pair_address: &Address) {
    // Add to global list
    let global_key = DataKey::AllDeployedPairs;
    let mut all_pairs: Vec<Address> = match env.storage().persistent().get(&global_key) {
        Some(pairs) => pairs,
        None => Vec::new(env),
    };
    all_pairs.push_back(pair_address.clone());
    env.storage().persistent().set(&global_key, &all_pairs);
    bump_persistent(env, &global_key);
}

pub fn get_all_deployed_pairs(env: &Env) -> Vec<Address> {
    let key = DataKey::AllDeployedPairs;
    match env.storage().persistent().get(&key) {
        Some(pairs) => {
            bump_persistent(env, &key);
            pairs
        }
        None => Vec::new(env),
    }
}
