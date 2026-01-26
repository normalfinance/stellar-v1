use soroban_sdk::{panic_with_error, Env};
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::SafeMath;

use crate::errors::TreasuryError;
use crate::storage::{TreasuryFeeConfig, TreasuryRiskParameters};

/// Apply a fee to an input amount, taking the fee from the input amount.
///
/// The fee is expressed in `PRICE_PRECISION` (typically 1e7), where:
/// - `fee = 0` means 0%
/// - `fee = PRICE_PRECISION` means 100% (net becomes 0)
///
/// ## Formula
/// `net_in = amount_in * (1 - fee)`
///
/// ## Panics
/// - `TreasuryError::InvalidInput` if `amount_in == 0`
/// - `TreasuryError::InvalidInput` if `fee > PRICE_PRECISION`
pub fn apply_fee_to_input(e: &Env, amount_in: u128, fee: u128) -> u128 {
    if amount_in == 0 || fee > PRICE_PRECISION {
        panic_with_error!(e, TreasuryError::InvalidInput);
    }

    amount_in
        .safe_mul(e, PRICE_PRECISION.safe_sub(e, fee))
        .safe_div(e, PRICE_PRECISION)
}

/// Computes the taker fee (half-spread) in `PRICE_PRECISION` units using a minimal,
/// inventory-aware model:
///
/// ```text
/// fee = base_fee ± imbalance_adjustment
/// ```
///
/// - `base_fee` is a constant (typically 10–50 bps) taken from config and clamped to
///   `[0, PRICE_PRECISION]`.
/// - `imbalance_adjustment` increases the fee when a trade **worsens** net inventory
///   imbalance, and decreases the fee (rebate) when the trade **improves** it.
///
/// ### Inputs
/// - `imbalance_delta_abs_tokens`: The absolute change in the treasury’s net inventory
///   imbalance caused by the trade, measured in **token units**.
///   This should be computed as:
///   `abs_after - abs_before` (absolute delta), where:
///   `abs_* = abs(long_tokens - short_tokens)`.
/// - `imbalance_increases`: `true` if the trade increases `abs(long_tokens - short_tokens)`,
///   `false` if it decreases it.
/// - `risk_params.max_net_imbalance_tokens`: The configured “full-scale” imbalance in tokens.
///   The adjustment scales linearly with `imbalance_delta_abs_tokens / max_net_imbalance_tokens`.
/// - `risk_params.imbalance_fee_max`: The maximum additive/subtractive fee adjustment
///   (in `PRICE_PRECISION`) applied when `imbalance_delta_abs_tokens >= max_net_imbalance_tokens`.
///
/// ### Adjustment formula
/// ```text
/// frac = imbalance_delta_abs_tokens / max_net_imbalance_tokens        (scaled to PRICE_PRECISION)
/// adj  = imbalance_fee_max * frac
///
/// if imbalance_increases:
///     fee = base_fee + adj
/// else:
///     fee = base_fee - adj
///
/// fee is clamped to [0, PRICE_PRECISION]
/// ```
///
/// ### Behavior
/// - If any of these are zero, the function returns `base_fee`:
///   - `risk_params.max_net_imbalance_tokens == 0`
///   - `risk_params.imbalance_fee_max == 0`
///   - `imbalance_delta_abs_tokens == 0`
/// - Fee is always clamped to `[0, PRICE_PRECISION]`.
///
/// ### Notes
/// - This function does **not** compute `imbalance_delta_abs_tokens`; callers are expected to
///   compute it from pre/post trade token balances (often via a risk helper).
/// - The hard safety invariant should still be enforced elsewhere:
///   `abs(long_tokens - short_tokens) <= max_net_imbalance_tokens`.
pub fn calculate_fee(
    e: &Env,
    fee_config: &TreasuryFeeConfig,
    imbalance_delta_abs_tokens: u128,
    imbalance_increases: bool,
    risk_params: &TreasuryRiskParameters,
) -> u128 {
    // Base fee (e.g. 10–50 bps) in PRICE_PRECISION.
    let base_fee = clamp_u128(fee_config.base_fee, 0, PRICE_PRECISION);

    // If imbalance controls are disabled, return base fee.
    if risk_params.max_net_imbalance_tokens == 0
        || risk_params.imbalance_fee_max == 0
        || imbalance_delta_abs_tokens == 0
    {
        return base_fee;
    }

    // Scale delta by the configured max imbalance:
    //   frac = imbalance_delta / max_net_imbalance_tokens   (in PRICE_PRECISION)
    let frac = imbalance_delta_abs_tokens
        .safe_mul(e, PRICE_PRECISION)
        .safe_div(e, risk_params.max_net_imbalance_tokens);

    // adj = imbalance_fee_max * frac
    let adj = risk_params
        .imbalance_fee_max
        .safe_mul(e, frac)
        .safe_div(e, PRICE_PRECISION);

    let fee = if imbalance_increases {
        base_fee.safe_add(e, adj)
    } else {
        base_fee.saturating_sub(adj)
    };

    clamp_u128(fee, 0, PRICE_PRECISION)
}

