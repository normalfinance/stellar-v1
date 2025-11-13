use crate::events::{Events, FactoryConfigEvents, FactoryEvents};
use crate::interface::{AdminInterface, LongShortPairFactoryTrait};
use crate::pair_utils::get_pair_salt;
use crate::storage::get_is_killed_create;
use crate::storage::get_lsp_contract_wasm;
use crate::storage::get_token_factory;
use crate::storage::set_is_killed_create;
use crate::storage::set_lsp_contract_wasm;
use crate::storage::set_token_factory;
use crate::storage::{
    add_deployed_pair, get_all_deployed_pairs, get_contract_sequence, get_deployed_pairs,
    set_contract_sequence,
};
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::emergency::{get_emergency_mode, set_emergency_mode};
use access_control::errors::AccessControlError;
use access_control::events::Events as AccessControlEvents;
use access_control::interface::TransferableContract;
use access_control::management::SingleAddressManagementTrait;
use access_control::role::{Role, SymbolRepresentation};
use access_control::transfer::TransferOwnershipTrait;
use access_control::utils::require_pause_or_emergency_pause_admin_or_owner;
use soroban_sdk::token::StellarAssetClient as SorobanTokenAdminClient;
use soroban_sdk::Bytes;
use soroban_sdk::IntoVal;
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, BytesN, Env, Symbol, Vec,
};
use upgrade::events::Events as UpgradeEvents;
use upgrade::interface::UpgradeableContract;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};

#[contract]
pub struct LongShortPairFactory;

// Factory configuration struct for query methods
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactoryConfig {
    pub token_factory: Address,
    pub lsp_contract_wasm: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatorParams {
    pub admin: Address,
    pub pair_name: Symbol,
    pub collateral_per_pair: u128,
    pub serialized_long_asset: Bytes,
    pub serialized_short_asset: Bytes,
    pub collateral_token: Address,
    pub oracle: Address,
    pub calculator: Address,
    pub pool: Address,
}

#[contractimpl]
impl LongShortPairFactory {
    // __constructor
    // Initializes the factory by setting the admin roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The address to be assigned the Admin role.
    //   - emergency_admin: The address to be assigned the EmergencyAdmin role.
    //   - token_factory: The address of the swap token_factory contract.
    //   - lsp_contract_wasm: The WASM hash (BytesN<32>) for the long short pair contract.
    pub fn __constructor(
        e: Env,
        admin: Address,
        emergency_admin: Address,
        token_factory: Address,
        lsp_contract_wasm: BytesN<32>,
    ) {
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &admin);
        access_control.commit_transfer_ownership(&Role::EmergencyAdmin, &emergency_admin);
        access_control.apply_transfer_ownership(&Role::EmergencyAdmin);

        set_token_factory(&e, &token_factory);
        set_lsp_contract_wasm(&e, &lsp_contract_wasm);
    }
}

#[contractimpl]
impl LongShortPairFactoryTrait for LongShortPairFactory {
    /**
     * @notice Creates a longShortPair contract and associated long and short tokens.
     * @param params Constructor params used to initialize the LSP. Key-valued object with the following structure:
     *     - `pairName`: Name of the long short pair contract.
     *     - `collateralPerPair`: How many units of collateral are required to mint one pair of synthetic tokens.
     *     - `collateralToken`: ERC20 token used as collateral in the LSP.
     *     - `calculator`: Contract providing settlement payout logic.
     * @return lspAddress the deployed address of the new long short pair contract.
     * @notice Created LSP is not registered within the registry as the LSP uses the Optimistic Oracle for settlement.
     * @notice The LSP constructor does a number of validations on input params. These are not repeated here.
     */
    fn deploy_lsp_contract(e: Env, params: CreatorParams) -> Address {
        params.admin.require_auth();

        let token_factory = get_token_factory(&e);

        // Deploy the Long Token SAC
        let token_long: Address = e.invoke_contract(
            &token_factory,
            &Symbol::new(&e, "create_token"),
            Vec::from_array(&e, [params.serialized_long_asset.clone().into_val(&e)]),
        );

        // Deploy the Short Token SAC
        let token_short: Address = e.invoke_contract(
            &token_factory,
            &Symbol::new(&e, "create_token"),
            Vec::from_array(&e, [params.serialized_short_asset.clone().into_val(&e)]),
        );

        // Deploy the LSP contract
        let sequence = get_contract_sequence(&e, params.admin.clone());
        set_contract_sequence(&e, params.admin.clone(), sequence + 1);

        let salt = get_pair_salt(&e, &params.admin, &sequence);

        let lsp_address = e.deployer().with_current_contract(salt).deploy_v2(
            get_lsp_contract_wasm(&e),
            (e.current_contract_address(), params.clone()),
        );

        // Give permissions to new lsp contract and then hand over ownership.
        SorobanTokenAdminClient::new(&e, &token_long).set_admin(&lsp_address);
        SorobanTokenAdminClient::new(&e, &token_short).set_admin(&lsp_address);

        // Add to LSP registry
        add_deployed_pair(&e, &params.admin, &lsp_address);

        // Emit enhanced deployment event
        let current_time = e.ledger().timestamp();

        Events::new(&e).long_short_pair_deployed(
            current_time,
            params.admin.clone(),
            lsp_address.clone(), // long_short_pair_address
        );

        lsp_address
    }
}

