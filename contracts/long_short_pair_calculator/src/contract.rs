use crate::errors::CalculatorError;
use crate::interface::LongShortPairCalculatorTrait;
use soroban_sdk::{contract, contractimpl, contractmeta, panic_with_error, Env};
use utils::constant::PRICE_PRECISION;

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "");

#[contract]
pub struct LongShortPairCalculator;

#[contractimpl]
impl LongShortPairCalculatorTrait for LongShortPairCalculator {
    /**
     * @notice Returns a number between 0 and 1 to indicate how much collateral each long and short token is entitled
     * to per collateralPerPair.
     * @param oracle_price price from the optimistic oracle for the LSP price identifier.
     * @return expiryPercentLong to indicate how much collateral should be sent between long and short tokens.
     */
    fn percent_long_collateral(
        e: Env,
        oracle_price: u128,
        lower_bound: u128,
        upper_bound: u128,
    ) -> u128 {
        if upper_bound == 0 && lower_bound == 0 {
            panic_with_error!(&e, CalculatorError::ParamsNotSetForCallingPair);
        }

        let lower = lower_bound;
        let upper = upper_bound;

        // Clamp oracle price
        let price = oracle_price.min(upper).max(lower);

        // (price - lower) / (upper - lower)
        let numerator = price.saturating_sub(lower);
        let denominator = upper.saturating_sub(lower);

        if denominator == 0 {
            return 0;
        }

        // Scale to percent with 1e7 precision
        // 10_000_000 = 100%, 5_000_000 = 50%
        let fraction = numerator
            .saturating_mul(PRICE_PRECISION)
            .checked_div(denominator)
            .unwrap_or(0);

        fraction
    }
}
