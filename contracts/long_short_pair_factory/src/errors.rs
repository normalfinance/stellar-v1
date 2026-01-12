use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum LongShortPairFactoryError {
    PairAlreadyExists = 401,
    ActionPaused = 403,
    PairNotFound = 404,
}
