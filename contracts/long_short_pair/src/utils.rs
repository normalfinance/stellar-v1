use soroban_sdk::{log, panic_with_error, Address, Env, IntoVal, Symbol, Vec};
use types::oracle::OraclePriceData;

use crate::errors::LongShortPairError;
use crate::storage::{get_calculator, set_collateral_percent_long};

pub fn get_oracle_price(e: &Env, oracle_addr: &Address) -> OraclePriceData {
    match e.try_invoke_contract::<OraclePriceData, soroban_sdk::Error>(
        oracle_addr,
        &Symbol::new(e, "get_price"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, LongShortPairError::FailedToGetOraclePrice),
        Ok(Ok(oracle_price_data)) => {
            return oracle_price_data;
        }
    }
}

pub fn sync_collateral_percent_long(e: &Env, oracle_price_data: OraclePriceData) {
    match e.try_invoke_contract::<u64, soroban_sdk::Error>(
        &get_calculator(e),
        &Symbol::new(e, "percent_long_collateral"),
        Vec::from_array(
            e,
            [
                e.current_contract_address().into_val(e),
                oracle_price_data.price.into_val(e),
            ],
        ),
    ) {
        Ok(Err(_)) | Err(_) => {
            panic_with_error!(e, LongShortPairError::FailedToGetCalculatorPercent)
        }
        Ok(Ok(new_collateral_percent_long)) => {
            log!(
                e,
                "new_collateral_percent_long",
                new_collateral_percent_long
            );

            // Validate calculator response
            if new_collateral_percent_long > 10_000 {
                panic_with_error!(e, LongShortPairError::InvalidCalculatorValue);
            }

            set_collateral_percent_long(e, &new_collateral_percent_long);
        }
    }
}
