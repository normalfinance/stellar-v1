use crate::storage::{get_token_collateral, get_token_long, get_token_short};
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{Address, Env};

// Transfer
fn transfer(e: &Env, token: Address, from: &Address, to: &Address, amount: i128) {
    SorobanTokenClient::new(e, &token).transfer(from, to, &amount);
}

pub fn transfer_token_long(e: &Env, from: &Address, to: &Address, amount: u128) {
    transfer(e, get_token_long(e), from, to, amount as i128);
}

pub fn transfer_token_short(e: &Env, from: &Address, to: &Address, amount: u128) {
    transfer(e, get_token_short(e), from, to, amount as i128);
}

pub fn transfer_token_collateral(e: &Env, from: &Address, to: &Address, amount: u128) {
    transfer(e, get_token_collateral(e), from, to, amount as i128);
}

// Contract balance
pub fn get_token_long_balance(e: &Env) -> u128 {
    SorobanTokenClient::new(&e, &get_token_long(e)).balance(&e.current_contract_address()) as u128
}

pub fn get_token_short_balance(e: &Env) -> u128 {
    SorobanTokenClient::new(&e, &get_token_short(e)).balance(&e.current_contract_address()) as u128
}

pub fn get_token_collateral_balance(e: &Env) -> u128 {
    SorobanTokenClient::new(&e, &get_token_collateral(e)).balance(&e.current_contract_address())
        as u128
}

// Balance
pub fn get_token_long_balance_of(e: &Env, user: &Address) -> u128 {
    SorobanTokenClient::new(&e, &get_token_long(e)).balance(user) as u128
}

pub fn get_token_short_balance_of(e: &Env, user: &Address) -> u128 {
    SorobanTokenClient::new(&e, &get_token_short(e)).balance(user) as u128
}

pub fn get_token_collateral_balance_of(e: &Env, user: &Address) -> u128 {
    SorobanTokenClient::new(&e, &get_token_collateral(e)).balance(user) as u128
}

// Mint
pub fn mint_token_long(e: &Env, to: &Address, amount: &i128) {
    SorobanTokenAdminClient::new(e, &get_token_long(e)).mint(to, &amount);
}

pub fn mint_token_short(e: &Env, to: &Address, amount: &i128) {
    SorobanTokenAdminClient::new(e, &get_token_short(e)).mint(to, &amount);
}

// Burn
pub fn burn_token_long(e: &Env, account: &Address, amount: &i128) {
    SorobanTokenClient::new(&e, &get_token_long(e)).burn(account, amount)
}

pub fn burn_token_short(e: &Env, account: &Address, amount: &i128) {
    SorobanTokenClient::new(&e, &get_token_short(e)).burn(account, amount)
}

pub fn burn_token_long_from(e: &Env, from: &Address, amount: &i128) {
    SorobanTokenClient::new(&e, &get_token_long(e)).burn_from(
        &e.current_contract_address(),
        from,
        amount,
    );
}

pub fn burn_token_short_from(e: &Env, from: &Address, amount: &i128) {
    SorobanTokenClient::new(&e, &get_token_short(e)).burn_from(
        &e.current_contract_address(),
        from,
        amount,
    );
}
