use crate::interface::NormalOracleTrait;
use crate::math::oracle::get_oracle_price;
use crate::storage::{
    get_asset, get_seconds_before_stale, get_too_volatile_ratio, put_asset, put_oracle,
    set_oracle_source, set_seconds_before_stale, set_too_volatile_ratio, GuardRails,
};
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::management::SingleAddressManagementTrait;
use access_control::role::Role;
use oracle::state::OraclePriceData;
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};
use types::oracle::OracleSource;

#[contract]
pub struct NormalOracle;

#[contractimpl]
impl NormalOracle {
    // __constructor
    // Initializes the factory by setting the admin roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: ...
    //   - asset: The address of the swap token_factory contract.
    //   - oracle_source: ...
    //   - oracle_addr: The WASM hash (BytesN<32>) for the long short pair contract.
    pub fn __constructor(
        e: Env,
        admin: Address,
        asset: Symbol,
        oracle_source: OracleSource,
        oracle_addr: Address,
    ) {
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &admin);

        put_asset(&e, asset);
        set_oracle_source(&e, &oracle_source);
        put_oracle(&e, oracle_addr);
    }
}

#[contractimpl]
impl NormalOracleTrait for NormalOracle {
    fn get_price(e: Env) -> OraclePriceData {
        get_oracle_price(&e, &get_asset(&e), e.ledger().timestamp())
    }

    fn set_seconds_before_stale(e: Env, admin: Address, stale_limit: u64) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        set_seconds_before_stale(&e, &stale_limit);
    }

    fn set_too_volatile_ratio(e: Env, admin: Address, too_volatile_ratio: u64) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        set_too_volatile_ratio(&e, &too_volatile_ratio);
    }

    fn get_guard_rails(e: Env) -> GuardRails {
        GuardRails {
            seconds_before_stale: get_seconds_before_stale(&e),
            too_volatile_ratio: get_too_volatile_ratio(&e),
        }
    }
}
