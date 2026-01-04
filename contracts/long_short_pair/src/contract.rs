use crate::errors::LongShortPairError;
use crate::events::{Events, LongShortPairEvents};
use crate::interface::{
    AdminInterfaceTrait, LongShortPairTrait, OracleInterfaceTrait, UpgradeableContract,
};
use crate::storage::{
    get_calculator, get_collateral_per_pair, get_collateral_percent_long, get_is_killed_mint,
    get_is_killed_redeem, get_oracle, get_token_collateral, set_calculator,
    set_collateral_per_pair, set_collateral_percent_long, set_is_killed_mint, set_is_killed_redeem,
    set_oracle, CollateralInfo,
};
use crate::storage::{
    get_lower_bound, get_upper_bound, put_token_collateral, set_lower_bound, set_status,
    set_upper_bound,
};
use crate::token::create_contract;
use soroban_sdk::token::{
    StellarAssetClient as SorobanTokenAdminClient, TokenClient as SorobanTokenClient,
};

use access_control::access::{AccessControl, AccessControlTrait};
use access_control::emergency::{get_emergency_mode, set_emergency_mode};
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::{MultipleAddressesManagementTrait, SingleAddressManagementTrait};
use access_control::role::Role;
use access_control::role::SymbolRepresentation;
use access_control::transfer::TransferOwnershipTrait;
use access_control::utils::{
    require_operations_admin_or_owner, require_owner, require_pause_admin_or_owner,
    require_pause_or_emergency_pause_admin_or_owner,
};
use oracle::get_oracle_price;
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, symbol_short, Address, BytesN,
    Env, IntoVal, Map, Symbol, Vec,
};
use token_pair::{
    burn_long_tokens, burn_short_tokens, get_token_long, get_token_short, get_user_balance_long,
    get_user_balance_short, mint_long_tokens, mint_short_tokens, put_token_long, put_token_short,
    Client as PairTokenClient,
};
use types::pair::{PairParams, PairStatus};
use upgrade::events::Events as UpgradeEvents;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};
use utils::constant::{PRICE_PRECISION, PRICE_PRECISION_I128};
use utils::math::safe_math::{PrecisionMath, SafeMath};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "");

#[contract]
pub struct LongShortPair;

