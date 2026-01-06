#![cfg(test)]

use crate::testutils::Setup;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

// kill switches
#[test]
fn test_kill_deposit() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(treasury.try_kill_deposit(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_kill_withdraw() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(treasury.try_kill_withdraw(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_kill_trade() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(treasury.try_kill_trade(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_unkill_deposit() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(treasury.try_unkill_deposit(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_unkill_withdraw() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(treasury.try_unkill_withdraw(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_unkill_trade() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.pause_admin, true),
    ] {
        assert_eq!(treasury.try_unkill_trade(&addr).is_ok(), is_ok);
    }
}

#[test]
fn test_add_pair() {
    let setup = Setup::default();
    let user = Address::generate(&setup.env);
    let pair = Address::generate(&setup.env);
    let token = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.emergency_admin, false),
        // (setup.pause_admin, false),
    ] {
        assert_eq!(
            setup
                .treasury
                .try_add_pair(&addr, &pair, &token, &token, &token)
                .is_ok(),
            is_ok
        );
    }
}
