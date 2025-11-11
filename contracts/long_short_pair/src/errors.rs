use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum LongShortPairError {
    AlreadyInitialized = 201,
    WrongInputVecSize = 202,
    InvalidOracle = 203,
}
