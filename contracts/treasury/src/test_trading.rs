#![cfg(test)]
extern crate std;

use crate::{storage::TreasuryPairBalances, testutils::Setup};

use soroban_sdk::{testutils::Address as _, Address};
use utils::constant::PRICE_PRECISION;

/* Buy Long */

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
    // ================================================================================

    // Mint user tokens
    setup
        .token_usdc_admin_client
        .mint(&admin, &(init_admin_usdc as i128));
    setup
        .token_usdc_admin_client
        .mint(&user1, &(init_user_usdc as i128));
    assert_eq!(setup.token_usdc.balance(&admin), init_admin_usdc as i128);
    assert_eq!(setup.token_usdc.balance(&user1), init_user_usdc as i128);

    // Mint pair
    setup.pair.mint(&admin, &pair_tokens_to_mint);
    let collateral_info = setup.pair.get_collateral_info();
    let collateral_used =
        (pair_tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;
    assert_eq!(
        setup.token_usdc.balance(&admin),
        (init_admin_usdc - collateral_used) as i128
    ); // consider collateral per pair
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
    // ================================================================================

    let fee_config = setup.treasury.get_pair_fee_config(&setup.pair.address);
    let usdc_less_fee =
        (usdc_to_trade * (PRICE_PRECISION - fee_config.taker_fee)) / PRICE_PRECISION;
    let usdc_fee = usdc_to_trade - usdc_less_fee;

    // Trade
    let expected_out = (usdc_less_fee * PRICE_PRECISION) / 5_000_000;
    let long_out = setup
        .treasury
        .buy_long(&user1, &setup.pair.address, &usdc_to_trade, &0);
    assert_eq!(long_out, expected_out);

    // Assertions
    // ================================================================================

    // [x] USDC transferred from user to Treasury
    assert_eq!(setup.token_usdc.balance(&user1), 0);
    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        (usdc_to_deposit + usdc_to_trade) as i128
    );

    // [x] Long transferred from Treasury to user
    assert_eq!(setup.token_long.balance(&user1), expected_out as i128);
    assert_eq!(
        setup.token_long.balance(&setup.treasury.address),
        (pair_tokens_to_deposit - expected_out) as i128
    );

    // [x] TreasuryPairBalance updated
    assert_eq!(
        setup.treasury.get_balances(&setup.pair.address),
        TreasuryPairBalances {
            token_quote: usdc_to_deposit + usdc_to_trade - usdc_fee,
            token_long: pair_tokens_to_deposit - expected_out,
            token_short: pair_tokens_to_deposit,
        }
    );

    // [x] Fee tracked
    assert_eq!(
        setup.treasury.get_protocol_fees(&setup.pair.address),
        usdc_fee
    );
}

/* Buy Short */

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
    // ================================================================================

    // Mint user tokens
    setup
        .token_usdc_admin_client
        .mint(&admin, &(init_admin_usdc as i128));
    setup
        .token_usdc_admin_client
        .mint(&user1, &(init_user_usdc as i128));
    assert_eq!(setup.token_usdc.balance(&admin), init_admin_usdc as i128);
    assert_eq!(setup.token_usdc.balance(&user1), init_user_usdc as i128);

    // Mint pair
    setup.pair.mint(&admin, &pair_tokens_to_mint);
    let collateral_info = setup.pair.get_collateral_info();
    let collateral_used =
        (pair_tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;
    assert_eq!(
        setup.token_usdc.balance(&admin),
        (init_admin_usdc - collateral_used) as i128
    ); // consider collateral per pair
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
    // ================================================================================

    let fee_config = setup.treasury.get_pair_fee_config(&setup.pair.address);
    let usdc_less_fee =
        (usdc_to_trade * (PRICE_PRECISION - fee_config.taker_fee)) / PRICE_PRECISION;
    let usdc_fee = usdc_to_trade - usdc_less_fee;

    // Trade
    let expected_out = (usdc_less_fee * PRICE_PRECISION) / 5_000_000;
    let short_out = setup
        .treasury
        .buy_short(&user1, &setup.pair.address, &usdc_to_trade, &0);
    assert_eq!(short_out, expected_out);

    // Assertions
    // ================================================================================

    // [x] USDC transferred from user to Treasury
    assert_eq!(setup.token_usdc.balance(&user1), 0);
    assert_eq!(
        setup.token_usdc.balance(&setup.treasury.address),
        (usdc_to_deposit + usdc_to_trade) as i128
    );

    // [x] Short transferred from Treasury to user
    assert_eq!(setup.token_short.balance(&user1), expected_out as i128);
    assert_eq!(
        setup.token_short.balance(&setup.treasury.address),
        (pair_tokens_to_deposit - expected_out) as i128
    );

    // [x] TreasuryPairBalance updated
    assert_eq!(
        setup.treasury.get_balances(&setup.pair.address),
        TreasuryPairBalances {
            token_quote: usdc_to_deposit + usdc_to_trade - usdc_fee,
            token_long: pair_tokens_to_deposit,
            token_short: pair_tokens_to_deposit - expected_out,
        }
    );

    // [x] Fee tracked
    assert_eq!(
        setup.treasury.get_protocol_fees(&setup.pair.address),
        usdc_fee
    );
}

