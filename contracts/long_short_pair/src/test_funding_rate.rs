#![cfg(test)]
extern crate std;

use crate::testutils::Setup;
use soroban_sdk::{testutils::Address as _, Address, Vec};
use utils::constant::{ONE_HOUR, ONE_MINUTE};
use utils::test_utils::jump;

const RESERVES_FOR_PRICE_ABOVE: [u128; 2] = [90_0000000, 5500_0000000];
const RESERVES_FOR_PRICE_IDEAL: [u128; 2] = [100_0000000, 5000_0000000];
const RESERVES_FOR_PRICE_BELOW: [u128; 2] = [110_0000000, 4500_0000000];

// Funding Period

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_update_funding_period_invalid() {
    let setup = Setup::default();
    setup.pair.update_funding_period(&setup.admin, &0);
}

#[test]
fn test_update_funding_period() {
    let setup = Setup::default();
    let new_funding_period = 1000000;

    setup
        .pair
        .update_funding_period(&setup.admin, &new_funding_period);

    let funding_info = setup.pair.get_funding_info();
    assert_eq!(funding_info.funding_period, new_funding_period);
}

// Funding Rate

#[test]
#[should_panic(expected = "Error(Contract, #206)")]
fn test_update_funding_rate_fails_with_bad_plane() {
    let setup = Setup::default();
    let plane = Address::generate(&setup.env);

    setup.pair.set_pool_plane(&setup.admin, &plane);

    // Move forward in time to allow funding update
    jump(&setup.env, ONE_HOUR);

    setup.pair.update_funding_rate(&setup.admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #211)")]
// TODO: write better tests to test this failure for any of the four reserves being zero
fn test_update_funding_rate_returns_zero_with_zero_liquidity() {
    let setup = Setup::default();

    // Set pool reserves to zero
    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, [0, 0]),
    );
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, [0, 0]),
    );

    // Move forward in time to allow funding update
    jump(&setup.env, ONE_HOUR);

    // Update the funding rate
    setup.pair.update_funding_rate(&setup.admin);
}

// Funding Rate - Pool Price Scenarios

// Long = ^ | Short = ^
#[test]
fn test_update_funding_rate_is_zero_when_long_overpriced_and_short_overpriced() {
    let setup = Setup::default();

    // Make the long pool price ^ by buying LONG
    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_ABOVE),
    );

    // Make the short pool price ^ by buying SHORT
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_ABOVE),
    );

    // Move forward in time to allow funding update
    jump(&setup.env, ONE_HOUR);

    // Update the funding rate
    setup.pair.update_funding_rate(&setup.admin);

    // Make sure the oracle price is fetched and collateral info is updated
    assert_eq!(
        setup.pair.get_collateral_info().collateral_percent_long,
        5000
    );

    // Funding rate should be zero since arbitrage can balance pool prices
    let funding_info = setup.pair.get_funding_info();

    assert_eq!(funding_info.cumulative_funding_index_long, 0);
    assert_eq!(funding_info.cumulative_funding_index_short, 0);
    assert_eq!(funding_info.last_funding_rate, 0);
    assert_eq!(funding_info.last24h_avg_funding_rate, 0);
    assert_eq!(
        funding_info.last_funding_rate_ts,
        setup.env.ledger().timestamp()
    );
}

// Long = ^ | Short = 0
#[test]
fn test_update_funding_rate_is_zero_when_long_overpriced_and_short_ideal() {
    let setup = Setup::default();

    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_ABOVE),
    );
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_IDEAL),
    );

    // Move forward in time to allow funding update
    jump(&setup.env, ONE_HOUR);

    // Update the funding rate
    setup.pair.update_funding_rate(&setup.admin);

    // Make sure the oracle price is fetched and collateral info is updated
    assert_eq!(
        setup.pair.get_collateral_info().collateral_percent_long,
        5000
    );

    // Funding rate should be zero since arbitrage can balance pool prices
    let funding_info = setup.pair.get_funding_info();
    assert_eq!(funding_info.cumulative_funding_index_long, 0);
    assert_eq!(funding_info.cumulative_funding_index_short, 0);
    assert_eq!(funding_info.last_funding_rate, 0);
    assert_eq!(funding_info.last24h_avg_funding_rate, 0);
    assert_eq!(
        funding_info.last_funding_rate_ts,
        setup.env.ledger().timestamp()
    );
}

