#![cfg(test)]
extern crate std;

use crate::storage::{get_collateral_percent_long, get_status, set_status};
use crate::testutils::Setup;

use soroban_sdk::{testutils::Address as _, Address, Vec};
use token_pair::{get_total_long_shares, get_total_short_shares};
use types::pair::{CollateralConfig, PairStatus, Side};
use utils::constant::{ONE_HOUR, PRICE_PRECISION};
use utils::test_utils::jump;

/* Mint */

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_mint_invalid_amount() {
    let setup = Setup::default();

    setup
        .pair
        .mint(&setup.users[1], &setup.token_usdc.address, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn test_mint_invalid_status() {
    let setup = Setup::default();
    let tokens_to_mint = 1_0000000;

    // Set additional test configuration
    setup.env.as_contract(&setup.pair.address, || {
        set_status(&setup.env, &PairStatus::Expired);
    });

    setup
        .pair
        .mint(&setup.users[1], &setup.token_usdc.address, &tokens_to_mint);
}

#[test]
#[should_panic(expected = "Error(Contract, #218)")]
fn test_mint_disabled_collateral_type() {
    let setup = Setup::default();
    let token = Address::generate(&setup.env);
    let tokens_to_mint = 1_0000000;

    setup.pair.set_collateral_config(
        &setup.admin,
        &(CollateralConfig {
            token: token.clone(),
            oracle: Address::generate(&setup.env),
            mint_enabled: false,
            redeem_enabled: true,
        }),
    );

    setup.pair.mint(&setup.users[1], &token, &tokens_to_mint);
}

#[test]
fn test_mint() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 1_0000000;
    let collateral_info = setup.pair.get_collateral_info();
    let collateral_required =
        (tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;

    // Setup
    // ================================================================================

    // Mint USDC to user
    setup
        .token_usdc_admin_client
        .mint(&user1, &(collateral_required as i128));
    assert_eq!(
        setup.token_usdc.balance(&user1) as u128,
        collateral_required
    );

    // Test
    // ================================================================================

    let collateral_used = setup
        .pair
        .mint(&user1, &setup.token_usdc.address, &tokens_to_mint);
    assert_eq!(collateral_used, collateral_required);

    // Assertions
    // ================================================================================

    // [x] USDC moved from user to pair
    assert_eq!(setup.token_usdc.balance(&user1), 0);
    assert_eq!(
        setup.token_usdc.balance(&setup.pair.address),
        collateral_required as i128
    );

    // [x] Long and Short token(s) minted to user
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_mint);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_mint);

    // [x] Long and Short total shares incremented
    setup.env.as_contract(&setup.pair.address, || {
        assert_eq!(get_total_long_shares(&setup.env), tokens_to_mint);
        assert_eq!(get_total_short_shares(&setup.env), tokens_to_mint);
    });
}

/* Redeem */

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_redeem_invalid_amount() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    setup.pair.redeem(&user1, &setup.token_usdc.address, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #218)")]
fn test_redeem_disabled_collateral_type() {
    let setup = Setup::default();
    let token = Address::generate(&setup.env);
    let tokens_to_mint = 1_0000000;

    setup.pair.set_collateral_config(
        &setup.admin,
        &(CollateralConfig {
            token: token.clone(),
            oracle: Address::generate(&setup.env),
            mint_enabled: true,
            redeem_enabled: false,
        }),
    );

    setup.pair.mint(&setup.users[1], &token, &tokens_to_mint);
}

#[test]
fn test_redeem() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 1_0000000;
    let tokens_to_redeem = 1_0000000;
    let collateral_info = setup.pair.get_collateral_info();
    let collateral_required =
        (tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;

    // Setup
    // ================================================================================

    // Mint USDC to user
    setup
        .token_usdc_admin_client
        .mint(&user1, &(collateral_required as i128));
    assert_eq!(
        setup.token_usdc.balance(&user1) as u128,
        collateral_required
    );

    // Mint pair tokens
    setup
        .pair
        .mint(&user1, &setup.token_usdc.address, &tokens_to_mint);
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_mint);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_mint);

    // Test
    // ================================================================================

    let collateral_used = setup
        .pair
        .redeem(&user1, &setup.token_usdc.address, &tokens_to_redeem);
    assert_eq!(collateral_used, collateral_required);

    // Assertions
    // ================================================================================

    // [x] USDC moved from pair to user
    assert_eq!(setup.token_usdc.balance(&user1), collateral_used as i128);
    assert_eq!(setup.token_usdc.balance(&setup.pair.address), 0);

    // [x] Long and Short token(s) burned from user
    assert_eq!(setup.token_long.balance(&user1) as u128, 0);
    assert_eq!(setup.token_short.balance(&user1) as u128, 0);

    // [x] Long and Short total shares decremented
    setup.env.as_contract(&setup.pair.address, || {
        assert_eq!(get_total_long_shares(&setup.env), 0);
        assert_eq!(get_total_short_shares(&setup.env), 0);
    });
}

