#![cfg(test)]
extern crate std;

use crate::{storage::TreasuryPairBalances, testutils::Setup};

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, IntoVal, Symbol, Vec,
};
use utils::constant::PRICE_PRECISION;

/* Mint and Sell Short -> Buying Long */

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_mint_and_sell_short_invalid_amount() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    setup
        .treasury
        .mint_and_sell_short(&user1, &setup.pair.address, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #213)")]
fn test_mint_and_sell_short_trading_kills() {
    let setup = Setup::default();
    let admin = setup.admin.clone();
    let user1 = setup.users[1].clone();

    setup.treasury.kill_trade(&admin);

    setup
        .treasury
        .mint_and_sell_short(&user1, &setup.pair.address, &1_0000000_u128);
}

#[test]
#[should_panic(expected = "Error(Contract, #501)")]
fn test_mint_and_sell_short_invalid_pair() {
    let setup = Setup::default();
    let user1 = setup.users[1].clone();
    let pair = Address::generate(&setup.env);

    setup
        .treasury
        .mint_and_sell_short(&user1, &pair, &1_0000000_u128);
}

#[test]
fn test_mint_and_sell_short() {
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

    // Test
    // ================================================================================

    let fee_config = setup.treasury.get_pair_fee_config(&setup.pair.address);
    let usdc_less_fee =
        (usdc_to_trade * (PRICE_PRECISION - fee_config.maker_fee)) / PRICE_PRECISION;
    let usdc_fee = usdc_to_trade - usdc_less_fee;

    // Trade
    let expected_out = (usdc_less_fee * PRICE_PRECISION) / 5_000_000;
    let long_out = setup
        .treasury
        .mint_and_sell_short(&user1, &setup.pair.address, &usdc_to_trade);

    assert_eq!(
        setup.env.auths()[0],
        (
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    setup.pair.address.clone(),
                    Symbol::new(&setup.env, "mint"),
                    Vec::from_array(
                        &setup.env,
                        [
                            user1.to_val(),
                            desired_amounts.to_val(),
                            (0_u128).into_val(&setup.env),
                        ]
                    ),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token1.address.clone(),
                        Symbol::new(&setup.env, "transfer"),
                        Vec::from_array(
                            &setup.env,
                            [
                                user1.to_val(),
                                liq_pool.address.to_val(),
                                (desired_amounts.get(0).unwrap() as i128).into_val(&setup.env),
                            ]
                        ),
                    )),
                    sub_invocations: std::vec![],
                }],
            },
        )
    );

    assert_eq!(long_out, expected_out);

    // Assertions
    // ================================================================================

    // [ ] Token(s) were minted by the Pair
    assert_eq!(setup.pair.get_collateral_info().total_collateral, 0);

    let token_pair_supplies = setup.pair.get_total_token_supplies();
    assert_eq!(token_pair_supplies.0, 0);
    assert_eq!(token_pair_supplies.1, 0);

    // ...
}
