#![no_std]

mod contract;
pub mod errors;
mod interface;
mod test;
mod testutils;

pub use contract::{LongShortPairCalculator, LongShortPairCalculatorClient};
