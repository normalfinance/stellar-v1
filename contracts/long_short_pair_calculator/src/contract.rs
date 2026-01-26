use crate::errors::CalculatorError;
use crate::interface::LongShortPairCalculatorTrait;
use soroban_sdk::{contract, contractimpl, contractmeta, panic_with_error, Env};
use utils::constant::PRICE_PRECISION;

contractmeta!(
    key = "Description",
    val = "Price logic to create linear Long Short Pairs"
);

/// Linear Long Short Pair Calculator.
///
/// Adds logic to create linear LSPs. The contract will payout a scaled amount of collateral
/// depending on where the price lands within a price range between an upper_bound and a lower_bound. If
/// price is within the price range then the collateral_percent_long is defined by
/// (oracle_price - lower_bound) / (upper_bound - lower_bound).
///
/// This number represent the amount of collateral from the collateral_per_pair that will be sent to the
/// long and short side. If the price is higher than the upper_bound then
/// collateral_percent_long = 1. if the price is lower than the lower bound then collateral_percent_long = 0.
///
/// For example, consider a linear LSP on the price of ETH collateralized in USDC with an upper_bound = 4000 and lower_bound = 2000 with a
/// collateral_per_pair of $100 (i.e each pair of long and shorts is worth $100 USDC). The collateral_percent_long
/// would equal 1 (each long worth $100 and short worth 0) if ETH price was > 4000 and it would equal 0 if < 2000
/// (each long is worthless and each short is worth $100). If between the two (say 3500) then collateral_percent_long
/// = (3500 - 2000) / (4000 - 2000) = 0.75. Therefore each long is worth $75 and each short is worth $25.
#[contract]
pub struct LongShortPairCalculator;

#[contractimpl]
impl LongShortPairCalculatorTrait for LongShortPairCalculator {
    /// Returns a number between 0 and 1 to indicate how much collateral each long and short token is entitled
    /// to per collateral_per_pair.
    /// @param oracle_price price from the oracle for the target asset.
    /// @param lower_bound lower price boundary from the Long Short Pair.
    /// @param upper_bound upper price boundary from the Long Short Pair.
    /// @return expiryPercentLong to indicate how much collateral should be sent between long and short tokens.
    fn percent_long_collateral(
        e: Env,
        oracle_price: u128,
        lower_bound: u128,
        upper_bound: u128,
    ) -> u128 {
        if upper_bound == 0 && lower_bound == 0 {
            panic_with_error!(&e, CalculatorError::InvalidBounds);
        }

        if lower_bound >= upper_bound {
            panic_with_error!(&e, CalculatorError::InvalidBounds);
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
