use soroban_sdk::{log, panic_with_error, Address, Env, IntoVal, Symbol, Vec};
use types::oracle::OraclePriceData;
use types::pair::PairStatus;
use utils::constant::PRICE_PRECISION;

use crate::errors::LongShortPairError;
use crate::storage::{
    get_calculator, get_lower_bound, get_oracle, get_upper_bound, set_collateral_percent_long,
    set_status,
};

pub fn get_oracle_price(e: &Env, oracle_addr: &Address) -> OraclePriceData {
    match e.try_invoke_contract::<OraclePriceData, soroban_sdk::Error>(
        oracle_addr,
        &Symbol::new(e, "get_price"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, LongShortPairError::FailedToGetOraclePrice),
        Ok(Ok(oracle_price_data)) => {
            log!(e, "oracle_price_data", oracle_price_data.price);
            return oracle_price_data;
        }
    }
}

pub fn sync_collateral(e: &Env) {
    let oracle_price_data = crate::utils::get_oracle_price(e, &get_oracle(e));

    let price_bounds = Vec::from_array(e, [get_lower_bound(e), get_upper_bound(e)]);
    let lower_bound = price_bounds.get(0).unwrap();
    let upper_bound = price_bounds.get(1).unwrap();

    match e.try_invoke_contract::<u128, soroban_sdk::Error>(
        &get_calculator(e),
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
            log!(
                e,
                "new_collateral_percent_long",
                new_collateral_percent_long
            );
            // TODO: enforce collateral invariant (long + short = 1)
            if new_collateral_percent_long > 10_000_000 {
                panic_with_error!(e, LongShortPairError::InvalidCalculatorValue);
            }

            set_collateral_percent_long(e, &new_collateral_percent_long);

            if oracle_price_data.price <= lower_bound || oracle_price_data.price >= upper_bound {
                set_status(e, &PairStatus::Settlement);
            }
        }
    }
}
