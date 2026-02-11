#![cfg(test)]

use crate::testutils::Setup;
use access_control::constants::ADMIN_ACTIONS_DELAY;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Symbol, Vec};
use utils::test_utils::{install_dummy_wasm, jump};

// test admin transfer ownership
#[test]
#[should_panic(expected = "Error(Contract, #2908)")]
fn test_admin_transfer_ownership_too_early() {
    let setup = Setup::default();
    let pair = setup.pair;
    let admin_original = setup.users[0].clone();
    let admin_new = Address::generate(&setup.env);

    pair.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(pair
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY - 1);
    pair.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2906)")]
fn test_admin_transfer_ownership_twice() {
    let setup = Setup::default();
    let pair = setup.pair;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    pair.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    pair.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_admin_transfer_ownership_not_committed() {
    let setup = Setup::default();
    let pair = setup.pair;
    let admin_original = setup.admin;

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    pair.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_admin_transfer_ownership_reverted() {
    let setup = Setup::default();
    let pair = setup.pair;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    pair.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(pair
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    pair.revert_transfer_ownership(&admin_original, &symbol_short!("Admin"));
    pair.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
fn test_admin_transfer_ownership() {
    let setup = Setup::default();
    let pair = setup.pair;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    pair.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(pair
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    pair.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));

    pair.commit_transfer_ownership(&admin_new, &symbol_short!("Admin"), &admin_new);
}

// test emergency admin transfer ownership
#[test]
#[should_panic(expected = "Error(Contract, #2908)")]
fn test_emergency_admin_transfer_ownership_too_early() {
    let setup = Setup::default();
    let pair = setup.pair;
    let emergency_admin_new = Address::generate(&setup.env);

    pair.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(pair
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(pair
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY - 1);
    pair.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2906)")]
fn test_emergency_admin_transfer_ownership_twice() {
    let setup = Setup::default();
    let pair = setup.pair;
    let emergency_admin_new = Address::generate(&setup.env);

    pair.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
    pair.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_emergency_admin_transfer_ownership_not_committed() {
    let setup = Setup::default();
    let pair = setup.pair;

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    pair.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_emergency_admin_transfer_ownership_reverted() {
    let setup = Setup::default();
    let pair = setup.pair;
    let emergency_admin_new = Address::generate(&setup.env);

    pair.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(pair
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(pair
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    pair.revert_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
    pair.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
fn test_emergency_admin_transfer_ownership() {
    let setup = Setup::default();
    let pair = setup.pair;
    let emergency_admin_new = Address::generate(&setup.env);

    pair.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(pair
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(pair
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    pair.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    // check emergency admin has changed
    assert!(pair
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_ok());
    assert!(pair
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_err());
}

#[test]
fn test_transfer_ownership_separate_deadlines() {
    let setup = Setup::default();
    let pair = setup.pair;
    let admin_new = Address::generate(&setup.env);
    let emergency_admin_new = Address::generate(&setup.env);

    assert_eq!(
        pair.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        setup.emergency_admin
    );
    assert_eq!(
        pair.get_future_address(&symbol_short!("Admin")),
        setup.admin
    );

    assert!(pair
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(pair
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    pair.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
    jump(&setup.env, 10);
    pair.commit_transfer_ownership(&setup.admin, &symbol_short!("Admin"), &admin_new);

    assert_eq!(
        pair.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        emergency_admin_new
    );
    assert_eq!(pair.get_future_address(&symbol_short!("Admin")), admin_new);

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1 - 10);
    pair.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
    assert!(pair
        .try_apply_transfer_ownership(&setup.admin, &symbol_short!("Admin"))
        .is_err());

    assert_eq!(
        pair.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        emergency_admin_new
    );

    jump(&setup.env, 10);
    pair.apply_transfer_ownership(&setup.admin, &symbol_short!("Admin"));

    assert_eq!(pair.get_future_address(&symbol_short!("Admin")), admin_new);

    // check ownership transfer is complete. new admin is capable to call protected methods
    //      and new emergency admin can change toggle emergency mode
    pair.commit_transfer_ownership(&admin_new, &Symbol::new(&setup.env, "Admin"), &setup.admin);
    assert!(pair
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_ok());
    assert!(pair
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_err());
}

// upgrade pair & token
#[test]
fn test_commit_upgrade() {
    let setup = Setup::default();
    let pair = setup.pair;
    let new_wasm = install_dummy_wasm(&setup.env);
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin, true),
        (setup.emergency_admin, false),
        (setup.rewards_admin, false),
        (setup.operations_admin, false),
        (setup.pause_admin, false),
        (setup.emergency_pause_admin, false),
    ] {
        assert_eq!(pair.try_commit_upgrade(&addr, &new_wasm).is_ok(), is_ok);
    }
}

#[test]
fn test_apply_upgrade_third_party_user() {
    let setup = Setup::default();
    let pair = setup.pair;
    let user = Address::generate(&setup.env);
    pair.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(pair.try_apply_upgrade(&user).is_err());
}

#[test]
fn test_apply_upgrade_emergency_admin() {
    let setup = Setup::default();
    let pair = setup.pair;
    pair.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(pair.try_apply_upgrade(&setup.emergency_admin).is_err());
}

#[test]
fn test_apply_upgrade_admin() {
    let setup = Setup::default();
    let pair = setup.pair;
    let new_wasm = install_dummy_wasm(&setup.env);

    assert_ne!(pair.version(), 130);

    pair.commit_upgrade(&setup.admin, &new_wasm);
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert_eq!(pair.apply_upgrade(&setup.admin), new_wasm);

    // check contracts updated, dummy contract version is 130
    assert_eq!(pair.version(), 130);
}

// emergency mode
#[test]
fn test_set_emergency_mode_third_party_user() {
    let setup = Setup::default();
    let pair = setup.pair;
    let user = Address::generate(&setup.env);
    assert!(pair.try_set_emergency_mode(&user, &false).is_err());
}

#[test]
fn test_set_emergency_mode_admin() {
    let setup = Setup::default();
    let pair = setup.pair;
    assert!(pair.try_set_emergency_mode(&setup.admin, &false).is_err());
}

#[test]
fn test_set_emergency_mode_emergency_admin() {
    let setup = Setup::default();
    let pair = setup.pair;
    assert!(pair
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());
}

// kill switches
#[test]
fn test_kill_mint() {
    let setup = Setup::default();
    let pair = setup.pair;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        (setup.rewards_admin, false),
        (setup.operations_admin, false),
        (setup.pause_admin, true),
        (setup.emergency_pause_admin, true),
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
        (setup.rewards_admin, false),
        (setup.operations_admin, false),
        (setup.pause_admin, true),
        (setup.emergency_pause_admin, true),
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
        (setup.rewards_admin, false),
        (setup.operations_admin, false),
        (setup.pause_admin, true),
        (setup.emergency_pause_admin, true),
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
        (setup.rewards_admin, false),
        (setup.operations_admin, false),
        (setup.pause_admin, true),
        (setup.emergency_pause_admin, true),
    ] {
        assert_eq!(pair.try_unkill_redeem(&addr).is_ok(), is_ok);
    }
}

// manage privileged addresses
#[test]
fn test_set_privileged_addresses() {
    let setup = Setup::default();
    let pair = setup.pair;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin.clone(), true),
        (setup.rewards_admin.clone(), false),
        (setup.operations_admin.clone(), false),
        (setup.pause_admin.clone(), false),
        (setup.emergency_pause_admin.clone(), false),
    ] {
        assert_eq!(
            pair.try_set_privileged_addrs(
                &addr,
                &setup.rewards_admin,
                &setup.operations_admin,
                &setup.pause_admin,
                &Vec::from_array(&setup.env, [setup.emergency_pause_admin.clone()])
            )
            .is_ok(),
            is_ok
        );
    }
}

#[test]
fn test_update_calculator() {
    let setup = Setup::default();
    let user = Address::generate(&setup.env);
    let calculator = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin, true),
        (setup.rewards_admin, false),
        (setup.operations_admin, true),
        (setup.pause_admin, false),
        (setup.emergency_pause_admin, false),
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
        (user, false),
        (setup.admin, true),
        (setup.rewards_admin, false),
        (setup.operations_admin, true),
        (setup.pause_admin, false),
        (setup.emergency_pause_admin, false),
    ] {
        assert_eq!(setup.pair.try_set_oracle(&addr, &oracle).is_ok(), is_ok);
    }
}
