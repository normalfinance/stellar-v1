#![no_std]

use paste::paste;
use soroban_sdk::{panic_with_error, Address, Env, Symbol};
pub use utils::bump::bump_instance;
use utils::errors::storage_errors::StorageError;
use utils::{
    generate_instance_storage_getter, generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

/********** Storage Key Types **********/

const KEY_TOKEN_LONG: &str = "TokenLong";
const KEY_TOKEN_SHORT: &str = "TokenShort";
const KEY_TOTAL_LONG_SHARES: &str = "TotalLongShares";
const KEY_TOTAL_SHORT_SHARES: &str = "TotalShortShares";

/********** Storage **********/

generate_instance_storage_getter_and_setter!(token_long, KEY_TOKEN_LONG, Address);
generate_instance_storage_getter_and_setter_with_default!(
    total_long_shares,
    KEY_TOTAL_LONG_SHARES,
    u128,
    0
);

generate_instance_storage_getter_and_setter!(token_short, KEY_TOKEN_SHORT, Address);
generate_instance_storage_getter_and_setter_with_default!(
    total_short_shares,
    KEY_TOTAL_SHORT_SHARES,
    u128,
    0
);
