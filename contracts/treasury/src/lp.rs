use soroban_sdk::{ panic_with_error, Address, Env };
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::PrecisionMath;

use crate::errors::TreasuryError;
use crate::storage::TreasuryPairBalances;

pub fn total_pairs(e: &Env, balances: &TreasuryPairBalances, collateral_per_pair: u128) -> u128 {
    if collateral_per_pair <= 0 {
        return 0;
    }

    let usdc_pairs = balances.token_quote.safe_fixed_div_floor(
        e,
        collateral_per_pair,
        PRICE_PRECISION
    );

    // min(long, short, usdc_pairs)
    let mut total_pairs = balances.token_long.min(balances.token_short);
    total_pairs = total_pairs.min(usdc_pairs);

    total_pairs
}

pub fn shares_to_mint(
    e: &Env,
    pair: &Address,
    total_pairs_before: u128,
    total_shares_before: u128,
    pairs_deposited: u128
) -> u128 {
    if pairs_deposited <= 0 {
        return 0;
    }

    // First deposit: bootstrap
    // (https://app.almanax.ai/scan/13ca3512-fbc7-4909-929a-53855e07d7af/findings/76c2a564-876d-4409-9d7b-e2abfdf8feba)
    if total_shares_before == 0 {
        // Require empty treasury
        // If you allow someone to "seed" shares when treasury already has assets, you can get weird edge cases / implicit donations.
        let balances = crate::storage::get_pair_balances(&e, pair);
        if balances.token_long != 0 || balances.token_short != 0 || balances.token_quote != 0 {
            panic_with_error!(e, TreasuryError::InvalidBalance);
        }

        return pairs_deposited;
    }

    // Normal deposits require an existing base
    if total_pairs_before <= 0 {
        // Defensive: if shares exist, pairs must exist.
        return 0;
    }

    // shares_minted = floor(pairs_deposited * total_shares / total_pairs)
    let numerator = pairs_deposited.safe_fixed_mul_floor(e, total_shares_before, PRICE_PRECISION);

    let minted = numerator.safe_fixed_div_floor(e, total_pairs_before, PRICE_PRECISION); // floor division

    // Prevent "dust deposits" that mint 0 shares
    if minted <= 0 {
        panic_with_error!(e, TreasuryError::DepositTooSmall);
    }

    minted
}

pub fn pairs_for_withdraw(e: &Env, total_pairs: u128, total_shares: u128, shares_in: u128) -> u128 {
    if shares_in == 0 || total_shares == 0 || total_pairs == 0 {
        return 0;
    }

    // pairs_out = floor(shares_in * total_pairs / total_shares)
    let num = shares_in.safe_fixed_mul_floor(e, total_pairs, PRICE_PRECISION);
    let pairs_out = num.safe_fixed_div_floor(e, total_shares, PRICE_PRECISION);

    if pairs_out == 0 {
        panic_with_error!(e, TreasuryError::WithdrawTooSmall);
    }

    pairs_out
}

#[cfg(test)]
mod tests {
    use crate::{ Treasury };

    use super::*;
    use soroban_sdk::{ testutils::Address as _, Address, Env };

    fn setup_test_contract(e: &Env) -> (Address, Address) {
        let admin = Address::generate(e);
        let treasury_contract = e.register(Treasury, (&admin,));

        (treasury_contract, admin)
    }

    fn complete_test_setup(e: &Env) -> (Address, Address) {
        let (treasury_contract, admin) = setup_test_contract(e);

        (treasury_contract, admin)
    }

    fn balances(long: u128, short: u128, quote: u128) -> TreasuryPairBalances {
        TreasuryPairBalances {
            token_long: long,
            token_short: short,
            token_quote: quote,
        }
    }

    // ---------- total_pairs tests ----------

    #[test]
    fn total_pairs_cpp_zero_returns_zero() {
        let e = Env::default();
        let balances = balances(10, 10, 10);
        assert_eq!(total_pairs(&e, &balances, 0), 0);
    }

    #[test]
    fn total_pairs_is_min_of_long_short_and_usdc_pairs() {
        let e = Env::default();

        // Choose easy numbers.
        // usdc_pairs = (quote * PREC) / cpp.
        // We'll make usdc_pairs intentionally the min.
        let cpp: u128 = 5 * PRICE_PRECISION; // 5 USDC per pair (scaled)
        let balances = balances(
            100 * PRICE_PRECISION, // long
            200 * PRICE_PRECISION, // short
            10 * PRICE_PRECISION // quote = 10 USDC (scaled)
        );

        // usdc_pairs = (10*PREC * PREC) / (5*PREC) = 2*PREC
        let usdc_pairs = balances.token_quote.safe_fixed_div_floor(&e, cpp, PRICE_PRECISION);

        assert_eq!(usdc_pairs, 2 * PRICE_PRECISION);

        let out = total_pairs(&e, &balances, cpp);
        assert_eq!(out, 2 * PRICE_PRECISION);
    }

