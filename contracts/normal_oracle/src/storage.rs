use oracle::state::HistoricalOracleData;
use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol};
use types::oracle::{OraclePriceData, OracleSource};
use utils::bump::{bump_instance, bump_persistent};
use utils::constant::{FIVE_MINUTE, PERCENTAGE_PRECISION_U64};
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

/********** Storage Types **********/

/// Guard-rail parameters applied to raw oracle updates before they are exposed
/// to downstream consumers (Treasury, Pair, etc.).
///
/// These values define *when* a price is considered stale or unsafe, and *how*
/// aggressively new oracle prices are clamped relative to historical values.
#[contracttype]
#[derive(Copy, Clone, Debug)]
pub struct GuardRails {
    /// Maximum age (in seconds) before an oracle price is considered stale.
    ///
    /// If exceeded, consumers may reject the price or treat the oracle as unhealthy.
    pub seconds_before_stale: u64,

    /// Maximum allowed relative price change between updates, expressed in
    /// `PERCENTAGE_PRECISION_U64` units.
    ///
    /// Used to detect abnormally volatile price jumps that may indicate oracle failure
    /// or manipulation.
    pub too_volatile_ratio: u64,

    /// Controls how tightly new oracle prices are clamped to historical prices.
    ///
    /// The allowed band is:
    /// ```text
    /// last_price ± (last_price / sanitize_clamp_denominator)
    /// ```
    ///
    /// Example:
    /// - `sanitize_clamp_denominator = 10` → ±10% per update
    pub sanitize_clamp_denominator: u128,
}

/********** Storage Key Types **********/

// Instance-scoped keys (global to this oracle proxy instance).
const KEY_ASSET: &str = "Asset";
const KEY_ORACLE: &str = "Oracle";
const KEY_ORACLE_SOURCE: &str = "OracleSource";
const KEY_SANITIZE_CLAMP_DENOMINATOR: &str = "SanitizeClampDenominator";
const KEY_STALE: &str = "SecondsBeforeStale";
const KEY_VOLATILE: &str = "TooVolatileRatio";

/// Persistent data keys for historical oracle state.
///
/// Historical data is kept in persistent storage so it survives across
/// ledger boundaries and can be used for TWAPs, volatility checks, and clamping.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Stores the rolling oracle history (TWAP, last price, timestamps, etc.).
    HistoricalData,
}

/********** Storage **********/

// Symbol representing the underlying asset being priced (e.g. "BTC", "ETH").
//
// Used for metadata and sanity checks by consumers.
generate_instance_storage_getter_and_setter!(asset, KEY_ASSET, Symbol);

// Address of the upstream oracle contract providing raw price data.
//
// This contract acts as a *proxy / sanitizer* in front of this oracle.
generate_instance_storage_getter_and_setter!(oracle, KEY_ORACLE, Address);

// Declares the type of upstream oracle being used (e.g. Pyth, Chainlink, etc.).
//
// This is informational and can also be used by clients to apply
// source-specific logic if needed.
generate_instance_storage_getter_and_setter!(oracle_source, KEY_ORACLE_SOURCE, OracleSource);

// Controls how aggressively incoming oracle prices are clamped relative to
// historical values.
//
// Defaults to `10`, meaning ±10% movement per update is allowed.
generate_instance_storage_getter_and_setter_with_default!(
    sanitize_clamp_denominator,
    KEY_SANITIZE_CLAMP_DENOMINATOR,
    u128,
    10 // ±10% allowed price move per update
);

// Maximum allowed age (in seconds) before an oracle price is considered stale.
//
// Defaults to `FIVE_MINUTE`.
generate_instance_storage_getter_and_setter_with_default!(
    seconds_before_stale,
    KEY_STALE,
    u64,
    FIVE_MINUTE as u64
);

// Maximum allowed relative price change between oracle updates.
//
// Expressed in `PERCENTAGE_PRECISION_U64` units.
// Default: ±20%.
generate_instance_storage_getter_and_setter_with_default!(
    too_volatile_ratio,
    KEY_VOLATILE,
    u64,
    PERCENTAGE_PRECISION_U64 / 5 // ±20%
);

// Historical Data

/// Loads the stored [`HistoricalOracleData`] for this oracle proxy.
///
/// If no historical data exists yet (first update), this function initializes
/// the history using the provided `oracle_price_data` and current timestamp.
///
/// ### Arguments
/// - `oracle_price_data`: The latest raw price data from the upstream oracle.
/// - `now`: Current ledger timestamp (seconds).
///
/// ### Returns
/// A fully-initialized [`HistoricalOracleData`] struct suitable for:
/// - TWAP calculations
/// - volatility checks
/// - price clamping
///
/// ### Notes
/// - Historical data is stored in persistent storage and TTL-bumped on access.
/// - This function never reverts for missing history; it deterministically
///   bootstraps from the first observed price.
pub(crate) fn get_historical_data(
    e: &Env,
    oracle_price_data: &OraclePriceData,
    now: u64,
) -> HistoricalOracleData {
    let key = DataKey::HistoricalData;
    match e.storage().persistent().get(&key) {
        Some(data) => {
            bump_persistent(e, &key);
            data
        }
        None => HistoricalOracleData::default(*oracle_price_data, now),
    }
}

/// Persists updated [`HistoricalOracleData`] after a successful oracle update.
///
/// Callers are responsible for ensuring:
/// - the data has passed all staleness, volatility, and clamp checks
/// - timestamps are monotonically increasing
///
/// This function only performs storage + TTL bumping.
pub(crate) fn put_historical_data(e: &Env, oracle_data: &HistoricalOracleData) {
    let key = DataKey::HistoricalData;
    e.storage().persistent().set(&key, oracle_data);
    bump_persistent(e, &key);
}
