use soroban_sdk::{panic_with_error, Env, Symbol, Vec};
use utils::state::oracle::OraclePriceData;

use crate::{errors::LongShortPairError, storage::get_normal_oracle};

pub fn get_oracle_price(e: &Env) -> OraclePriceData {
    match e.try_invoke_contract::<OraclePriceData, soroban_sdk::Error>(
        &get_normal_oracle(e),
        &Symbol::new(e, "get_price"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, LongShortPairError::FailedToGetOraclePrice),
        Ok(Ok(oracle_price_data)) => {
            return oracle_price_data;
        }
    }
}
