#![no_std]

use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{contracttype, panic_with_error, Address, Env};
use utils::bump::bump_instance;

#[derive(Clone)]
#[contracttype]
enum DataKey {
    TokenLong,
    TokenShort,
    TotalLongShares,
    TotalShortShares,
}

pub mod token {
    soroban_sdk::contractimport!(file = "../../wasm/token_long_short_pair.wasm");
}
pub use token::{self as token_contract, Client};
use utils::errors::storage_errors::StorageError;

/**
 * Long
 */

pub fn get_token_long(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::TokenLong) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn put_token_long(e: &Env, contract: Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::TokenLong, &contract)
}

pub fn get_user_balance_long(e: &Env, user: &Address) -> u128 {
    SorobanTokenClient::new(e, &get_token_long(e)).balance(user) as u128
}

pub fn get_total_long_shares(e: &Env) -> u128 {
    bump_instance(e);
    e.storage()
        .instance()
        .get(&DataKey::TotalLongShares)
        .unwrap_or(0)
}

pub fn put_total_long_shares(e: &Env, value: u128) {
    bump_instance(e);
    e.storage()
        .instance()
        .set(&DataKey::TotalLongShares, &value)
}

pub fn burn_long_tokens(e: &Env, from: &Address, amount: u128) {
    let total_share = get_total_long_shares(e);
    put_total_long_shares(e, total_share - amount);

    let long_contract = get_token_long(e);
    SorobanTokenClient::new(e, &long_contract).burn(from, &(amount as i128));
}

pub fn mint_long_tokens(e: &Env, to: &Address, amount: i128) {
    let total_share = get_total_long_shares(e);
    put_total_long_shares(e, total_share + (amount as u128));

    let long_contract_id = get_token_long(e);
    SorobanTokenAdminClient::new(e, &long_contract_id).mint(to, &amount);
}

/**
 * Short
 */

pub fn get_token_short(e: &Env) -> Address {
    bump_instance(e);
    match e.storage().instance().get(&DataKey::TokenShort) {
        Some(v) => v,
        None => panic_with_error!(e, StorageError::ValueNotInitialized),
    }
}

pub fn put_token_short(e: &Env, contract: Address) {
    bump_instance(e);
    e.storage().instance().set(&DataKey::TokenShort, &contract)
}

pub fn get_user_balance_short(e: &Env, user: &Address) -> u128 {
    SorobanTokenClient::new(e, &get_token_short(e)).balance(user) as u128
}

pub fn get_total_short_shares(e: &Env) -> u128 {
    bump_instance(e);
    e.storage()
        .instance()
        .get(&DataKey::TotalShortShares)
        .unwrap_or(0)
}

pub fn put_total_short_shares(e: &Env, value: u128) {
    bump_instance(e);
    e.storage()
        .instance()
        .set(&DataKey::TotalShortShares, &value)
}

pub fn burn_short_tokens(e: &Env, from: &Address, amount: u128) {
    let total_share = get_total_short_shares(e);
    put_total_short_shares(e, total_share - amount);

    let short_contract = get_token_short(e);
    SorobanTokenClient::new(e, &short_contract).burn(from, &(amount as i128));
}

pub fn mint_short_tokens(e: &Env, to: &Address, amount: i128) {
    let total_share = get_total_short_shares(e);
    put_total_short_shares(e, total_share + (amount as u128));

    let short_contract_id = get_token_short(e);
    SorobanTokenAdminClient::new(e, &short_contract_id).mint(to, &amount);
}
