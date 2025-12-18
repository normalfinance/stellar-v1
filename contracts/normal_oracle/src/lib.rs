#![no_std]

mod contract;
pub mod errors;
mod interface;
mod math;
mod storage;
mod test;
mod test_permissions;
mod testutils;

pub use crate::contract::{NormalOracle, NormalOracleClient};
