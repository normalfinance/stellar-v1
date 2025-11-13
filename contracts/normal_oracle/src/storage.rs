use soroban_sdk::{ Address, Env, Symbol, contracttype, panic_with_error };
use utils::bump::{ bump_instance };
use utils::{ errors::storage_errors::StorageError };

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Asset,
    ReflectorOracle,
}

// Asset
pub fn get_asset(e: &Env) -> Symbol {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::Asset) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn put_asset(e: &Env, asset: Symbol) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::Asset, &asset)
}

// Oracle
pub fn get_reflector_oracle(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::ReflectorOracle) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn put_reflector_oracle(e: &Env, contract: Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::ReflectorOracle, &contract)
}
