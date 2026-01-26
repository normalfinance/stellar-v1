#![cfg(test)]

use crate::storage::TreasuryFeeConfig;
use crate::testutils::{create_treasury_contract, Setup};
use access_control::constants::ADMIN_ACTIONS_DELAY;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};
use utils::test_utils::{install_dummy_wasm, jump};

// test admin transfer ownership
#[test]
#[should_panic(expected = "Error(Contract, #2908)")]
fn test_admin_transfer_ownership_too_early() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    treasury.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(treasury
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY - 1);
    treasury.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2906)")]
fn test_admin_transfer_ownership_twice() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    treasury.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    treasury.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_admin_transfer_ownership_not_committed() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let admin_original = setup.admin;

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    treasury.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_admin_transfer_ownership_reverted() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    treasury.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(treasury
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    treasury.revert_transfer_ownership(&admin_original, &symbol_short!("Admin"));
    treasury.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
fn test_admin_transfer_ownership() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    treasury.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(treasury
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    treasury.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));

    treasury.commit_transfer_ownership(&admin_new, &symbol_short!("Admin"), &admin_new);
}

// test emergency admin transfer ownership
#[test]
#[should_panic(expected = "Error(Contract, #2908)")]
fn test_emergency_admin_transfer_ownership_too_early() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let emergency_admin_new = Address::generate(&setup.env);

    treasury.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(treasury
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(treasury
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY - 1);
    treasury.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2906)")]
fn test_emergency_admin_transfer_ownership_twice() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let emergency_admin_new = Address::generate(&setup.env);

    treasury.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
    treasury.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_emergency_admin_transfer_ownership_not_committed() {
    let setup = Setup::default();
    let treasury = setup.treasury;

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    treasury.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_emergency_admin_transfer_ownership_reverted() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let emergency_admin_new = Address::generate(&setup.env);

    treasury.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(treasury
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(treasury
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    treasury.revert_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
    treasury.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
fn test_emergency_admin_transfer_ownership() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let emergency_admin_new = Address::generate(&setup.env);

    treasury.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(treasury
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(treasury
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    treasury.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    // check emergency admin has changed
    assert!(treasury
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_ok());
    assert!(treasury
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_err());
}

#[test]
fn test_transfer_ownership_separate_deadlines() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let admin_new = Address::generate(&setup.env);
    let emergency_admin_new = Address::generate(&setup.env);

    assert_eq!(
        treasury.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        setup.emergency_admin
    );
    assert_eq!(
        treasury.get_future_address(&symbol_short!("Admin")),
        setup.admin
    );

    assert!(treasury
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(treasury
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    treasury.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
    jump(&setup.env, 10);
    treasury.commit_transfer_ownership(&setup.admin, &symbol_short!("Admin"), &admin_new);

    assert_eq!(
        treasury.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        emergency_admin_new
    );
    assert_eq!(
        treasury.get_future_address(&symbol_short!("Admin")),
        admin_new
    );

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1 - 10);
    treasury.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
    assert!(treasury
        .try_apply_transfer_ownership(&setup.admin, &symbol_short!("Admin"))
        .is_err());

    assert_eq!(
        treasury.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        emergency_admin_new
    );

    jump(&setup.env, 10);
    treasury.apply_transfer_ownership(&setup.admin, &symbol_short!("Admin"));

    assert_eq!(
        treasury.get_future_address(&symbol_short!("Admin")),
        admin_new
    );

    // check ownership transfer is complete. new admin is capable to call protected methods
    //      and new emergency admin can change toggle emergency mode
    treasury.commit_transfer_ownership(&admin_new, &Symbol::new(&setup.env, "Admin"), &setup.admin);
    assert!(treasury
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_ok());
    assert!(treasury
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_err());
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_get_future_address_empty() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let emergency_admin = Address::generate(&env);
    let treasury = create_treasury_contract(&env, &admin, &admin);

    // treasury.init_admin(&admin);
    treasury.commit_transfer_ownership(
        &admin,
        &Symbol::new(&env, "EmergencyAdmin"),
        &emergency_admin,
    );
    treasury.apply_transfer_ownership(&admin, &Symbol::new(&env, "EmergencyAdmin"));
    assert_eq!(
        treasury.get_future_address(&Symbol::new(&env, "EmergencyAdmin")),
        emergency_admin
    );
    treasury.apply_transfer_ownership(&admin, &Symbol::new(&env, "EmergencyAdmin"));
}

// upgrade
#[test]
fn test_commit_upgrade_third_party_user() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let user = Address::generate(&setup.env);
    assert!(treasury
        .try_commit_upgrade(&user, &install_dummy_wasm(&setup.env))
        .is_err());
}

#[test]
fn test_commit_upgrade_emergency_admin() {
    let setup = Setup::default();
    let treasury = setup.treasury;

    treasury.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &setup.emergency_admin,
    );
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1); // delay is mandatory since emergency admin was set during initialization
    treasury.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    assert!(treasury
        .try_commit_upgrade(&setup.emergency_admin, &install_dummy_wasm(&setup.env))
        .is_err());
}

#[test]
fn test_commit_upgrade_admin() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    assert!(treasury
        .try_commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env))
        .is_ok());
}

#[test]
fn test_apply_upgrade_third_party_user() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let user = Address::generate(&setup.env);
    treasury.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(treasury.try_apply_upgrade(&user).is_err());
}

#[test]
fn test_apply_upgrade_emergency_admin() {
    let setup = Setup::default();
    let treasury = setup.treasury;

    treasury.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &setup.emergency_admin,
    );
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1); // delay is mandatory since emergency admin was set during initialization
    treasury.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    treasury.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(treasury.try_apply_upgrade(&setup.emergency_admin).is_err());
}

#[test]
fn test_apply_upgrade_admin() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    treasury.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert_ne!(treasury.version(), 130);
    assert!(treasury.try_apply_upgrade(&setup.admin).is_ok());
    assert_eq!(treasury.version(), 130);
}

// emergency mode
#[test]
fn test_set_emergency_mode_third_party_user() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    let user = Address::generate(&setup.env);
    assert!(treasury.try_set_emergency_mode(&user, &false).is_err());
}

#[test]
fn test_set_emergency_mode_admin() {
    let setup = Setup::default();
    let treasury = setup.treasury;
    assert!(treasury
        .try_set_emergency_mode(&setup.admin, &false)
        .is_err());
}

#[test]
fn test_set_emergency_mode_emergency_admin() {
    let setup = Setup::default();
    let treasury = setup.treasury;

    treasury.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &setup.emergency_admin,
    );
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1); // delay is mandatory since emergency admin was set during initialization
    treasury.apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    assert!(treasury
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());
}

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

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        // (setup.emergency_admin, false),
        // (setup.pause_admin, false),
    ] {
        assert_eq!(setup.treasury.try_add_pair(&addr, &pair).is_ok(), is_ok);
    }
}
