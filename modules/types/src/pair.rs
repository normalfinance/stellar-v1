use soroban_sdk::{contracttype, Address, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairParams {
    // Access control
    pub admin: Address,
    pub emergency_admin: Address,
    pub operations_admin: Address,
    pub pause_admin: Address,
    pub emergency_pause_admins: Vec<Address>,
    pub rewards_admin: Address,
    pub system_fee_admin: Address,

    // Config
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Long,
    Short,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    Buy,
    Sell,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairPriceBounds {
    pub lower: u128,
    pub upper: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairAmounts {
    pub long: u128,
    pub short: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairAmountsWithUSDC {
    pub long: u128,
    pub short: u128,
    pub usdc: u128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairTokens {
    pub long: Address,
    pub short: Address,
    pub collateral: Address,
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
    pub tokens: PairTokens,
    pub price_bounds: PairPriceBounds,
    pub collateral: CollateralInfo,
    pub calculator: Address,
    pub oracle: Address,
}
