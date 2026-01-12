use soroban_sdk::contracttype;
use types::oracle::OraclePriceData;
use utils::constant::PRICE_PRECISION;

use crate::errors::OracleError;

// #[contracttype]
// #[derive(Default, Clone, Copy, Debug)]
// pub struct OraclePriceData {
//     pub price: u128,
//     pub delay: Delay,
// }

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
    pub last_update_ts: u64, // unix_timestamp of last snapshot.
}

impl HistoricalOracleData {
    pub fn default_quote_oracle() -> Self {
        HistoricalOracleData {
            last_price: PRICE_PRECISION,
            last_price_twap: PRICE_PRECISION,
            ..HistoricalOracleData::default()
        }
    }

    pub fn default_price(price: u128) -> Self {
        HistoricalOracleData {
            last_price: price,
            last_price_twap: price,
            ..HistoricalOracleData::default()
        }
    }

    pub fn default_with_current_oracle(price_data: OraclePriceData) -> Self {
        HistoricalOracleData {
            last_price: price_data.price,
            last_price_twap: price_data.price,
            ..HistoricalOracleData::default()
        }
    }
}
