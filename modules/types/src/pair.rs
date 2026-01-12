use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
pub struct PairParams {
    // Config
    pub admin: Address,
    pub asset: Symbol,
    pub oracle: Address,

    // Collateral
    pub collateral_token: Address,
    pub collateral_per_pair: u128,
    pub calculator: Address,

    // Pair tokens
    pub long_token: Address,
    pub short_token: Address,

    // Price boundaries
    pub lower_bound: u128,
    pub upper_bound: u128,
}

#[contracttype]
#[derive(Clone)]
pub enum Direction {
    Long,
    Short,
}

// NOTE: Only update by appending value, DO NOT reorder them
// (https://app.almanax.ai/scan/13ca3512-fbc7-4909-929a-53855e07d7af/findings/907bf2df-518b-484d-a1fd-86bd91eb19d9)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairStatus {
    Inactive,
    // NOT HERE!
    Active,
    Expired,
    // UPDATE HERE!
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollateralInfo {
    pub collateral_token: Address,
    pub total_collateral: u128,
    pub collateral_per_pair: u128,
    pub collateral_percent_long: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairSummary {
    pub asset: Symbol,
    pub status: PairStatus,
    pub long_token: Address,
    pub short_token: Address,
    pub price_bounds: (u128, u128),
    pub collateral: CollateralInfo,
    pub calculator: Address,
    pub oracle: Address,
}
