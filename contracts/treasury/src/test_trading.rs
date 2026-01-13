#![cfg(test)]
extern crate std;

use crate::testutils::Setup;

use soroban_sdk::{testutils::Address as _, Address};
use types::pair::PairAmountsWithUSDC;
use utils::constant::PRICE_PRECISION;

// Assumes your default oracle prices are:
// long = 5_000_000, short = 5_000_000, usdc = PRICE_PRECISION (or equivalent).
// Your existing tests already rely on 5_000_000 as the long/short "price".
const DEFAULT_TOKEN_PRICE: u128 = 5_000_000;

// -------------------------------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------------------------------

fn bootstrap_with_liquidity(
    setup: &Setup,
    pair_tokens_to_mint: u128,
    pair_tokens_to_deposit: u128,
    init_admin_usdc: u128,
) {
    let admin = setup.admin.clone();

    // Mint admin USDC
    setup
        .token_usdc_admin_client
        .mint(&admin, &(init_admin_usdc as i128));
    assert_eq!(setup.token_usdc.balance(&admin), init_admin_usdc as i128);

    // Mint pair (admin receives long+short, and posts collateral in USDC)
    setup.pair.mint(&admin, &pair_tokens_to_mint);

    // Deposit liquidity into treasury (moves long/short + some usdc via collateral mechanics)
    setup
        .treasury
        .deposit(&admin, &setup.pair.address, &pair_tokens_to_deposit);

    // After deposit, admin should have deposited tokens; USDC should be 0 in your current behavior
    assert_eq!(setup.token_usdc.balance(&admin), 0);
    assert_eq!(setup.token_long.balance(&admin), 0);
    assert_eq!(setup.token_short.balance(&admin), 0);
}

fn mint_user_usdc(setup: &Setup, user: &Address, amount: u128) {
    setup.token_usdc_admin_client.mint(user, &(amount as i128));
    assert_eq!(setup.token_usdc.balance(user), amount as i128);
}

// fee is expressed in PRICE_PRECISION units (same as your taker_base_fee)
fn apply_fee_on_input(amount_in: u128, fee: u128) -> (u128, u128) {
    let less_fee = (amount_in * (PRICE_PRECISION - fee)) / PRICE_PRECISION;
    let fee_amt = amount_in - less_fee;
    (less_fee, fee_amt)
}

fn apply_fee_on_output(gross_out: u128, fee: u128) -> (u128, u128) {
    let less_fee = (gross_out * (PRICE_PRECISION - fee)) / PRICE_PRECISION;
    let fee_amt = gross_out - less_fee;
    (less_fee, fee_amt)
}

