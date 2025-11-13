mod liquidity_calculator_client {
    soroban_sdk::contractimport!(file = "../../wasm/liquidity_pool_liquidity_calculator.wasm");
}

pub use crate::liquidity_calculator::liquidity_calculator_client::Client as LiquidityCalculatorClient;
