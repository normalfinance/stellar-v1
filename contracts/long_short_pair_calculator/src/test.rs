#![cfg(test)]
extern crate std;

use crate::testutils::Setup;

#[test]
fn test_below_lower_bound_returns_zero() {
    let setup = Setup::default();

    let result = setup.calculator.percent_long_collateral(&0, &0, &200);

    // Since below lower bound, should clamp to 0
    assert_eq!(result, 0, "price below lower bound should map to 0");
}

#[test]
fn test_below_lower_bountest_at_lower_bound_returns_zerod_returns_zero() {
    let setup = Setup::default();

    let result = setup.calculator.percent_long_collateral(&50, &0, &200);
    assert_eq!(
        result, 2_500_000,
        "price == lower bound should map to 2_500_000"
    );
}

#[test]
fn test_midpoint_returns_half() {
    let setup = Setup::default();

    let result = setup.calculator.percent_long_collateral(&100, &0, &200);
    assert_eq!(
        result, 5_000_000,
        "price == upper bound should map to 5_000_000"
    )
}

#[test]
fn test_upper_bound_returns_one() {
    let setup = Setup::default();

    let result = setup.calculator.percent_long_collateral(&200, &0, &200);
    assert_eq!(
        result, 10_000_000,
        "price == upper bound should map to 10_000_000"
    );
}

#[test]
fn test_above_upper_bound_clamped() {
    let setup = Setup::default();

    let result = setup.calculator.percent_long_collateral(&210, &0, &200);
    assert_eq!(
        result, 10_000_000,
        "price above upper bound should clamp to 10_000_000"
    );
}

#[test]
fn test_near_lower_bound_quarter_value() {
    let setup = Setup::default();

    let result = setup.calculator.percent_long_collateral(&75, &0, &200);

    assert!(result < 5_000_000, "expected <5_000_000, got {}", result);
}

#[test]
fn test_near_upper_bound_three_quarters() {
    let setup = Setup::default();

    let result = setup.calculator.percent_long_collateral(&125, &0, &200);

    assert!(result > 5_000_000, "expected >5_000_000, got {}", result);
}
