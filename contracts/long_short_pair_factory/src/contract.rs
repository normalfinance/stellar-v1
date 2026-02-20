use crate::errors::LongShortPairFactoryError;
use crate::events::{Events, FactoryConfigEvents, FactoryEvents};
use crate::factory_interface::{AdminInterface, LongShortPairFactoryTrait};
use crate::pair_interface::PairInterfaceTrait;
use access_control::utils::require_operations_admin_or_owner;
use soroban_sdk::{
    contract, contractimpl, contractmeta, contracttype, panic_with_error, Address, BytesN, Env,
    Map, Symbol, Vec,
};
use soroban_sdk::{symbol_short, IntoVal};
use types::pair::PairParams;

// Access control
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::emergency::{get_emergency_mode, set_emergency_mode};
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::{Role, SymbolRepresentation};
use access_control::transfer::TransferOwnershipTrait;

// Upgrade
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};

contractmeta!(
    key = "Description",
    val = "Factory for creating and interacting with Long Short Pair contracts"
);

#[contract]
pub struct LongShortPairFactory;

// Factory configuration struct for query methods
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactoryConfig {
    pub pair_contract_wasm: BytesN<32>,
}

#[contractimpl]
impl LongShortPairFactory {
    // __constructor
    // Initializes the factory by setting the access roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The address to be assigned the Admin role.
    //   - rewards_admin - The address of the rewards admin.
    //   - operations_admin - The address of the operations admin.
    //   - system_fee_admin - The address of the system fee admin.
    //   - pair_contract_wasm: The WASM hash (BytesN<32>) for the long short pair contract.
    pub fn __constructor(
        e: Env,
        admin: Address,
        emergency_admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        system_fee_admin: Address,
        pair_contract_wasm: BytesN<32>,
    ) {
        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, LongShortPairFactoryError::AlreadyInitialized);
        }

        access_control.set_role_address(&Role::Admin, &admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &emergency_admin);
        access_control.set_role_address(&Role::OperationsAdmin, &operations_admin);
        access_control.set_role_address(&Role::RewardsAdmin, &rewards_admin);
        access_control.set_role_address(&Role::SystemFeeAdmin, &system_fee_admin);

        crate::storage::set_pair_contract_wasm(&e, &pair_contract_wasm);
    }
}

#[contractimpl]
impl LongShortPairFactoryTrait for LongShortPairFactory {
    /// @notice Creates a Long Short Pair contract.
    /// @param params Constructor params used to initialize the LSP:
    ///     - `admin`: Address of the Factory admin.
    ///     - `params`: Config parameters of the Long Short Pair contract.
    /// @return pair_address the deployed address of the new Long Short Pair contract.
    fn deploy_pair_contract(e: Env, admin: Address, params: PairParams) -> Address {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        if get_emergency_mode(&e) {
            panic_with_error!(&e, LongShortPairFactoryError::ActionPaused);
        }

        // Deploy the pair contract
        let salt = crate::pair_utils::get_pair_salt(&e, &params.asset);

        let pair_address = e
            .deployer()
            .with_current_contract(salt.clone())
            .deploy_v2(crate::storage::get_pair_contract_wasm(&e), (params,));

        // Add to pair registry
        crate::storage::add_deployed_pair(&e, &pair_address);
        crate::storage::put_pair(&e, salt, &pair_address);

        Events::new(&e).pair_deployed(e.ledger().timestamp(), admin.clone(), pair_address.clone());

        pair_address
    }
}

#[contractimpl]
impl PairInterfaceTrait for LongShortPairFactory {
    fn mint(e: Env, user: Address, asset: Symbol, tokens_to_mint: u128) -> u128 {
        user.require_auth();

        let salt = crate::pair_utils::get_pair_salt(&e, &asset);
        let pair_address = crate::storage::get_pair(&e, salt);

        let minted_tokens: u128 = e.invoke_contract(
            &pair_address,
            &symbol_short!("mint"),
            Vec::from_array(&e, [user.clone().into_val(&e), tokens_to_mint.into_val(&e)]),
        );

        Events::new(&e).mint(
            user,
            asset,
            pair_address,
            0,
            tokens_to_mint,
            e.ledger().timestamp(),
        );

        minted_tokens
    }

    fn redeem(e: Env, user: Address, asset: Symbol, tokens_to_redeem: u128) -> u128 {
        user.require_auth();

        let salt = crate::pair_utils::get_pair_salt(&e, &asset);
        let pair_address = crate::storage::get_pair(&e, salt);

        let collateral: u128 = e.invoke_contract(
            &pair_address,
            &symbol_short!("redeem"),
            Vec::from_array(
                &e,
                [user.clone().into_val(&e), tokens_to_redeem.into_val(&e)],
            ),
        );

        Events::new(&e).redeem(
            user,
            asset,
            pair_address,
            collateral,
            e.ledger().timestamp(),
        );

        collateral
    }

    fn redeem_one(
        e: Env,
        user: Address,
        asset: Symbol,
        token: Address,
        tokens_to_redeem: u128,
    ) -> u128 {
        user.require_auth();

        let salt = crate::pair_utils::get_pair_salt(&e, &asset);
        let pair_address = crate::storage::get_pair(&e, salt);

        let collateral: u128 = e.invoke_contract(
            &pair_address,
            &Symbol::new(&e, "redeem_one"),
            Vec::from_array(
                &e,
                [
                    user.clone().into_val(&e),
                    token.into_val(&e),
                    tokens_to_redeem.into_val(&e),
                ],
            ),
        );

        Events::new(&e).redeem_one(
            user,
            asset,
            pair_address,
            token,
            tokens_to_redeem,
            e.ledger().timestamp(),
        );

        collateral
    }
}

