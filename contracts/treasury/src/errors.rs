use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum TreasuryError {
    AlreadyInitialized = 201,
    WrongInputVecSize = 202,
    InvalidOracle = 203,
    InvalidInput = 204,
    FailedToGetCalculatorPercent = 207,
    FailedToGetOraclePrice = 209,
    InvalidCalculatorValue = 212,
    ActionPaused = 213,
    ZeroTvl = 214,
    InsufficientInventory = 215,
    Slippage = 216,
    InsufficientShares = 217,
}
