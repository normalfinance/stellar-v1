#![no_std]

mod contract;
pub mod errors;
mod interface;
mod math;
mod state;
mod storage;
mod test;
mod testutils;

pub use crate::contract::{NormalOracle, NormalOracleClient};
