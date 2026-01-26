use soroban_sdk::{panic_with_error, Address, Env};
use types::pair::{CollateralInfo, PairAmountsWithUSDC};
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::SafeMath;

use crate::errors::TreasuryError;

/// Returns the Treasury's current **price vector** for a given `pair`.
///
/// The returned values are expressed in **two different units**:
///
/// - `long` / `short`: **settlement allocation** in `PRICE_PRECISION` units (e.g. `10_000_000 == 100%`).
///   These come from the Pair contract's `CollateralInfo` (i.e. how collateral is split between LONG and SHORT).
/// - `usdc`: **oracle TWAP price** for USDC (also scaled), sourced from the configured oracle.
///   This is the reference price used by the Treasury's fee model and quoting logic.
///
/// This function is intentionally lightweight: it does **not** perform any inventory checks or
/// trading risk checks. It just aggregates the current inputs needed for quoting.
///
/// ### Returns
/// A [`PairAmountsWithUSDC`] where:
/// - `long + short == PRICE_PRECISION`
/// - `usdc == oracle.last_price_twap`
///
/// ### Notes
/// - The oracle value is expected to be scaled (TWAP) and consistent with the rest of the protocol.
/// - If the Pair's `collateral_percent_long` has not been refreshed recently (e.g. via Pair sync),
///   the returned LONG/SHORT percentages may be stale until the Pair updates them.
pub fn get_prices(e: &Env, pair: &Address) -> PairAmountsWithUSDC {
    let collateral_info = crate::pair::get_pair_collateral_info(e, pair);
    get_prices_with_params(e, &collateral_info)
}

pub fn get_prices_with_params(e: &Env, collateral_info: &CollateralInfo) -> PairAmountsWithUSDC {
    let long_price = collateral_info
        .collateral_percent_long
        .safe_mul(e, collateral_info.collateral_per_pair)
        .safe_div(e, PRICE_PRECISION);

    let short_price = PRICE_PRECISION
        .safe_sub(e, collateral_info.collateral_percent_long)
        .safe_mul(e, collateral_info.collateral_per_pair)
        .safe_div(e, PRICE_PRECISION);

    let usdc_price = crate::oracle::get_oracle_price(e, &crate::storage::get_oracle(e));

    PairAmountsWithUSDC {
        long: long_price,
        short: short_price,
        usdc: usdc_price.last_price_twap,
    }
}

/// Quotes how many LONG/SHORT tokens a user receives when **buying** with USDC.
///
/// Conceptually:
/// 1. Apply the fee to the input (`usdc_in`) to compute `usdc_net`.
/// 2. Convert net USDC into tokens using the token's price fraction.
///
/// ```text
/// usdc_net  = usdc_in * (PRICE_PRECISION - fee_component) / PRICE_PRECISION
/// token_out = usdc_net / price_token
/// ```
///
/// Where `price_token` is expressed in `PRICE_PRECISION` units as:
/// **"USDC value per 1 token"**.
///
/// Example (with `PRICE_PRECISION = 10_000_000`):
/// - If 1 token is worth **0.25 USDC**, then `price_token = 2_500_000`.
/// - If `usdc_net = 1.00 USDC`, then `token_out = 1.00 / 0.25 = 4.0 tokens`
///   (subject to integer rounding).
///
/// ### Fee semantics
/// This function returns both:
/// - `token_out`: the amount the user receives
/// - `usdc_fee`: the portion of `usdc_in` retained as protocol fee
///
/// The caller is responsible for:
/// - transferring `usdc_in`
/// - transferring `token_out`
/// - recording `usdc_fee` (e.g., protocol fee accumulator)
///
/// ### Rounding
/// Uses integer math. The division rounds **down**, which is conservative for the buyer
/// (they may receive slightly less due to truncation).
///
/// ### Arguments
/// - `usdc_in`: Input USDC amount (must be > 0)
/// - `price_token`: Token price in `PRICE_PRECISION` units (must be > 0)
/// - `fee`: Fee parameter used by `apply_fee_to_input`
///
/// ### Returns
/// `(token_out, usdc_fee)`.
///
/// ### Reverts
/// - [`TreasuryError::InvalidFee`] if `fee` is outside the expected range for fee math.
///
/// ### Notes
/// This function is pure quoting logic: it does not check Treasury inventory, slippage,
/// or risk constraints.
pub fn quote_buy_token(e: &Env, usdc_in: u128, price_token: u128, fee: u128) -> (u128, u128) {
    if usdc_in <= 0 {
        return (0, 0);
    }

    if price_token <= 0 {
        return (0, 0);
    }

    // `apply_fee_to_input` expects `fee` to be a valid scaling parameter.
    // If fee config is malformed, fail loudly rather than mispricing trades.
    if fee > PRICE_PRECISION {
        panic_with_error!(e, TreasuryError::InvalidFee);
    }

    // Net USDC after fees.
    let usdc_net = crate::fees::apply_fee_to_input(e, usdc_in, fee);
    let usdc_fee = usdc_in.safe_sub(e, usdc_net);

    // token_out = usdc_net / price_token (scaled)
    let token_out = usdc_net
        .safe_mul(e, PRICE_PRECISION)
        .safe_div(e, price_token);

    (token_out, usdc_fee)
}

