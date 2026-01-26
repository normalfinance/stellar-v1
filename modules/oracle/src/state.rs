use soroban_sdk::contracttype;
use types::oracle::OraclePriceData;

use crate::errors::OracleError;

// ordered by "severity"
#[contracttype]
#[derive(Clone, Copy, PartialEq, Debug, Eq, Default)]
pub enum OracleValidity {
    NonPositive,
    TooVolatile,
    StaleForPair,
    Frozen,
    #[default]
    Valid,
}

impl OracleValidity {
    pub fn get_error_code(&self) -> OracleError {
        match self {
            OracleValidity::NonPositive => OracleError::OracleNonPositive,
            OracleValidity::TooVolatile => OracleError::OracleTooVolatile,
            OracleValidity::StaleForPair => OracleError::OracleStaleForPair,
            OracleValidity::Frozen => unreachable!(),
            OracleValidity::Valid => unreachable!(),
        }
    }
}

#[contracttype]
#[derive(Default, Clone, Copy, Eq, PartialEq, Debug)]
pub struct HistoricalOracleData {
    pub last_price: u128,
    pub last_price_twap: u128,
    pub last_update_ts: u64, // the last ts the Normal oracle was updated
    pub last_delay_ts: u64,  // delay from the oracle provider
}

impl HistoricalOracleData {
    pub fn default(oracle_price_data: OraclePriceData, now: u64) -> Self {
        HistoricalOracleData {
            last_price: oracle_price_data.price,
            last_price_twap: oracle_price_data.price,
            last_update_ts: now,
            last_delay_ts: oracle_price_data.delay.as_seconds(),
        }
    }
}
