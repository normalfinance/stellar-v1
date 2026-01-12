use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum CalculatorError {
    AlreadyInitialized = 201,
    InvalidBounds = 202,
    ParamsAlreadySet = 203,
    ParamsNotSetForCallingPair = 204,
}
