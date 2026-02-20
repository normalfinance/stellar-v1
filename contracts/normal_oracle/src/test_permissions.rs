#![cfg(test)]

use crate::testutils::{create_normal_oracle_contract, Setup};
use access_control::constants::ADMIN_ACTIONS_DELAY;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};
use types::oracle::OracleSource;
use utils::test_utils::{install_dummy_wasm, jump};

// test admin transfer ownership
#[test]
#[should_panic(expected = "Error(Contract, #2908)")]
fn test_admin_transfer_ownership_too_early() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    normal_oracle.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(normal_oracle
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY - 1);
    normal_oracle.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2906)")]
fn test_admin_transfer_ownership_twice() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    normal_oracle.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    normal_oracle.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_admin_transfer_ownership_not_committed() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let admin_original = setup.admin;

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    normal_oracle.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_admin_transfer_ownership_reverted() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    normal_oracle.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(normal_oracle
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    normal_oracle.revert_transfer_ownership(&admin_original, &symbol_short!("Admin"));
    normal_oracle.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));
}

#[test]
fn test_admin_transfer_ownership() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let admin_original = setup.admin;
    let admin_new = Address::generate(&setup.env);

    normal_oracle.commit_transfer_ownership(&admin_original, &symbol_short!("Admin"), &admin_new);
    // check admin not changed yet by calling protected method
    assert!(normal_oracle
        .try_revert_transfer_ownership(&admin_new, &symbol_short!("Admin"))
        .is_err());
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    normal_oracle.apply_transfer_ownership(&admin_original, &symbol_short!("Admin"));

    normal_oracle.commit_transfer_ownership(&admin_new, &symbol_short!("Admin"), &admin_new);
}

// test emergency admin transfer ownership
#[test]
#[should_panic(expected = "Error(Contract, #2908)")]
fn test_emergency_admin_transfer_ownership_too_early() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let emergency_admin_new = Address::generate(&setup.env);

    normal_oracle.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(normal_oracle
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(normal_oracle
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY - 1);
    normal_oracle
        .apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2906)")]
fn test_emergency_admin_transfer_ownership_twice() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let emergency_admin_new = Address::generate(&setup.env);

    normal_oracle.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
    normal_oracle.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_emergency_admin_transfer_ownership_not_committed() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    normal_oracle
        .apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
#[should_panic(expected = "Error(Contract, #2907)")]
fn test_emergency_admin_transfer_ownership_reverted() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let emergency_admin_new = Address::generate(&setup.env);

    normal_oracle.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(normal_oracle
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(normal_oracle
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    normal_oracle
        .revert_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
    normal_oracle
        .apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
}

#[test]
fn test_emergency_admin_transfer_ownership() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let emergency_admin_new = Address::generate(&setup.env);

    normal_oracle.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );

    // check emergency admin not changed yet by calling protected method
    assert!(normal_oracle
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(normal_oracle
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    normal_oracle
        .apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    // check emergency admin has changed
    assert!(normal_oracle
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_ok());
    assert!(normal_oracle
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_err());
}

#[test]
fn test_transfer_ownership_separate_deadlines() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let admin_new = Address::generate(&setup.env);
    let emergency_admin_new = Address::generate(&setup.env);

    assert_eq!(
        normal_oracle.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        setup.emergency_admin
    );
    assert_eq!(
        normal_oracle.get_future_address(&symbol_short!("Admin")),
        setup.admin
    );

    assert!(normal_oracle
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_err());
    assert!(normal_oracle
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());

    normal_oracle.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &emergency_admin_new,
    );
    jump(&setup.env, 10);
    normal_oracle.commit_transfer_ownership(&setup.admin, &symbol_short!("Admin"), &admin_new);

    assert_eq!(
        normal_oracle.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        emergency_admin_new
    );
    assert_eq!(
        normal_oracle.get_future_address(&symbol_short!("Admin")),
        admin_new
    );

    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1 - 10);
    normal_oracle
        .apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));
    assert!(normal_oracle
        .try_apply_transfer_ownership(&setup.admin, &symbol_short!("Admin"))
        .is_err());

    assert_eq!(
        normal_oracle.get_future_address(&Symbol::new(&setup.env, "EmergencyAdmin")),
        emergency_admin_new
    );

    jump(&setup.env, 10);
    normal_oracle.apply_transfer_ownership(&setup.admin, &symbol_short!("Admin"));

    assert_eq!(
        normal_oracle.get_future_address(&symbol_short!("Admin")),
        admin_new
    );

    // check ownership transfer is complete. new admin is capable to call protected methods
    //      and new emergency admin can change toggle emergency mode
    normal_oracle.commit_transfer_ownership(
        &admin_new,
        &Symbol::new(&setup.env, "Admin"),
        &setup.admin,
    );
    assert!(normal_oracle
        .try_set_emergency_mode(&emergency_admin_new, &false)
        .is_ok());
    assert!(normal_oracle
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
    let oracle = Address::generate(&env);
    let normal_oracle = create_normal_oracle_contract(
        &env,
        &admin,
        &emergency_admin,
        &admin,
        &admin,
        &Vec::from_array(&env, [admin.clone()]),
        &Symbol::new(&env, "BTC"),
        &OracleSource::Reflector,
        &oracle,
    );

    // normal_oracle.init_admin(&admin);
    normal_oracle.commit_transfer_ownership(
        &admin,
        &Symbol::new(&env, "EmergencyAdmin"),
        &emergency_admin,
    );
    normal_oracle.apply_transfer_ownership(&admin, &Symbol::new(&env, "EmergencyAdmin"));
    assert_eq!(
        normal_oracle.get_future_address(&Symbol::new(&env, "EmergencyAdmin")),
        emergency_admin
    );
    normal_oracle.apply_transfer_ownership(&admin, &Symbol::new(&env, "EmergencyAdmin"));
}