#[contractimpl]
impl LongShortPairTrait for LongShortPair {
    fn initialize(e: Env, params: PairParams) {
        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, LongShortPairError::AlreadyInitialized);
        }
        access_control.set_role_address(&Role::Admin, &params.admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &params.privileged_addrs.0);
        access_control.set_role_address(&Role::PauseAdmin, &params.privileged_addrs.1);

        put_token_collateral(&e, params.collateral_token.clone());

        // Init pair tokens
        let long_token_contract = create_contract(
            &e,
            params.pair_token_wasm_hash.clone(),
            &params.asset,
            &params.collateral_token,
            &symbol_short!("LONG"),
        );
        PairTokenClient::new(&e, &long_token_contract).initialize(
            &e.current_contract_address(),
            &7u32,
            &"Long Token".into_val(&e),
            &"LONG".into_val(&e),
        );

        let short_token_contract = create_contract(
            &e,
            params.pair_token_wasm_hash,
            &params.asset,
            &params.collateral_token,
            &symbol_short!("SHORT"),
        );
        PairTokenClient::new(&e, &short_token_contract).initialize(
            &e.current_contract_address(),
            &7u32,
            &"Short Token".into_val(&e),
            &"SHORT".into_val(&e),
        );

        put_token_long(&e, long_token_contract);
        put_token_short(&e, short_token_contract);

        // TODO: validate oracle

        set_oracle(&e, &params.oracle);
        set_calculator(&e, &params.pair_calculator);
        set_collateral_per_pair(&e, &params.collateral_per_pair);

        set_lower_bound(&e, &params.lower_bound);
        set_upper_bound(&e, &params.upper_bound);

        set_status(&e, &PairStatus::Active);
    }

    /**
     * @notice Creates a pair of long and short tokens equal in number to tokensToCreate. Pulls the required collateral
     * amount into this contract, defined by the collateralPerPair value.
     * @dev The caller must approve this contract to transfer `tokensToCreate * collateralPerPair` amount of collateral.
     * @param tokensToCreate number of long and short synthetic tokens to create.
     * @return collateralUsed total collateral used to mint the synthetics.
     */
    fn mint(e: Env, user: Address, tokens_to_mint: u128) -> u128 {
        user.require_auth();

        if tokens_to_mint <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        let current_time = e.ledger().timestamp();

        let collateral_used =
            tokens_to_mint.safe_fixed_mul_floor(&e, get_collateral_per_pair(&e), PRICE_PRECISION);

        log!(&e, "collateral_used", collateral_used);

        SorobanTokenClient::new(&e, &get_token_collateral(&e)).transfer(
            &user,
            &e.current_contract_address(),
            &(collateral_used as i128),
        );

        mint_long_tokens(&e, &user, tokens_to_mint as i128);
        mint_short_tokens(&e, &user, tokens_to_mint as i128);

        Events::new(&e).tokens_minted(current_time, user, collateral_used, tokens_to_mint);

        tokens_to_mint
    }

    /**
     * @notice Redeems a pair of long and short tokens equal in number to tokensToRedeem. Returns the commensurate
     * amount of collateral to the caller for the pair of tokens, defined by the collateralPerPair value.
     * @dev This contract must have the `Burner` role for the `longToken` and `shortToken` in order to call `burnFrom`.
     * @dev The caller does not need to approve this contract to transfer any amount of `tokensToRedeem` since long
     * and short tokens are burned, rather than transferred, from the caller.
     * @dev This method can be called either pre or post expiration.
     * @param tokensToRedeem number of long and short synthetic tokens to redeem.
     * @return collateralReturned total collateral returned in exchange for the pair of synthetics.
     */
    fn redeem(e: Env, user: Address, tokens_to_redeem: u128) -> u128 {
        user.require_auth();

        if tokens_to_redeem <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        let current_time = e.ledger().timestamp();

        // Get the oracle price and store it. Also sets collateral_percent_long to inform settlement. Reverts if either:
        // a) the price request has not resolved (either a normal expiration call or early expiration call) or b) If the
        // the contract was attempted to be settled early but the price returned is the ignore oracle price.
        // Note that we use the bool receivedSettlementPrice over checking for price != 0 as 0 is a valid price.
        // Self::sync_collateral_percent_long(e.clone());
        let oracle_price_data = crate::utils::get_oracle_price(&e, &get_oracle(&e));
        log!(&e, "price", oracle_price_data.price);

        crate::utils::sync_collateral_percent_long(&e, oracle_price_data);

        burn_long_tokens(&e, &user, tokens_to_redeem);
        burn_short_tokens(&e, &user, tokens_to_redeem);

        let collateral =
            tokens_to_redeem.safe_fixed_mul_floor(&e, get_collateral_per_pair(&e), PRICE_PRECISION);

        SorobanTokenClient::new(&e, &get_token_collateral(&e)).transfer(
            &e.current_contract_address(),
            &user,
            &(collateral as i128),
        );

        Events::new(&e).tokens_redeemed(
            current_time,
            user,
            collateral,
            collateral,
            tokens_to_redeem,
        );

        collateral
    }

    fn sync_collateral_percent_long(e: Env) {
        let oracle_price_data = crate::utils::get_oracle_price(&e, &get_oracle(&e));
        log!(&e, "price", oracle_price_data.price);

        let price_bounds = Self::get_price_bounds(e.clone());
        let lower_bound = price_bounds.get(0).unwrap();
        let upper_bound = price_bounds.get(1).unwrap();

        crate::utils::sync_collateral_percent_long(&e, oracle_price_data);
    }

    fn get_tokens(e: Env) -> Vec<Address> {
        Vec::from_array(&e, [get_token_long(&e), get_token_short(&e)])
    }

    fn get_price_bounds(e: Env) -> Vec<u128> {
        Vec::from_array(&e, [get_lower_bound(&e), get_upper_bound(&e)])
    }

    fn get_position_tokens(e: Env, user: Address) -> Vec<u128> {
        Vec::from_array(
            &e,
            [
                get_user_balance_long(&e, &user),
                get_user_balance_short(&e, &user),
            ],
        )
    }

    fn get_collateral_info(e: Env) -> CollateralInfo {
        CollateralInfo {
            collateral_per_pair: get_collateral_per_pair(&e),
            collateral_percent_long: get_collateral_percent_long(&e),
        }
    }
}

