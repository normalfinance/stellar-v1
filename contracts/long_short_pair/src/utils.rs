use crate::errors::LongShortPairError;
use oracle::state::HistoricalOracleData;
use soroban_sdk::{panic_with_error, Address, Env, IntoVal, Symbol, Vec};
use types::pair::PairStatus;

/// Fetches the latest oracle price bundle from the configured oracle contract.
///
/// This is a thin wrapper around `Env::try_invoke_contract` that standardizes:
/// - the invoked method name (`"get_price"`)
/// - the expected return type ([`HistoricalOracleData`])
/// - error handling (converts any oracle failure into a `LongShortPairError`)
///
/// ### Error handling semantics
/// The oracle call is treated as **required** for correct Pair operation.
/// If the oracle invocation fails for any reason (host error, missing method,
/// contract error, unexpected result), this function **reverts** with
/// [`LongShortPairError::FailedToGetOraclePrice`].
///
/// ### Arguments
/// - `e`: Soroban environment.
/// - `oracle_addr`: Address of the oracle contract.
///
/// ### Returns
/// Returns a [`HistoricalOracleData`] struct (for example, containing `last_price_twap`).
///
/// ### Reverts
/// - [`LongShortPairError::FailedToGetOraclePrice`] if the oracle call fails or returns an error.
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

/// Synchronizes the Pair's settlement state using the oracle TWAP and calculator contract.
///
/// This routine updates the **settlement allocation** (`collateral_percent_long`) based on the
/// current oracle TWAP and the Pair's configured price bounds. It also transitions the Pair to
/// [`PairStatus::Expired`] once the TWAP crosses or touches either bound.
///
/// In other words, this does **not** move collateral or reconcile token balances. It only updates
/// **price-derived state** used for settlement:
/// - `collateral_percent_long` (in `PRICE_PRECISION` units)
/// - `last_update_ts`
/// - and potentially `status`/`expiration_ts` when bounds are breached.
///
/// ### Behavior
/// 1. If the Pair is already [`PairStatus::Expired`], this is a no-op.
/// 2. Calls the oracle to retrieve [`HistoricalOracleData`] and reads `last_price_twap`.
/// 3. Calls the calculator contract's `"percent_long_collateral"` to compute a new
///    `collateral_percent_long` from:
///    - `last_price_twap`
///    - `lower_bound`
///    - `upper_bound`
/// 4. Validates the calculator output is within `[0, 10_000_000]` (i.e., `PRICE_PRECISION`).
/// 5. Writes:
///    - `last_update_ts = ledger.timestamp()`
///    - `collateral_percent_long = new value`
/// 6. If `last_price_twap <= lower_bound` or `last_price_twap >= upper_bound`,
///    sets:
///    - `status = Expired`
///    - `expiration_ts = ledger.timestamp()`
///
/// ### Reverts
/// - [`LongShortPairError::FailedToGetOraclePrice`] if the oracle call fails.
/// - [`LongShortPairError::FailedToGetCalculatorPercent`] if the calculator call fails.
/// - [`LongShortPairError::InvalidCalculatorValue`] if the calculator returns a value > `PRICE_PRECISION`.
///
/// ### Notes
/// - This function is typically called inside `redeem` / `redeem_one` flows to ensure
///   the most recent settlement allocation is used.
/// - Expiration is triggered solely by the TWAP crossing the bounds, not by time.
pub fn sync_collateral(e: &Env) {
    let status = crate::storage::get_status(e);

    // Once expired, settlement is fixed and should not be updated further.
    if status == PairStatus::Expired {
        return;
    }

    // Pull oracle data (TWAP) and compute a fresh settlement allocation from the calculator.
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
            // Calculator should always output a value in PRICE_PRECISION units (0..=1e7).
            if new_collateral_percent_long > 10_000_000 {
                panic_with_error!(e, LongShortPairError::InvalidCalculatorValue);
            }

            let current_time = e.ledger().timestamp();

            // Update settlement allocation + timestamp.
            crate::storage::set_last_update_ts(e, &current_time);
            crate::storage::set_collateral_percent_long(e, &new_collateral_percent_long);

            // Expire the pair if the TWAP has reached or crossed either boundary.
            if historical_oracle_data.last_price_twap <= lower_bound
                || historical_oracle_data.last_price_twap >= upper_bound
            {
                crate::storage::set_status(e, &PairStatus::Expired);
                crate::storage::set_expiration_ts(e, &current_time);
            }
        }
    }
}
