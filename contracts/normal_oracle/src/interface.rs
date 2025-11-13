use soroban_sdk::Env;
use utils::state::oracle::OraclePriceData;

pub trait NormalOracleTrait {
    fn get_price(e: Env) -> OraclePriceData;
}
