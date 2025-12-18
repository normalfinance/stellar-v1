#![cfg(test)]

use crate::testutils::Setup;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

#[test]
fn test_set_seconds_before_stale() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [(user.clone(), false), (setup.admin.clone(), true)] {
        assert_eq!(
            normal_oracle
                .try_set_seconds_before_stale(&addr, &5000)
                .is_ok(),
            is_ok
        );
    }
}

#[test]
fn test_set_too_volatile_ratio() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [(user.clone(), false), (setup.admin.clone(), true)] {
        assert_eq!(
            normal_oracle
                .try_set_too_volatile_ratio(&addr, &5000)
                .is_ok(),
            is_ok
        );
    }
}
