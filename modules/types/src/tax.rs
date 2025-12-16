use soroban_sdk::contracttype;

// Rate Table Entry for configurable tax tables
#[derive(Clone)]
#[contracttype]
pub struct RateTableEntry {
    pub deviation: u128, // Price deviation scaled by PRICE_PRECISION
    pub rate: u32,       // Tax rate fraction
}