// Long = ^ | Short = ⌄
#[test]
fn test_update_funding_rate_is_applied_when_long_overpriced_and_short_underpriced() {
    let setup = Setup::default();

    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_ABOVE),
    );
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_BELOW),
    );

    // Move forward in time to allow funding update
    jump(&setup.env, ONE_HOUR);

    // Update the funding rate
    setup.pair.update_funding_rate(&setup.admin);

    // Make sure the oracle price is fetched and collateral info is updated
    assert_eq!(
        setup.pair.get_collateral_info().collateral_percent_long,
        5000
    ); // 50%

    // Funding rate should be zero since arbitrage can balance pool prices
    let funding_info = setup.pair.get_funding_info();
    assert_eq!(funding_info.cumulative_funding_index_long, -16);
    assert_eq!(funding_info.cumulative_funding_index_short, 16);
    assert_eq!(funding_info.last_funding_rate, 16);
    assert_eq!(funding_info.last24h_avg_funding_rate, 0);
    assert_eq!(
        funding_info.last_funding_rate_ts,
        setup.env.ledger().timestamp()
    );
}

// Long = 0 | Short = ^
#[test]
fn test_update_funding_rate_is_zero_when_long_ideal_and_short_overpriced() {
    let setup = Setup::default();

    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_IDEAL),
    );
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_ABOVE),
    );

    jump(&setup.env, ONE_HOUR);

    setup.pair.update_funding_rate(&setup.admin);

    // Funding rate should be zero since arbitrage can balance pool prices
    let funding_info = setup.pair.get_funding_info();
    assert_eq!(funding_info.cumulative_funding_index_long, 0);
    assert_eq!(funding_info.cumulative_funding_index_short, 0);
    assert_eq!(funding_info.last_funding_rate, 0);
    assert_eq!(funding_info.last24h_avg_funding_rate, 0);
    assert_eq!(
        funding_info.last_funding_rate_ts,
        setup.env.ledger().timestamp()
    );
}

// Long = 0 | Short = 0
#[test]
fn test_update_funding_rate_is_zero_when_long_ideal_and_short_ideal() {
    let setup = Setup::default();

    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_IDEAL),
    );
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_IDEAL),
    );

    jump(&setup.env, ONE_HOUR);

    setup.pair.update_funding_rate(&setup.admin);

    // Funding rate should be zero since arbitrage can balance pool prices
    let funding_info = setup.pair.get_funding_info();
    assert_eq!(funding_info.cumulative_funding_index_long, 0);
    assert_eq!(funding_info.cumulative_funding_index_short, 0);
    assert_eq!(funding_info.last_funding_rate, 0);
    assert_eq!(funding_info.last24h_avg_funding_rate, 0);
    assert_eq!(
        funding_info.last_funding_rate_ts,
        setup.env.ledger().timestamp()
    );
}

// Long = 0 | Short = ⌄
#[test]
fn test_update_funding_rate_is_zero_when_long_ideal_and_short_underpriced() {
    let setup = Setup::default();

    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_IDEAL),
    );
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_BELOW),
    );

    jump(&setup.env, ONE_HOUR);

    setup.pair.update_funding_rate(&setup.admin);

    // Funding rate should be zero since arbitrage can balance pool prices
    let funding_info = setup.pair.get_funding_info();
    assert_eq!(funding_info.cumulative_funding_index_long, 0);
    assert_eq!(funding_info.cumulative_funding_index_short, 0);
    assert_eq!(funding_info.last_funding_rate, 0);
    assert_eq!(funding_info.last24h_avg_funding_rate, 0);
    assert_eq!(
        funding_info.last_funding_rate_ts,
        setup.env.ledger().timestamp()
    );
}

// Long = ⌄ | Short = ^
#[test]
fn test_update_funding_rate_is_applied_when_long_underpriced_and_short_overpriced() {
    let setup = Setup::default();

    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_BELOW),
    );
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_ABOVE),
    );

    jump(&setup.env, ONE_HOUR);

    setup.pair.update_funding_rate(&setup.admin);

    // Funding rate should be zero since arbitrage can balance pool prices
    let funding_info = setup.pair.get_funding_info();
    assert_eq!(funding_info.cumulative_funding_index_long, 0);
    assert_eq!(funding_info.cumulative_funding_index_short, 0);
    assert_eq!(funding_info.last_funding_rate, 0);
    assert_eq!(funding_info.last24h_avg_funding_rate, 0);
    assert_eq!(
        funding_info.last_funding_rate_ts,
        setup.env.ledger().timestamp()
    );
}

