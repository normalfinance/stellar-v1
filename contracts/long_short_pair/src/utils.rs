use crate::errors::LongShortPairError;
use oracle::state::HistoricalOracleData;
use soroban_sdk::{panic_with_error, Address, Env, IntoVal, Symbol, Vec};
use types::pair::PairStatus;

pub fn get_oracle_price(e: &Env, oracle_addr: &Address) -> HistoricalOracleData {
    match e.try_invoke_contract::<HistoricalOracleData, soroban_sdk::Error>(
        oracle_addr,
        &Symbol::new(e, "get_price"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, LongShortPairError::FailedToGetOraclePrice),
        Ok(Ok(historical_oracle_data)) => {
            return historical_oracle_data;
        }
    }
}

pub fn sync_collateral(e: &Env) {
    let status = crate::storage::get_status(e);

    if status == PairStatus::Expired {
        return;
    }

    let historical_oracle_data = crate::utils::get_oracle_price(e, &crate::storage::get_oracle(e));

    let lower_bound = crate::storage::get_lower_bound(e);
    let upper_bound = crate::storage::get_upper_bound(e);

    match e.try_invoke_contract::<u128, soroban_sdk::Error>(
        &crate::storage::get_calculator(e),
        &Symbol::new(e, "percent_long_collateral"),
        Vec::from_array(
            e,
            [
                historical_oracle_data.last_price_twap.into_val(e),
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

            crate::storage::set_last_update_ts(e, &current_time);
            crate::storage::set_collateral_percent_long(e, &new_collateral_percent_long);

            if historical_oracle_data.last_price_twap <= lower_bound
                || historical_oracle_data.last_price_twap >= upper_bound
            {
                crate::storage::set_status(e, &PairStatus::Expired);
                crate::storage::set_expiration_ts(e, &current_time);
            }
        }
    }
}
