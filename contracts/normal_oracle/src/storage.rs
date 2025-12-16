use oracle::state::HistoricalOracleData;
use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol};
use types::oracle::OracleSource;
use utils::bump::{bump_instance, bump_persistent};
use utils::constant::{FIVE_MINUTE, PERCENTAGE_PRECISION_U64};
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // Config
    Asset, // symbol

    // Oracle
    OracleSource,
    Oracle, // address

    HistoricalData,

    // Guard Rails
    SecondsBeforeStale,
    TooVolatileRatio,
}

#[contracttype]
#[derive(Copy, Clone, Debug)]
pub struct GuardRails {
    pub seconds_before_stale: u64,
    pub too_volatile_ratio: u64,
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

// Source
generate_instance_storage_getter_and_setter!(oracle_source, DataKey::OracleSource, OracleSource);

pub fn get_oracle(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::Oracle) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn put_oracle(e: &Env, contract: Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::Oracle, &contract)
}

// Guard Rails
generate_instance_storage_getter_and_setter_with_default!(
    seconds_before_stale,
    DataKey::SecondsBeforeStale,
    u64,
    FIVE_MINUTE as u64
);
generate_instance_storage_getter_and_setter_with_default!(
    too_volatile_ratio,
    DataKey::TooVolatileRatio,
    u64,
    PERCENTAGE_PRECISION_U64 / 5 // ±20%
);

// Historical Data
pub(crate) fn get_historical_data(e: &Env) -> HistoricalOracleData {
    let key = DataKey::HistoricalData;
    match e.storage().persistent().get(&key) {
        Some(value) => {
            bump_persistent(e, &key);
            value
        }
        None => HistoricalOracleData::default_quote_oracle(),
    }
}

pub(crate) fn put_historical_data(e: &Env, oracle_data: &HistoricalOracleData) {
    let key = DataKey::HistoricalData;
    e.storage().persistent().set(&key, oracle_data);
    bump_persistent(e, &key);
}