// Long = ⌄ | Short = 0
#[test]
fn test_update_funding_rate_is_zero_when_long_underpriced_and_short_ideal() {
    let setup = Setup::default();

    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_BELOW),
    );
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_IDEAL),
    );

    jump(&setup.env, ONE_HOUR);

    setup.pair.update_funding_rate(&setup.admin);

    // Funding rate should be zero since arbitrage can balance pool prices
    let funding_info = setup.pair.get_funding_info();
    assert_eq!(funding_info.cumulative_funding_index_long, 0);
    assert_eq!(funding_info.cumulative_funding_index_short, 0);
    assert_eq!(funding_info.last_funding_rate, 0);
    assert_eq!(funding_info.last24h_avg_funding_rate, 0);
    assert_eq!(
        funding_info.last_funding_rate_ts,
        setup.env.ledger().timestamp()
    );
}

// Long = ⌄ | Short = ⌄
#[test]
fn test_update_funding_rate_is_zero_when_long_underpriced_and_short_underpriced() {
    let setup = Setup::default();

    setup.plane.update(
        &setup.long_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_BELOW),
    );
    setup.plane.update(
        &setup.short_pool,
        &setup.pool_type,
        &setup.pool_init_args,
        &Vec::from_array(&setup.env, RESERVES_FOR_PRICE_BELOW),
    );

    // Move forward in time to allow funding update
    jump(&setup.env, ONE_HOUR);

    // Update the funding rate
    setup.pair.update_funding_rate(&setup.admin);

    // Make sure the oracle price is fetched and collateral info is updated
    assert_eq!(
        setup.pair.get_collateral_info().collateral_percent_long,
        5000
    );

    // Funding rate should be zero since arbitrage can balance pool prices
    let funding_info = setup.pair.get_funding_info();

    assert_eq!(funding_info.cumulative_funding_index_long, 0);
    assert_eq!(funding_info.cumulative_funding_index_short, 0);
    assert_eq!(funding_info.last_funding_rate, 0);
    assert_eq!(funding_info.last24h_avg_funding_rate, 0);
    assert_eq!(
        funding_info.last_funding_rate_ts,
        setup.env.ledger().timestamp()
    );
}

// #[test]
// fn test_update_funding_rate_resets_to_zero_after_value() {

// }

// #[test]
// fn test_update_funding_rate_applies_to_redemption() {
//     let setup = Setup::default();
//     let user1 = setup.users[1].clone();

//     // Mint pair tokens
//     setup.pair.mint(&user1, &1_0000000);

//     // Adjust pool prices to require funding
//     setup.plane.update(
//         &setup.long_pool,
//         &setup.pool_type,
//         &setup.pool_init_args,
//         &Vec::from_array(&setup.env, [110_0000000, 4500_0000000])
//     );
//     setup.plane.update(
//         &setup.short_pool,
//         &setup.pool_type,
//         &setup.pool_init_args,
//         &Vec::from_array(&setup.env, [90_0000000, 5500_0000000])
//     );

//     // Move forward in time to allow funding update
//     jump(&setup.env, ONE_HOUR);

//     // Update the funding rate
//     setup.pair.update_funding_rate(&setup.admin);

//     // Ensure cumulative funding index is applied to the redemption
//     let expected_collateral_adjustment =
//     let collateral_adjusted = setup.pair.redeem(&user1, &1_0000000);

//     assert_eq!(collateral_adjusted, 0);

// }

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_update_funding_rate_while_paused() {
    let setup = Setup::default();
    setup.pair.kill_update_funding(&setup.admin);

    setup.pair.update_funding_rate(&setup.admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #205)")]
fn test_update_funding_rate_too_early() {
    let setup = Setup::default();

    setup.pair.update_funding_rate(&setup.admin);

    jump(&setup.env, ONE_MINUTE as u64);

    setup.pair.update_funding_rate(&setup.admin);
}

// #[test]
// #[should_panic(expected = "Error(Contract, #210)")]
// fn test_update_funding_rate_without_pools() {
//     let setup = Setup::default();

//     // Set pools
//     let pool_long = Address::generate(&setup.env);
//     let pool_short = Address::generate(&setup.env);
//     setup.pair.set_pools(&setup.admin, &pool_long, &pool_short);

//     setup.pair.update_funding_rate(&setup.admin);
// }
