use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum LongShortPairError {
    AlreadyInitialized = 201,
    WrongInputVecSize = 202,
    InvalidOracle = 203,
    InvalidInput = 204,
    FundingWasNotUpdated = 205,
    FailedToGetPoolReserves = 206,
    FailedToGetCalculatorPercent = 207,
    FailedToUpdateTokenScalingFactor = 208,
    FailedToGetOraclePrice = 209,
    PoolsNotSet = 210,
}
