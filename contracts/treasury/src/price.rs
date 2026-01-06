use soroban_sdk::{panic_with_error, Address, Env, Symbol, Vec};
use types::pair::CollateralInfo;
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::SafeMath;

use crate::errors::TreasuryError;
use crate::storage::TreasuryPairBalances;

pub fn get_pair_collateral_info(e: &Env, pair: &Address) -> CollateralInfo {
    match e.try_invoke_contract::<CollateralInfo, soroban_sdk::Error>(
        pair,
        &Symbol::new(e, "get_collateral_info"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, TreasuryError::FailedToGetOraclePrice),
        Ok(Ok(collateral_info)) => {
            return collateral_info;
        }
    }
}

pub fn get_prices(e: &Env, pair: &Address) -> (u128, u128) {
    let collateral_info = get_pair_collateral_info(e, pair);

    let long_price = collateral_info.collateral_percent_long as u128;
    let short_price = PRICE_PRECISION.safe_sub(e, long_price);

    (long_price, short_price)
}

pub fn tvl(e: &Env, balances: &TreasuryPairBalances, long_price: u128, short_price: u128) -> u128 {
    let value_long = balances
        .token_long
        .safe_mul(e, long_price)
        .safe_div(e, PRICE_PRECISION);
    let value_short = balances
        .token_short
        .safe_mul(e, short_price)
        .safe_div(e, PRICE_PRECISION);

    balances
        .token_quote
        .safe_add(e, value_long)
        .safe_add(e, value_short)
}

/// Apply fee to an input amount (fee taken from the input, stays in treasury in USDC trades).
/// net_in = in * (1 - fee)
pub fn apply_fee_to_input(amount_in: u128, fee: u128) -> u128 {
    (amount_in * (PRICE_PRECISION - fee)) / PRICE_PRECISION
}

/// Convert USDC -> token using oracle price.
/// token_out = usdc_net / price_token
///
/// price_token is in [0..ONE] representing “USDC value per 1 token”
/// e.g. if token is worth 0.25 USDC, price_token = 2_500_000
pub fn quote_buy_token(e: &Env, usdc_in: u128, price_token: u128, fee: u128) -> (u128, u128) {
    if usdc_in <= 0 {
        return (0, 0);
    }

    if price_token <= 0 {
        return (0, 0);
    }

    let usdc_net = apply_fee_to_input(usdc_in, fee);
    let usdc_fee = usdc_in.safe_sub(e, usdc_net);

    // token_out = usdc_net / price
    let token_out = usdc_net
        .safe_mul(e, PRICE_PRECISION)
        .safe_div(e, price_token);

    (token_out, usdc_fee)
}

/// Convert token -> USDC using oracle price.
/// usdc_out = token_in * price_token (then optionally fee on output)
///
/// Here we take fee from the output for symmetry: net_out = out * (1 - fee)
pub fn quote_sell_token(e: &Env, token_in: u128, price_token: u128, fee: u128) -> (u128, u128) {
    if token_in <= 0 {
        return (0, 0);
    }

    if price_token <= 0 {
        return (0, 0);
    }

    let gross = token_in
        .safe_mul(e, price_token)
        .safe_div(e, PRICE_PRECISION);

    let usdc_net = apply_fee_to_input(gross, fee); // take fee from output (same helper)
    let usdc_fee = gross.safe_sub(e, usdc_net);

    (usdc_net, usdc_fee)
}
