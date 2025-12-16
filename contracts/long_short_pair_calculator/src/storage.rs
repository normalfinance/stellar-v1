use soroban_sdk::{contracttype, Address, Env};
use utils::bump::bump_persistent;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    LongShortPairParams(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearLongShortPairParameters {
    pub upper_bound: u128,
    pub lower_bound: u128,
}

pub fn get_params(e: &Env, pair: Address) -> LinearLongShortPairParameters {
    let key = DataKey::LongShortPairParams(pair);
    match e
        .storage()
        .persistent()
        .get::<DataKey, LinearLongShortPairParameters>(&key)
    {
        Some(params) => {
            bump_persistent(e, &key);
            params
        }
        None => LinearLongShortPairParameters {
            upper_bound: 0,
            lower_bound: 0,
        },
    }
}

pub fn set_params(e: &Env, pair: Address, params: LinearLongShortPairParameters) {
    let key = DataKey::LongShortPairParams(pair);
    e.storage().persistent().set(&key, &params);
    bump_persistent(e, &key);
}
