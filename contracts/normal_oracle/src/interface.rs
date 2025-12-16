use oracle::state::OraclePriceData;
use soroban_sdk::{Address, Env};

use crate::storage::GuardRails;

pub trait NormalOracleTrait {
    fn get_price(e: Env) -> OraclePriceData;

    fn get_guard_rails(e: Env) -> GuardRails;

    fn set_seconds_before_stale(e: Env, admin: Address, stale_limit: u64);

    fn set_too_volatile_ratio(e: Env, admin: Address, too_volatile_ratio: u64);
}
