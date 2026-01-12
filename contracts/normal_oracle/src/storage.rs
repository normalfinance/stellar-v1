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

/********** Storage Types **********/

#[contracttype]
#[derive(Copy, Clone, Debug)]
pub struct GuardRails {
    pub seconds_before_stale: u64,
    pub too_volatile_ratio: u64,
}

/********** Storage Key Types **********/

const KEY_ASSET: &str = "Asset";
const KEY_ORACLE: &str = "Oracle";
const KEY_ORACLE_SOURCE: &str = "OracleSource";
const KEY_STALE: &str = "SecondsBeforeStale";
const KEY_VOLATILE: &str = "TooVolatileRatio";

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    HistoricalData,
}

/********** Storage **********/

generate_instance_storage_getter_and_setter!(asset, KEY_ASSET, Symbol);
generate_instance_storage_getter_and_setter!(oracle, KEY_ORACLE, Address);
generate_instance_storage_getter_and_setter!(oracle_source, KEY_ORACLE_SOURCE, OracleSource);
generate_instance_storage_getter_and_setter_with_default!(
    seconds_before_stale,
    KEY_STALE,
    u64,
    FIVE_MINUTE as u64
);
generate_instance_storage_getter_and_setter_with_default!(
    too_volatile_ratio,
    KEY_VOLATILE,
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
