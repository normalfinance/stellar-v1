#![cfg(test)]
extern crate std;

use crate::testutils::Setup;
#[test]
fn test_set_long_short_pair_parameters() {
    let setup = Setup::default();

    let upper_bound = 100_000000;
    let lower_bound = 10_000000;

    setup
        .calculator
        .set_long_short_pair_parameters(&setup.pair, &upper_bound, &lower_bound);

    let price = 75_0000000;
    let long_pct = setup
        .calculator
        .pct_long_colat_at_expiry(&setup.pair, &price);

    assert_eq!(long_pct, 0); // TODO:
}

#[test]
#[should_panic(expected = "Error(Contract, #202)")]
fn test_set_long_short_pair_parameters_upper_less_than_lower() {
    let setup = Setup::default();

    setup
        .calculator
        .set_long_short_pair_parameters(&setup.pair, &5_000000, &10_000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #203)")]
fn test_set_long_short_pair_parameters_already_set() {
    let setup = Setup::default();

    setup
        .calculator
        .set_long_short_pair_parameters(&setup.pair, &100_000000, &10_000000);
    setup
        .calculator
        .set_long_short_pair_parameters(&setup.pair, &120_000000, &20_000000);
}
