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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairConfig {
    pub pair: Address,
    pub long: Address,
    pub short: Address,
    pub usdc: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasurySummary {
    pub config: PairConfig,
    pub balances: PairAmountsWithUSDC,
    pub prices: PairAmountsWithUSDC,
    // pub total_pairs: u128,
    pub total_shares: u128,
    pub fee_config: TreasuryFeeConfig,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryUserSummary {
    pub summary: TreasurySummary,
    pub user_shares: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryFeeConfig {
    pub maker_base_fee: u128, // max = MAX_BASE_FEE
    pub taker_base_fee: u128, // max = MAX_BASE_FEE
    pub implied_volatility: u128,
    pub reaction_time_secs: u128, // defualt to 10mins
    pub coefficient_a: u128, // How much fee do we charge for being wrong about price movement during Δt?
    pub coefficient_c: u128, // How much extra do we charge when the treasury is already unbalanced?
    pub coefficient_d: u128, // How aggressively do we defend against holding assets that are going to zero?
    pub bound_power: u32,    // how sharply the bound defense ramps; max = MAX_BOUND_POWER
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryRiskParameters {
    pub toxic_threshold: u128, // similar to collateral_percent_long
}

/********** Storage Key Types **********/

const KEY_USDC_FLOOR_FRACTION: &str = "UsdcFloorFraction";
const KEY_ORACLE: &str = "Oracle";
const KEY_IS_KILLED_DEPOSIT: &str = "IsKilledDeposit";
const KEY_IS_KILLED_WITHDRAW: &str = "IsKilledWithdraw";
const KEY_IS_KILLED_TRADE: &str = "IsKilledTrade";

#[contracttype]
#[derive(Clone)]
pub struct UserSharesKey {
    pub pair: Address,
    pub user: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum TreasuryDataKey {
    // map of pair to Config
    Config(Address),
    // map of pair to Balances
    Balances(Address),
    // map of pair to RiskParameters
    RiskParameters(Address),
    // map of pair to Total LP share supply
    TotalShares(Address),
    // map of (pair, user) to LP share balance
    UserShares(UserSharesKey),
    // map of pair to FeeConfig
    FeeConfig(Address),
    // map of pair to collectable protocol fees
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

// Pair details
pub(crate) fn has_config(e: &Env, pair: &Address) -> bool {
    let key = TreasuryDataKey::Config(pair.clone());
    e.storage().persistent().has(&key)
}

pub(crate) fn get_config(env: &Env, pair: &Address) -> PairConfig {
    let key = TreasuryDataKey::Config(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(config) => {
            bump_persistent(env, &key);
            config
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

pub(crate) fn set_config(env: &Env, pair: &Address, config: &PairConfig) {
    let key = TreasuryDataKey::Config(pair.clone());
    env.storage().persistent().set(&key, config);
    bump_persistent(env, &key);
}

// Risk parameters
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

pub(crate) fn set_risk_parameters(env: &Env, pair: &Address, params: &TreasuryRiskParameters) {
    let key = TreasuryDataKey::RiskParameters(pair.clone());
    env.storage().persistent().set(&key, params);
    bump_persistent(env, &key);
}

// Pair balances
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

pub(crate) fn set_balances(env: &Env, pair: &Address, balances: &PairAmountsWithUSDC) {
    let key = TreasuryDataKey::Balances(pair.clone());
    env.storage().persistent().set(&key, balances);
    bump_persistent(env, &key);
}

// Shares
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

pub(crate) fn set_total_shares(env: &Env, pair: &Address, shares: u128) {
    let key = TreasuryDataKey::TotalShares(pair.clone());
    env.storage().persistent().set(&key, &shares);
    bump_persistent(env, &key);
}

// User Shares
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

pub(crate) fn set_user_shares(env: &Env, pair: &Address, user: &Address, shares: u128) {
    let key = TreasuryDataKey::UserShares(UserSharesKey {
        pair: pair.clone(),
        user: user.clone(),
    });
    env.storage().persistent().set(&key, &shares);
    bump_persistent(env, &key);
}

// Fee Config
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

pub(crate) fn set_fee_config(env: &Env, pair: &Address, config: &TreasuryFeeConfig) {
    let key = TreasuryDataKey::FeeConfig(pair.clone());
    env.storage().persistent().set(&key, config);
    bump_persistent(env, &key);
}

// Protocol fee
pub(crate) fn get_protocol_fees(env: &Env, pair: &Address) -> u128 {
    let key = TreasuryDataKey::ProtocolFees(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(details) => {
            bump_persistent(env, &key);
            details
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

pub(crate) fn set_protocol_fees(env: &Env, pair: &Address, fees: &u128) {
    let key = TreasuryDataKey::ProtocolFees(pair.clone());
    env.storage().persistent().set(&key, fees);
    bump_persistent(env, &key);
}
