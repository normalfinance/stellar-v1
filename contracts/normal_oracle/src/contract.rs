use crate::errors::NormalOracleError;
use crate::interface::NormalOracleTrait;
use crate::storage::GuardRails;
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::management::SingleAddressManagementTrait;
use access_control::role::Role;
use soroban_sdk::{contract, contractimpl, contractmeta, panic_with_error, Address, Env, Symbol};
use types::oracle::{OraclePriceData, OracleSource};

contractmeta!(key = "Description", val = "");

#[contract]
pub struct NormalOracle;

#[contractimpl]
impl NormalOracleTrait for NormalOracle {
    fn initialize(
        e: Env,
        admin: Address,
        asset: Symbol,
        oracle_source: OracleSource,
        oracle_addr: Address,
    ) {
        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, NormalOracleError::AlreadyInitialized);
        }
        access_control.set_role_address(&Role::Admin, &admin);

        crate::storage::put_asset(&e, asset);
        crate::storage::set_oracle_source(&e, &oracle_source);
        crate::storage::put_oracle(&e, oracle_addr);
    }

    fn get_price(e: Env) -> OraclePriceData {
        let now = e.ledger().timestamp();
        let asset = crate::storage::get_asset(&e);

        assert!(now > 0, "now timestamp must be positive");

        let oracle_addr = crate::storage::get_oracle(&e);
        let oracle_source = crate::storage::get_oracle_source(&e);

        match oracle_source {
            OracleSource::Reflector => {
                return crate::math::oracle::get_reflector_oracle_price(
                    &e,
                    &oracle_addr,
                    &asset,
                    now,
                );
            }
        }
    }

    fn set_seconds_before_stale(e: Env, admin: Address, stale_limit: u64) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_seconds_before_stale(&e, &stale_limit);
    }

    fn set_too_volatile_ratio(e: Env, admin: Address, too_volatile_ratio: u64) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_too_volatile_ratio(&e, &too_volatile_ratio);
    }

    fn get_guard_rails(e: Env) -> GuardRails {
        GuardRails {
            seconds_before_stale: crate::storage::get_seconds_before_stale(&e),
            too_volatile_ratio: crate::storage::get_too_volatile_ratio(&e),
        }
    }
}
