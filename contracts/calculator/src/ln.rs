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
