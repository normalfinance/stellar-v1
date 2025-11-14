use crate::constants::FEE_MULTIPLIER;
use crate::storage::{
    get_base_asset, get_fee_fraction, get_quote_asset, get_reserve_a, get_reserve_b,
};
use liquidity_pool_validation_errors::LiquidityPoolValidationError;
use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{panic_with_error, Env, Symbol};
use utils::constant::PRICE_PRECISION_I64;
use utils::{
    constant::PRICE_PRECISION,
    math::safe_math::{PrecisionMath, SafeConversion, SafeMath},
};

pub fn get_deposit_amounts(
    e: &Env,
    desired_a: u128,
    min_a: u128,
    desired_b: u128,
    min_b: u128,
    reserve_a: u128,
    reserve_b: u128,
) -> (u128, u128) {
    if reserve_a == 0 && reserve_b == 0 {
        return (desired_a, desired_b);
    }

    let amount_b = desired_a.fixed_mul_floor(e, &reserve_b, &reserve_a);
    if amount_b <= desired_b {
        if amount_b < min_b {
            panic_with_error!(e, LiquidityPoolValidationError::InvalidDepositAmount);
        }
        (desired_a, amount_b)
    } else {
        let amount_a = desired_b.fixed_mul_floor(&e, &reserve_a, &reserve_b);
        if amount_a > desired_a || amount_a < min_a {
            panic_with_error!(e, LiquidityPoolValidationError::InvalidDepositAmount);
        }
        (amount_a, desired_b)
    }
}

pub fn get_amount_out(
    e: &Env,
    in_amount: u128,    // dx  – exact tokens the trader wants to sell
    reserve_sell: u128, // x
    reserve_buy: u128,  // y
) -> (u128, u128) {
    if in_amount == 0 {
        return (0, 0);
    }

    let fee_fraction = get_fee_fraction(e) as u128; // e.g. 30 => 0.3 %
    let in_after_fee = (in_amount * (FEE_MULTIPLIER - fee_fraction)) / FEE_MULTIPLIER;
    let raw_out = in_after_fee.fixed_mul_floor(e, &reserve_buy, &(reserve_sell + in_after_fee));
    (raw_out, in_amount - in_after_fee) // fee is taken on input
}

pub fn get_amount_out_strict_receive(
    e: &Env,
    out_amount: u128,   // dy  – exact tokens the trader wants to receive
    reserve_sell: u128, // x
    reserve_buy: u128,  // y
) -> (u128, u128) {
    if out_amount == 0 {
        return (0, 0);
    }
    if out_amount >= reserve_buy {
        panic_with_error!(e, LiquidityPoolValidationError::InsufficientBalance);
    }

    let fee_fraction = get_fee_fraction(&e) as u128;

    // ----------  Step 1: dx_after_fee = ceil(x·dy / (y-dy))  ----------
    let dx_after_fee = reserve_sell.fixed_mul_ceil(e, &out_amount, &(reserve_buy - out_amount));

    // ----------  Step 2: gross-up for fee on *input* side  -------------
    // dx_before_fee = ceil( dx_after_fee / (1-f) )
    let dx_before_fee =
        dx_after_fee.fixed_mul_ceil(e, &FEE_MULTIPLIER, &(FEE_MULTIPLIER - fee_fraction));

    // ----------  Step 3: fee = dx_before_fee - dx_after_fee -----------
    let fee = dx_before_fee - dx_after_fee;

    (dx_before_fee, fee)
}

pub fn is_token_a_synthetic(e: &Env) -> bool {
    get_quote_asset(e) == Symbol::new(e, "USDC")
}

pub fn pool_price(e: &Env) -> u128 {
    let reserve_a = get_reserve_a(e);
    let reserve_b = get_reserve_b(e);

    if reserve_a == 0 || reserve_b == 0 {
        return 0;
    }

    if is_token_a_synthetic(e) {
        reserve_b.safe_fixed_div_round(e, reserve_a, PRICE_PRECISION)
    } else {
        reserve_a.safe_fixed_div_round(e, reserve_b, PRICE_PRECISION)
    }
}

// Calculates the peg price between the base and quote assets based on oracle prices.
//
// Returns `quote / base` to represent the current price ratio. If either price is zero,
// returns 0 to indicate an invalid state.
//
// # Arguments
// * `e` - Soroban environment reference.
//
// # Returns
// * `u128` — The derived peg price (scaled by `PRICE_PRECISION`), or 0 if invalid.
pub fn peg_price(e: &Env, current_time: u64) -> u128 {
    let base_oracle_price_data =
        crate::oracle::get_oracle_price_with_validity(e, &get_base_asset(e), current_time);
    let quote_oracle_price_data =
        crate::oracle::get_oracle_price_with_validity(e, &get_quote_asset(e), current_time);

    if base_oracle_price_data.last_price_twap == 0 || quote_oracle_price_data.last_price_twap == 0 {
        return 0;
    }

    // Calculate quote_oracle_price / base_oracle_price with round-to-nearest to reduce bias
    if is_token_a_synthetic(e) {
        quote_oracle_price_data
            .last_price_twap
            .safe_fixed_div_round(e, base_oracle_price_data.last_price_twap, PRICE_PRECISION)
    } else {
        base_oracle_price_data.last_price_twap.safe_fixed_div_round(
            e,
            quote_oracle_price_data.last_price_twap,
            PRICE_PRECISION,
        )
    }
}

pub fn calculate_price_difference(e: &Env, price_a: u128, price_b: u128) -> i128 {
    // Use safe conversions to prevent overflow
    let price_a_i128 = price_a.safe_to_i128(e);
    let price_b_i128 = price_b.safe_to_i128(e);

    price_a_i128.safe_sub(e, price_b_i128)
}

pub fn is_swap_risk_increasing(e: &Env, pool_price: u128, peg_price: u128, out_idx: u32) -> bool {
    if pool_price == 0 || peg_price == 0 {
        return false;
    }

    if out_idx > 1 {
        panic_with_error!(&e, LiquidityPoolValidationError::OutTokenOutOfBounds);
    }

    // Calculate difference: pool_price - peg_price
    // i.e. $105 (pool) - $100 (peg) = 5 -> positive diff -> pool price needs lowered
    // i.e. $95 (pool) - $100 (peg) = -5 -> negative diff -> pool price needs increased
    let price_diff = calculate_price_difference(e, pool_price, peg_price);

    // No price difference
    if price_diff == 0 {
        return false;
    }

    let token_a_synthetic = is_token_a_synthetic(e);

    let buying_normal_token =
        if (token_a_synthetic && out_idx == 0) || (!token_a_synthetic && out_idx == 1) {
            true
        } else {
            false
        };

    // Pool price higher than peg, buys = risk increasing
    if price_diff > 0 {
        if buying_normal_token {
            return false;
        } else {
            return true;
        }
    }

    // Pool price higher than peg, sells will be risk reducing
    if buying_normal_token {
        true
    } else {
        false
    }
}
