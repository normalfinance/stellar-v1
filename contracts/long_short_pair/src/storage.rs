use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol};
pub use utils::bump::bump_instance;
use utils::bump::bump_persistent;
use utils::errors::storage_errors::StorageError;
use utils::generate_instance_storage_getter;
use utils::{
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

use crate::state::oracle::{HistoricalOracleData, OracleGuardRails};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenLong,
    TokenShort,
    TokenCollateral,

    CollateralPerPair,
    // CollateralTotal,
    // Number between 0 and 1e18 to allocate collateral between long & short tokens at redemption. 0 entitles each short
    // to collateralPerPair and each long to 0. 1e18 makes each long worth collateralPerPair and short 0.
    ExpiryPercentLong,

    Calculator, // address
    Oracle,

    OracleGuardRails, // a set of oracle price data validations and protections.
    HistoricalOracleData(Symbol),

    IsKilledCreate,
    IsKilledRedeem,
    IsKilledSettle,
}

// Collateral
generate_instance_storage_getter_and_setter_with_default!(
    collateral_per_pair,
    DataKey::CollateralPerPair,
    u128,
    1000000000 // $100
);
// generate_instance_storage_getter_and_setter_with_default!(
//     collateral_total,
//     DataKey::CollateralTotal,
//     u128,
//     0
// );
generate_instance_storage_getter_and_setter_with_default!(
    expiry_percent_long,
    DataKey::ExpiryPercentLong,
    u128,
    0
);

// Paused Ops
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_create,
    DataKey::IsKilledCreate,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_redeem,
    DataKey::IsKilledRedeem,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_settle,
    DataKey::IsKilledSettle,
    bool,
    false
);

generate_instance_storage_getter_and_setter!(oracle, DataKey::Oracle, Address);
generate_instance_storage_getter_and_setter!(calculator, DataKey::Calculator, Address);

// Oracle
generate_instance_storage_getter_and_setter_with_default!(
    oracle_guard_rails,
    DataKey::OracleGuardRails,
    OracleGuardRails,
    OracleGuardRails::default()
);

pub(crate) fn get_historical_oracle_data(e: &Env, asset: &Symbol) -> HistoricalOracleData {
    let key = DataKey::HistoricalOracleData(asset.clone());
    match e.storage().persistent().get(&key) {
        Some(value) => {
            bump_persistent(e, &key);
            value
        }
        None => HistoricalOracleData::default_quote_oracle(),
    }
}

pub(crate) fn put_historical_oracle_data(
    e: &Env,
    asset: &Symbol,
    oracle_data: &HistoricalOracleData,
) {
    let key = DataKey::HistoricalOracleData(asset.clone());
    e.storage().persistent().set(&key, oracle_data);
    bump_persistent(e, &key);
}

// Token Getters
pub fn get_token_long(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::TokenLong) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn get_token_short(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::TokenShort) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn get_token_collateral(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::TokenCollateral) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

// Token - Setters
pub fn put_token_long(e: &Env, contract: Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::TokenLong, &contract)
}

pub fn put_token_short(e: &Env, contract: Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::TokenShort, &contract)
}

pub fn put_token_collateral(e: &Env, contract: Address) {
    bump_instance(e);
    e.storage()
        .instance()
        .set(&DataKey::TokenCollateral, &contract)
}
