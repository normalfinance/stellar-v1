use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{panic_with_error, Env};
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::{PrecisionMath, SafeConversion, SafeMath};

use crate::errors::LiquidityPoolError;
use crate::storage::{get_is_killed_tax, get_tax_rate_table};

pub fn calculate_tax_rate(e: &Env, pool_price: u128, peg_price: u128) -> u32 {
    // Guard against zero division
    if peg_price == 0 || pool_price == 0 {
        return 0;
    }

    // Calculate absolute deviation: |pool_price - peg_price| / peg_price
    let price_diff = if pool_price > peg_price {
        pool_price.safe_sub(e, peg_price)
    } else {
        peg_price.safe_sub(e, pool_price)
    };

    // abs_deviation = |price_diff| / peg_price (scaled by PRICE_PRECISION)
    let abs_deviation = price_diff.safe_fixed_div_round(e, peg_price, PRICE_PRECISION);

    // Check if we have a configured rate table
    let rate_table = get_tax_rate_table(e);

    if rate_table.len() <= 0 {
        panic_with_error!(e, LiquidityPoolError::NoTaxTableFound);
    }

    // Use table lookup with step function
    // Find the entry with the highest deviation <= actual deviation
    let mut best_rate = 0_u32;
    let mut found = false;

    for i in 0..rate_table.len() {
        let entry = rate_table.get(i).unwrap();
        if entry.deviation <= abs_deviation {
            best_rate = entry.rate;
            found = true;
        } else {
            break; // Table should be sorted, so we can stop here
        }
    }

    // If deviation is below all table points, return 0
    // If deviation matches or exceeds a point, return that rate
    if found {
        return best_rate;
    } else {
        return 0;
    }
}

// pub fn calculate_price_spread_pct(e: &Env, price_a: u128, price_b: u128) -> i64 {
//     let price_deviation = calculate_price_deviation(e, price_a, price_b);

//     // Safe conversion to i64 with overflow protection
//     let price_spread = price_deviation.safe_to_i64(e);
//     let price_a_i64 = price_a.safe_to_i64(e);

//     // Calculate (price_spread * PRICE_PRECISION_I64) / price_a_i64 using safe arithmetic
//     let numerator = price_spread.safe_mul(e, PRICE_PRECISION_I64);
//     numerator.safe_div(e, price_a_i64)
// }

/// Calculates the tax amount to be collected from a trade.
/// Returns the tax amount in token units.
pub fn calculate_tax_amount(e: &Env, pool_price: u128, peg_price: u128, out_amount: u128) -> u128 {
    if get_is_killed_tax(&e) {
        return 0;
    }

    if pool_price == 0 || peg_price == 0 || out_amount == 0 {
        return 0;
    }

    let tax_rate = calculate_tax_rate(e, pool_price, peg_price);

    // Apply tax rate to trade amount: out_amount * tax_rate / PRICE_PRECISION
    out_amount.fixed_mul_floor(e, &(tax_rate as u128), &PRICE_PRECISION)
}

#[cfg(test)]
mod test {
    use soroban_sdk::Vec;

    use super::*;
    use crate::{
        storage::{set_tax_rate_table, RateTableEntry},
        LiquidityPoolSynthetic,
    };

    fn test_env_with_contract() -> (Env, soroban_sdk::Address) {
        let e = Env::default();
        e.mock_all_auths();
        let contract_id = e.register_contract(None, LiquidityPoolSynthetic);
        (e, contract_id)
    }

    #[test]
    fn test_calculate_tax_rate_zero_prices() {
        let (e, contract_id) = test_env_with_contract();

        let (tax_rate_0, tax_rate_1, tax_rate_2) = e.as_contract(&contract_id, || {
            let tax_rate_0 = calculate_tax_rate(&e, 0, 0);
            let tax_rate_1 = calculate_tax_rate(&e, 1_0000000, 0);
            let tax_rate_2 = calculate_tax_rate(&e, 0, 1_0000000);
            (tax_rate_0, tax_rate_1, tax_rate_2)
        });

        assert_eq!(tax_rate_0, 0);
        assert_eq!(tax_rate_1, 0);
        assert_eq!(tax_rate_2, 0);
    }

