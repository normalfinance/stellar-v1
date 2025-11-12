// ============================================================
// Logarithmic payoff mapping using 1e7 fixed-point precision.
// Suitable for Stellar-native decimal standard (7 dp).
// ============================================================

const ONE: i128 = 10_000_000; // 1e7 scale
const LN2_1E7: i128 = 693_1472; // ln(2) ≈ 0.6931472 × 1e7

// Fixed-point multiply/divide helpers (scale = 1e7)
fn fmul(a: i128, b: i128) -> i128 {
    (a * b) / ONE
}

/// Approximate natural log (ln) for x in [0.5, 2.0] with 4-term Taylor series.
/// ln(x) ≈ (x-1) - (x-1)^2/2 + (x-1)^3/3 - (x-1)^4/4
/// Returns ln(x) scaled by 1e7 (ln(e)=1e7)
pub fn flog(x: i128) -> i128 {
    if x <= 0 {
        return 0;
    }

    let mut v = x;
    let mut k: i32 = 0;
    while v > 2 * ONE {
        v /= 2;
        k += 1;
    }
    while v < ONE / 2 {
        v *= 2;
        k -= 1;
    }

    let t = v - ONE; // (x - 1)
    let t2 = fmul(t, t);
    let t3 = fmul(t2, t);
    let t4 = fmul(t3, t);

    // ln(v)
    let ln_v = t - t2 / 2 + t3 / 3 - t4 / 4;

    ln_v + LN2_1E7 * (k as i128)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to pretty-print scaled values as f64
    fn as_f64(x: i128) -> f64 {
        (x as f64) / (ONE as f64)
    }

    #[test]
    fn test_ln_of_one_is_zero() {
        let result = flog(ONE);
        assert!(
            result.abs() < 10, // within 1e-6 absolute error
            "ln(1) should be 0, got {} (~{})",
            result,
            as_f64(result)
        );
    }

    #[test]
    fn test_ln_of_two_is_ln2() {
        let result = flog(2 * ONE);
        let expected = LN2_1E7;
        let diff = (result - expected).abs();
        assert!(
            diff < 500, // <5e-5 tolerance
            "ln(2) expected {}, got {}, diff {}",
            expected,
            result,
            diff
        );
    }

    #[test]
    fn test_ln_of_half_is_negative_ln2() {
        let result = flog(ONE / 2);
        let expected = -LN2_1E7;
        let diff = (result - expected).abs();
        assert!(
            diff < 500,
            "ln(0.5) expected {}, got {}, diff {}",
            expected,
            result,
            diff
        );
    }

    #[test]
    fn test_monotonic_increase() {
        let a = flog((7 * ONE) / 10); // ln(0.7)
        let b = flog(ONE); // ln(1.0)
        let c = flog((15 * ONE) / 10); // ln(1.5)
        assert!(a < b && b < c, "ln(x) must increase with x");
    }

    #[test]
    fn test_scaling_accuracy() {
        // ln(1.1) ≈ 0.09531, scaled 0.09531 * 1e7 = 953100
        let result = flog((11 * ONE) / 10);
        let expected = 953_100;
        let diff = (result - expected).abs();
        assert!(
            diff < 10_000,
            "ln(1.1) expected ~{}, got {}, diff {}",
            expected,
            result,
            diff
        );
    }

    #[test]
    fn test_large_range_clamping() {
        // 8.0 -> 3*ln(2)
        let result = flog(8 * ONE);
        let expected = LN2_1E7 * 3;
        let diff = (result - expected).abs();
        assert!(
            diff < 10_000,
            "ln(8) expected ~{}, got {}, diff {}",
            expected,
            result,
            diff
        );
    }

    #[test]
    fn test_non_positive_input_returns_zero() {
        assert_eq!(flog(0), 0);
        assert_eq!(flog(-1), 0);
    }
}
