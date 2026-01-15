#![cfg(test)]
extern crate std;

use crate::testutils::Setup;

use soroban_sdk::{testutils::Address as _, Address, Env, Vec};
use types::pair::PairAmountsWithUSDC;
use utils::{
    constant::{ONE_HOUR, PRICE_PRECISION},
    test_utils::jump,
};

// -------------------------------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------------------------------

const TOXIC_THRESHOLD: u128 = 9_000_000; // 90%
const LONG_TOXIC_PRICE: u128 = 1_000_000; // 10%

fn mint_user_usdc(setup: &Setup, user: &Address, amount: u128) {
    setup.token_usdc_admin_client.mint(user, &(amount as i128));
    assert_eq!(setup.token_usdc.balance(user), amount as i128);
}

fn bootstrap_with_liquidity(
    setup: &Setup,
    pair_tokens_to_mint: u128,
    pair_tokens_to_deposit: u128,
    init_admin_usdc: u128,
) {
    let admin = setup.admin.clone();

    // Mint admin USDC
    mint_user_usdc(setup, &admin, init_admin_usdc);

    // Mint pair
    setup.pair.mint(&admin, &pair_tokens_to_mint);

    // Deposit
    setup
        .treasury
        .deposit(&admin, &setup.pair.address, &pair_tokens_to_deposit);

    // In your current behavior, admin ends with 0 balances after deposit
    assert_eq!(setup.token_usdc.balance(&admin), 0);
    assert_eq!(setup.token_long.balance(&admin), 0);
    assert_eq!(setup.token_short.balance(&admin), 0);
}

/// Fee delta in USDC: protocol_fees_after - protocol_fees_before
fn protocol_fee_delta(setup: &Setup, pair: &Address, before: u128) -> u128 {
    let after = setup.treasury.get_protocol_fees(pair);
    after - before
}

/// Expected buy out based on realized fee delta
// fn expected_buy_out_from_realized_fee(usdc_in: u128, fee_charged: u128) -> u128 {
//     let net_in = usdc_in - fee_charged;
//     (net_in * PRICE_PRECISION) / DEFAULT_TOKEN_PRICE
// }

/// Expected sell out based on realized fee delta (fee taken from output)
fn expected_sell_out_from_realized_fee(gross_out: u128, fee_charged: u128) -> u128 {
    gross_out - fee_charged
}

/// (A) Set USDC floor via contract admin method.
/// Your contract has `set_usdc_floor(admin, floor_fraction)` (not "...fraction").
fn set_usdc_floor_fraction(setup: &Setup, floor_fraction: u128) {
    let admin = setup.admin.clone();
    setup.treasury.set_usdc_floor(&admin, &floor_fraction);
}

/// (B) Set risk parameters via storage (since contract doesn’t expose admin setter).
/// IMPORTANT: must be inside env.as_contract() to access contract storage.
fn set_risk_params_for_pair(
    setup: &Setup,
    toxic_threshold: u128,
    max_net_imbalance_tokens: u128,
    imbalance_fee_max: u128,
) {
    let env: &Env = &setup.env;
    let pair = setup.pair.address.clone();
    let treasury = setup.treasury.address.clone();

    env.as_contract(&treasury, || {
        crate::storage::set_risk_parameters(
            env,
            &pair,
            &(crate::storage::TreasuryRiskParameters {
                toxic_threshold,
                max_net_imbalance_tokens,
                imbalance_fee_max,
            }),
        );
    });
}

/// (C) Set oracle prices for the pair.
/// 🚨 EDIT THIS FUNCTION to match your mock oracle client API.
/// Goal: make treasury read long/short normalized prices you choose.
fn set_pair_price(setup: &Setup, new_price: i128, timestamp: u64) {
    // oracle price
    let new_prices: Vec<i128> = Vec::from_array(&setup.env, [new_price, 1_00000000000000]);
    setup.reflector_client.set_price(&new_prices, &timestamp);

    setup.pair.sync_collateral();
}

