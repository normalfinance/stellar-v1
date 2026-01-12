#![cfg(test)]
extern crate std;

use crate::testutils::Setup;
use utils::constant::{FIVE_MINUTE, PERCENTAGE_PRECISION_U64, PRICE_PRECISION_I128};

// TODO: test double init

#[test]
fn test_get_price() {
    let setup = Setup::default();

    let price_data = setup.normal_oracle.get_price();
    assert_eq!(
        price_data.last_price as i128,
        setup.initial_asset_price / PRICE_PRECISION_I128
    );
}

#[test]
fn test_set_seconds_before_stale() {
    let setup = Setup::default();
    let stale_limit = (FIVE_MINUTE as u64) * 2;

    // Test the default
    let guard_rails = setup.normal_oracle.get_guard_rails();
    assert_eq!(guard_rails.seconds_before_stale, FIVE_MINUTE as u64);

    setup
        .normal_oracle
        .set_seconds_before_stale(&setup.admin, &stale_limit);
    let updated_guard_rails = setup.normal_oracle.get_guard_rails();

    assert_eq!(updated_guard_rails.seconds_before_stale, stale_limit);
}

#[test]
fn test_set_too_volatile_ratio() {
    let setup = Setup::default();
    let too_volatile_ratio = PERCENTAGE_PRECISION_U64 / 10;

    // Test the default
    let guard_rails = setup.normal_oracle.get_guard_rails();
    assert_eq!(guard_rails.too_volatile_ratio, PERCENTAGE_PRECISION_U64 / 5);

    setup
        .normal_oracle
        .set_too_volatile_ratio(&setup.admin, &too_volatile_ratio);
    let updated_guard_rails = setup.normal_oracle.get_guard_rails();

    assert_eq!(updated_guard_rails.too_volatile_ratio, too_volatile_ratio);
}
