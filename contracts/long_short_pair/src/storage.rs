use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol};
use types::pair::PairStatus;
pub use utils::bump::bump_instance;
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
    Asset,
    Status, // NOTE: Only update by appending value, DO NOT reorder them

    // Price boundaries
    LowerBound,
    UpperBound,

    // Collateral config
    CollateralToken, // USDC
    TotalCollateral,
    CollateralPerPair,
    // Number between 0 and 1 to allocate collateral between long & short tokens at redemption. 0 entitles each short
    // to collateral_per_pair and each long to 0. 1 makes each long worth collateral_per_pair and short 0.
    CollateralPercentLong,

    // Addresses
    Calculator,

    // Oracle
    Oracle,
    MaxPriceDivergence,

    // Timestamps
    LastUpdateTs,
    ExpirationTs,

    // Paused ops
    IsKilledMint,
    IsKilledRedeem,
}

generate_instance_storage_getter_and_setter!(asset, DataKey::Asset, Symbol);

generate_instance_storage_getter_and_setter_with_default!(
    status,
    DataKey::Status,
    PairStatus,
    PairStatus::Inactive
);

// Price boundaries
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

// Collateral
generate_instance_storage_getter_and_setter!(collateral_token, DataKey::CollateralToken, Address);
generate_instance_storage_getter_and_setter_with_default!(
    total_collateral,
    DataKey::TotalCollateral,
    u128,
    0
);
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

// Addresses
generate_instance_storage_getter_and_setter!(oracle, DataKey::Oracle, Address);
generate_instance_storage_getter_and_setter!(calculator, DataKey::Calculator, Address);

// Timestamps
generate_instance_storage_getter_and_setter_with_default!(
    last_update_ts,
    DataKey::LastUpdateTs,
    u64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    expiration_ts,
    DataKey::ExpirationTs,
    u64,
    0
);

// Guard Rails
generate_instance_storage_getter_and_setter_with_default!(
    max_price_divergence,
    DataKey::MaxPriceDivergence,
    u64,
    1_000_000 // 10%
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
