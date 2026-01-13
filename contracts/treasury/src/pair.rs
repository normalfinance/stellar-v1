use soroban_sdk::{panic_with_error, Address, Env, IntoVal, Symbol, Vec};
use types::pair::{CollateralInfo, PairTokens};

use crate::errors::TreasuryError;

pub fn get_pair_tokens(e: &Env, pair: &Address) -> PairTokens {
    // Validate the pair exists
    crate::storage::has_config(e, pair);

    match e.try_invoke_contract::<PairTokens, soroban_sdk::Error>(
        pair,
        &Symbol::new(e, "get_tokens"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, TreasuryError::FailedToCallPairContract),
        Ok(Ok(tokens)) => {
            return tokens;
        }
    }
}

pub fn get_pair_collateral_info(e: &Env, pair: &Address) -> CollateralInfo {
    // Validate the pair exists
    crate::storage::has_config(e, pair);

    match e.try_invoke_contract::<CollateralInfo, soroban_sdk::Error>(
        pair,
        &Symbol::new(e, "get_collateral_info"),
        Vec::from_array(e, []),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, TreasuryError::FailedToCallPairContract),
        Ok(Ok(collateral_info)) => {
            return collateral_info;
        }
    }
}

pub fn mint_pair_as_treasury(e: &Env, pair: &Address, tokens_to_mint: u128) -> u128 {
    // Validate the pair exists
    crate::storage::has_config(e, pair);

    match e.try_invoke_contract::<u128, soroban_sdk::Error>(
        pair,
        &Symbol::new(e, "mint"),
        Vec::from_array(
            e,
            [
                e.current_contract_address().into_val(e),
                tokens_to_mint.into_val(e),
            ],
        ),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, TreasuryError::FailedToCallPairContract),
        Ok(Ok(collateral_used)) => {
            return collateral_used;
        }
    }
}

pub fn redeem_pair_as_treasury(e: &Env, pair: &Address, tokens_to_redeem: u128) -> u128 {
    // Validate the pair exists
    crate::storage::has_config(e, pair);

    match e.try_invoke_contract::<u128, soroban_sdk::Error>(
        pair,
        &Symbol::new(e, "redeem"),
        Vec::from_array(
            e,
            [
                e.current_contract_address().into_val(e),
                tokens_to_redeem.into_val(e),
            ],
        ),
    ) {
        Ok(Err(_)) | Err(_) => panic_with_error!(e, TreasuryError::FailedToCallPairContract),
        Ok(Ok(collateral_returned)) => {
            return collateral_returned;
        }
    }
}