#[contractimpl]
impl AdminInterfaceTrait for LongShortPair {
    // Calculator
    fn set_calculator(e: Env, admin: Address, calculator: Address) {
        admin.require_auth();
        require_owner(&e, &admin);

        set_calculator(&e, &calculator);
    }

    // Sets the privileged addresses.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `pause_admin` - The address of the pause admin.
    // * `emergency_pause_admin` - The addresses of the emergency pause admins.
    fn set_privileged_addrs(e: Env, admin: Address, pause_admin: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        access_control.set_role_address(&Role::PauseAdmin, &pause_admin);
    }

    // Returns a map of privileged roles.
    //
    // # Returns
    //
    // A map of privileged roles to their respective addresses.
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>> {
        let access_control = AccessControl::new(&e);
        let mut result: Map<Symbol, Vec<Address>> = Map::new(&e);
        for role in [Role::Admin, Role::EmergencyAdmin, Role::PauseAdmin] {
            result.set(
                role.as_symbol(&e),
                match access_control.get_role_safe(&role) {
                    Some(v) => Vec::from_array(&e, [v]),
                    None => Vec::new(&e),
                },
            );
        }

        result
    }

    fn set_oracle(e: Env, admin: Address, oracle: Address) {
        admin.require_auth();
        require_owner(&e, &admin);

        set_oracle(&e, &oracle);
    }

    // Stops the pair mints instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_mint(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_mint(&e, &true);
        Events::new(&e).kill_mint();
    }

    // Stops the pair redemptions instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_redeem(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_redeem(&e, &true);
        Events::new(&e).kill_redeem();
    }

    // Resumes the pair mints.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_mint(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_mint(&e, &false);
        Events::new(&e).unkill_mint();
    }

    // Resumes the pair redemptions.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_redeem(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_redeem(&e, &false);
        Events::new(&e).unkill_redeem();
    }

    // Get create killswitch status.
    fn get_is_killed_mint(e: Env) -> bool {
        get_is_killed_mint(&e)
    }

    // Get redeem killswitch status.
    fn get_is_killed_redeem(e: Env) -> bool {
        get_is_killed_redeem(&e)
    }
}

#[contractimpl]
impl OracleInterfaceTrait for LongShortPair {
    fn get_price(e: Env) -> u128 {
        get_collateral_percent_long(&e) as u128
    }
}

// The `TransferableContract` trait provides the interface for transferring ownership of the contract.
#[contractimpl]
impl TransferableContract for LongShortPair {
    // Commits an ownership transfer.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `role_name` - The name of the role to transfer ownership of. The role must be one of the following:
    //     * `Admin`
    //     * `EmergencyAdmin`
    // * `new_address` - New address for the role
    fn commit_transfer_ownership(e: Env, admin: Address, role_name: Symbol, new_address: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        access_control.commit_transfer_ownership(&role, &new_address);
        AccessControlEvents::new(&e).commit_transfer_ownership(role, new_address);
    }

