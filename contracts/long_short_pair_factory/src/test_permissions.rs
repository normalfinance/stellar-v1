#![cfg(test)]

use crate::testutils::Setup;
use access_control::constants::ADMIN_ACTIONS_DELAY;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Symbol};
use utils::test_utils::{install_dummy_wasm, jump};

// kill switches
#[test]
fn test_kill_create() {
    let setup = Setup::default();
    let factory = setup.factory;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.emergency_admin, false),
        // (setup.pause_admin, true),
        // (setup.emergency_pause_admin, true),
    ] {
        assert_eq!(factory.try_kill_create(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_unkill_create() {
    let setup = Setup::default();
    let factory = setup.factory;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.emergency_admin, false),
        // (setup.pause_admin, true),
        // (setup.emergency_pause_admin, true),
    ] {
        assert_eq!(factory.try_unkill_create(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_set_pair_contract_wasm() {
    let setup = Setup::default();
    let user = Address::generate(&setup.env);
    let new_hash = install_dummy_wasm(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.emergency_admin, false),
        // (setup.pause_admin, false),
        // (setup.emergency_pause_admin, false),
    ] {
        assert_eq!(
            setup
                .factory
                .try_set_pair_contract_wasm(&addr, &new_hash)
                .is_ok(),
            is_ok
        );
    }
}
