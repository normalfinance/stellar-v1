use soroban_sdk::{Address, Env};
use types::pair::PairAmountsWithUSDC;
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::SafeMath;

pub fn get_prices(e: &Env, pair: &Address) -> PairAmountsWithUSDC {
    let collateral_info = crate::pair::get_pair_collateral_info(e, pair);

    let usdc_price = crate::oracle::get_oracle_price(e, &crate::storage::get_oracle(e));

    PairAmountsWithUSDC {
        long: collateral_info.collateral_percent_long,
        short: PRICE_PRECISION.safe_sub(e, collateral_info.collateral_percent_long),
        usdc: usdc_price.last_price_twap,
    }
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

    let usdc_net = crate::fees::apply_fee_to_input(e, usdc_in, fee);
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

    let usdc_net = crate::fees::apply_fee_to_input(e, gross, fee); // take fee from output (same helper)
    let usdc_fee = gross.safe_sub(e, usdc_net);

    (usdc_net, usdc_fee)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    // ----------------------------
    // quote_buy_token() tests
    // ----------------------------

    #[test]
    fn quote_buy_token_returns_zero_on_zero_inputs() {
        let e = Env::default();
        let (out, fee) = quote_buy_token(&e, 0, PRICE_PRECISION, 0);
        assert_eq!((out, fee), (0, 0));

        let (out2, fee2) = quote_buy_token(&e, 100, 0, 0);
        assert_eq!((out2, fee2), (0, 0));
    }

    #[test]
    fn quote_buy_token_no_fee_price_one_to_one() {
        let e = Env::default();

        // price_token = 1.0 USDC per token => token_out = usdc_in
        let usdc_in = 1_000_000u128;
        let price = PRICE_PRECISION;
        let fee = 0;

        let (token_out, usdc_fee) = quote_buy_token(&e, usdc_in, price, fee);
        assert_eq!(usdc_fee, 0);
        assert_eq!(token_out, usdc_in);
    }

    #[test]
    fn quote_buy_token_no_fee_quarter_price_gives_4x_tokens() {
        let e = Env::default();

        // token worth 0.25 USDC => price_token = 0.25 * PREC
        // token_out = usdc_in / 0.25 = 4x usdc_in
        let usdc_in = 1_000_000u128;
        let price = PRICE_PRECISION / 4; // 0.25
        let fee = 0;

        let (token_out, usdc_fee) = quote_buy_token(&e, usdc_in, price, fee);
        assert_eq!(usdc_fee, 0);
        assert_eq!(token_out, usdc_in * 4);
    }

    #[test]
    fn quote_buy_token_with_fee_fee_is_taken_from_input() {
        let e = Env::default();

        let usdc_in = 1_000_000u128;
        let price = PRICE_PRECISION; // 1:1
        let fee = PRICE_PRECISION / 100; // 1%

        let (token_out, usdc_fee) = quote_buy_token(&e, usdc_in, price, fee);

        // net = 990_000, fee = 10_000
        let usdc_net = crate::fees::apply_fee_to_input(&e, usdc_in, fee);
        assert_eq!(usdc_fee, usdc_in - usdc_net);
        assert_eq!(token_out, usdc_net); // 1:1 price
    }

    // ----------------------------
    // quote_sell_token() tests
    // ----------------------------

    #[test]
    fn quote_sell_token_returns_zero_on_zero_inputs() {
        let e = Env::default();
        let (out, fee) = quote_sell_token(&e, 0, PRICE_PRECISION, 0);
        assert_eq!((out, fee), (0, 0));

        let (out2, fee2) = quote_sell_token(&e, 100, 0, 0);
        assert_eq!((out2, fee2), (0, 0));
    }

    #[test]
    fn quote_sell_token_no_fee_price_one_to_one() {
        let e = Env::default();

        let token_in = 1_000_000u128;
        let price = PRICE_PRECISION;
        let fee = 0;

        let (usdc_out, usdc_fee) = quote_sell_token(&e, token_in, price, fee);
        assert_eq!(usdc_fee, 0);
        assert_eq!(usdc_out, token_in);
    }

    #[test]
    fn quote_sell_token_no_fee_quarter_price_gives_quarter_usdc() {
        let e = Env::default();

        // price_token = 0.25 USDC => selling 1 token gives 0.25 USDC
        let token_in = 1_000_000u128;
        let price = PRICE_PRECISION / 4; // 0.25
        let fee = 0;

        let (usdc_out, usdc_fee) = quote_sell_token(&e, token_in, price, fee);
        assert_eq!(usdc_fee, 0);
        assert_eq!(usdc_out, token_in / 4);
    }

    #[test]
    fn quote_sell_token_with_fee_fee_is_taken_from_output() {
        let e = Env::default();

        let token_in = 1_000_000u128;
        let price = PRICE_PRECISION; // gross = 1_000_000
        let fee = PRICE_PRECISION / 100; // 1%

        let (usdc_net, usdc_fee) = quote_sell_token(&e, token_in, price, fee);

        let gross = token_in; // 1:1
        let expected_net = crate::fees::apply_fee_to_input(&e, gross, fee);
        assert_eq!(usdc_net, expected_net);
        assert_eq!(usdc_fee, gross - expected_net);
    }

    // ----------------------------
    // Round-trip sanity: buy then sell at same price, same fee
    // This won’t be perfectly symmetric due to integer flooring, so we assert bounds.
    // ----------------------------

    #[test]
    fn buy_then_sell_same_price_fee_never_increases_usdc() {
        let e = Env::default();

        let usdc_in = 1_000_000u128;
        let price = PRICE_PRECISION / 3; // ~0.333..., introduces rounding
        let fee = PRICE_PRECISION / 100; // 1%

        let (token_out, fee_in) = quote_buy_token(&e, usdc_in, price, fee);
        let (usdc_out, fee_out) = quote_sell_token(&e, token_out, price, fee);

        // Fees are non-negative, and round-trip should not create money.
        assert!(fee_in > 0 || fee == 0);
        assert!(fee_out >= 0);

        assert!(usdc_out <= usdc_in);
    }
}