    #[test]
    fn total_pairs_limited_by_long() {
        let e = Env::default();

        let cpp: u128 = 1 * PRICE_PRECISION; // 1 USDC per pair
        let balances = balances(
            3 * PRICE_PRECISION, // long is smallest
            10 * PRICE_PRECISION,
            10 * PRICE_PRECISION // quote enough for 10 pairs
        );

        let out = total_pairs(&e, &balances, cpp);
        assert_eq!(out, 3 * PRICE_PRECISION);
    }

    #[test]
    fn total_pairs_limited_by_short() {
        let e = Env::default();

        let cpp: u128 = 1 * PRICE_PRECISION;
        let balances = balances(
            10 * PRICE_PRECISION,
            4 * PRICE_PRECISION, // short is smallest
            10 * PRICE_PRECISION
        );

        let out = total_pairs(&e, &balances, cpp);
        assert_eq!(out, 4 * PRICE_PRECISION);
    }

    #[test]
    fn total_pairs_usdc_pairs_uses_floor_division() {
        let e = Env::default();

        // cpp = 3 USDC per pair (scaled)
        let cpp: u128 = 3 * PRICE_PRECISION;

        // quote = 10 USDC (scaled)
        // usdc_pairs = floor((10*PREC*PREC)/(3*PREC)) = floor((10/3)*PREC) = 3*PREC + floor(1/3*PREC)
        // It's strictly < 4*PREC, so min should reflect the floored result.
        let balances = balances(100 * PRICE_PRECISION, 100 * PRICE_PRECISION, 10 * PRICE_PRECISION);

        let usdc_pairs = balances.token_quote.safe_fixed_div_floor(&e, cpp, PRICE_PRECISION);

        // Must be < 4*PREC (since 10/3 = 3.333...)
        assert!(usdc_pairs < 4 * PRICE_PRECISION);
        assert!(usdc_pairs >= 3 * PRICE_PRECISION);

        let out = total_pairs(&e, &balances, cpp);
        assert_eq!(out, usdc_pairs);
    }

    // ---------- shares_to_mint tests ----------

    #[test]
    fn shares_to_mint_returns_zero_if_pairs_deposited_zero() {
        let e = Env::default();
        let pair = Address::generate(&e);
        // storage value irrelevant for this early return
        let minted = shares_to_mint(&e, &pair, 100, 100, 0);
        assert_eq!(minted, 0);
    }

    #[test]
    fn shares_to_mint_first_deposit_requires_empty_and_mints_pairs_deposited() {
        let e = Env::default();
        let (contract_address, _) = complete_test_setup(&e);

        e.as_contract(&contract_address, || {
            crate::storage::set_pair_balances(&e, &contract_address, &balances(0, 0, 0));
        });

        let minted = shares_to_mint(
            &e,
            &contract_address,
            0, // total_pairs_before ignored on first deposit
            0, // total_shares_before == 0 => bootstrap path
            7 * PRICE_PRECISION
        );

        assert_eq!(minted, 7 * PRICE_PRECISION);
    }

    #[test]
    #[should_panic]
    fn shares_to_mint_first_deposit_panics_if_treasury_not_empty() {
        let e = Env::default();
        let (contract_address, _) = complete_test_setup(&e);

        e.as_contract(&contract_address, || {
            crate::storage::set_pair_balances(&e, &contract_address, &balances(1, 0, 0));
        });

        // Should hit the assert!(balances all zero) and panic
        let _ = shares_to_mint(&e, &contract_address, 0, 0, 1 * PRICE_PRECISION);
    }

    #[test]
    fn shares_to_mint_normal_deposit_matches_fixed_floor_formula() {
        let e = Env::default();
        let (contract_address, _) = complete_test_setup(&e);

        e.as_contract(&contract_address, || {
            crate::storage::set_pair_balances(&e, &contract_address, &balances(0, 0, 0));
        });

        // Pick values that avoid dust and make the math exact.
        // With your helpers:
        // numerator = floor(pairs_deposited * total_shares / PREC)
        // minted   = floor(numerator * PREC / total_pairs_before)
        //
        // If we set everything in PREC units, it collapses cleanly.

        let total_pairs_before: u128 = 100 * PRICE_PRECISION;
        let total_shares_before: u128 = 200 * PRICE_PRECISION;
        let pairs_deposited: u128 = 10 * PRICE_PRECISION;

        // Expected economic result: minted = pairs_deposited * total_shares_before / total_pairs_before
        // = 10*PREC * 200*PREC / (100*PREC) = 20*PREC
        let expected: u128 = 20 * PRICE_PRECISION;

        let minted = shares_to_mint(
            &e,
            &contract_address,
            total_pairs_before,
            total_shares_before,
            pairs_deposited
        );

        assert_eq!(minted, expected);
    }

