#![cfg(test)]
extern crate std;

use crate::testutils::Setup;

/// Convenience to read output as f64 percent (0.0–1.0)
fn as_percent(x: u128) -> f64 {
    (x as f64) / 10_000_000.0
}

#[test]
#[should_panic(expected = "Error(Contract, #204)")]
fn test_panics_if_params_not_set() {
    let setup = Setup::default();

    // No params stored for caller
    setup.calculator.percent_long_collateral(&setup.pair, &100);
}

#[test]
#[should_panic(expected = "Error(Contract, #202)")]
fn test_set_parameters_upper_less_than_lower() {
    let setup = Setup::default();

    setup
        .calculator
        .set_parameters(&setup.pair, &10_000000, &5_000000);
}

#[test]
#[should_panic(expected = "Error(Contract, #203)")]
fn test_set_long_short_pair_parameters_already_set() {
    let setup = Setup::default();

    setup
        .calculator
        .set_parameters(&setup.pair, &10_000000, &100_000000);
    setup
        .calculator
        .set_parameters(&setup.pair, &20_000000, &120_000000);
}

#[test]
fn test_below_lower_bound_returns_zero() {
    let setup = Setup::default();
    setup.calculator.set_parameters(&setup.pair, &50, &150);

    let result = setup.calculator.percent_long_collateral(&setup.pair, &25);

    // Since below lower bound, should clamp to 0
    assert_eq!(result, 0, "price below lower bound should map to 0");
}

#[test]
fn test_below_lower_bountest_at_lower_bound_returns_zerod_returns_zero() {
    let setup = Setup::default();
    setup.calculator.set_parameters(&setup.pair, &50, &150);

    let result = setup.calculator.percent_long_collateral(&setup.pair, &50);

    assert_eq!(result, 0, "price == lower bound should map to 0");
}

#[test]
fn test_midpoint_returns_half() {
    let setup = Setup::default();
    setup.calculator.set_parameters(&setup.pair, &50, &150);

    let result = setup.calculator.percent_long_collateral(&setup.pair, &100);

    let pct = as_percent(result);
    assert!(
        (pct - 0.5).abs() < 0.0001,
        "midpoint should map to 0.5, got {}",
        pct
    );
}

#[test]
fn test_upper_bound_returns_one() {
    let setup = Setup::default();
    setup.calculator.set_parameters(&setup.pair, &50, &150);

    let result = setup.calculator.percent_long_collateral(&setup.pair, &150);
    assert_eq!(result, 10_000_000, "price == upper bound should map to 1.0");
}

#[test]
fn test_above_upper_bound_clamped() {
    let setup = Setup::default();
    setup.calculator.set_parameters(&setup.pair, &50, &150);

    let result = setup.calculator.percent_long_collateral(&setup.pair, &200);
    assert_eq!(
        result, 10_000_000,
        "price above upper bound should clamp to 1.0"
    );
}

#[test]
fn test_near_lower_bound_quarter_value() {
    let setup = Setup::default();
    setup.calculator.set_parameters(&setup.pair, &50, &150);

    let result = setup.calculator.percent_long_collateral(&setup.pair, &75);
    let pct = as_percent(result);
    assert!((pct - 0.25).abs() < 0.001, "expected ~0.25, got {}", pct);
}

#[test]
fn test_near_upper_bound_three_quarters() {
    let setup = Setup::default();
    setup.calculator.set_parameters(&setup.pair, &50, &150);

    let result = setup.calculator.percent_long_collateral(&setup.pair, &125);
    let pct = as_percent(result);
    assert!((pct - 0.75).abs() < 0.001, "expected ~0.75, got {}", pct);
}