// --------------------------------------
// BUY LONG
// --------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn buy_long_invalid_amount() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup.treasury.buy_long(&user, &setup.pair.address, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn buy_long_invalid_pair() {
    let setup = Setup::default();
    let user = setup.users[1].clone();
    let bad_pair = Address::generate(&setup.env);

    setup.treasury.buy_long(&user, &bad_pair, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn buy_long_trading_paused() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    setup.treasury.kill_trade(&admin);

    setup
        .treasury
        .buy_long(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn buy_long_min_long_out_insufficient_inventory() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup
        .treasury
        .buy_long(&user, &setup.pair.address, &1_0000000, &1_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn buy_long_insufficient_inventory() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup
        .treasury
        .buy_long(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #216)")]
fn buy_long_slippage() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    mint_user_usdc(&setup, &user, 1_0000000);

    setup
        .treasury
        .buy_long(&user, &setup.pair.address, &1_0000000, &10_0000000);
}

#[test]
#[should_panic]
fn buy_long_hard_cap() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);
    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 1, 0);

    mint_user_usdc(&setup, &user, 5_0000000);
    setup
        .treasury
        .buy_long(&user, &setup.pair.address, &5_0000000, &0);
}

#[test]
fn buy_long_happy_path() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    mint_user_usdc(&setup, &user, 1_0000000);

    let (out, fee) = setup
        .treasury
        .buy_long(&user, &setup.pair.address, &1_0000000, &0);
    assert!(out > 0);

    assert_eq!(setup.token_usdc.balance(&user), 0);
    assert_eq!(setup.token_long.balance(&user), out as i128);
    assert_eq!(setup.token_short.balance(&user), 0);

    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        1001_0000000
    );
    assert_eq!(
        setup.token_long.balance(&setup.treasury.address),
        (10_0000000 - out) as i128
    );
    assert_eq!(
        setup.token_short.balance(&setup.treasury.address),
        10_0000000
    );

    assert_eq!(
        setup.treasury.get_balances(&setup.pair.address),
        PairAmountsWithUSDC {
            long: 10_0000000 - out,
            short: 10_0000000,
            usdc: 1001_0000000 - fee,
        }
    );

    assert_eq!(setup.treasury.get_protocol_fees(&setup.pair.address), fee);
}

// --------------------------------------
// BUY SHORT
// --------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn buy_short_invalid_amount() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup.treasury.buy_short(&user, &setup.pair.address, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn buy_short_invalid_pair() {
    let setup = Setup::default();
    let user = setup.users[1].clone();
    let bad_pair = Address::generate(&setup.env);

    setup.treasury.buy_short(&user, &bad_pair, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn buy_short_trading_paused() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    setup.treasury.kill_trade(&admin);

    setup
        .treasury
        .buy_short(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn buy_short_min_long_out_insufficient_inventory() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup
        .treasury
        .buy_short(&user, &setup.pair.address, &1_0000000, &1_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn buy_short_insufficient_inventory() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup
        .treasury
        .buy_short(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #216)")]
fn buy_short_slippage() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    mint_user_usdc(&setup, &user, 1_0000000);

    setup
        .treasury
        .buy_short(&user, &setup.pair.address, &1_0000000, &10_0000000);
}

#[test]
#[should_panic]
fn buy_short_hard_cap() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);
    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 1, 0);

    mint_user_usdc(&setup, &user, 5_0000000);
    setup
        .treasury
        .buy_short(&user, &setup.pair.address, &5_0000000, &0);
}

#[test]
fn buy_short_happy_path() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    mint_user_usdc(&setup, &user, 1_0000000);

    let (out, fee) = setup
        .treasury
        .buy_short(&user, &setup.pair.address, &1_0000000, &0);
    assert!(out > 0);

    assert_eq!(setup.token_usdc.balance(&user), 0);
    assert_eq!(setup.token_long.balance(&user), 0);
    assert_eq!(setup.token_short.balance(&user), out as i128);

    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        1001_0000000
    );
    assert_eq!(
        setup.token_long.balance(&setup.treasury.address),
        10_0000000
    );
    assert_eq!(
        setup.token_short.balance(&setup.treasury.address),
        (10_0000000 - out) as i128
    );

    assert_eq!(
        setup.treasury.get_balances(&setup.pair.address),
        PairAmountsWithUSDC {
            long: 10_0000000,
            short: 10_0000000 - out,
            usdc: 1001_0000000 - fee,
        }
    );

    assert_eq!(setup.treasury.get_protocol_fees(&setup.pair.address), fee);
}