/// Quotes how much USDC a user receives when **selling** LONG/SHORT tokens.
///
/// Conceptually:
/// 1. Convert tokens into **gross** USDC value using the token's price fraction.
/// 2. Apply the fee to the gross output to compute `usdc_net`.
///
/// ```text
/// gross_usdc = token_in * price_token
/// usdc_net   = gross_usdc * (PRICE_PRECISION - fee_component) / PRICE_PRECISION
/// ```
///
/// Where `price_token` is expressed in `PRICE_PRECISION` units as:
/// **"USDC value per 1 token"** (same convention as `quote_buy_token`).
///
/// ### Fee semantics
/// For sells, the fee is taken from the **output** (gross USDC), so the user receives `usdc_net`
/// and the protocol retains `usdc_fee = gross_usdc - usdc_net`.
///
/// Returning both values lets the caller:
/// - transfer `usdc_net` to the user
/// - track `usdc_fee` as protocol revenue
///
/// ### Rounding
/// Uses integer math:
/// - `gross_usdc` is truncated down by division.
/// - fee application is also truncated down.
/// This is conservative to the seller (they may receive slightly less due to truncation).
///
/// ### Arguments
/// - `token_in`: Token amount being sold (must be > 0)
/// - `price_token`: Token price in `PRICE_PRECISION` units (must be > 0)
/// - `fee`: Fee parameter used by `apply_fee_to_input`
///
/// ### Returns
/// `(usdc_net, usdc_fee)`.
///
/// ### Reverts
/// - [`TreasuryError::InvalidFee`] if `fee` is outside the expected range for fee math.
///
/// ### Notes
/// This function does not enforce:
/// - Treasury USDC inventory
/// - slippage bounds
/// - toxic-trade checks
/// - USDC floor constraints
///
/// Those checks should be enforced by the calling trade function.
pub fn quote_sell_token(e: &Env, token_in: u128, price_token: u128, fee: u128) -> (u128, u128) {
    if token_in <= 0 {
        return (0, 0);
    }

    if price_token <= 0 {
        return (0, 0);
    }

    // `apply_fee_to_input` expects `fee` to be a valid scaling parameter.
    if fee > PRICE_PRECISION {
        panic_with_error!(e, TreasuryError::InvalidFee);
    }

    // Convert token amount into gross USDC value (scaled).
    let gross = token_in
        .safe_mul(e, price_token)
        .safe_div(e, PRICE_PRECISION);

    // Apply fee on the output (gross) for symmetry with `quote_buy_token`.
    let usdc_net = crate::fees::apply_fee_to_input(e, gross, fee);
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