/* Sell Long */

// #[test]
// fn test_sell_long() {
//     let setup = Setup::default();

//     let admin = setup.admin.clone();
//     let user1 = setup.users[1].clone();

//     let init_admin_usdc = 2000_0000000_u128;
//     let init_user_usdc = 1_0000000_u128;

//     let pair_tokens_to_mint = 10_0000000_u128;
//     let pair_tokens_to_deposit = 10_0000000_u128;
//     let usdc_to_deposit = 1000_0000000_u128;
//     let usdc_to_trade = 1_0000000_u128;

//     // Setup
//     // ================================================================================

//     // Mint user tokens
//     setup.token_usdc_admin_client.mint(&admin, &(init_admin_usdc as i128));
//     setup.token_usdc_admin_client.mint(&user1, &(init_user_usdc as i128));
//     assert_eq!(setup.token_usdc.balance(&admin), init_admin_usdc as i128);
//     assert_eq!(setup.token_usdc.balance(&user1), init_user_usdc as i128);

//     // Mint pair
//     setup.pair.mint(&admin, &pair_tokens_to_mint);
//     let collateral_info = setup.pair.get_collateral_info();
//     let collateral_used =
//         (pair_tokens_to_mint * collateral_info.collateral_per_pair) / PRICE_PRECISION;
//     assert_eq!(setup.token_usdc.balance(&admin), (init_admin_usdc - collateral_used) as i128); // consider collateral per pair
//     assert_eq!(setup.token_long.balance(&admin), pair_tokens_to_mint as i128);
//     assert_eq!(setup.token_short.balance(&admin), pair_tokens_to_mint as i128);

//     // Deposit liquidity
//     setup.treasury.deposit(
//         &admin,
//         &setup.pair.address,
//         &pair_tokens_to_deposit,
//         &pair_tokens_to_deposit,
//         &usdc_to_deposit
//     );
//     assert_eq!(setup.token_usdc.balance(&admin), 0);
//     assert_eq!(setup.token_long.balance(&admin), 0);
//     assert_eq!(setup.token_short.balance(&admin), 0);

//     // Test
//     // ================================================================================

//     let fee = setup.treasury.get_pair_fee(&setup.pair.address);
//     let usdc_less_fee = (usdc_to_trade * (PRICE_PRECISION - fee)) / PRICE_PRECISION;
//     let usdc_fee = usdc_to_trade - usdc_less_fee;

//     // Trade
//     let expected_out = (usdc_less_fee * PRICE_PRECISION) / 5_000_000;
//     let long_out = setup.treasury.buy_long(&user1, &setup.pair.address, &usdc_to_trade, &0);
//     assert_eq!(long_out, expected_out);

//     // Assertions
//     // ================================================================================

//     // [x] USDC transferred from user to Treasury
//     assert_eq!(setup.token_usdc.balance(&user1), 0);
//     assert_eq!(
//         setup.token_usdc.balance(&setup.treasury.address),
//         (usdc_to_deposit + usdc_to_trade) as i128
//     );

//     // [x] Long transferred from Treasury to user
//     assert_eq!(setup.token_long.balance(&user1), expected_out as i128);
//     assert_eq!(
//         setup.token_long.balance(&setup.treasury.address),
//         (pair_tokens_to_deposit - expected_out) as i128
//     );

//     // [x] TreasuryPairBalance updated
//     assert_eq!(setup.treasury.get_balances(&setup.pair.address), TreasuryPairBalances {
//         token_quote: usdc_to_deposit + usdc_to_trade - usdc_fee,
//         token_long: pair_tokens_to_deposit - expected_out,
//         token_short: pair_tokens_to_deposit,
//     });

//     // [x] Fee tracked
//     assert_eq!(setup.treasury.get_protocol_fees(&setup.pair.address), usdc_fee);
// }