// --------------------------------------
// SELL LONG
// --------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn sell_long_invalid_amount() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup.treasury.sell_long(&user, &setup.pair.address, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn sell_long_invalid_pair() {
    let setup = Setup::default();
    let user = setup.users[1].clone();
    let bad_pair = Address::generate(&setup.env);

    setup.treasury.sell_long(&user, &bad_pair, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn sell_long_trading_paused() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    setup.treasury.kill_trade(&admin);

    setup
        .treasury
        .sell_long(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn sell_long_min_usdc_out_insufficient_inventory() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup
        .treasury
        .sell_long(&user, &setup.pair.address, &1_0000000, &1_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn sell_long_insufficient_inventory() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    setup
        .treasury
        .sell_long(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #216)")]
fn sell_long_slippage() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    mint_user_usdc(&setup, &user, 100_0000000);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    // Mint the user some pair tokens
    setup.pair.mint(&user, &1_0000000);

    setup
        .treasury
        .sell_long(&user, &setup.pair.address, &1_0000000, &50_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #222)")]
fn sell_long_toxic() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    // Mint the user some pair tokens
    mint_user_usdc(&setup, &user, 100_0000000);
    setup.pair.mint(&user, &1_0000000);

    jump(&setup.env, ONE_HOUR);
    set_pair_price(&setup, 9_00000000000000, setup.env.ledger().timestamp());

    setup
        .treasury
        .sell_long(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #221)")]
fn sell_long_usdc_floor() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    set_usdc_floor_fraction(&setup, PRICE_PRECISION / 10);

    // Mint the user some pair tokens
    mint_user_usdc(&setup, &user, 2000_0000000);
    setup.pair.mint(&user, &20_0000000);

    setup
        .treasury
        .sell_long(&user, &setup.pair.address, &20_0000000, &0);
}

#[test]
fn sell_long_usdc_floor_zero_works() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    set_usdc_floor_fraction(&setup, 0);

    // Mint the user some pair tokens
    mint_user_usdc(&setup, &user, 2000_0000000);
    setup.pair.mint(&user, &20_0000000);

    setup
        .treasury
        .sell_long(&user, &setup.pair.address, &20_0000000, &0);
}

// #[test]
// #[should_panic]
// fn sell_long_hard_cap() {
//     let setup = Setup::default();
//     let user = setup.users[1].clone();

//     bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);
//     set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 1, 0);

//     mint_user_usdc(&setup, &user, 5_0000000);
//     setup.treasury.sell_long(&user, &setup.pair.address, &5_0000000, &0);
// }

#[test]
fn sell_long_happy_path() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    // Mint the user some pair tokens
    mint_user_usdc(&setup, &user, 100_0000000);
    setup.pair.mint(&user, &1_0000000);

    let (out, fee) = setup
        .treasury
        .sell_long(&user, &setup.pair.address, &1_0000000, &0);
    assert!(out > 0);

    assert_eq!(setup.token_usdc.balance(&user), out as i128);
    assert_eq!(setup.token_long.balance(&user), 0);
    assert_eq!(setup.token_short.balance(&user), 1_0000000);

    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        (1000_0000000 - out) as i128
    );
    assert_eq!(
        setup.token_long.balance(&setup.treasury.address),
        11_0000000
    );
    assert_eq!(
        setup.token_short.balance(&setup.treasury.address),
        10_0000000
    );

    assert_eq!(
        setup.treasury.get_balances(&setup.pair.address),
        PairAmountsWithUSDC {
            long: 11_0000000,
            short: 10_0000000,
            usdc: 1000_0000000 - out - fee,
        }
    );

    assert_eq!(setup.treasury.get_protocol_fees(&setup.pair.address), fee);
}

// --------------------------------------
// SELL SHORT
// --------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn sell_short_invalid_amount() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup
        .treasury
        .sell_short(&user, &setup.pair.address, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn sell_short_invalid_pair() {
    let setup = Setup::default();
    let user = setup.users[1].clone();
    let bad_pair = Address::generate(&setup.env);

    setup.treasury.sell_short(&user, &bad_pair, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn sell_short_trading_paused() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    setup.treasury.kill_trade(&admin);

    setup
        .treasury
        .sell_short(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn sell_short_min_usdc_out_insufficient_inventory() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    setup
        .treasury
        .sell_short(&user, &setup.pair.address, &1_0000000, &1_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn sell_short_insufficient_inventory() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    setup
        .treasury
        .sell_short(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #216)")]
fn sell_short_slippage() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    mint_user_usdc(&setup, &user, 100_0000000);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    // Mint the user some pair tokens
    setup.pair.mint(&user, &1_0000000);

    setup
        .treasury
        .sell_short(&user, &setup.pair.address, &1_0000000, &50_0000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #222)")]
fn sell_short_toxic() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    // Mint the user some pair tokens
    mint_user_usdc(&setup, &user, 100_0000000);
    setup.pair.mint(&user, &1_0000000);

    jump(&setup.env, ONE_HOUR);
    set_pair_price(&setup, 191_00000000000000, setup.env.ledger().timestamp());

    mint_user_usdc(&setup, &user, 1_0000000);
    setup
        .treasury
        .sell_short(&user, &setup.pair.address, &1_0000000, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #221)")]
fn sell_short_usdc_floor() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    set_usdc_floor_fraction(&setup, PRICE_PRECISION / 10);

    // Mint the user some pair tokens
    mint_user_usdc(&setup, &user, 2000_0000000);
    setup.pair.mint(&user, &20_0000000);

    setup
        .treasury
        .sell_short(&user, &setup.pair.address, &20_0000000, &0);
}

#[test]
fn sell_short_usdc_floor_zero_works() {
    let setup = Setup::default();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    set_usdc_floor_fraction(&setup, 0);

    // Mint the user some pair tokens
    mint_user_usdc(&setup, &user, 2000_0000000);
    setup.pair.mint(&user, &20_0000000);

    setup
        .treasury
        .sell_short(&user, &setup.pair.address, &20_0000000, &0);
}

// #[test]
// #[should_panic]
// fn sell_short_hard_cap() {
//     let setup = Setup::default();
//     let user = setup.users[1].clone();

//     bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);
//     set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 1, 0);

//     mint_user_usdc(&setup, &user, 5_0000000);
//     setup.treasury.sell_short(&user, &setup.pair.address, &5_0000000, &0);
// }

#[test]
fn sell_short_happy_path() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    set_risk_params_for_pair(&setup, TOXIC_THRESHOLD, 0, 0);

    // Mint the user some pair tokens
    mint_user_usdc(&setup, &user, 100_0000000);
    setup.pair.mint(&user, &1_0000000);

    let (out, fee) = setup
        .treasury
        .sell_short(&user, &setup.pair.address, &1_0000000, &0);
    assert!(out > 0);

    assert_eq!(setup.token_usdc.balance(&user), out as i128);
    assert_eq!(setup.token_long.balance(&user), 1_0000000);
    assert_eq!(setup.token_short.balance(&user), 0);

    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        (1000_0000000 - out) as i128
    );
    assert_eq!(
        setup.token_long.balance(&setup.treasury.address),
        10_0000000
    );
    assert_eq!(
        setup.token_short.balance(&setup.treasury.address),
        11_0000000
    );

    assert_eq!(
        setup.treasury.get_balances(&setup.pair.address),
        PairAmountsWithUSDC {
            long: 10_0000000,
            short: 11_0000000,
            usdc: 1000_0000000 - out - fee,
        }
    );

    assert_eq!(setup.treasury.get_protocol_fees(&setup.pair.address), fee);
}

// --------------------------------------
// CLAIM FEES
// --------------------------------------

#[test]
fn claim_protocol_fees_on_zero_returns_zero() {
    let setup = Setup::default();
    let admin = setup.admin.clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    assert_eq!(setup.treasury.get_protocol_fees(&setup.pair.address), 0);

    let fee_collected = setup
        .treasury
        .claim_protocol_fees(&admin, &setup.pair.address, &admin);
    assert_eq!(fee_collected, 0);

    assert_eq!(setup.treasury.get_protocol_fees(&setup.pair.address), 0);

    assert_eq!(setup.token_usdc.balance(&admin), 0);
    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        1000_0000000
    );
}

#[test]
fn claim_protocol_fees() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user = setup.users[1].clone();

    bootstrap_with_liquidity(&setup, 10_0000000, 10_0000000, setup.collateral_per_pair * 20);

    mint_user_usdc(&setup, &user, 1_0000000);

    let (out, fee) = setup
        .treasury
        .buy_long(&user, &setup.pair.address, &1_0000000, &0);

    assert_eq!(
        setup.treasury.get_balances(&setup.pair.address),
        PairAmountsWithUSDC {
            long: 10_0000000 - out,
            short: 10_0000000,
            usdc: 1001_0000000 - fee,
        }
    );

    assert_eq!(setup.treasury.get_protocol_fees(&setup.pair.address), fee);

    let fee_collected = setup
        .treasury
        .claim_protocol_fees(&admin, &setup.pair.address, &admin);

    assert_eq!(setup.treasury.get_protocol_fees(&setup.pair.address), 0);

    assert_eq!(setup.token_usdc.balance(&admin), fee_collected as i128);
    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        (1001_0000000 - fee_collected) as i128
    );
}
