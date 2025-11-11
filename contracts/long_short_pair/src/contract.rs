use crate::errors::LongShortPairError;
use crate::events::{Events, LongShortPairEvents};
use crate::interface::{AdminInterfaceTrait, LongShortPairTrait, UpgradeableContract};
use crate::storage::set_calculator;
use crate::storage::set_oracle;
use crate::storage::{
    get_collateral_per_pair, get_expiry_percent_long, get_is_killed_create, get_is_killed_redeem,
    get_is_killed_settle, set_is_killed_create, set_is_killed_redeem, set_is_killed_settle,
};
use crate::storage::{
    get_token_long, get_token_short, put_token_collateral, put_token_long, put_token_short,
};
use crate::token::burn_token_short_from;
use crate::token::{
    burn_token_long, burn_token_short, mint_token_long, mint_token_short, transfer_token_collateral,
};
use crate::token::{burn_token_long_from, get_token_long_balance_of, get_token_short_balance_of};
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
    require_pause_admin_or_owner, require_pause_or_emergency_pause_admin_or_owner,
};
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, Address, BytesN, Env, Map, Symbol, Vec,
};
use upgrade::events::Events as UpgradeEvents;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "");

#[contract]
pub struct LongShortPair;

#[contractimpl]
impl LongShortPairTrait for LongShortPair {
    // Initializes the long short pair.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin user.
    // * `privileged_addrs` - (
    //      emergency admin,
    //      rewards admin,
    //      operations admin,
    //      pause admin,
    //      emergency pause admins
    //      system fee admin,
    //  ).
    // * `tokens` - The address of the long token, short token, and collateral.
    // * `oracle` - The address of the oracle.
    // * `calculator` - The address of the calculator.
    fn initialize(
        e: Env,
        admin: Address,
        privileged_addrs: (Address, Address, Address, Address, Vec<Address>, Address),
        tokens: Vec<Address>,
        oracle: Address,
        calculator: Address,
    ) {
        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, LongShortPairError::AlreadyInitialized);
        }
        access_control.set_role_address(&Role::Admin, &admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &privileged_addrs.0);
        access_control.set_role_address(&Role::RewardsAdmin, &privileged_addrs.1);
        access_control.set_role_address(&Role::OperationsAdmin, &privileged_addrs.2);
        access_control.set_role_address(&Role::PauseAdmin, &privileged_addrs.3);
        access_control.set_role_addresses(&Role::EmergencyPauseAdmin, &privileged_addrs.4);
        access_control.set_role_address(&Role::SystemFeeAdmin, &privileged_addrs.5);

        set_oracle(&e, &oracle);
        set_calculator(&e, &calculator);

        if tokens.len() != 3 {
            panic_with_error!(&e, LongShortPairError::WrongInputVecSize);
        }

        let token_long = tokens.get(0).unwrap();
        let token_short = tokens.get(1).unwrap();
        let token_collateral = tokens.get(2).unwrap();

        put_token_long(&e, token_long);
        put_token_short(&e, token_short);
        put_token_collateral(&e, token_collateral);
    }

    /**
     * @notice Creates a pair of long and short tokens equal in number to tokensToCreate. Pulls the required collateral
     * amount into this contract, defined by the collateralPerPair value.
     * @dev The caller must approve this contract to transfer `tokensToCreate * collateralPerPair` amount of collateral.
     * @param tokensToCreate number of long and short synthetic tokens to create.
     * @return collateralUsed total collateral used to mint the synthetics.
     */
    fn create(e: Env, user: Address, tokens_to_create: u128) -> u128 {
        user.require_auth();

        let current_time = e.ledger().timestamp();

        let collateral_used = tokens_to_create * get_collateral_per_pair(&e);

        transfer_token_collateral(&e, &user, &e.current_contract_address(), collateral_used);

        mint_token_long(&e, &user, &(tokens_to_create as i128));
        mint_token_short(&e, &user, &(tokens_to_create as i128));

        Events::new(&e).tokens_created(current_time, user, collateral_used, tokens_to_create);

        tokens_to_create
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

        let current_time = e.ledger().timestamp();

        burn_token_long_from(&e, &user, &(tokens_to_redeem as i128));
        burn_token_short_from(&e, &user, &(tokens_to_redeem as i128));

        let collateral_returned = tokens_to_redeem * get_collateral_per_pair(&e);

        transfer_token_collateral(
            &e,
            &user,
            &e.current_contract_address(),
            collateral_returned,
        );

        Events::new(&e).tokens_redeemed(current_time, user, collateral_returned, tokens_to_redeem);

        collateral_returned
    }

    /**
     * @notice Settle long and/or short tokens in for collateral at a rate informed by the contract settlement.
     * @dev Uses financialProductLibrary to compute the redemption rate between long and short tokens.
     * @dev This contract must have the `Burner` role for the `longToken` and `shortToken` in order to call `burnFrom`.
     * @dev The caller does not need to approve this contract to transfer any amount of `tokensToRedeem` since long
     * and short tokens are burned, rather than transferred, from the caller.
     * @dev This function can be called before or after expiration to facilitate early expiration. If a price has
     * not yet been resolved for either normal or early expiration yet then it will revert.
     * @param long_tokens_to_redeem number of long tokens to settle.
     * @param short_tokens_to_redeem number of short tokens to settle.
     * @return collateralReturned total collateral returned in exchange for the pair of synthetics.
     */
    fn settle(
        e: Env,
        user: Address,
        long_tokens_to_redeem: u128,
        short_tokens_to_redeem: u128,
    ) -> u128 {
        user.require_auth();

        let current_time = e.ledger().timestamp();

        burn_token_long(&e, &user, &(long_tokens_to_redeem as i128));
        burn_token_short(&e, &user, &(short_tokens_to_redeem as i128));

        let collateral_per_pair = get_collateral_per_pair(&e);
        let expiry_percent_long = get_expiry_percent_long(&e);

        // expiry_percent_long is a number between 0 and 1e18. 0 means all collateral goes to short tokens and 1e18 means
        // all collateral goes to the long token. Total collateral returned is the sum of payouts.
        let long_collateral_redeemed =
            long_tokens_to_redeem * collateral_per_pair * expiry_percent_long;
        let short_collateral_redeemed =
            short_tokens_to_redeem * collateral_per_pair * (1 - expiry_percent_long);

        let collateral_returned = long_collateral_redeemed + short_collateral_redeemed;
        transfer_token_collateral(
            &e,
            &e.current_contract_address(),
            &user,
            collateral_returned,
        );

        Events::new(&e).position_settled(
            current_time,
            user,
            collateral_returned,
            long_tokens_to_redeem,
            short_tokens_to_redeem,
        );

        collateral_returned
    }

    // fn sync(e: Env, user: Address) -> u128 {
    //     let (balance_long, balance_1) = (
    //         get_token_long_balance(&e, &e.current_contract_address()),
    //         get_balance_1(&e),
    //     );
    //     update(&e, balance_0, balance_1);
    // }

    // fn skim(e: Env, to: Address) -> u128 {
    //     let (balance_long, balance_1) = (
    //         get_token_long_balance(&e, &e.current_contract_address()),
    //         get_balance_1(&e),
    //     );
    //     let (reserve_0, reserve_1) = (get_reserve_0(&e), get_reserve_1(&e));
    //     let skimmed_0 = balance_0.checked_sub(reserve_0).unwrap();
    //     let skimmed_1 = balance_1.checked_sub(reserve_1).unwrap();
    //     transfer_token_0_from_pair(&e, &to, skimmed_0);
    //     transfer_token_1_from_pair(&e, &to, skimmed_1);
    //     event::skim(&e, skimmed_0, skimmed_1);
    // }

    fn get_tokens(e: Env) -> Vec<Address> {
        Vec::from_array(&e, [get_token_long(&e), get_token_short(&e)])
    }

    fn get_expiration_price(e: Env, user: Address) -> u128 {
        0
    }

    fn get_position_tokens(e: Env, user: Address) -> Vec<u128> {
        Vec::from_array(
            &e,
            [
                get_token_long_balance_of(&e, &user),
                get_token_short_balance_of(&e, &user),
            ],
        )
    }
}