// upgrade
#[test]
fn test_commit_upgrade_third_party_user() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let user = Address::generate(&setup.env);
    assert!(normal_oracle
        .try_commit_upgrade(&user, &install_dummy_wasm(&setup.env))
        .is_err());
}

#[test]
fn test_commit_upgrade_emergency_admin() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;

    normal_oracle.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &setup.emergency_admin,
    );
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1); // delay is mandatory since emergency admin was set during initialization
    normal_oracle
        .apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    assert!(normal_oracle
        .try_commit_upgrade(&setup.emergency_admin, &install_dummy_wasm(&setup.env))
        .is_err());
}

#[test]
fn test_commit_upgrade_admin() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    assert!(normal_oracle
        .try_commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env))
        .is_ok());
}

#[test]
fn test_apply_upgrade_third_party_user() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let user = Address::generate(&setup.env);
    normal_oracle.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(normal_oracle.try_apply_upgrade(&user).is_err());
}

#[test]
fn test_apply_upgrade_emergency_admin() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;

    normal_oracle.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &setup.emergency_admin,
    );
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1); // delay is mandatory since emergency admin was set during initialization
    normal_oracle
        .apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    normal_oracle.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert!(normal_oracle
        .try_apply_upgrade(&setup.emergency_admin)
        .is_err());
}

#[test]
fn test_apply_upgrade_admin() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    normal_oracle.commit_upgrade(&setup.admin, &install_dummy_wasm(&setup.env));
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1);
    assert_ne!(normal_oracle.version(), 130);
    assert!(normal_oracle.try_apply_upgrade(&setup.admin).is_ok());
    assert_eq!(normal_oracle.version(), 130);
}

// emergency mode
#[test]
fn test_set_emergency_mode_third_party_user() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let user = Address::generate(&setup.env);
    assert!(normal_oracle.try_set_emergency_mode(&user, &false).is_err());
}

#[test]
fn test_set_emergency_mode_admin() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    assert!(normal_oracle
        .try_set_emergency_mode(&setup.admin, &false)
        .is_err());
}

#[test]
fn test_set_emergency_mode_emergency_admin() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;

    normal_oracle.commit_transfer_ownership(
        &setup.admin,
        &Symbol::new(&setup.env, "EmergencyAdmin"),
        &setup.emergency_admin,
    );
    jump(&setup.env, ADMIN_ACTIONS_DELAY + 1); // delay is mandatory since emergency admin was set during initialization
    normal_oracle
        .apply_transfer_ownership(&setup.admin, &Symbol::new(&setup.env, "EmergencyAdmin"));

    assert!(normal_oracle
        .try_set_emergency_mode(&setup.emergency_admin, &false)
        .is_ok());
}

// TODO: add killswitches for freezing the oracle

// manage privileged addresses
#[test]
fn test_set_privileged_addresses() {
    let setup = Setup::default();
    let oracle = setup.normal_oracle;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user, false),
        (setup.admin.clone(), true),
        (setup.operations_admin.clone(), false),
        (setup.pause_admin.clone(), false),
        (setup.emergency_pause_admin.clone(), false),
    ] {
        assert_eq!(
            oracle
                .try_set_privileged_addrs(
                    &addr,
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
fn test_set_seconds_before_stale() {
    let setup = Setup::default();
    let normal_oracle = setup.normal_oracle;
    let user = Address::generate(&setup.env);

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        (setup.operations_admin.clone(), true),
        (setup.pause_admin.clone(), false),
        (setup.emergency_pause_admin.clone(), false),
    ] {
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

    for (addr, is_ok) in [
        (user.clone(), false),
        (setup.admin.clone(), true),
        (setup.operations_admin.clone(), true),
        (setup.pause_admin.clone(), false),
        (setup.emergency_pause_admin.clone(), false),
    ] {
        assert_eq!(
            normal_oracle
                .try_set_too_volatile_ratio(&addr, &5000)
                .is_ok(),
            is_ok
        );
    }
}
