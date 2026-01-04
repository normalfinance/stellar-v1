use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env};
use types::pair::PairStatus;
pub use utils::bump::bump_instance;
use utils::bump::bump_persistent;
use utils::constant::PERCENTAGE_PRECISION_U64;
use utils::errors::storage_errors::StorageError;
use utils::generate_instance_storage_getter;
use utils::{
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

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

    LowerBound,
    UpperBound,

    Status,

    CollateralPerPair,
    // Number between 0 and 1 to allocate collateral between long & short tokens at redemption. 0 entitles each short
    // to collateralPerPair and each long to 0. 1 makes each long worth collateralPerPair and short 0.
    CollateralPercentLong,

    // Addresses
    Calculator,
    Oracle,

    // Guard Rails
    MaxRatioPercentDivergence,

    LastUpdateTs,

    // Paused ops
    IsKilledMint,
    IsKilledRedeem,
}

generate_instance_storage_getter_and_setter_with_default!(
    last_update_ts,
    DataKey::LastUpdateTs,
    u64,
    0
);

generate_instance_storage_getter_and_setter_with_default!(
    lower_bound,
    DataKey::LowerBound,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    upper_bound,
    DataKey::UpperBound,
    u128,
    0
);

generate_instance_storage_getter_and_setter_with_default!(
    status,
    DataKey::Status,
    PairStatus,
    PairStatus::Inactive
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

generate_instance_storage_getter_and_setter!(oracle, DataKey::Oracle, Address);
generate_instance_storage_getter_and_setter!(calculator, DataKey::Calculator, Address);

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