#[contractimpl]
impl AdminInterface for LongShortPairFactory {
    //   _______    _______  ___________  ___________  _______   _______    ________
    //  /" _   "|  /"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (: ( \___) (: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \/ \       \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //  //  \ ___  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    // (:   _(  _|(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    //  \_______)  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    // Query Methods - Factory Configuration
    fn get_factory_config(e: Env) -> FactoryConfig {
        FactoryConfig {
            token_factory: get_token_factory(&e),
            lsp_contract_wasm: get_lsp_contract_wasm(&e),
        }
    }

    // Individual getters for factory configuration
    fn get_token_factory(e: Env) -> Address {
        get_token_factory(&e)
    }

    fn get_lsp_contract_wasm(e: Env) -> BytesN<32> {
        get_lsp_contract_wasm(&e)
    }

    // LSP Registry Query Methods
    fn get_deployed_pairs(e: Env, operator: Address) -> Vec<Address> {
        get_deployed_pairs(&e, &operator)
    }

    fn get_all_deployed_pairs(e: Env) -> Vec<Address> {
        get_all_deployed_pairs(&e)
    }

    fn get_pair_count(e: Env, operator: Address) -> u32 {
        let pairs = get_deployed_pairs(&e, &operator);
        pairs.len()
    }

    fn get_total_pair_count(e: Env) -> u32 {
        let all_pairs = get_all_deployed_pairs(&e);
        all_pairs.len()
    }

    //   ________  _______  ___________  ___________  _______   _______    ________
    //  /"       )/"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (:   \___/(: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \___  \   \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //   __/  \\  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    //  /" \   :)(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    // (_______/  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    // set_lsp_contract_wasm
    // Updates the WASM hash for the long short pair contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - lsp_contract_wasm: The new WASM hash (BytesN<32>) for the swap fee contract.
    fn set_lsp_contract_wasm(e: Env, admin: Address, lsp_contract_wasm: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let old_wasm = get_lsp_contract_wasm(&e);
        set_lsp_contract_wasm(&e, &lsp_contract_wasm);

        let current_time = e.ledger().timestamp();
        Events::new(&e).lsp_wasm_updated(
            current_time,
            admin.clone(),
            old_wasm.clone(),
            lsp_contract_wasm.clone(),
            1,
        );
    }

    //    _______     __       ____  ____   ________  _______  ________
    //   |   __ "\   /""\     ("  _||_ " | /"       )/"     "||"      "\
    //   (. |__) :) /    \    |   (  ) : |(:   \___/(: ______)(.  ___  :)
    //   |:  ____/ /' /\  \   (:  |  | . ) \___  \   \/    |  |: \   ) ||
    //   (|  /    //  __'  \   \\ \__/ //   __/  \\  // ___)_ (| (___\ ||
    //  /|__/ \  /   /  \\  \  /\\ __ //\  /" \   :)(:      "||:       :)
    // (_______)(___/    \___)(__________)(_______/  \_______)(________/

    fn kill_create(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_create(&e, &true);
        Events::new(&e).factory_paused(e.ledger().timestamp(), admin);
    }

    fn unkill_create(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_create(&e, &false);
        Events::new(&e).factory_unpaused(e.ledger().timestamp(), admin);
    }

    fn get_is_killed_create(e: Env) -> bool {
        get_is_killed_create(&e)
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

#[contractimpl]
impl TransferableContract for LongShortPairFactory {
    // commit_transfer_ownership
    // Commits to transferring ownership of a given role.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - role_name: The symbol representing the role (e.g., "Admin" or "EmergencyAdmin").
    //   - new_address: The new address to assume the role.
    fn commit_transfer_ownership(e: Env, admin: Address, role_name: Symbol, new_address: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        access_control.commit_transfer_ownership(&role, &new_address);
        AccessControlEvents::new(&e).commit_transfer_ownership(role, new_address);
    }

    // apply_transfer_ownership
    // Applies the pending ownership transfer for a role.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - role_name: The symbol representing the role.
    fn apply_transfer_ownership(e: Env, admin: Address, role_name: Symbol) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name.clone());
        let old_address = access_control.get_role(&role);
        let new_address = access_control.apply_transfer_ownership(&role);

        // Emit factory admin updated event if this is an Admin role transfer
        if role_name == Symbol::new(&e, "Admin") {
            let current_time = e.ledger().timestamp();
            Events::new(&e).factory_admin_updated(current_time, old_address, new_address.clone());
        }

        AccessControlEvents::new(&e).apply_transfer_ownership(role, new_address);
    }

    // revert_transfer_ownership
    // Reverts a pending ownership transfer for a role.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - role_name: The symbol representing the role.
    fn revert_transfer_ownership(e: Env, admin: Address, role_name: Symbol) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let role = Role::from_symbol(&e, role_name);
        access_control.revert_transfer_ownership(&role);
        AccessControlEvents::new(&e).revert_transfer_ownership(role);
    }

    // get_future_address
    // Returns the pending future address for a role if an ownership transfer is committed;
    // otherwise, returns the current role address.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - role_name: The symbol representing the role.
    //
    // Returns:
    //   - The Address scheduled to assume the role, or the current address if none pending.
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
