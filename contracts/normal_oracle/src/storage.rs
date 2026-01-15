use oracle::state::HistoricalOracleData;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol};
use types::oracle::{OraclePriceData, OracleSource};
use utils::{bump::bump_persistent, errors::storage_errors::StorageError};

/********** Storage Types **********/

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleConfig {
    /// Symbol representing the underlying asset being priced (e.g. "BTC", "ETH").
    /// Used for metadata and sanity checks by consumers.
    pub asset: Symbol,

    /// Declares the type of upstream oracle being used (e.g. Pyth, Chainlink, etc.).
    /// This is informational and can also be used by clients to apply
    /// source-specific logic if needed.
    pub source: OracleSource,

    /// Address of the upstream oracle contract providing raw price data.
    /// This contract acts as a *proxy / sanitizer* in front of this oracle.
    pub oracle: Address,
}

/// Guard-rail parameters applied to raw oracle updates before they are exposed
/// to downstream consumers (Treasury, Pair, etc.).
///
/// These values define *when* a price is considered stale or unsafe, and *how*
/// aggressively new oracle prices are clamped relative to historical values.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleGuardRails {
    /// Maximum age (in seconds) before an oracle price is considered stale.
    /// If exceeded, consumers may reject the price or treat the oracle as unhealthy.
    pub seconds_before_stale: u64,

    /// Maximum allowed relative price change between updates, expressed in
    /// `PERCENTAGE_PRECISION_U64` units.
    /// Used to detect abnormally volatile price jumps that may indicate oracle failure
    /// or manipulation.
    pub too_volatile_ratio: u64,

    /// Controls how tightly new oracle prices are clamped to historical prices.
    /// The allowed band is:
    /// ```text
    /// last_price ± (last_price / sanitize_clamp_denominator)
    /// ```
    /// Example:
    /// - `sanitize_clamp_denominator = 10` → ±10% per update
    pub sanitize_clamp_denominator: u128,
}

/********** Storage Key Types **********/

/// Persistent data keys for historical oracle state.
///
/// Historical data is kept in persistent storage so it survives across
/// ledger boundaries and can be used for TWAPs, volatility checks, and clamping.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    //
    Config(Symbol),
    /// Stores the rolling oracle history (TWAP, last price, timestamps, etc.).
    HistoricalData(Symbol),
    //
    GuardRails(Symbol),
}

/********** Storage **********/

pub(crate) fn has_config(e: &Env, asset: &Symbol) -> bool {
    let key = DataKey::Config(asset.clone());
    e.storage().persistent().has(&key)
}

pub(crate) fn get_config(e: &Env, asset: &Symbol) -> OracleConfig {
    let key = DataKey::Config(asset.clone());
    match e.storage().persistent().get(&key) {
        Some(data) => {
            bump_persistent(e, &key);
            data
        }
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub(crate) fn put_config(e: &Env, config: &OracleConfig) {
    let key = DataKey::Config(config.asset.clone());
    e.storage().persistent().set(&key, config);
    bump_persistent(e, &key);
}

pub(crate) fn remove_config(e: &Env, asset: &Symbol) {
    let key = DataKey::Config(asset.clone());
    e.storage().persistent().remove(&key);
}

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
    asset: &Symbol,
    oracle_price_data: &OraclePriceData,
    now: u64,
) -> HistoricalOracleData {
    let key = DataKey::HistoricalData(asset.clone());
    match e.storage().persistent().get(&key) {
        Some(data) => {
            bump_persistent(e, &key);
            data
        }
        None => HistoricalOracleData::default(*oracle_price_data, now),
    }
}

pub(crate) fn remove_historical_data(e: &Env, asset: &Symbol) {
    let key = DataKey::HistoricalData(asset.clone());
    e.storage().persistent().remove(&key);
}

/// Persists updated [`HistoricalOracleData`] after a successful oracle update.
///
/// Callers are responsible for ensuring:
/// - the data has passed all staleness, volatility, and clamp checks
/// - timestamps are monotonically increasing
///
/// This function only performs storage + TTL bumping.
pub(crate) fn put_historical_data(e: &Env, asset: &Symbol, oracle_data: &HistoricalOracleData) {
    let key = DataKey::HistoricalData(asset.clone());
    e.storage().persistent().set(&key, oracle_data);
    bump_persistent(e, &key);
}

pub(crate) fn get_guard_rails(e: &Env, asset: &Symbol) -> OracleGuardRails {
    let key = DataKey::GuardRails(asset.clone());
    match e.storage().persistent().get(&key) {
        Some(data) => {
            bump_persistent(e, &key);
            data
        }
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub(crate) fn put_guard_rails(e: &Env, asset: &Symbol, guard_rails: &OracleGuardRails) {
    let key = DataKey::GuardRails(asset.clone());
    e.storage().persistent().set(&key, guard_rails);
    bump_persistent(e, &key);
}

pub(crate) fn remove_guard_rails(e: &Env, asset: &Symbol) {
    let key = DataKey::GuardRails(asset.clone());
    e.storage().persistent().remove(&key);
}
