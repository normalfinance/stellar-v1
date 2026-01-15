use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_sdk::{panic_with_error, Address, Env, Symbol};
use types::oracle::OracleSource;
use utils::constant::{PERCENTAGE_PRECISION_U64, TWENTY_FOUR_HOUR};

use crate::{
    errors::NormalOracleError,
    storage::{OracleConfig, OracleGuardRails},
};

pub fn validate_guard_rails(e: &Env, guard_rails: &OracleGuardRails) {
    if guard_rails.seconds_before_stale == 0 || guard_rails.seconds_before_stale > TWENTY_FOUR_HOUR
    {
        panic_with_error!(&e, NormalOracleError::InvalidInput);
    }

    if guard_rails.too_volatile_ratio > PERCENTAGE_PRECISION_U64 {
        panic_with_error!(&e, NormalOracleError::InvalidInput);
    }

    if guard_rails.sanitize_clamp_denominator > 10_000 {
        panic_with_error!(&e, NormalOracleError::InvalidInput);
    }
}

pub fn validate_oracle(e: &Env, config: &OracleConfig) {
    // FIXME: Source
    if config.source.eq(&OracleSource::Reflector) {
        panic_with_error!(e, NormalOracleError::InvalidOracleSource);
    }

    // call oracle contract to check if asset exists & it's alive
    let client = PriceFeedClient::new(e, &config.oracle);
    let result = client.try_lastprice(&Asset::Other(config.asset.clone()));

    if result.is_err() {
        panic_with_error!(e, NormalOracleError::FailedToGetOraclePrice);
    }
}
