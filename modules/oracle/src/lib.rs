#![no_std]

use soroban_sdk::{panic_with_error, Address, Env, Symbol, Vec};

use crate::{errors::OracleError, state::OraclePriceData};

pub mod errors;
pub mod math;
pub mod state;

pub fn get_oracle_price(e: &Env, oracle_addr: &Address) -> OraclePriceData {
    match e.try_invoke_contract::<OraclePriceData, soroban_sdk::Error>(
        oracle_addr,
        &Symbol::new(e, "get_price"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, OracleError::FailedToGetOraclePrice),
        Ok(Ok(oracle_price_data)) => {
            return oracle_price_data;
        }
    }
}
