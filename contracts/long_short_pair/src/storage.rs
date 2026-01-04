use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env};
pub use utils::bump::bump_instance;
use utils::bump::bump_persistent;
use utils::constant::{
    ONE_HOUR, PERCENTAGE_PRECISION_U64, PERCENT_MULTIPLIER, PERCENT_MULTIPLIER_I128,
    PERCENT_MULTIPLIER_I64, PERCENT_MULTIPLIER_U64,
};
use utils::errors::storage_errors::StorageError;
use utils::generate_instance_storage_getter;
use utils::{
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

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
    TokenCollateral,

    CollateralPerPair,
    // Number between 0 and 1 to allocate collateral between long & short tokens at redemption. 0 entitles each short
    // to collateralPerPair and each long to 0. 1 makes each long worth collateralPerPair and short 0.
    CollateralPercentLong,

    // Addresses
    Calculator,
    Oracle,
    PoolPlane,
    PoolLong,
    PoolShort,

    // Guard Rails
    MaxRatioPercentDivergence,

    // Funding
    SanitizeClampDenominator,
    FundingCheckpoint(Address),
    CumulativeFundingIndexLong,
    CumulativeFundingIndexShort,
    LastFundingRate,
    Last24hAvgFundingRate, // estimate of last 24h of funding rate perp market (unit is quote per base)
    LastFundingRateTs,
    FundingPeriod,
    FundingClamp, // max/min the

    LastUpdateTs,

    // Paused ops
    IsKilledMint,
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
    ONE_HOUR
);
generate_instance_storage_getter_and_setter_with_default!(
    funding_clamp,
    DataKey::FundingClamp,
    i128,
    PERCENT_MULTIPLIER_I128 // 100%
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
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    collateral_percent_long,
    DataKey::CollateralPercentLong,
    u64,
    5000 // 50%
);

// Paused Ops
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_mint,
    DataKey::IsKilledMint,
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

generate_instance_storage_getter_and_setter!(oracle, DataKey::Oracle, Address);
generate_instance_storage_getter_and_setter!(calculator, DataKey::Calculator, Address);
generate_instance_storage_getter_and_setter!(pool_plane, DataKey::PoolPlane, Address);
generate_instance_storage_getter_and_setter!(pool_long, DataKey::PoolLong, Address);
generate_instance_storage_getter_and_setter!(pool_short, DataKey::PoolShort, Address);

pub(crate) fn has_pools(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::PoolLong)
        && e.storage().instance().has(&DataKey::PoolShort)
}

// Guard Rails
generate_instance_storage_getter_and_setter_with_default!(
    max_ratio_percent_divergence,
    DataKey::MaxRatioPercentDivergence,
    u64,
    PERCENTAGE_PRECISION_U64 / 10 // 10%
);

// Token
pub fn get_token_collateral(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::TokenCollateral) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

// Token - Setters
pub fn put_token_collateral(e: &Env, contract: Address) {
    bump_instance(e);
    e.storage()
        .instance()
        .set(&DataKey::TokenCollateral, &contract)
}
