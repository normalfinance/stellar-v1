use oracle::state::HistoricalOracleData;
use soroban_sdk::{Address, Env, Symbol};
use types::oracle::OraclePriceData;

use crate::storage::{OracleConfig, OracleGuardRails};

pub trait NormalOracleTrait {
    fn get_oracle_price(e: Env, asset: Symbol) -> OraclePriceData;

    fn get_price(e: Env, asset: Symbol) -> HistoricalOracleData;

    fn get_price_and_update(e: Env, asset: Symbol) -> HistoricalOracleData;

    fn get_config(e: Env, asset: Symbol) -> OracleConfig;

    fn get_guard_rails(e: Env, asset: Symbol) -> OracleGuardRails;
}

pub trait AdminInterfaceTrait {
    fn add_asset(e: Env, admin: Address, config: OracleConfig, guard_rails: OracleGuardRails);

    fn remove_asset(e: Env, admin: Address, asset: Symbol);

    fn set_guard_rails(e: Env, admin: Address, asset: Symbol, guard_rails: OracleGuardRails);
}
