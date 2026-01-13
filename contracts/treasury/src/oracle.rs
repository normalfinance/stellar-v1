use oracle::state::HistoricalOracleData;
use soroban_sdk::{panic_with_error, Address, Env, Symbol, Vec};

use crate::errors::TreasuryError;

pub fn get_oracle_price(e: &Env, oracle_addr: &Address) -> HistoricalOracleData {
    match e.try_invoke_contract::<HistoricalOracleData, soroban_sdk::Error>(
        oracle_addr,
        &Symbol::new(e, "get_price"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, TreasuryError::FailedToGetOraclePrice),
        Ok(Ok(historical_oracle_data)) => {
            return historical_oracle_data;
        }
    }
}
