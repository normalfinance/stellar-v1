use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};
use soroban_sdk::{Address, Env};
use utils::math::safe_math::{SafeConversion, SafeMath};

pub fn get_user_balance_long(e: &Env, user: &Address) -> u128 {
    SorobanTokenClient::new(e, &crate::storage::get_token_long(e))
        .balance(user)
        .safe_to_u128(e)
}

pub fn burn_long_tokens(e: &Env, from: &Address, amount: u128) {
    let total_share = crate::storage::get_total_long_shares(e);
    crate::storage::set_total_long_shares(e, &total_share.safe_sub(e, amount));

    SorobanTokenClient::new(e, &crate::storage::get_token_long(e))
        .burn(from, &amount.safe_to_i128(&e));
}

pub fn mint_long_tokens(e: &Env, to: &Address, amount: u128) {
    let total_share = crate::storage::get_total_long_shares(e);
    crate::storage::set_total_long_shares(e, &total_share.safe_add(e, amount));

    SorobanTokenAdminClient::new(e, &crate::storage::get_token_long(e))
        .mint(to, &amount.safe_to_i128(e));
}

pub fn get_user_balance_short(e: &Env, user: &Address) -> u128 {
    SorobanTokenClient::new(e, &crate::storage::get_token_short(e))
        .balance(user)
        .safe_to_u128(e)
}

pub fn burn_short_tokens(e: &Env, from: &Address, amount: u128) {
    let total_share = crate::storage::get_total_short_shares(e);
    crate::storage::set_total_short_shares(e, &total_share.safe_sub(e, amount));

    SorobanTokenClient::new(e, &crate::storage::get_token_short(e))
        .burn(from, &amount.safe_to_i128(&e));
}

pub fn mint_short_tokens(e: &Env, to: &Address, amount: u128) {
    let total_share = crate::storage::get_total_short_shares(e);
    crate::storage::set_total_short_shares(e, &total_share.safe_add(e, amount));

    SorobanTokenAdminClient::new(e, &crate::storage::get_token_short(e))
        .mint(to, &amount.safe_to_i128(e));
}
