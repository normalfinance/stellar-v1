use crate::errors::CalculatorError;
use crate::interface::CalculatorTrait;
use crate::storage::{ get_params, set_params, LinearLongShortPairParameters };
use soroban_sdk::{ contract, contractimpl, contractmeta, panic_with_error, Address, Env };

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "");

#[contract]
pub struct Calculator;

#[contractimpl]
impl CalculatorTrait for Calculator {
    fn set_parameters(e: Env, long_short_pair: Address, lower_bound: u128, upper_bound: u128) {
        if upper_bound <= lower_bound {
            panic_with_error!(&e, CalculatorError::InvalidBounds);
        }

        let params = get_params(&e, long_short_pair.clone());

        if params.upper_bound != 0 || params.lower_bound != 0 {
            panic_with_error!(&e, CalculatorError::ParamsAlreadySet);
        }

        let new_params = LinearLongShortPairParameters {
            upper_bound,
            lower_bound,
        };
        set_params(&e, long_short_pair, new_params);
    }

    /**
     * @notice Returns a number between 0 and 1 to indicate how much collateral each long and short token is entitled
     * to per collateralPerPair.
     * @param oracle_price price from the optimistic oracle for the LSP price identifier.
     * @return expiryPercentLong to indicate how much collateral should be sent between long and short tokens.
     */
    fn percent_long_collateral(e: Env, caller: Address, oracle_price: u128) -> u64 {
        let params = get_params(&e, caller); // user is the calling LongShortPair contract

        if params.upper_bound == 0 && params.lower_bound == 0 {
            panic_with_error!(&e, CalculatorError::ParamsNotSetForCallingLSP);
        }

        let lower = params.lower_bound;
        let upper = params.upper_bound;

        // Clamp oracle price
        let price = oracle_price.min(upper).max(lower);

        // (price - lower) / (upper - lower)
        let numerator = price.saturating_sub(lower);
        let denominator = upper.saturating_sub(lower);

        if denominator == 0 {
            return 0;
        }

        // Scale result to 1e7 precision (0–1)
        let fraction = numerator
            .saturating_mul(1_0000000_u128)
            .checked_div(denominator)
            .unwrap_or(0);

        fraction as u64
    }
}