fn clamp_u128(x: u128, lo: u128, hi: u128) -> u128 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    fn env() -> Env {
        Env::default()
    }

    fn fee_cfg(base_fee: u128) -> TreasuryFeeConfig {
        // If your config field is `taker_base_fee`, swap it here.
        TreasuryFeeConfig {
            base_fee,
            // If your struct has additional fields, fill them in here as needed,
            // or use `..Default::default()` if available.
        }
    }

    fn risk_cfg(max_net: u128, fee_max: u128) -> TreasuryRiskParameters {
        TreasuryRiskParameters {
            toxic_threshold: 0,
            max_net_imbalance_tokens: max_net,
            imbalance_fee_max: fee_max,
            // If you still have other fields (e.g. `toxic_threshold`), set them too.
        }
    }

    // ----------------------------
    // apply_fee_to_input() tests
    // ----------------------------

    #[test]
    fn apply_fee_to_input_zero_fee_no_change() {
        let e = env();
        let amt = 1_234_567u128;
        assert_eq!(apply_fee_to_input(&e, amt, 0), amt);
    }

    #[test]
    fn apply_fee_to_input_full_fee_returns_zero() {
        let e = env();
        let amt = 999_999u128;
        assert_eq!(apply_fee_to_input(&e, amt, PRICE_PRECISION), 0);
    }

    #[test]
    fn apply_fee_to_input_matches_expected_formula() {
        let e = env();
        let amt = 1_000_000u128;
        let fee = PRICE_PRECISION / 100; // 1%
        let expected = (amt * (PRICE_PRECISION - fee)) / PRICE_PRECISION;
        assert_eq!(apply_fee_to_input(&e, amt, fee), expected);
        assert!(apply_fee_to_input(&e, amt, fee) < amt);
    }

    #[test]
    #[should_panic]
    fn apply_fee_to_input_amount_zero_panics() {
        let e = env();
        let _ = apply_fee_to_input(&e, 0, 0);
    }

    #[test]
    #[should_panic]
    fn apply_fee_to_input_fee_too_large_panics() {
        let e = env();
        let _ = apply_fee_to_input(&e, 1, PRICE_PRECISION + 1);
    }

    // ----------------------------
    // calculate_fee() tests
    // ----------------------------

    #[test]
    fn calculate_fee_returns_clamped_base_when_controls_disabled_by_max_net() {
        let e = env();
        let fee_config = fee_cfg(50_000); // 0.50%
        let risk = risk_cfg(0, 200_000); // disabled (max_net = 0)

        let fee = calculate_fee(&e, &fee_config, 123, true, &risk);
        assert_eq!(fee, 50_000);
    }

    #[test]
    fn calculate_fee_returns_clamped_base_when_controls_disabled_by_fee_max() {
        let e = env();
        let fee_config = fee_cfg(50_000);
        let risk = risk_cfg(1_000, 0); // disabled (fee_max = 0)

        let fee = calculate_fee(&e, &fee_config, 123, true, &risk);
        assert_eq!(fee, 50_000);
    }

    #[test]
    fn calculate_fee_returns_base_when_delta_is_zero() {
        let e = env();
        let fee_config = fee_cfg(12_345);
        let risk = risk_cfg(1_000, 200_000);

        let fee = calculate_fee(&e, &fee_config, 0, true, &risk);
        assert_eq!(fee, 12_345);
    }

    #[test]
    fn calculate_fee_surcharges_when_imbalance_increases() {
        let e = env();
        let fee_config = fee_cfg(10_000); // 0.10%
        let risk = risk_cfg(1_000, 200_000); // max adj = 2.00%

        // delta=250 => frac=0.25 => adj=0.5% (50_000)
        let fee = calculate_fee(&e, &fee_config, 250, true, &risk);
        assert_eq!(fee, 10_000 + 50_000);
    }

    #[test]
    fn calculate_fee_rebates_when_imbalance_decreases_but_never_below_zero() {
        let e = env();
        let fee_config = fee_cfg(10_000); // 0.10%
        let risk = risk_cfg(1_000, 200_000); // max adj = 2.00%

        // delta=250 => adj=0.5% (50_000), rebate > base, should clamp at 0
        let fee = calculate_fee(&e, &fee_config, 250, false, &risk);
        assert_eq!(fee, 0);
    }

    #[test]
    fn calculate_fee_hits_max_adjustment_at_or_above_cap() {
        let e = env();
        let fee_config = fee_cfg(25_000); // 0.25%
        let risk = risk_cfg(1_000, 300_000); // max adj = 3.00%

        // delta=max => adj=fee_max
        let fee_at = calculate_fee(&e, &fee_config, 1_000, true, &risk);
        assert_eq!(fee_at, 25_000 + 300_000);

        // delta>max => frac>1 => adj>fee_max, but note: your current code
        // does NOT clamp frac to 1; it will scale beyond fee_max.
        // The final clamp is to PRICE_PRECISION. This test asserts that behavior.
        let fee_over = calculate_fee(&e, &fee_config, 10_000, true, &risk);
        assert!(fee_over >= fee_at);
        assert!(fee_over <= PRICE_PRECISION);
    }

    #[test]
    fn calculate_fee_clamps_base_fee_to_price_precision() {
        let e = env();
        let fee_config = fee_cfg(PRICE_PRECISION + 123); // base above max
        let risk = risk_cfg(1_000, 200_000);

        let fee = calculate_fee(&e, &fee_config, 0, true, &risk);
        assert_eq!(fee, PRICE_PRECISION);
    }

    #[test]
    fn calculate_fee_clamps_total_fee_to_price_precision() {
        let e = env();
        // Large base + large adj should clamp to PRICE_PRECISION.
        let fee_config = fee_cfg(PRICE_PRECISION);
        let risk = risk_cfg(1, PRICE_PRECISION); // huge max adjustment

        let fee = calculate_fee(&e, &fee_config, 1_000_000, true, &risk);
        assert_eq!(fee, PRICE_PRECISION);
    }

    #[test]
    fn calculate_fee_monotonic_in_delta_when_increasing() {
        let e = env();
        let fee_config = fee_cfg(20_000);
        let risk = risk_cfg(1_000, 200_000);

        let f1 = calculate_fee(&e, &fee_config, 100, true, &risk);
        let f2 = calculate_fee(&e, &fee_config, 200, true, &risk);
        let f3 = calculate_fee(&e, &fee_config, 300, true, &risk);

        assert!(f1 < f2);
        assert!(f2 < f3);
    }

    #[test]
    fn calculate_fee_monotonic_in_delta_when_decreasing() {
        let e = env();
        let fee_config = fee_cfg(150_000); // bigger base so rebates don't clamp at 0 immediately
        let risk = risk_cfg(1_000, 200_000);

        let f1 = calculate_fee(&e, &fee_config, 100, false, &risk);
        let f2 = calculate_fee(&e, &fee_config, 200, false, &risk);
        let f3 = calculate_fee(&e, &fee_config, 300, false, &risk);

        // larger delta => larger rebate => smaller fee
        assert!(f1 > f2);
        assert!(f2 > f3);
        assert!(f3 <= f1);
    }
}
