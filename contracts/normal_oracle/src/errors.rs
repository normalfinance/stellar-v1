use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum NormalOracleError {
    AlreadyInitialized = 201,
    AssetSupported = 202,
    AssetNotSupported = 203,
    FailedToGetOraclePrice = 204,
    InvalidInput = 205,
    InvalidOracleSource = 206,
}
