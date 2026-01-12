use crate::errors::LongShortPairError;
use soroban_sdk::{panic_with_error, Address, Env, IntoVal, Symbol, Vec};
use types::oracle::OraclePriceData;
use types::pair::PairStatus;

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

pub fn sync_collateral(e: &Env) {
    let oracle_price_data = crate::utils::get_oracle_price(e, &crate::storage::get_oracle(e));

    let lower_bound = crate::storage::get_lower_bound(e);
    let upper_bound = crate::storage::get_upper_bound(e);

    match e.try_invoke_contract::<u128, soroban_sdk::Error>(
        &crate::storage::get_calculator(e),
        &Symbol::new(e, "percent_long_collateral"),
        Vec::from_array(
            e,
            [
                oracle_price_data.price.into_val(e),
                lower_bound.into_val(e),
                upper_bound.into_val(e),
            ],
        ),
    ) {
        Ok(Err(_)) | Err(_) => {
            panic_with_error!(e, LongShortPairError::FailedToGetCalculatorPercent)
        }
        Ok(Ok(new_collateral_percent_long)) => {
            if new_collateral_percent_long > 10_000_000 {
                panic_with_error!(e, LongShortPairError::InvalidCalculatorValue);
            }

            let current_time = e.ledger().timestamp();
            let max_price_divergence = crate::storage::get_max_price_divergence(e);
            // TODO: apply divergence

            crate::storage::set_last_update_ts(e, &current_time);
            crate::storage::set_collateral_percent_long(e, &new_collateral_percent_long);

            if oracle_price_data.price <= lower_bound || oracle_price_data.price >= upper_bound {
                crate::storage::set_status(e, &PairStatus::Expired);
                crate::storage::set_expiration_ts(e, &current_time);
            }
        }
    }
}
