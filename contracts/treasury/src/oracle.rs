use oracle::state::HistoricalOracleData;
use soroban_sdk::{panic_with_error, Address, Env, Symbol, Vec};

use crate::errors::TreasuryError;

/// Fetches the latest oracle price bundle from the configured oracle contract.
///
/// This is a thin wrapper around `Env::try_invoke_contract` that standardizes:
/// - the invoked method name (`"get_price"`)
/// - the expected return type ([`HistoricalOracleData`])
/// - error handling (converts any oracle failure into a `TreasuryError`)
///
/// ### Error handling semantics
/// The oracle call is treated as **required** for correct Pair operation.
/// If the oracle invocation fails for any reason (host error, missing method,
/// contract error, unexpected result), this function **reverts** with
/// [`TreasuryError::FailedToGetOraclePrice`].
///
/// ### Arguments
/// - `e`: Soroban environment.
/// - `oracle_addr`: Address of the oracle contract.
///
/// ### Returns
/// Returns a [`HistoricalOracleData`] struct (for example, containing `last_price_twap`).
///
/// ### Reverts
/// - [`TreasuryError::FailedToGetOraclePrice`] if the oracle call fails or returns an error.
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

pub fn update_oracle_price(e: &Env, oracle_addr: &Address) -> HistoricalOracleData {
    match e.try_invoke_contract::<HistoricalOracleData, soroban_sdk::Error>(
        oracle_addr,
        &Symbol::new(e, "update_price"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, TreasuryError::FailedToGetOraclePrice),
        Ok(Ok(historical_oracle_data)) => {
            return historical_oracle_data;
        }
    }
}
