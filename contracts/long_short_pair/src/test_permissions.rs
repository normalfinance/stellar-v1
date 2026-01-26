#![cfg(test)]

use crate::testutils::Setup;
use access_control::constants::ADMIN_ACTIONS_DELAY;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Symbol, Vec};
use utils::test_utils::{install_dummy_wasm, jump};

// kill switches
#[test]
fn test_kill_mint() {
    let setup = Setup::default();
    let pair = setup.pair;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(pair.try_kill_mint(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_kill_redeem() {
    let setup = Setup::default();
    let pair = setup.pair;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(pair.try_kill_redeem(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_unkill_mint() {
    let setup = Setup::default();
    let pair = setup.pair;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(pair.try_unkill_mint(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_unkill_redeem() {
    let setup = Setup::default();
    let pair = setup.pair;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(pair.try_unkill_redeem(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_update_calculator() {
    let setup = Setup::default();
    let user = Address::generate(&setup.env);
    let calculator = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.emergency_admin, false),
        // (setup.pause_admin, false),
    ] {
        assert_eq!(
            setup.pair.try_set_calculator(&addr, &calculator).is_ok(),
            is_ok
        );
    }
}

#[test]
fn test_update_oracle() {
    let setup = Setup::default();
    let user = Address::generate(&setup.env);
    let oracle = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.emergency_admin, false),
        // (setup.pause_admin, false),
    ] {
        assert_eq!(setup.pair.try_set_oracle(&addr, &oracle).is_ok(), is_ok);
    }
}
