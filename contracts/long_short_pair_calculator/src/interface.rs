use soroban_sdk::{Address, Env};

pub trait LongShortPairCalculatorTrait {
    /**
     * @notice Enables any address to set the parameters for an associated financial product.
     * @param longShortPair address of the LSP contract.
     * @param upperBound the upper price that the linear LSP will operate within.
     * @param lowerBound the lower price that the linear LSP will operate within.
     * @dev Note: a) Any address can set these parameters b) existing LSP parameters for address not set.
     * c) upperBound > lowerBound.
     * d) parameters can only be set once to prevent the deployer from changing the parameters after the fact.
     * e) For safety, parameters should be set before depositing any synthetic tokens in a liquidity pool.
     * f) longShortPair must expose an expirationTimestamp method to validate it is correctly deployed.
     */
    fn set_parameters(e: Env, long_short_pair: Address, lower_bound: u128, upper_bound: u128);

    /**
     * @notice Returns a number between 0 and 1 to indicate how much collateral each long and short token is entitled
     * to per collateralPerPair.
     * @param oracle_price price from a Normal Oracle for the LSP price identifier.
     * @return expiryPercentLong to indicate how much collateral should be sent between long and short tokens.
     */
    fn percent_long_collateral(e: Env, caller: Address, oracle_price: u128) -> u64;
}
