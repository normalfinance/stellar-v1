use paste::paste;
use soroban_sdk::{panic_with_error, Address, Env, Symbol};
use types::pair::PairStatus;
pub use utils::bump::bump_instance;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

/********** Storage Key Types **********/

const KEY_ASSET: &str = "Asset";
const KEY_STATUS: &str = "Status"; // NOTE: Only update Status enum by appending value, DO NOT reorder it
const KEY_LOWER_BOUND: &str = "LowerBound";
const KEY_UPPER_BOUND: &str = "UpperBound";
const KEY_COLLATERAL_TOKEN: &str = "CollateralToken";
const KEY_TOTAL_COLLATERAL: &str = "TotalCollateral";
const KEY_COLLATERAL_PER_PAIR: &str = "CollateralPerPair";
const KEY_COLLATERAL_PERCENT_LONG: &str = "CollateralPercentLong";
const KEY_CALCULATOR: &str = "Calculator";
const KEY_ORACLE: &str = "Oracle";
const KEY_MAX_PRICE_DIVERGENCE: &str = "MaxPriceDivergence";
const KEY_LAST_UPDATE_TS: &str = "LastUpdateTs";
const KEY_EXPERIRATION_TS: &str = "ExpirationTs";
const KEY_IS_KILLED_MINT: &str = "IsKilledMint";
const KEY_IS_KILLED_REDEEM: &str = "IsKilledRedeem";

/********** Storage **********/

generate_instance_storage_getter_and_setter!(asset, KEY_ASSET, Symbol);

generate_instance_storage_getter_and_setter_with_default!(
    status,
    KEY_STATUS,
    PairStatus,
    PairStatus::Inactive
);

// Price boundaries
generate_instance_storage_getter_and_setter_with_default!(
    lower_bound,
    KEY_LOWER_BOUND,
    u128,
    1 // basically zero to maximize short value
);
generate_instance_storage_getter_and_setter_with_default!(upper_bound, KEY_UPPER_BOUND, u128, 0);

// Collateral
generate_instance_storage_getter_and_setter!(collateral_token, KEY_COLLATERAL_TOKEN, Address);
generate_instance_storage_getter_and_setter_with_default!(
    total_collateral,
    KEY_TOTAL_COLLATERAL,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    collateral_per_pair,
    KEY_COLLATERAL_PER_PAIR,
    u128,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    collateral_percent_long,
    KEY_COLLATERAL_PERCENT_LONG,
    u128,
    5_000_000 // 50%
);

// Addresses
generate_instance_storage_getter_and_setter!(oracle, KEY_ORACLE, Address);
generate_instance_storage_getter_and_setter!(calculator, KEY_CALCULATOR, Address);

// Timestamps
generate_instance_storage_getter_and_setter_with_default!(
    last_update_ts,
    KEY_LAST_UPDATE_TS,
    u64,
    0
);
generate_instance_storage_getter_and_setter_with_default!(
    expiration_ts,
    KEY_EXPERIRATION_TS,
    u64,
    0
);

// Guard Rails
generate_instance_storage_getter_and_setter_with_default!(
    max_price_divergence,
    KEY_MAX_PRICE_DIVERGENCE,
    u64,
    1_000_000 // 10%
);

// Paused Ops
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_mint,
    KEY_IS_KILLED_MINT,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_redeem,
    KEY_IS_KILLED_REDEEM,
    bool,
    false
);