#[contractimpl]
impl AdminInterfaceTrait for LongShortPair {
    // Sets the privileged addresses.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `rewards_admin` - The address of the rewards admin.
    // * `operations_admin` - The address of the operations admin.
    // * `pause_admin` - The address of the pause admin.
    // * `emergency_pause_admin` - The addresses of the emergency pause admins.
    // * `system_fee_admin` - The address of the system fee admin.
    fn set_privileged_addrs(
        e: Env,
        admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        pause_admin: Address,
        emergency_pause_admins: Vec<Address>,
        system_fee_admin: Address,
    ) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        access_control.set_role_address(&Role::RewardsAdmin, &rewards_admin);
        access_control.set_role_address(&Role::OperationsAdmin, &operations_admin);
        access_control.set_role_address(&Role::PauseAdmin, &pause_admin);
        access_control.set_role_addresses(&Role::EmergencyPauseAdmin, &emergency_pause_admins);
        access_control.set_role_address(&Role::SystemFeeAdmin, &system_fee_admin);
        AccessControlEvents::new(&e).set_privileged_addrs(
            rewards_admin,
            operations_admin,
            pause_admin,
            emergency_pause_admins,
            system_fee_admin,
        );
    }

    // Returns a map of privileged roles.
    //
    // # Returns
    //
    // A map of privileged roles to their respective addresses.
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>> {
        let access_control = AccessControl::new(&e);
        let mut result: Map<Symbol, Vec<Address>> = Map::new(&e);
        for role in [
            Role::Admin,
            Role::EmergencyAdmin,
            Role::RewardsAdmin,
            Role::OperationsAdmin,
            Role::PauseAdmin,
            Role::SystemFeeAdmin,
        ] {
            result.set(
                role.as_symbol(&e),
                match access_control.get_role_safe(&role) {
                    Some(v) => Vec::from_array(&e, [v]),
                    None => Vec::new(&e),
                },
            );
        }

        result.set(
            Role::EmergencyPauseAdmin.as_symbol(&e),
            access_control.get_role_addresses(&Role::EmergencyPauseAdmin),
        );

        result
    }

    fn set_oracle(e: Env, admin: Address, oracle: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_oracle(&e, &oracle);
    }

    // Stops the pool deposits instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_create(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_create(&e, &true);
        Events::new(&e).kill_create();
    }

    // Stops the pool swaps instantly.
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

    // Stops the pool claims instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_settle(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_settle(&e, &true);
        Events::new(&e).kill_settle();
    }

    // Resumes the pool deposits.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_create(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_create(&e, &false);
        Events::new(&e).unkill_create();
    }

    // Resumes the pool swaps.
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

    // Resumes the pool claims.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_settle(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_settle(&e, &false);
        Events::new(&e).unkill_settle();
    }

    // Get create killswitch status.
    fn get_is_killed_create(e: Env) -> bool {
        get_is_killed_create(&e)
    }

    // Get redeem killswitch status.
    fn get_is_killed_redeem(e: Env) -> bool {
        get_is_killed_redeem(&e)
    }

    // Get settle killswitch status.
    fn get_is_killed_settle(e: Env) -> bool {
        get_is_killed_settle(&e)
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
