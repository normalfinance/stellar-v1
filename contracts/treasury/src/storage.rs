use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env};
pub use utils::bump::bump_instance;
use utils::bump::bump_persistent;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryPairDetails {
    pub pair: Address,
    pub token_quote: Address,
    pub token_long: Address,
    pub token_short: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryPairBalances {
    pub token_quote: u128,
    pub token_long: u128,
    pub token_short: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryPairSummary {
    pub details: TreasuryPairDetails,
    pub balances: TreasuryPairBalances,
    pub prices: (u128, u128),
    pub total_pairs: u128,
    pub total_shares: u128,
    pub fee_config: TreasuryFeeConfig,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryUserPairSummary {
    pub pair_summary: TreasuryPairSummary,
    pub user_shares: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryFeeConfig {
    pub maker_fee: u128,
    pub taker_fee: u128,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // Store which tokens this pair uses, config, etc.
    PairDetails(Address),  // pair
    PairBalances(Address), // pair

    // Total LP share supply per pair
    TotalShares(Address), // pair

    // LP share balance for (pair, user)
    UserShares(Address, Address), // (pair, user)

    FeeConfig(Address), // pair > TreasuryFeeConfig
    ProtocolFees(Address),

    // Paused ops
    IsKilledDeposit,
    IsKilledWithdraw,
    IsKilledTrade,
}

// Paused Ops
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_deposit,
    DataKey::IsKilledDeposit,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_withdraw,
    DataKey::IsKilledWithdraw,
    bool,
    false
);
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_trade,
    DataKey::IsKilledTrade,
    bool,
    false
);

// Pair details
pub(crate) fn get_pair_details(env: &Env, pair: &Address) -> TreasuryPairDetails {
    let key = DataKey::PairDetails(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(details) => {
            bump_persistent(env, &key);
            details
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

pub(crate) fn set_pair_details(env: &Env, pair: &Address, details: &TreasuryPairDetails) {
    let key = DataKey::PairDetails(pair.clone());
    env.storage().persistent().set(&key, details);
    bump_persistent(env, &key);
}

// Pair balances
pub(crate) fn get_pair_balances(env: &Env, pair: &Address) -> TreasuryPairBalances {
    let key = DataKey::PairBalances(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(balances) => {
            bump_persistent(env, &key);
            balances
        }
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

pub(crate) fn set_pair_balances(env: &Env, pair: &Address, balances: &TreasuryPairBalances) {
    let key = DataKey::PairBalances(pair.clone());
    env.storage().persistent().set(&key, balances);
    bump_persistent(env, &key);
}

// Shares
pub(crate) fn get_total_shares(env: &Env, pair: &Address) -> u128 {
    let key = DataKey::TotalShares(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(total_shares) => {
            bump_persistent(env, &key);
            total_shares
        }
        None => 0,
    }
}

pub(crate) fn set_total_shares(env: &Env, pair: &Address, shares: u128) {
    let key = DataKey::TotalShares(pair.clone());
    env.storage().persistent().set(&key, &shares);
    bump_persistent(env, &key);
}

// User Shares
pub(crate) fn get_user_shares(env: &Env, pair: &Address, user: &Address) -> u128 {
    let key = DataKey::UserShares(pair.clone(), user.clone());
    match env.storage().persistent().get(&key) {
        Some(user_shares) => {
            bump_persistent(env, &key);
            user_shares
        }
        None => 0,
    }
}

pub(crate) fn set_user_shares(env: &Env, pair: &Address, user: &Address, shares: u128) {
    let key = DataKey::UserShares(pair.clone(), user.clone());
    env.storage().persistent().set(&key, &shares);
    bump_persistent(env, &key);
}

// Fee Config
pub(crate) fn get_fee_config(env: &Env, pair: &Address) -> TreasuryFeeConfig {
    let key = DataKey::FeeConfig(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(fee_config) => {
            bump_persistent(env, &key);
            fee_config
        }
        None => TreasuryFeeConfig {
            maker_fee: 0,
            taker_fee: 0,
        },
    }
}

pub(crate) fn set_fee_config(env: &Env, pair: &Address, fee_config: TreasuryFeeConfig) {
    let key = DataKey::FeeConfig(pair.clone());
    env.storage().persistent().set(&key, &fee_config);
    bump_persistent(env, &key);
}

// Protocol fee
pub(crate) fn get_protocol_fees(env: &Env, pair: &Address) -> u128 {
    let key = DataKey::ProtocolFees(pair.clone());
    match env.storage().persistent().get(&key) {
        Some(details) => {
            bump_persistent(env, &key);
            details
        }
        None => 0,
    }
}

pub(crate) fn set_protocol_fees(env: &Env, pair: &Address, fees: &u128) {
    let key = DataKey::ProtocolFees(pair.clone());
    env.storage().persistent().set(&key, fees);
    bump_persistent(env, &key);
}