#[contractimpl]
impl AdminInterface for LongShortPairFactory {
    // Sets the privileged addresses.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    // * `rewards_admin` - The address of the rewards admin.
    // * `operations_admin` - The address of the operations admin.
    // * `system_fee_admin` - The address of the system fee admin.
    fn set_privileged_addrs(
        e: Env,
        admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        system_fee_admin: Address,
    ) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        access_control.set_role_address(&Role::RewardsAdmin, &rewards_admin);
        access_control.set_role_address(&Role::OperationsAdmin, &operations_admin);
        access_control.set_role_address(&Role::SystemFeeAdmin, &system_fee_admin);
        AccessControlEvents::new(&e).set_factory_privileged_addrs(
            rewards_admin,
            operations_admin,
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
        ] {
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

    fn get_factory_config(e: Env) -> FactoryConfig {
        FactoryConfig {
            pair_contract_wasm: crate::storage::get_pair_contract_wasm(&e),
        }
    }

    fn get_pair_contract_wasm(e: Env) -> BytesN<32> {
        crate::storage::get_pair_contract_wasm(&e)
    }

    fn get_all_deployed_pairs(e: Env) -> Vec<Address> {
        crate::storage::get_all_deployed_pairs(&e)
    }

    fn get_total_pair_count(e: Env) -> u32 {
        let all_pairs = crate::storage::get_all_deployed_pairs(&e);
        all_pairs.len()
    }

    fn get_pair_by_asset(e: Env, asset: Symbol) -> Address {
        let salt = crate::pair_utils::get_pair_salt(&e, &asset);
        crate::storage::get_pair(&e, salt)
    }

    // set_pair_contract_wasm
    // Updates the WASM hash for the long short pair contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - pair_contract_wasm: The new WASM hash (BytesN<32>) for the swap fee contract.
    fn set_pair_contract_wasm(e: Env, admin: Address, pair_contract_wasm: BytesN<32>) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        let old_wasm = crate::storage::get_pair_contract_wasm(&e);
        crate::storage::set_pair_contract_wasm(&e, &pair_contract_wasm);

        let current_time = e.ledger().timestamp();
        Events::new(&e).pair_wasm_updated(
            current_time,
            admin.clone(),
            old_wasm.clone(),
            pair_contract_wasm.clone(),
            1,
        );
    }
}

#[contractimpl]
impl UpgradeableContract for LongShortPairFactory {
    // version
    // Returns the current version number of the contract.
    //
    // Returns:
    //   - A u32 representing the version.
    fn version() -> u32 {
        100
    }

    // Get contract type symbolic name
    fn contract_name(e: Env) -> Symbol {
        Symbol::new(&e, "LongShortPairFactory")
    }

    // commit_upgrade
    // Commits a new WASM hash as a pending upgrade.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - new_wasm_hash: The new WASM hash (BytesN<32>) to be committed.
    fn commit_upgrade(e: Env, admin: Address, new_wasm_hash: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        commit_upgrade(&e, &new_wasm_hash);
        UpgradeEvents::new(&e).commit_upgrade(Vec::from_array(&e, [new_wasm_hash.clone()]));
    }

    // apply_upgrade
    // Applies the previously committed upgrade.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //
    // Returns:
    //   - The new WASM hash (BytesN<32>) that was applied.
    fn apply_upgrade(e: Env, admin: Address) -> BytesN<32> {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        let new_wasm_hash = apply_upgrade(&e);
        UpgradeEvents::new(&e).apply_upgrade(Vec::from_array(&e, [new_wasm_hash.clone()]));
        new_wasm_hash
    }

    // revert_upgrade
    // Reverts a pending upgrade that has not yet been applied.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    fn revert_upgrade(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);
        revert_upgrade(&e);
        UpgradeEvents::new(&e).revert_upgrade();
    }

    // set_emergency_mode
    // Sets or unsets emergency mode for instant upgrades.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - emergency_admin: The emergency admin address (must be authorized).
    //   - value: Boolean indicating whether to enable (true) or disable (false) emergency mode.
    fn set_emergency_mode(e: Env, emergency_admin: Address, value: bool) {
        emergency_admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&emergency_admin, &Role::EmergencyAdmin);
        set_emergency_mode(&e, &value);

        let current_time = e.ledger().timestamp();
        // Emit factory pause/unpause events based on emergency mode
        if value {
            Events::new(&e).factory_paused(current_time, emergency_admin.clone());
        } else {
            Events::new(&e).factory_unpaused(current_time, emergency_admin.clone());
        }

        AccessControlEvents::new(&e).set_emergency_mode(value);
    }

    // get_emergency_mode
    // Returns the current emergency mode state.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //
    // Returns:
    //   - A boolean indicating whether emergency mode is active.
    fn get_emergency_mode(e: Env) -> bool {
        get_emergency_mode(&e)
    }
}

// The `TransferableContract` trait provides the interface for transferring ownership of the contract.
#[contractimpl]
impl TransferableContract for LongShortPairFactory {
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