    // Applies the committed ownership transfer.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `role_name` - The name of the role to transfer ownership of. The role must be one of the following:
    //     * `Admin`
    //     * `EmergencyAdmin`
    fn apply_transfer_ownership(e: Env, admin: Address, role_name: Symbol) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        let new_address = access_control.apply_transfer_ownership(&role);
        AccessControlEvents::new(&e).apply_transfer_ownership(role, new_address);
    }

    // Reverts the committed ownership transfer.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `role_name` - The name of the role to transfer ownership of. The role must be one of the following:
    //     * `Admin`
    //     * `EmergencyAdmin`
    fn revert_transfer_ownership(e: Env, admin: Address, role_name: Symbol) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        access_control.revert_transfer_ownership(&role);
        AccessControlEvents::new(&e).revert_transfer_ownership(role);
    }

    // Returns the future address for the role.
    // The future address is the address that the ownership of the role will be transferred to.
    // The future address is set using the `commit_transfer_ownership` function.
    // The address will be defaulted to the current address if the transfer is not committed.
    //
    // # Arguments
    //
    // * `role_name` - The name of the role to get the future address for. The role must be one of the following:
    //    * `Admin`
    //    * `EmergencyAdmin`
    fn get_future_address(e: Env, role_name: Symbol) -> Address {
        let access_control = AccessControl::new(&e);
        let role = Role::from_symbol(&e, role_name);
        match access_control.get_transfer_ownership_deadline(&role) {
            0 => match access_control.get_role_safe(&role) {
                Some(address) => address,
                None => panic_with_error!(&e, AccessControlError::RoleNotFound),
            },
            _ => access_control.get_future_address(&role),
        }
    }
}

// The `UpgradeableContract` trait provides the interface for upgrading the contract.
#[contractimpl]
impl UpgradeableContract for LongShortPair {
    // Returns the version of the contract.
    //
    // # Returns
    //
    // The version of the contract as a u32.
    fn version() -> u32 {
        100
    }

    // Get contract type symbolic name
    fn contract_name(e: Env) -> Symbol {
        Symbol::new(&e, "LongShortPair")
    }

    // Commits a new wasm hash for a future upgrade.
    // The upgrade will be available through `apply_upgrade` after the standard upgrade delay
    // unless the system is in emergency mode.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `new_wasm_hash` - The new wasm hash to commit.
    fn commit_upgrade(e: Env, admin: Address, new_wasm_hash: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        commit_upgrade(&e, &new_wasm_hash);
        UpgradeEvents::new(&e).commit_upgrade(Vec::from_array(&e, [new_wasm_hash.clone()]));
    }

    // Applies the committed upgrade.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn apply_upgrade(e: Env, admin: Address) -> BytesN<32> {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        let new_wasm_hash = apply_upgrade(&e);
        UpgradeEvents::new(&e).apply_upgrade(Vec::from_array(&e, [new_wasm_hash.clone()]));
        new_wasm_hash
    }

    // Reverts the committed upgrade.
    // This can be used to cancel a previously committed upgrade.
    // The upgrade will be canceled only if it has not been applied yet.
    // If the upgrade has already been applied, it cannot be reverted.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn revert_upgrade(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        revert_upgrade(&e);
        UpgradeEvents::new(&e).revert_upgrade();
    }

    // Sets the emergency mode.
    // When the emergency mode is set to true, the contract will allow instant upgrades without the delay.
    // This is useful in case of critical issues that need to be fixed immediately.
    // When the emergency mode is set to false, the contract will require the standard upgrade delay.
    // The emergency mode can only be set by the emergency admin.
    //
    // # Arguments
    //
    // * `emergency_admin` - The address of the emergency admin.
    // * `value` - The value to set the emergency mode to.
    fn set_emergency_mode(e: Env, emergency_admin: Address, value: bool) {
        emergency_admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&emergency_admin, &Role::EmergencyAdmin);
        set_emergency_mode(&e, &value);
        AccessControlEvents::new(&e).set_emergency_mode(value);
    }

    // Returns the emergency mode flag value.
    fn get_emergency_mode(e: Env) -> bool {
        get_emergency_mode(&e)
    }
}
