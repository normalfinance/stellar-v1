use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol};
use types::pair::PairAmountsWithUSDC;
pub use utils::bump::bump_instance;
use utils::bump::bump_persistent;
use utils::constant::PRICE_PRECISION;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

/********** Storage Types **********/

/// Static configuration for a supported LongShortPair.
///
/// Stored per-pair and used to find the token contracts the Treasury should interact with.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairConfig {
    /// The LongShortPair contract address.
    pub pair: Address,
    /// LONG token contract address.
    pub long: Address,
    /// SHORT token contract address.
    pub short: Address,
    /// Collateral token contract address (USDC).
    pub usdc: Address,
}

/// Convenient aggregation used by frontend/indexers.
///
/// Note: this is not stored directly; it is assembled from other stored keys.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasurySummary {
    pub config: PairConfig,
    /// Treasury inventory (what the Treasury actually holds / accounts for).
    pub balances: PairAmountsWithUSDC,
    /// Quoting inputs (LONG/SHORT settlement fractions + USDC oracle TWAP).
    pub prices: PairAmountsWithUSDC,
    /// Total LP share supply for this pair.
    pub total_shares: u128,
    pub fee_config: TreasuryFeeConfig,
}

/// A per-user view that combines [`TreasurySummary`] plus the user's LP share balance.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryUserSummary {
    pub summary: TreasurySummary,
    pub user_shares: u128,
}

/// Fee model parameters for a pair.
///
/// `base_fee` are expressed in `PRICE_PRECISION` units.
/// The remaining parameters are model-specific knobs used by `calculate_fee(...)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryFeeConfig {
    pub base_fee: u128, // max = MAX_BASE_FEE
}

/// Risk parameters enforced by the Treasury when executing trades.
///
/// Values are interpreted by `crate::risk::*` checks.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryRiskParameters {
    /// Threshold in `PRICE_PRECISION` units used by toxic-trade logic.
    pub toxic_threshold: u128, // similar to collateral_percent_long

    /// Hard cap on absolute net inventory imbalance in **token units**:
    /// `abs(long_balance - short_balance) <= max_net_imbalance_tokens`.
    ///
    /// Set to 0 to disable the guard (not recommended).
    pub max_net_imbalance_tokens: u128,

    /// Maximum fee adjustment (additive or subtractive) driven by whether a trade
    /// increases or decreases `abs(net_imbalance)`; expressed in `PRICE_PRECISION`.
    ///
    /// Example: `50_000` = 0.50% max surcharge/rebate.
    pub imbalance_fee_max: u128,
}

/********** Storage Key Types **********/

// Instance (non-pair-specific) keys.
const KEY_USDC_FLOOR_FRACTION: &str = "UsdcFloorFraction";
const KEY_ORACLE: &str = "Oracle";
const KEY_IS_KILLED_DEPOSIT: &str = "IsKilledDeposit";
const KEY_IS_KILLED_WITHDRAW: &str = "IsKilledWithdraw";
const KEY_IS_KILLED_TRADE: &str = "IsKilledTrade";
const KEY_IS_KILLED_WITHDRAW_FLOOR: &str = "IsKilledWithdrawFloor";

/// Composite key for `(pair, user)` LP share balances.
///
/// Stored under [`TreasuryDataKey::UserShares`].
#[contracttype]
#[derive(Clone)]
pub struct UserSharesKey {
    pub pair: Address,
    pub user: Address,
}

/// Persistent storage keys for all per-pair state.
///
/// Everything here is stored in **persistent** storage and must be TTL-bumped
/// (`bump_persistent`) on read/write to avoid expiry.
#[derive(Clone)]
#[contracttype]
pub enum TreasuryDataKey {
    /// Pair -> token/config addresses.
    Config(Address),
    /// Pair -> Treasury inventory amounts.
    Balances(Address),
    /// Pair -> risk knobs (toxicity threshold, etc.).
    RiskParameters(Address),
    /// Pair -> total LP share supply.
    TotalShares(Address),
    /// (Pair, User) -> user's LP share balance.
    UserShares(UserSharesKey),
    /// Pair -> fee configuration.
    FeeConfig(Address),
    /// Pair -> accumulated protocol fees (in USDC units).
    ProtocolFees(Address),
}

/********** Storage **********/

generate_instance_storage_getter_and_setter!(oracle, KEY_ORACLE, Address);
generate_instance_storage_getter_and_setter_with_default!(
    usdc_floor_fraction,
    KEY_USDC_FLOOR_FRACTION,
    u128,
    PRICE_PRECISION / 10 // 10%
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_deposit,
    KEY_IS_KILLED_DEPOSIT,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_withdraw,
    KEY_IS_KILLED_WITHDRAW,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_trade,
    KEY_IS_KILLED_TRADE,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_withdraw_floor,
    KEY_IS_KILLED_WITHDRAW_FLOOR,
    bool,
    true
);

// Pair details

/// Returns `true` if this pair has been registered via `add_pair` (i.e. config exists).
pub(crate) fn has_config(e: &Env, pair: &Address) -> bool {
    let key = TreasuryDataKey::Config(pair.clone());
    e.storage().persistent().has(&key)
}

