#![cfg(test)]

use crate::testutils::Setup;
use access_control::constants::ADMIN_ACTIONS_DELAY;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;
use utils::test_utils::{install_dummy_wasm, jump};

#[test]
fn test_commit_upgrade() {
    let setup = Setup::default();
    let factory = setup.factory;
    let new_wasm = install_dummy_wasm(&setup.env);
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin, true),
        (setup.emergency_admin, false),
        (setup.rewards_admin, false),
        (setup.operations_admin, false),
        (setup.system_fee_admin, false),
    ] {
        assert_eq!(factory.try_commit_upgrade(&addr, &new_wasm).is_ok(), is_ok);
    }
}

#[test]
fn test_apply_upgrade_third_party_user() {
    let setup = Setup::default();
    let factory = setup.factory;
    let user = Address::generate(&setup.env);
    factory.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(factory.try_apply_upgrade(&user).is_err());
}

#[test]
fn test_apply_upgrade_emergency_admin() {
    let setup = Setup::default();
    let factory = setup.factory;
    factory.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(factory.try_apply_upgrade(&setup.emergency_admin).is_err());
}

#[test]
fn test_apply_upgrade_admin() {
    let setup = Setup::default();
    let factory = setup.factory;
    let new_wasm = install_dummy_wasm(&setup.env);

    assert_ne!(factory.version(), 130);

    factory.commit_upgrade(&setup.admin, &new_wasm);
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert_eq!(factory.apply_upgrade(&setup.admin), new_wasm);

    // check contracts updated, dummy contract version is 130
    assert_eq!(factory.version(), 130);
}

// emergency mode
#[test]
fn test_set_emergency_mode_third_party_user() {
    let setup = Setup::default();
    let factory = setup.factory;
    let user = Address::generate(&setup.env);
    assert!(factory.try_set_emergency_mode(&user, &false).is_err());
}

#[test]
fn test_set_emergency_mode_admin() {
    let setup = Setup::default();
    let factory = setup.factory;
    assert!(factory
        .try_set_emergency_mode(&setup.admin, &false)
        .is_err());
}

#[test]
fn test_set_emergency_mode_emergency_admin() {
    let setup = Setup::default();
    let factory = setup.factory;
    assert!(factory
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());
}

// manage privileged addresses
#[test]
fn test_set_privileged_addresses() {
    let setup = Setup::default();
    let factory = setup.factory;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin.clone(), true),
        (setup.rewards_admin.clone(), false),
        (setup.operations_admin.clone(), false),
        (setup.system_fee_admin.clone(), false),
    ] {
        assert_eq!(
            factory
                .try_set_privileged_addrs(
                    &addr,
                    &setup.rewards_admin,
                    &setup.operations_admin,
                    &setup.system_fee_admin,
                )
                .is_ok(),
            is_ok
        );
    }
}

#[test]
fn test_set_pair_contract_wasm() {
    let setup = Setup::default();
    let user = Address::generate(&setup.env);
    let new_hash = install_dummy_wasm(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin, true),
        (setup.rewards_admin, false),
        (setup.operations_admin, true),
        (setup.system_fee_admin, false),
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