    #[test]
    #[should_panic]
    fn shares_to_mint_dust_deposit_panics_deposit_too_small() {
        let e = Env::default();
        let (contract_address, _) = complete_test_setup(&e);

        e.as_contract(&contract_address, || {
            crate::storage::set_pair_balances(&e, &contract_address, &balances(0, 0, 0));
        });

        // Force minted == 0 deterministically:
        //
        // numerator = floor(pairs_deposited * total_shares_before / PREC)
        // If total_shares_before < PREC and pairs_deposited is small, numerator can be 0.
        //
        // Choose:
        //   pairs_deposited = 1
        //   total_shares_before = 1
        // => numerator = floor(1*1 / PREC) = 0
        // => minted = floor(0 * PREC / total_pairs_before) = 0
        //
        // That must trigger DepositTooSmall panic.
        let total_pairs_before: u128 = 1 * PRICE_PRECISION;
        let total_shares_before: u128 = 1;
        let pairs_deposited: u128 = 1;

        let _ = shares_to_mint(
            &e,
            &contract_address,
            total_pairs_before,
            total_shares_before,
            pairs_deposited
        );
    }

    #[test]
    fn shares_to_mint_returns_zero_if_pairs_before_zero_but_shares_exist() {
        let e = Env::default();
        let (contract_address, _) = complete_test_setup(&e);

        e.as_contract(&contract_address, || {
            crate::storage::set_pair_balances(&e, &contract_address, &balances(0, 0, 0));
        });

        // Defensive branch: shares exist but pairs_before == 0
        let minted = shares_to_mint(&e, &contract_address, 0, 123, 10);
        assert_eq!(minted, 0);
    }

    // ---------- pairs_for_withdraw tests ----------

    #[test]
    fn pairs_for_withdraw_returns_zero_on_any_zero_input() {
        let e = Env::default();

        assert_eq!(pairs_for_withdraw(&e, 0, 100, 10), 0);
        assert_eq!(pairs_for_withdraw(&e, 100, 0, 10), 0);
        assert_eq!(pairs_for_withdraw(&e, 100, 100, 0), 0);

        // all zero
        assert_eq!(pairs_for_withdraw(&e, 0, 0, 0), 0);
    }

    #[test]
    fn pairs_for_withdraw_exact_pro_rata_case() {
        let e = Env::default();

        // Use values in PREC units to make the math land exactly.
        // total_pairs = 100 * PREC
        // total_shares = 200 * PREC
        // shares_in = 10 * PREC
        //
        // pairs_out = floor(shares_in * total_pairs / total_shares)
        //          = 10*PREC * 100*PREC / (200*PREC) = 5*PREC
        let total_pairs = 100 * PRICE_PRECISION;
        let total_shares = 200 * PRICE_PRECISION;
        let shares_in = 10 * PRICE_PRECISION;

        let out = pairs_for_withdraw(&e, total_pairs, total_shares, shares_in);
        assert_eq!(out, 5 * PRICE_PRECISION);
    }

    #[test]
    fn pairs_for_withdraw_uses_floor_rounding() {
        let e = Env::default();

        // total_pairs = 10 * PREC
        // total_shares = 3 * PREC
        // shares_in = 1 * PREC
        //
        // Expected: floor(10/3) * PREC = 3*PREC + remainder, but strictly < 4*PREC.
        let total_pairs = 10 * PRICE_PRECISION;
        let total_shares = 3 * PRICE_PRECISION;
        let shares_in = 1 * PRICE_PRECISION;

        let out = pairs_for_withdraw(&e, total_pairs, total_shares, shares_in);

        // Must be >= 3*PREC and < 4*PREC due to floor
        assert!(out >= 3 * PRICE_PRECISION);
        assert!(out < 4 * PRICE_PRECISION);
    }

    #[test]
    #[should_panic]
    fn pairs_for_withdraw_panics_on_dust_withdraw() {
        let e = Env::default();

        // Force pairs_out == 0 deterministically with your fixed-point helpers:
        //
        // num = floor(shares_in * total_pairs / PREC)
        // If shares_in=1 and total_pairs=1 => num = floor(1*1/PREC)=0
        // pairs_out = floor(num * PREC / total_shares) = 0
        // => should panic WithdrawTooSmall
        let total_pairs = 1;
        let total_shares = 1 * PRICE_PRECISION;
        let shares_in = 1;

        let _ = pairs_for_withdraw(&e, total_pairs, total_shares, shares_in);
    }
}