// -------------------------------------------------------------------------------------------------
// Buy Long
// -------------------------------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_buy_long_invalid_amount() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    setup.treasury.buy_long(&user1, &setup.pair.address, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn test_buy_long_trading_kills() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    setup.treasury.kill_trade(&admin);

    setup
        .treasury
        .buy_long(&user1, &setup.pair.address, &1_0000000_u128, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn test_buy_long_invalid_pair() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let pair = Address::generate(&setup.env);

    setup.treasury.buy_long(&user1, &pair, &1_0000000_u128, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #216)")]
fn test_buy_long_enforces_slippage() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    setup.treasury.buy_long(
        &user1,
        &setup.pair.address,
        &1_0000000_u128,
        &100_0000000_u128,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn test_buy_long_enforces_sufficient_inventory() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    // No liquidity deposited => inventory should be 0
    setup
        .treasury
        .buy_long(&user1, &setup.pair.address, &1_0000000_u128, &0);
}

#[test]
fn test_buy_long() {
    let setup = Setup::default();

    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    let init_admin_usdc = 2000_0000000_u128;
    let init_user_usdc = 1_0000000_u128;

    let pair_tokens_to_mint = 10_0000000_u128;
    let pair_tokens_to_deposit = 10_0000000_u128;
    let usdc_to_deposit = 1000_0000000_u128;
    let usdc_to_trade = 1_0000000_u128;

    // Setup
    mint_user_usdc(&setup, &admin, init_admin_usdc);
    mint_user_usdc(&setup, &user1, init_user_usdc);

    // Mint pair, compute collateral used (assert matches your existing expectations)
    setup.pair.mint(&admin, &pair_tokens_to_mint);
    let collateral_info = setup.pair.get_collateral_info();
    let collateral_used =
        (pair_tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;

    assert_eq!(
        setup.token_usdc.balance(&admin),
        (init_admin_usdc - collateral_used) as i128
    );
    assert_eq!(
        setup.token_long.balance(&admin),
        pair_tokens_to_mint as i128
    );
    assert_eq!(
        setup.token_short.balance(&admin),
        pair_tokens_to_mint as i128
    );

    // Deposit liquidity
    setup
        .treasury
        .deposit(&admin, &setup.pair.address, &pair_tokens_to_deposit);
    assert_eq!(setup.token_usdc.balance(&admin), 0);
    assert_eq!(setup.token_long.balance(&admin), 0);
    assert_eq!(setup.token_short.balance(&admin), 0);

    // Test
    let fee_config = setup.treasury.get_fee_config(&setup.pair.address);
    let (usdc_less_fee, usdc_fee) = apply_fee_on_input(usdc_to_trade, fee_config.taker_base_fee);

    let expected_out = (usdc_less_fee * PRICE_PRECISION) / DEFAULT_TOKEN_PRICE;
    let long_out = setup
        .treasury
        .buy_long(&user1, &setup.pair.address, &usdc_to_trade, &0);
    assert_eq!(long_out, expected_out);

    // Assertions
    assert_eq!(setup.token_usdc.balance(&user1), 0);
    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        (usdc_to_deposit + usdc_to_trade) as i128
    );

    assert_eq!(setup.token_long.balance(&user1), expected_out as i128);
    assert_eq!(
        setup.token_long.balance(&setup.treasury.address),
        (pair_tokens_to_deposit - expected_out) as i128
    );

    assert_eq!(
        setup.treasury.get_balances(&setup.pair.address),
        PairAmountsWithUSDC {
            usdc: usdc_to_deposit + usdc_to_trade - usdc_fee,
            long: pair_tokens_to_deposit - expected_out,
            short: pair_tokens_to_deposit,
        }
    );

    assert_eq!(
        setup.treasury.get_protocol_fees(&setup.pair.address),
        usdc_fee
    );
}

// -------------------------------------------------------------------------------------------------
// Buy Short
// -------------------------------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_buy_short_invalid_amount() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    setup
        .treasury
        .buy_short(&user1, &setup.pair.address, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn test_buy_short_trading_kills() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    setup.treasury.kill_trade(&admin);

    setup
        .treasury
        .buy_short(&user1, &setup.pair.address, &1_0000000_u128, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn test_buy_short_invalid_pair() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let pair = Address::generate(&setup.env);

    setup.treasury.buy_short(&user1, &pair, &1_0000000_u128, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #216)")]
fn test_buy_short_enforces_slippage() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    setup.treasury.buy_short(
        &user1,
        &setup.pair.address,
        &1_0000000_u128,
        &100_0000000_u128,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn test_buy_short_enforces_sufficient_inventory() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    setup
        .treasury
        .buy_short(&user1, &setup.pair.address, &1_0000000_u128, &0);
}

#[test]
fn test_buy_short() {
    let setup = Setup::default();

    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    let init_admin_usdc = 2000_0000000_u128;
    let init_user_usdc = 1_0000000_u128;

    let pair_tokens_to_mint = 10_0000000_u128;
    let pair_tokens_to_deposit = 10_0000000_u128;
    let usdc_to_deposit = 1000_0000000_u128;
    let usdc_to_trade = 1_0000000_u128;

    // Setup
    mint_user_usdc(&setup, &admin, init_admin_usdc);
    mint_user_usdc(&setup, &user1, init_user_usdc);

    setup.pair.mint(&admin, &pair_tokens_to_mint);
    let collateral_info = setup.pair.get_collateral_info();
    let collateral_used =
        (pair_tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;

    assert_eq!(
        setup.token_usdc.balance(&admin),
        (init_admin_usdc - collateral_used) as i128
    );
    assert_eq!(
        setup.token_long.balance(&admin),
        pair_tokens_to_mint as i128
    );
    assert_eq!(
        setup.token_short.balance(&admin),
        pair_tokens_to_mint as i128
    );

    setup
        .treasury
        .deposit(&admin, &setup.pair.address, &pair_tokens_to_deposit);
    assert_eq!(setup.token_usdc.balance(&admin), 0);
    assert_eq!(setup.token_long.balance(&admin), 0);
    assert_eq!(setup.token_short.balance(&admin), 0);

    // Test
    let fee_config = setup.treasury.get_fee_config(&setup.pair.address);
    let (usdc_less_fee, usdc_fee) = apply_fee_on_input(usdc_to_trade, fee_config.taker_base_fee);

    let expected_out = (usdc_less_fee * PRICE_PRECISION) / DEFAULT_TOKEN_PRICE;
    let short_out = setup
        .treasury
        .buy_short(&user1, &setup.pair.address, &usdc_to_trade, &0);
    assert_eq!(short_out, expected_out);

    // Assertions
    assert_eq!(setup.token_usdc.balance(&user1), 0);
    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        (usdc_to_deposit + usdc_to_trade) as i128
    );

    assert_eq!(setup.token_short.balance(&user1), expected_out as i128);
    assert_eq!(
        setup.token_short.balance(&setup.treasury.address),
        (pair_tokens_to_deposit - expected_out) as i128
    );

    assert_eq!(
        setup.treasury.get_balances(&setup.pair.address),
        PairAmountsWithUSDC {
            usdc: usdc_to_deposit + usdc_to_trade - usdc_fee,
            long: pair_tokens_to_deposit,
            short: pair_tokens_to_deposit - expected_out,
        }
    );

    assert_eq!(
        setup.treasury.get_protocol_fees(&setup.pair.address),
        usdc_fee
    );
}

// -------------------------------------------------------------------------------------------------
// Sell Long
// -------------------------------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn test_sell_long_invalid_pair() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let pair = Address::generate(&setup.env);

    setup.treasury.sell_long(&user1, &pair, &1_0000000_u128, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_sell_long_invalid_amount() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    setup
        .treasury
        .sell_long(&user1, &setup.pair.address, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn test_sell_long_trading_kills() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    setup.treasury.kill_trade(&admin);

    setup
        .treasury
        .sell_long(&user1, &setup.pair.address, &1_0000000_u128, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #216)")]
fn test_sell_long_enforces_slippage() {
    let setup = Setup::default();

    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    // Bootstrap: mint USDC, mint pair, deposit
    setup
        .token_usdc_admin_client
        .mint(&admin, &(2_000_0000000_i128));
    setup.pair.mint(&admin, &10_0000000_u128);
    setup
        .treasury
        .deposit(&admin, &setup.pair.address, &10_0000000_u128);

    // User buys long to get inventory to sell
    setup
        .token_usdc_admin_client
        .mint(&user1, &(1_0000000_i128));
    let long_out = setup
        .treasury
        .buy_long(&user1, &setup.pair.address, &1_0000000_u128, &0);
    assert!(long_out > 0);

    // Now sell but demand too much USDC out => Slippage (216)
    setup.treasury.sell_long(
        &user1,
        &setup.pair.address,
        &long_out,
        &10_0000000_u128, // too high min
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn test_sell_long_enforces_sufficient_usdc_inventory() {
    let setup = Setup::default();

    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    // Bootstrap with SMALL liquidity so treasury USDC inventory is tight
    setup
        .token_usdc_admin_client
        .mint(&admin, &(100_0000000_i128));
    setup.pair.mint(&admin, &1_0000000_u128);
    setup
        .treasury
        .deposit(&admin, &setup.pair.address, &1_0000000_u128);

    // Give user USDC, buy long, then sell it back.
    setup
        .token_usdc_admin_client
        .mint(&user1, &(1_0000000_i128));
    let long_out = setup
        .treasury
        .buy_long(&user1, &setup.pair.address, &1_0000000_u128, &0);

    // Depending on your deposit mechanics, this often trips insufficient treasury USDC.
    setup
        .treasury
        .sell_long(&user1, &setup.pair.address, &long_out, &0);
}

#[test]
fn test_sell_long_happy_path_updates_balances_and_fees() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    let init_admin_usdc = 2000_0000000_u128;
    let init_user_usdc = 2_0000000_u128;

    let pair_tokens_to_mint = 10_0000000_u128;
    let pair_tokens_to_deposit = 10_0000000_u128;
    let usdc_to_trade_buy = 1_0000000_u128;

    // Bootstrap inventory
    bootstrap_with_liquidity(
        &setup,
        pair_tokens_to_mint,
        pair_tokens_to_deposit,
        init_admin_usdc,
    );

    // Give user USDC and buy long
    mint_user_usdc(&setup, &user1, init_user_usdc);
    let fee_config = setup.treasury.get_fee_config(&setup.pair.address);

    let (usdc_less_fee_buy, usdc_fee_buy) =
        apply_fee_on_input(usdc_to_trade_buy, fee_config.taker_base_fee);
    let expected_long_out = (usdc_less_fee_buy * PRICE_PRECISION) / DEFAULT_TOKEN_PRICE;

    let long_out = setup
        .treasury
        .buy_long(&user1, &setup.pair.address, &usdc_to_trade_buy, &0);
    assert_eq!(long_out, expected_long_out);

    // Sell half of the long back
    let long_in = long_out / 2;

    // expected sell math (gross then fee on output)
    let gross_usdc_out = (long_in * DEFAULT_TOKEN_PRICE) / PRICE_PRECISION;
    let (expected_usdc_out, usdc_fee_sell) =
        apply_fee_on_output(gross_usdc_out, fee_config.taker_base_fee);

    let usdc_out = setup
        .treasury
        .sell_long(&user1, &setup.pair.address, &long_in, &0);
    assert_eq!(usdc_out, expected_usdc_out);

    // User balances: spent 1 USDC, got some back from sell, and retains remaining long
    assert_eq!(
        setup.token_usdc.balance(&user1),
        (init_user_usdc - usdc_to_trade_buy + usdc_out) as i128
    );
    assert_eq!(
        setup.token_long.balance(&user1),
        (long_out - long_in) as i128
    );

    // Treasury balances: long increased by long_in, usdc decreased by usdc_out
    let bal = setup.treasury.get_balances(&setup.pair.address);
    assert_eq!(bal.long, pair_tokens_to_deposit - long_out + long_in);

    // Fees accumulate from both trades
    assert_eq!(
        setup.treasury.get_protocol_fees(&setup.pair.address),
        usdc_fee_buy + usdc_fee_sell
    );
}

// -------------------------------------------------------------------------------------------------
// Sell Short
// -------------------------------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn test_sell_short_invalid_pair() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let pair = Address::generate(&setup.env);

    setup
        .treasury
        .sell_short(&user1, &pair, &1_0000000_u128, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_sell_short_invalid_amount() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    setup
        .treasury
        .sell_short(&user1, &setup.pair.address, &0, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn test_sell_short_trading_kills() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    setup.treasury.kill_trade(&admin);

    setup
        .treasury
        .sell_short(&user1, &setup.pair.address, &1_0000000_u128, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #216)")]
fn test_sell_short_enforces_slippage() {
    let setup = Setup::default();

    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    setup
        .token_usdc_admin_client
        .mint(&admin, &(2_000_0000000_i128));
    setup.pair.mint(&admin, &10_0000000_u128);
    setup
        .treasury
        .deposit(&admin, &setup.pair.address, &10_0000000_u128);

    setup
        .token_usdc_admin_client
        .mint(&user1, &(1_0000000_i128));
    let short_out = setup
        .treasury
        .buy_short(&user1, &setup.pair.address, &1_0000000_u128, &0);
    assert!(short_out > 0);

    setup
        .treasury
        .sell_short(&user1, &setup.pair.address, &short_out, &10_0000000_u128);
}

#[test]
#[should_panic(expected = "Error(Contract, #215)")]
fn test_sell_short_enforces_sufficient_usdc_inventory() {
    let setup = Setup::default();

    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    setup
        .token_usdc_admin_client
        .mint(&admin, &(100_0000000_i128));
    setup.pair.mint(&admin, &1_0000000_u128);
    setup
        .treasury
        .deposit(&admin, &setup.pair.address, &1_0000000_u128);

    setup
        .token_usdc_admin_client
        .mint(&user1, &(1_0000000_i128));
    let short_out = setup
        .treasury
        .buy_short(&user1, &setup.pair.address, &1_0000000_u128, &0);

    setup
        .treasury
        .sell_short(&user1, &setup.pair.address, &short_out, &0);
}

#[test]
fn test_sell_short_happy_path_updates_balances_and_fees() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();

    let init_admin_usdc = 2000_0000000_u128;
    let init_user_usdc = 2_0000000_u128;

    let pair_tokens_to_mint = 10_0000000_u128;
    let pair_tokens_to_deposit = 10_0000000_u128;
    let usdc_to_trade_buy = 1_0000000_u128;

    bootstrap_with_liquidity(
        &setup,
        pair_tokens_to_mint,
        pair_tokens_to_deposit,
        init_admin_usdc,
    );

    mint_user_usdc(&setup, &user1, init_user_usdc);
    let fee_config = setup.treasury.get_fee_config(&setup.pair.address);

    let (usdc_less_fee_buy, usdc_fee_buy) =
        apply_fee_on_input(usdc_to_trade_buy, fee_config.taker_base_fee);
    let expected_short_out = (usdc_less_fee_buy * PRICE_PRECISION) / DEFAULT_TOKEN_PRICE;

    let short_out = setup
        .treasury
        .buy_short(&user1, &setup.pair.address, &usdc_to_trade_buy, &0);
    assert_eq!(short_out, expected_short_out);

    // Sell half back
    let short_in = short_out / 2;
    let gross_usdc_out = (short_in * DEFAULT_TOKEN_PRICE) / PRICE_PRECISION;
    let (expected_usdc_out, usdc_fee_sell) =
        apply_fee_on_output(gross_usdc_out, fee_config.taker_base_fee);

    let usdc_out = setup
        .treasury
        .sell_short(&user1, &setup.pair.address, &short_in, &0);
    assert_eq!(usdc_out, expected_usdc_out);

    assert_eq!(
        setup.token_usdc.balance(&user1),
        (init_user_usdc - usdc_to_trade_buy + usdc_out) as i128
    );
    assert_eq!(
        setup.token_short.balance(&user1),
        (short_out - short_in) as i128
    );

    let bal = setup.treasury.get_balances(&setup.pair.address);
    assert_eq!(bal.short, pair_tokens_to_deposit - short_out + short_in);

    assert_eq!(
        setup.treasury.get_protocol_fees(&setup.pair.address),
        usdc_fee_buy + usdc_fee_sell
    );
}