    #[test]
    fn test_calculate_tax_rate_at_peg() {
        let (e, contract_id) = test_env_with_contract();

        let tax_rate = e.as_contract(&contract_id, || {
            // Configure a simple table
            let mut table = Vec::new(&e);
            // 0% deviation -> 100 (0.10%)
            table.push_back(RateTableEntry {
                deviation: 0,
                rate: 100,
            });
            // 2% deviation (0.02 * PRICE_PRECISION) -> 1100 (1.1%)
            table.push_back(RateTableEntry {
                deviation: 200000,
                rate: 1100,
            });
            // 5% deviation -> 5500 (5.5%)
            table.push_back(RateTableEntry {
                deviation: 500000,
                rate: 5500,
            });
            // 10% deviation -> 27000 (27%)
            table.push_back(RateTableEntry {
                deviation: 1000000,
                rate: 27000,
            });

            set_tax_rate_table(&e, &table);

            let tax_rate = calculate_tax_rate(&e, 1_0000000, 1_0000000);
            tax_rate
        });

        assert_eq!(tax_rate, 0);
    }

    #[test]
    fn test_calculate_tax_rate_small_deviation() {
        let (e, contract_id) = test_env_with_contract();

        let tax_rate = e.as_contract(&contract_id, || {
            // Configure a simple table
            let mut table = Vec::new(&e);
            // 0% deviation -> 100 (0.10%)
            table.push_back(RateTableEntry {
                deviation: 0,
                rate: 100,
            });
            // 2% deviation (0.02 * PRICE_PRECISION) -> 1100 (1.1%)
            table.push_back(RateTableEntry {
                deviation: 200000,
                rate: 1100,
            });
            // 5% deviation -> 5500 (5.5%)
            table.push_back(RateTableEntry {
                deviation: 500000,
                rate: 5500,
            });
            // 10% deviation -> 27000 (27%)
            table.push_back(RateTableEntry {
                deviation: 1000000,
                rate: 27000,
            });

            set_tax_rate_table(&e, &table);

            let tax_rate = calculate_tax_rate(&e, 1_0100000, 1_0000000);
            tax_rate
        });

        // With small deviation (1%), tax rate should be close to base rate
        // The actual increase depends on the tax curve parameters
        assert!(tax_rate > 0);
        assert!(tax_rate < 10000); // Should be reasonable
    }

    #[test]
    fn test_calculate_tax_rate_large_deviation() {
        let (e, contract_id) = test_env_with_contract();

        let tax_rate = e.as_contract(&contract_id, || {
            // Configure a simple table
            let mut table = Vec::new(&e);
            // 0% deviation -> 100 (0.10%)
            table.push_back(RateTableEntry {
                deviation: 0,
                rate: 100,
            });
            // 2% deviation (0.02 * PRICE_PRECISION) -> 1100 (1.1%)
            table.push_back(RateTableEntry {
                deviation: 200000,
                rate: 1100,
            });
            // 5% deviation -> 5500 (5.5%)
            table.push_back(RateTableEntry {
                deviation: 500000,
                rate: 5500,
            });
            // 10% deviation -> 27000 (27%)
            table.push_back(RateTableEntry {
                deviation: 1000000,
                rate: 27000,
            });

            set_tax_rate_table(&e, &table);

            let tax_rate = calculate_tax_rate(&e, 1_2000000, 1_0000000);
            tax_rate
        });

        // Verify tax rate is within valid bounds
        assert!(tax_rate > 0);
    }

