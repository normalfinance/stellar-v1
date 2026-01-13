use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum TreasuryError {
    AlreadyInitialized = 201,
    InvalidInput = 204,
    FailedToCallPairContract = 209,
    ActionPaused = 213,
    InsufficientInventory = 215,
    Slippage = 216,
    InsufficientShares = 217,
    DepositTooSmall = 218,
    WithdrawTooSmall = 219,
    InvalidBalance = 220,
    CannotPassFloor = 221,
    ToxicSideNotAccepted = 222,
    FailedToGetOraclePrice = 223,
    InvalidFee = 224,
}
