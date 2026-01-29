use oracle::state::HistoricalOracleData;
use soroban_sdk::{Address, Env};
use types::oracle::OraclePriceData;

use crate::storage::GuardRails;

pub trait NormalOracleTrait {
    fn get_oracle_price(e: Env) -> OraclePriceData;

    fn get_price(e: Env) -> HistoricalOracleData;

    fn get_guard_rails(e: Env) -> GuardRails;

    fn update_price(e: Env) -> HistoricalOracleData;

    fn set_seconds_before_stale(e: Env, admin: Address, stale_limit: u64);

    fn set_too_volatile_ratio(e: Env, admin: Address, too_volatile_ratio: u64);

    fn set_sanitize_clamp_denominator(e: Env, admin: Address, sanitize_clamp_denominator: u128);
}