    #[test]
    fn test_calculate_tax_amount_zero_prices() {
        let (e, contract_id) = test_env_with_contract();
        let trade_amount = 100_0000000;

        let (tax_amount_0, tax_amount_1, tax_amount_2) = e.as_contract(&contract_id, || {
            let tax_amount_0 = calculate_tax_amount(&e, trade_amount, 0, 0);
            let tax_amount_1 = calculate_tax_amount(&e, trade_amount, 1_0000000, 0);
            let tax_amount_2 = calculate_tax_amount(&e, trade_amount, 0, 1_0000000);
            (tax_amount_0, tax_amount_1, tax_amount_2)
        });

        assert_eq!(tax_amount_0, 0);
        assert_eq!(tax_amount_1, 0);
        assert_eq!(tax_amount_2, 0);
    }

    #[test]
    fn test_calculate_tax_amount_at_peg() {
        let (e, contract_id) = test_env_with_contract();
        let trade_amount = 100_0000000;

        let tax_amount = e.as_contract(&contract_id, || {
            // Configure a simple table
            let mut table = Vec::new(&e);
            // 0% deviation -> 100 (0.10%)
            table.push_back(RateTableEntry {
                deviation: 0,
                rate: 100,
            });
            // 2% deviation (0.02 * PRICE_PRECISION) -> 1100 (1.1%)
            table.push_back(RateTableEntry {
                deviation: 200000,
                rate: 1100,
            });
            // 5% deviation -> 5500 (5.5%)
            table.push_back(RateTableEntry {
                deviation: 500000,
                rate: 5500,
            });
            // 10% deviation -> 27000 (27%)
            table.push_back(RateTableEntry {
                deviation: 1000000,
                rate: 27000,
            });

            set_tax_rate_table(&e, &table);

            calculate_tax_amount(&e, trade_amount, 1_0000000, 1_0000000)
        });

        // At peg, only base tax applies
        // With base_tax_fraction = 100 and PRICE_PRECISION = 10_000_000:
        // 100_0000000 * 100 / 10_000_000 = 10000
        assert_eq!(tax_amount, 10000);
    }

    #[test]
    fn test_calculate_tax_amount_with_deviation() {
        let (e, contract_id) = test_env_with_contract();
        let trade_amount = 100_0000000;

        let (base_tax_amount, tax_amount_with_dev) = e.as_contract(&contract_id, || {
            let base_tax_amount = calculate_tax_amount(&e, trade_amount, 1_0000000, 1_0000000);
            // 10% deviation
            let tax_amount_with_dev = calculate_tax_amount(&e, trade_amount, 1_1000000, 1_0000000);
            (base_tax_amount, tax_amount_with_dev)
        });

        // Verify tax amount is within valid bounds (at least base tax, less than half of trade)
        assert!(tax_amount_with_dev >= base_tax_amount);
        assert!(tax_amount_with_dev < trade_amount / 2); // Less than 50% of trade amount
    }

    #[test]
    fn test_calculate_tax_rate_with_empty_table() {
        // Test that empty table falls back to exponential formula
        let (e, contract_id) = test_env_with_contract();

        let (table_len, tax_rate) = e.as_contract(&contract_id, || {
            // Verify table is empty
            let table = crate::storage::get_tax_rate_table(&e);

            // Calculate with 5% deviation - should use exponential formula
            let tax_rate = calculate_tax_rate(&e, 1_0500000, 1_0000000);

            (table.len(), tax_rate)
        });

        // Should use exponential formula when table is empty
        assert_eq!(table_len, 0);
        // Should return a valid tax rate (>= base tax of 100)
        assert!(tax_rate >= 100);
    }