/// Loads the stored token/config addresses for a pair.
///
/// Reverts if the pair has not been initialized (no config found).
pub(crate) fn get_config(env: &Env, pair: &Address) -> PairConfig {
    let key = TreasuryDataKey::Config(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(config) => {
            // Prevent TTL expiry of critical config.
            bump_persistent(env, &key);
            config
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

/// Stores token/config addresses for a pair (called from admin `add_pair`).
pub(crate) fn set_config(env: &Env, pair: &Address, config: &PairConfig) {
    let key = TreasuryDataKey::Config(pair.clone());
    env.storage().persistent().set(&key, config);
    bump_persistent(env, &key);
}

// Risk parameters

/// Loads risk parameters for a pair (toxicity threshold, etc.).
///
/// Reverts if missing (pair not initialized).
pub(crate) fn get_risk_parameters(env: &Env, pair: &Address) -> TreasuryRiskParameters {
    let key = TreasuryDataKey::RiskParameters(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(params) => {
            bump_persistent(env, &key);
            params
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

/// Stores risk parameters for a pair (called from admin `add_pair` / updates).
pub(crate) fn set_risk_parameters(env: &Env, pair: &Address, params: &TreasuryRiskParameters) {
    let key = TreasuryDataKey::RiskParameters(pair.clone());
    env.storage().persistent().set(&key, params);
    bump_persistent(env, &key);
}

// Pair balances

/// Loads the Treasury's accounted inventory for a pair.
///
/// These balances are the source-of-truth used for inventory checks and NAV calculations.
/// Reverts if missing (pair not initialized).
pub(crate) fn get_balances(env: &Env, pair: &Address) -> PairAmountsWithUSDC {
    let key = TreasuryDataKey::Balances(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(balances) => {
            bump_persistent(env, &key);
            balances
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

/// Stores the Treasury's accounted inventory for a pair.
///
/// Callers should ensure these balances remain consistent with token transfers.
pub(crate) fn set_balances(env: &Env, pair: &Address, balances: &PairAmountsWithUSDC) {
    let key = TreasuryDataKey::Balances(pair.clone());
    env.storage().persistent().set(&key, balances);
    bump_persistent(env, &key);
}

// Shares

/// Loads the total LP share supply for a pair.
///
/// Shares represent pro-rata ownership of the Treasury inventory for that pair.
/// Reverts if missing (pair not initialized).
pub(crate) fn get_total_shares(env: &Env, pair: &Address) -> u128 {
    let key = TreasuryDataKey::TotalShares(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(total_shares) => {
            bump_persistent(env, &key);
            total_shares
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

/// Stores the total LP share supply for a pair.
pub(crate) fn set_total_shares(env: &Env, pair: &Address, shares: u128) {
    let key = TreasuryDataKey::TotalShares(pair.clone());
    env.storage().persistent().set(&key, &shares);
    bump_persistent(env, &key);
}

// User Shares

/// Loads a user's LP share balance for a pair.
///
/// - If `return_zero == true`, missing storage returns `0` (useful for "view" calls).
/// - If `return_zero == false`, missing storage reverts (useful when missing shares is an invariant violation).
///
/// Note: This key is in persistent storage; callers should ensure share keys are TTL-bumped
/// on any path that must preserve withdrawability.
pub(crate) fn get_user_shares(
    env: &Env,
    pair: &Address,
    user: &Address,
    return_zero: bool,
) -> u128 {
    let key = TreasuryDataKey::UserShares(UserSharesKey {
        pair: pair.clone(),
        user: user.clone(),
    });
    match env.storage().persistent().get(&key) {
        Some(user_shares) => {
            bump_persistent(env, &key);
            user_shares
        }
        None => {
            if return_zero {
                0
            } else {
                panic_with_error!(env, StorageError::ValueNotInitialized)
            }
        }
    }
}

/// Stores a user's LP share balance for a pair.
pub(crate) fn set_user_shares(env: &Env, pair: &Address, user: &Address, shares: u128) {
    let key = TreasuryDataKey::UserShares(UserSharesKey {
        pair: pair.clone(),
        user: user.clone(),
    });
    env.storage().persistent().set(&key, &shares);
    bump_persistent(env, &key);
}

// Fee Config

/// Loads the fee configuration for a pair (maker/taker base fees + model parameters).
///
/// Reverts if missing (pair not initialized).
pub(crate) fn get_fee_config(env: &Env, pair: &Address) -> TreasuryFeeConfig {
    let key = TreasuryDataKey::FeeConfig(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(fee_config) => {
            bump_persistent(env, &key);
            fee_config
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

/// Stores the fee configuration for a pair.
pub(crate) fn set_fee_config(env: &Env, pair: &Address, config: &TreasuryFeeConfig) {
    let key = TreasuryDataKey::FeeConfig(pair.clone());
    env.storage().persistent().set(&key, config);
    bump_persistent(env, &key);
}

// Protocol fee

/// Loads accumulated protocol fees (denominated in USDC) for a pair.
///
/// This is the amount claimable by the admin via `claim_protocol_fees`.
/// Reverts if missing (pair not initialized).
pub(crate) fn get_protocol_fees(env: &Env, pair: &Address) -> u128 {
    let key = TreasuryDataKey::ProtocolFees(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(fees) => {
            bump_persistent(env, &key);
            fees
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

/// Stores accumulated protocol fees (denominated in USDC) for a pair.
pub(crate) fn set_protocol_fees(env: &Env, pair: &Address, fees: &u128) {
    let key = TreasuryDataKey::ProtocolFees(pair.clone());
    env.storage().persistent().set(&key, fees);
    bump_persistent(env, &key);
}
