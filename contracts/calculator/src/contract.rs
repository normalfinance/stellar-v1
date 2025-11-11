use crate::errors::CalculatorError;
use crate::interface::CalculatorTrait;
use crate::ln::flog;
use crate::storage::{get_params, set_params, LinearLongShortPairParameters};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{contract, contractimpl, contractmeta, panic_with_error, Address, Env};
use utils::constant::{PRICE_PRECISION, PRICE_PRECISION_I128};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "");

#[contract]
pub struct Calculator;

#[contractimpl]
impl CalculatorTrait for Calculator {
    fn set_long_short_pair_parameters(
        e: Env,
        long_short_pair: Address,
        upper_bound: u128,
        lower_bound: u128,
    ) {
        if upper_bound <= lower_bound {
            panic_with_error!(&e, CalculatorError::InvalidBounds);
        }

        let params = get_params(&e, long_short_pair.clone());

        if params.upper_bound == 0 && params.lower_bound == 0 {
            panic_with_error!(&e, CalculatorError::ParamsAlreadySet);
        }

        let new_params = LinearLongShortPairParameters {
            upper_bound,
            lower_bound,
        };
        set_params(&e, long_short_pair, new_params);
    }

    /**
     * @notice Returns a number between 0 and 1e18 to indicate how much collateral each long and short token is entitled
     * to per collateralPerPair.
     * @param expiry_price price from the optimistic oracle for the LSP price identifier.
     * @return expiryPercentLong to indicate how much collateral should be sent between long and short tokens.
     */
    fn pct_long_colat_at_expiry(e: Env, caller: Address, price: u128) -> u128 {
        let params = get_params(&e, caller); // user is the calling LongShortPair contract

        if params.upper_bound == 0 || params.lower_bound == 0 {
            panic_with_error!(&e, CalculatorError::ParamsAlreadySet);
        }

        // clamp
        if price <= params.lower_bound {
            return params.lower_bound;
        }
        if price >= params.upper_bound {
            return params.upper_bound;
        }

        // geometric midpoint
        let midpoint = params
            .lower_bound
            .fixed_mul_ceil(params.upper_bound, PRICE_PRECISION)
            .unwrap()
            .isqrt() as i128;

        // ln(P / mid) / (2 * ln(U / L))
        let ratio = flog(
            (price as i128)
                .fixed_div_ceil(midpoint, PRICE_PRECISION_I128)
                .unwrap(),
        );
        let denominator = flog(
            (params.upper_bound as i128)
                .fixed_div_ceil(params.lower_bound as i128, PRICE_PRECISION_I128)
                .unwrap(),
        ) * 2;

        let one: i128 = 10_000_000;
        let two: i128 = 20_000_000;
        let mut fraction =
            one.fixed_div_ceil(two, PRICE_PRECISION_I128).unwrap() + ratio / denominator;

        // clamp 0–1
        if fraction < 0 {
            fraction = 0;
        }
        if fraction > one {
            fraction = one;
        }

        return fraction as u128;
    }
}