    #[test]
    fn test_calculate_tax_rate_with_configured_table() {
        use crate::storage::{set_tax_rate_table, RateTableEntry};
        use soroban_sdk::Vec;

        let (e, contract_id) = test_env_with_contract();

        let tax_rate_2pct = e.as_contract(&contract_id, || {
            // Configure a simple table
            let mut table = Vec::new(&e);
            // 0% deviation -> 100 (0.10%)
            table.push_back(RateTableEntry {
                deviation: 0,
                rate: 100,
            });
            // 2% deviation (0.02 * PRICE_PRECISION) -> 1100 (1.1%)
            table.push_back(RateTableEntry {
                deviation: 200000,
                rate: 1100,
            });
            // 5% deviation -> 5500 (5.5%)
            table.push_back(RateTableEntry {
                deviation: 500000,
                rate: 5500,
            });
            // 10% deviation -> 27000 (27%)
            table.push_back(RateTableEntry {
                deviation: 1000000,
                rate: 27000,
            });

            set_tax_rate_table(&e, &table);

            // Test exact match at 2%
            calculate_tax_rate(&e, 1_0200000, 1_0000000)
        });

        // Should return exact rate for 2% deviation
        assert_eq!(tax_rate_2pct, 1100);
    }

    #[test]
    fn test_calculate_tax_rate_step_function() {
        use crate::storage::{set_tax_rate_table, RateTableEntry};
        use soroban_sdk::Vec;

        let (e, contract_id) = test_env_with_contract();

        let (rate_1_5pct, rate_3pct, rate_between) = e.as_contract(&contract_id, || {
            // Configure table
            let mut table = Vec::new(&e);
            table.push_back(RateTableEntry {
                deviation: 0,
                rate: 100,
            });
            table.push_back(RateTableEntry {
                deviation: 200000,
                rate: 1100,
            }); // 2%
            table.push_back(RateTableEntry {
                deviation: 500000,
                rate: 5500,
            }); // 5%

            set_tax_rate_table(&e, &table);

            // Test step function: 1.5% deviation (between 0 and 2%)
            let rate_1_5pct = calculate_tax_rate(&e, 1_0150000, 1_0000000);

            // Test 3% deviation (between 2% and 5%)
            let rate_3pct = calculate_tax_rate(&e, 1_0300000, 1_0000000);

            // Test deviation between table points uses nearest lower
            let rate_between = calculate_tax_rate(&e, 1_0250000, 1_0000000);

            (rate_1_5pct, rate_3pct, rate_between)
        });

        // 1.5% should use rate for 0% (step function)
        assert_eq!(rate_1_5pct, 100);

        // 3% should use rate for 2% (nearest lower point)
        assert_eq!(rate_3pct, 1100);

        // 2.5% should also use rate for 2%
        assert_eq!(rate_between, 1100);
    }

    #[test]
    fn test_calculate_tax_rate_above_all_table_entries() {
        use crate::storage::{set_tax_rate_table, RateTableEntry};
        use soroban_sdk::Vec;

        let (e, contract_id) = test_env_with_contract();

        let rate_high = e.as_contract(&contract_id, || {
            // Configure table with max at 10%
            let mut table = Vec::new(&e);
            table.push_back(RateTableEntry {
                deviation: 0,
                rate: 100,
            });
            table.push_back(RateTableEntry {
                deviation: 1000000,
                rate: 27000,
            }); // 10%

            set_tax_rate_table(&e, &table);

            // Test 20% deviation (above all entries)
            calculate_tax_rate(&e, 1_2000000, 1_0000000)
        });

        // Should return the highest rate in table
        assert_eq!(rate_high, 27000);
    }

    #[test]
    fn test_calculate_tax_rate_below_all_table_entries() {
        use crate::storage::{set_tax_rate_table, RateTableEntry};
        use soroban_sdk::Vec;

        let (e, contract_id) = test_env_with_contract();

        let rate_low = e.as_contract(&contract_id, || {
            // Configure table starting at 2%
            let mut table = Vec::new(&e);
            table.push_back(RateTableEntry {
                deviation: 200000,
                rate: 1100,
            }); // 2%
            table.push_back(RateTableEntry {
                deviation: 500000,
                rate: 5500,
            }); // 5%

            set_tax_rate_table(&e, &table);

            // Test 1% deviation (below all entries)
            calculate_tax_rate(&e, 1_0100000, 1_0000000)
        });

        // Should return 0 when below all table points
        assert_eq!(rate_low, 0);
    }
}
