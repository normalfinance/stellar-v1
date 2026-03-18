use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum LongShortPairError {
    AlreadyInitialized = 201,
    InvalidOracle = 203,
    InvalidInput = 204,
    FailedToGetPoolReserves = 206,
    FailedToGetCalculatorPercent = 207,
    FailedToUpdateTokenScalingFactor = 208,
    FailedToGetOraclePrice = 209,
    PoolsNotSet = 210,
    FundingRateRequiresPoolLiquidity = 211,
    InvalidCalculatorValue = 212,
    MintingDisabled = 213,
    InvalidStatus = 214,
    InsufficientInventory = 215,
    ActionPaused = 216,
    PairExpired = 217,
    CollateralTypeDisabled = 218,
}