#[test]
fn test_redeem_after_price_change() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 1_0000000;
    let tokens_to_redeem = 1_0000000;
    let collateral_info = setup.pair.get_collateral_info();
    let collateral_required =
        (tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;

    // Setup
    // ================================================================================

    // Mint USDC to user
    setup
        .token_usdc_admin_client
        .mint(&user1, &(collateral_required as i128));
    assert_eq!(
        setup.token_usdc.balance(&user1) as u128,
        collateral_required
    );

    // Mint pair tokens
    setup
        .pair
        .mint(&user1, &setup.token_usdc.address, &tokens_to_mint);
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_mint);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_mint);

    // Test
    // ================================================================================

    // Update oracle price
    jump(&setup.env, ONE_HOUR);
    let new_prices: Vec<i128> = Vec::from_array(&setup.env, [110_00000000000000, 1_00000000000000]);
    setup
        .reflector_client
        .set_price(&new_prices, &(setup.start_time + ONE_HOUR));

    let collateral_used = setup
        .pair
        .redeem(&user1, &setup.token_usdc.address, &tokens_to_redeem);
    assert_eq!(collateral_used, collateral_required);

    // Assertions
    // ================================================================================

    // [x] Ensure price is updated
    setup.env.as_contract(&setup.pair.address, || {
        assert_eq!(get_collateral_percent_long(&setup.env), 5_500_000); // 55%
    });

    // [x] USDC moved from pair to user
    assert_eq!(setup.token_usdc.balance(&user1), collateral_used as i128);
    assert_eq!(setup.token_usdc.balance(&setup.pair.address), 0);

    // [x] Long and Short token(s) burned from user
    assert_eq!(setup.token_long.balance(&user1) as u128, 0);
    assert_eq!(setup.token_short.balance(&user1) as u128, 0);

    // [x] Long and Short total shares decremented
    setup.env.as_contract(&setup.pair.address, || {
        assert_eq!(get_total_long_shares(&setup.env), 0);
        assert_eq!(get_total_short_shares(&setup.env), 0);
    });
}

/* Redeem One */

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_redeem_one_invalid_amount() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    setup
        .pair
        .redeem_one(&user1, &setup.token_usdc.address, &Side::Long, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #214)")]
fn test_redeem_one_invalid_status() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    setup
        .pair
        .redeem_one(&user1, &setup.token_usdc.address, &Side::Long, &1_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #218)")]
fn test_redeem_one_disabled_collateral_type() {
    let setup = Setup::default();
    let token = Address::generate(&setup.env);
    let tokens_to_mint = 1_0000000;

    setup.pair.set_collateral_config(
        &setup.admin,
        &(CollateralConfig {
            token: token.clone(),
            oracle: Address::generate(&setup.env),
            mint_enabled: true,
            redeem_enabled: false,
        }),
    );

    setup.pair.mint(&setup.users[1], &token, &tokens_to_mint);
}

#[test]
fn test_redeem_one_long_when_price_at_upper_bound() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let tokens_to_mint = 1_0000000;
    let tokens_to_redeem = 1_0000000;
    let collateral_info = setup.pair.get_collateral_info();
    let collateral_required =
        (tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;

    // Setup
    // ================================================================================

    // Mint USDC to user
    setup
        .token_usdc_admin_client
        .mint(&user1, &(collateral_required as i128));
    assert_eq!(
        setup.token_usdc.balance(&user1) as u128,
        collateral_required
    );

    // Mint pair tokens
    setup
        .pair
        .mint(&user1, &setup.token_usdc.address, &tokens_to_mint);
    assert_eq!(setup.token_long.balance(&user1) as u128, tokens_to_mint);
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_mint);

    // Update oracle price
    jump(&setup.env, ONE_HOUR);
    let new_prices: Vec<i128> = Vec::from_array(&setup.env, [201_00000000000000, 1_00000000000000]);
    setup
        .reflector_client
        .set_price(&new_prices, &(setup.start_time + ONE_HOUR));

    // Test
    // ================================================================================

    let collateral_returned = setup.pair.redeem_one(
        &user1,
        &setup.token_usdc.address,
        &Side::Long,
        &tokens_to_redeem,
    );

    // Assertions
    // ================================================================================

    // [x] Collateral returned should equal 100% of the collateral per pair
    assert_eq!(collateral_returned, collateral_info.collateral_per_pair);

    // [x] USDC moved from pair to user
    assert_eq!(
        setup.token_usdc.balance(&user1),
        collateral_returned as i128
    );
    assert_eq!(setup.token_usdc.balance(&setup.pair.address), 0);

    // [x] Long token(s) burned from user
    assert_eq!(setup.token_long.balance(&user1) as u128, 0);

    // [x] Short token(s) NOT transferred or burned (they're worthless)
    assert_eq!(setup.token_short.balance(&user1) as u128, tokens_to_mint);

    setup.env.as_contract(&setup.pair.address, || {
        // [x] Status is automatically updated to Settlement
        assert_eq!(get_status(&setup.env), PairStatus::Expired);

        // [x] Long and Short total shares updated
        assert_eq!(get_total_long_shares(&setup.env), 0);
        assert_eq!(get_total_short_shares(&setup.env), tokens_to_mint);
    });
}

// TODO: test other price boundaries
