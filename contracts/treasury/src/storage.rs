use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol};
pub use utils::bump::bump_instance;
use utils::bump::bump_persistent;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

/********** Storage Types **********/

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

/********** Storage Key Types **********/

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
    // map of pair to PairDetails
    PairDetails(Address),
    // map of pair to TreasuryPairBalances
    PairBalances(Address),
    // map of pair to Total LP share supply
    TotalShares(Address),
    // map of (pair, user) to LP share balance
    UserShares(UserSharesKey),
    // map of pair to TreasuryFeeConfig
    FeeConfig(Address),
    // map of pair to collectable protocol fees
    ProtocolFees(Address),
}

/********** Storage **********/

// Paused Ops
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
pub(crate) fn get_pair_details(env: &Env, pair: &Address) -> TreasuryPairDetails {
    let key = TreasuryDataKey::PairDetails(pair.clone());
    bump_persistent(env, &key);
    match env.storage().persistent().get(&key) {
        Some(details) => details,
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

pub(crate) fn set_pair_details(env: &Env, pair: &Address, details: &TreasuryPairDetails) {
    let key = TreasuryDataKey::PairDetails(pair.clone());
    env.storage().persistent().set(&key, details);
    bump_persistent(env, &key);
}

// Pair balances
pub(crate) fn get_pair_balances(env: &Env, pair: &Address) -> TreasuryPairBalances {
    let key = TreasuryDataKey::PairBalances(pair.clone());
    bump_persistent(env, &key);
    match env.storage().persistent().get(&key) {
        Some(balances) => balances,
        None => panic_with_error!(env, StorageError::ValueNotInitialized),
    }
}

pub(crate) fn set_pair_balances(env: &Env, pair: &Address, balances: &TreasuryPairBalances) {
    let key = TreasuryDataKey::PairBalances(pair.clone());
    env.storage().persistent().set(&key, balances);
    bump_persistent(env, &key);
}

// Shares
pub(crate) fn get_total_shares(env: &Env, pair: &Address) -> u128 {
    let key = TreasuryDataKey::TotalShares(pair.clone());
    bump_persistent(env, &key);
    match env.storage().persistent().get(&key) {
        Some(total_shares) => total_shares,
        None => 0,
    }
}

pub(crate) fn set_total_shares(env: &Env, pair: &Address, shares: u128) {
    let key = TreasuryDataKey::TotalShares(pair.clone());
    env.storage().persistent().set(&key, &shares);
    bump_persistent(env, &key);
}

// User Shares
pub(crate) fn get_user_shares(env: &Env, pair: &Address, user: &Address) -> u128 {
    let key = TreasuryDataKey::UserShares(UserSharesKey {
        pair: pair.clone(),
        user: user.clone(),
    });
    bump_persistent(env, &key);
    match env.storage().persistent().get(&key) {
        Some(user_shares) => user_shares,
        None => 0,
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
    bump_persistent(env, &key);
    match env.storage().persistent().get(&key) {
        Some(fee_config) => fee_config,
        None => TreasuryFeeConfig {
            maker_fee: 0,
            taker_fee: 0,
        },
    }
}

pub(crate) fn set_fee_config(env: &Env, pair: &Address, fee_config: TreasuryFeeConfig) {
    let key = TreasuryDataKey::FeeConfig(pair.clone());
    env.storage().persistent().set(&key, &fee_config);
    bump_persistent(env, &key);
}

// Protocol fee
pub(crate) fn get_protocol_fees(env: &Env, pair: &Address) -> u128 {
    let key = TreasuryDataKey::ProtocolFees(pair.clone());
    bump_persistent(env, &key);
    match env.storage().persistent().get(&key) {
        Some(details) => details,
        None => 0,
    }
}

pub(crate) fn set_protocol_fees(env: &Env, pair: &Address, fees: &u128) {
    let key = TreasuryDataKey::ProtocolFees(pair.clone());
    env.storage().persistent().set(&key, fees);
    bump_persistent(env, &key);
}
