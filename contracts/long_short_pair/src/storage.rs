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

/// Asset identifier for this pair (e.g., "BTC", "ETH", etc.).
///
/// This is informational / used for display and deterministic addressing in factories.
const KEY_ASSET: &str = "Asset";

/// Current lifecycle status of the pair.
///
/// ⚠️ **IMPORTANT:** Only update the `PairStatus` enum by **appending** new variants.
/// Do **not** reorder existing variants, because the serialized representation is stored on-chain.
const KEY_STATUS: &str = "Status";

/// Lower bound of the price range used by the calculator to map price → settlement allocation.
///
/// Typically set near zero (or a small sentinel like `1`) to allow near-full-short exposure.
const KEY_LOWER_BOUND: &str = "LowerBound";

/// Upper bound of the price range used by the calculator to map price → settlement allocation.
///
/// When the oracle price crosses `upper_bound` (or `lower_bound`), the pair expires and fixes
/// the final collateral split.
const KEY_UPPER_BOUND: &str = "UpperBound";

/// Collateral token contract address (e.g., USDC).
const KEY_COLLATERAL_TOKEN: &str = "CollateralToken";

/// Total collateral currently held/recognized by this pair contract.
///
/// This value should track the collateral backing outstanding synthetic supply. It is
/// incremented on mint and decremented on redeem.
const KEY_TOTAL_COLLATERAL: &str = "TotalCollateral";

/// Collateral required to mint **one** long+short pair, scaled by `PRICE_PRECISION`.
///
/// Example: if 1 pair requires 1.0 USDC and `PRICE_PRECISION = 10_000_000`,
/// then `collateral_per_pair = 10_000_000`.
const KEY_COLLATERAL_PER_PAIR: &str = "CollateralPerPair";

/// Current settlement allocation (what fraction of collateral is attributed to LONG),
/// scaled by `PRICE_PRECISION`.
///
/// - `collateral_percent_long = 5_000_000` means 50% long / 50% short.
/// - `collateral_percent_long = 10_000_000` means 100% long / 0% short.
///
/// This value is updated by `sync_collateral` (unless the pair is expired).
const KEY_COLLATERAL_PERCENT_LONG: &str = "CollateralPercentLong";

/// Calculator contract address used to compute `collateral_percent_long` from oracle data and bounds.
const KEY_CALCULATOR: &str = "Calculator";

/// Oracle contract address used as the price source for `sync_collateral`.
const KEY_ORACLE: &str = "Oracle";

/// Timestamp (ledger seconds) when `collateral_percent_long` was last updated.
const KEY_LAST_UPDATE_TS: &str = "LastUpdateTs";

/// Timestamp (ledger seconds) when the pair became `Expired` (if applicable).
///
/// After expiration, `collateral_percent_long` should no longer change, and redemption rules
/// may allow one-sided redemption (depending on your contract logic).
const KEY_EXPIRATION_TS: &str = "ExpirationTs";

/// Emergency killswitch for mint operations.
const KEY_IS_KILLED_MINT: &str = "IsKilledMint";

/// Emergency killswitch for redeem operations.
const KEY_IS_KILLED_REDEEM: &str = "IsKilledRedeem";

/********** Storage **********/

// Asset symbol stored in instance storage.
generate_instance_storage_getter_and_setter!(asset, KEY_ASSET, Symbol);

// Pair lifecycle status.
//
// Default is `Inactive` until initialized by the constructor.
generate_instance_storage_getter_and_setter_with_default!(
    status,
    KEY_STATUS,
    PairStatus,
    PairStatus::Inactive
);

// -------------------------------------------------------------------------------------
// Price boundaries
// -------------------------------------------------------------------------------------

// Lower bound used by the calculator for price → collateral split.
//
// Default is `1` as a “near-zero” sentinel (avoids literal 0 if your math treats 0 specially).
generate_instance_storage_getter_and_setter_with_default!(
    lower_bound,
    KEY_LOWER_BOUND,
    u128,
    1 // effectively near-zero to maximize short value
);

// Upper bound used by the calculator for price → collateral split.
//
// Default `0` indicates “not configured” (your constructor should set a real value).
generate_instance_storage_getter_and_setter_with_default!(upper_bound, KEY_UPPER_BOUND, u128, 0);

// -------------------------------------------------------------------------------------
// Collateral accounting
// -------------------------------------------------------------------------------------

// Collateral token address (e.g., USDC).
generate_instance_storage_getter_and_setter!(collateral_token, KEY_COLLATERAL_TOKEN, Address);

// Total collateral held by the pair contract to back outstanding supply.
//
// Updated on mint/redeem (and potentially reconciled by a sync routine, if you have one).
generate_instance_storage_getter_and_setter_with_default!(
    total_collateral,
    KEY_TOTAL_COLLATERAL,
    u128,
    0
);

// Collateral required per 1 long+short pair, scaled by `PRICE_PRECISION`.
generate_instance_storage_getter_and_setter_with_default!(
    collateral_per_pair,
    KEY_COLLATERAL_PER_PAIR,
    u128,
    0
);

// Current allocation fraction assigned to LONG, scaled by `PRICE_PRECISION`.
//
// Updated by `sync_collateral` while the pair is `Active`.
generate_instance_storage_getter_and_setter_with_default!(
    collateral_percent_long,
    KEY_COLLATERAL_PERCENT_LONG,
    u128,
    5_000_000 // 50%
);

// -------------------------------------------------------------------------------------
// External dependencies
// -------------------------------------------------------------------------------------

// Oracle address used to fetch price data for `sync_collateral`.
generate_instance_storage_getter_and_setter!(oracle, KEY_ORACLE, Address);

// Calculator address used to compute collateral split from price and bounds.
generate_instance_storage_getter_and_setter!(calculator, KEY_CALCULATOR, Address);

// -------------------------------------------------------------------------------------
// Timestamps
// -------------------------------------------------------------------------------------

// Last time `collateral_percent_long` was updated.
generate_instance_storage_getter_and_setter_with_default!(
    last_update_ts,
    KEY_LAST_UPDATE_TS,
    u64,
    0
);

// Time when the pair transitioned to `Expired`.
generate_instance_storage_getter_and_setter_with_default!(expiration_ts, KEY_EXPIRATION_TS, u64, 0);

// -------------------------------------------------------------------------------------
// Emergency pause flags
// -------------------------------------------------------------------------------------

// If `true`, minting is disabled.
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_mint,
    KEY_IS_KILLED_MINT,
    bool,
    false
);

// If `true`, redemptions are disabled.
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_redeem,
    KEY_IS_KILLED_REDEEM,
    bool,
    false
);
