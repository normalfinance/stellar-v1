use soroban_sdk::Env;

pub trait LongShortPairCalculatorTrait {
    /**
     * @notice Returns a number between 0 and 1 to indicate how much collateral each long and short token is entitled
     * to per collateralPerPair.
     * @param oracle_price price from a Normal Oracle for the LSP price identifier.
     * @return expiryPercentLong to indicate how much collateral should be sent between long and short tokens.
     */
    fn percent_long_collateral(
        e: Env,
        lower_bound: u128,
        upper_bound: u128,
        oracle_price: u128,
    ) -> u64;
}
