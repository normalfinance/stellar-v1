use soroban_sdk::{panic_with_error, Address, Env, Symbol};

pub fn validate_tokens_contracts(e: &Env, tokens: &Vec<Address>) {
    // call token contract to check if token exists & it's alive
    for token in tokens.iter() {
        SorobanTokenClient::new(e, &token).balance(&e.current_contract_address());
    }
}

pub fn validate_pools_contracts(e: &Env, pools: &Vec<Address>) {
    // call token contract to check if token exists & it's alive
    for pool in pools.iter() {
        match e.try_invoke_contract::<u64, soroban_sdk::Error>(
            &pool,
            &Symbol::new(&e, "get_reserves"),
            Vec::from_array(&e, []),
        ) {
            Ok(Err(_)) | Err(_) => {
                panic_with_error!(e, LongShortPairError::FailedToGetCalculatorPercent);
            }
            Ok(Ok(_)) => {}
        }
    }
}
