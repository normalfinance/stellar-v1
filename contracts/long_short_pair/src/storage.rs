use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env};
use types::pair::PairStatus;
pub use utils::bump::bump_instance;
use utils::constant::PERCENTAGE_PRECISION_U64;
use utils::errors::storage_errors::StorageError;
use utils::generate_instance_storage_getter;
use utils::{
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

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
    1 // basically zero to maximize short value
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
    u128,
    5_000_000 // 50%
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
generate_instance_storage_getter_and_setter!(token_collateral, DataKey::TokenCollateral, Address);

// Guard Rails
generate_instance_storage_getter_and_setter_with_default!(
    max_ratio_percent_divergence,
    DataKey::MaxRatioPercentDivergence,
    u64,
    PERCENTAGE_PRECISION_U64 / 10 // 10%
);
