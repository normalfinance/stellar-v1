use crate::interface::NormalOracleTrait;
use crate::math::oracle::get_oracle_price;
use crate::storage::{get_asset, put_asset, put_reflector_oracle};
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};
use utils::state::oracle::OraclePriceData;

#[contract]
pub struct NormalOracle;

#[contractimpl]
impl NormalOracle {
    // __constructor
    // Initializes the factory by setting the admin roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - asset: The address of the swap token_factory contract.
    //   - oracle: The WASM hash (BytesN<32>) for the long short pair contract.
    pub fn __constructor(e: Env, asset: Symbol, oracle: Address) {
        put_asset(&e, asset);
        put_reflector_oracle(&e, oracle);
    }
}

#[contractimpl]
impl NormalOracleTrait for NormalOracle {
    fn get_price(e: Env) -> OraclePriceData {
        get_oracle_price(&e, &get_asset(&e), e.ledger().timestamp())
    }
}
