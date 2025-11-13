use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum WrapperError {
    AlreadyInitialized = 201,
    InvalidToken = 202,
    InsufficientBalance = 203,
}
