use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol};
pub use utils::bump::bump_instance;
use utils::bump::bump_persistent;
use utils::constant::ONE_HOUR;
use utils::errors::storage_errors::StorageError;
use utils::generate_instance_storage_getter;
use utils::{
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

use oracle::state::{HistoricalOracleData, OracleGuardRails};

use crate::funding::FundingCheckpoint;

// Factory configuration struct for query methods
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollateralInfo {
    pub collateral_per_pair: u128,
    pub collateral_percent_long: u64,
}
// Factory configuration struct for query methods
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundingInfo {
    pub sanitize_clamp_denominator: i64,
    pub cumulative_funding_index_long: i64,
    pub cumulative_funding_index_short: i64,
    pub last_funding_rate: i64,
    pub last24h_avg_funding_rate: i64,
    pub last_funding_rate_ts: u64,
    pub funding_period: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    TokenLong,
    TokenShort,
    TokenCollateral,

    CollateralPerPair,
    // Number between 0 and 1 to allocate collateral between long & short tokens at redemption. 0 entitles each short
    // to collateralPerPair and each long to 0. 1 makes each long worth collateralPerPair and short 0.
    CollateralPercentLong,

    // Addresses
    Pool,
    Calculator,
    NormalOracle,

    // Oracle
    GuardRails, // a set of oracle price data validations and protections.
    OracleData,

    // Funding
    SanitizeClampDenominator,
    FundingCheckpoint(Address),
    CumulativeFundingIndexLong,
    CumulativeFundingIndexShort,
    LastFundingRate,
    Last24hAvgFundingRate, // estimate of last 24h of funding rate perp market (unit is quote per base)
    LastFundingRateTs,
    FundingPeriod,

    LastUpdateTs,

    // Paused ops
    IsKilledCreate,
    IsKilledRedeem,
    IsKilledUpdateFunding,
}

// Funding
generate_instance_storage_getter_and_setter_with_default!(
    sanitize_clamp_denominator,
    DataKey::SanitizeClampDenominator,
    i64,
    0
);

pub(crate) fn get_user_funding_checkpoint(e: &Env, user: &Address) -> FundingCheckpoint {
    let key = DataKey::FundingCheckpoint(user.clone());
    match e.storage().persistent().get(&key) {
        Some(value) => {
            bump_persistent(e, &key);
            value
        }
        None => FundingCheckpoint::new(user.clone()),
    }
}

pub(crate) fn put_user_funding_checkpoint(e: &Env, user: &Address, checkpoint: &FundingCheckpoint) {
    let key = DataKey::FundingCheckpoint(user.clone());
    e.storage().persistent().set(&key, checkpoint);
    bump_persistent(e, &key);
}

generate_instance_storage_getter_and_setter_with_default!(
    cumulative_funding_index_long,
    DataKey::CumulativeFundingIndexLong,
    i64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    cumulative_funding_index_short,
    DataKey::CumulativeFundingIndexShort,
    i64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    last_funding_rate,
    DataKey::LastFundingRate,
    i64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    last_24h_avg_funding_rate,
    DataKey::Last24hAvgFundingRate,
    i64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    last_funding_rate_ts,
    DataKey::LastFundingRateTs,
    u64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    funding_period,
    DataKey::FundingPeriod,
    u64,
    ONE_HOUR * 8 // 8 hours
);

generate_instance_storage_getter_and_setter_with_default!(
    last_update_ts,
    DataKey::LastUpdateTs,
    u64,
    0
);

// Collateral
generate_instance_storage_getter_and_setter_with_default!(
    collateral_per_pair,
    DataKey::CollateralPerPair,
    u128,
    1000000000 // $100.00
);
generate_instance_storage_getter_and_setter_with_default!(
    collateral_percent_long,
    DataKey::CollateralPercentLong,
    u64,
    5000 // 50%
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
    is_killed_update_funding,
    DataKey::IsKilledUpdateFunding,
    bool,
    false
);

generate_instance_storage_getter_and_setter!(normal_oracle, DataKey::NormalOracle, Address);
generate_instance_storage_getter_and_setter!(calculator, DataKey::Calculator, Address);
generate_instance_storage_getter_and_setter!(pool, DataKey::Pool, Address);

// Oracle
generate_instance_storage_getter_and_setter_with_default!(
    guard_rails,
    DataKey::GuardRails,
    OracleGuardRails,
    OracleGuardRails::default()
);

pub(crate) fn get_historical_oracle_data(e: &Env) -> HistoricalOracleData {
    let key = DataKey::OracleData;
    match e.storage().persistent().get(&key) {
        Some(value) => {
            bump_persistent(e, &key);
            value
        }
        None => HistoricalOracleData::default_quote_oracle(),
    }
}

pub(crate) fn put_historical_oracle_data(e: &Env, oracle_data: &HistoricalOracleData) {
    let key = DataKey::OracleData;
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
