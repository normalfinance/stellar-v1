#![no_std]

pub mod token {
    soroban_sdk::contractimport!(file = "../../wasm/soroban_token_contract.wasm");
}
pub use token::{self as token_contract, Client};

pub mod storage;
pub mod utils;

pub use storage::*;
pub use utils::*;
