use crate::errors::LongShortPairError;
use crate::events::{Events, LongShortPairEvents};
use crate::interface::{AdminInterfaceTrait, LongShortPairTrait, OracleInterfaceTrait};
use crate::storage::{get_is_killed_mint, get_is_killed_redeem};
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, Address, BytesN, Env, Symbol, Vec,
};
use types::pair::{CollateralInfo, PairParams, PairStatus, PairSummary};
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::{PrecisionMath, SafeMath};

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
    val = "A DeFi primitive for financial derivatives using long and short tokens"
);

#[contract]
pub struct LongShortPair;

#[contractimpl]
impl LongShortPairTrait for LongShortPair {
    fn initialize(e: Env, params: PairParams) {
        params.admin.require_auth();

        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, LongShortPairError::AlreadyInitialized);
        }
        access_control.set_role_address(&Role::Admin, &params.admin);
        access_control.set_role_address(&Role::PauseAdmin, &params.admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &params.admin);

        crate::storage::set_asset(&e, &params.asset);
        crate::storage::set_status(&e, &PairStatus::Active);

        // Collateral
        crate::storage::set_collateral_token(&e, &params.collateral_token);
        crate::storage::set_collateral_per_pair(&e, &params.collateral_per_pair);

        // Tokens
        token_pair::put_token_long(&e, params.long_token);
        token_pair::put_token_short(&e, params.short_token);

        // Addresses
        crate::storage::set_oracle(&e, &params.oracle);
        crate::storage::set_calculator(&e, &params.calculator);

        // Price boundaries
        crate::storage::set_lower_bound(&e, &params.lower_bound);
        crate::storage::set_upper_bound(&e, &params.upper_bound);
    }

    /// Creates a pair of long and short tokens equal in number to tokens_to_mint. Pulls the required collateral
    /// amount into this contract, defined by the collateral_per_pair value.
    /// @param tokens_to_mint number of long and short synthetic tokens to create.
    /// @return collateral_used total collateral used to mint the synthetics.
    fn mint(e: Env, user: Address, tokens_to_mint: u128) -> u128 {
        user.require_auth();

        if tokens_to_mint <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        if get_is_killed_mint(&e) {
            panic_with_error!(&e, LongShortPairError::ActionPaused);
        }

        // Do not allow minting unless pair is active
        if crate::storage::get_status(&e) != PairStatus::Active {
            panic_with_error!(&e, LongShortPairError::MintingDisabled);
        }

        let current_time = e.ledger().timestamp();

        let collateral_used = tokens_to_mint.safe_fixed_mul_floor(
            &e,
            crate::storage::get_collateral_per_pair(&e),
            PRICE_PRECISION,
        );

        SorobanTokenClient::new(&e, &crate::storage::get_collateral_token(&e)).transfer(
            &user,
            &e.current_contract_address(),
            &(collateral_used as i128),
        );

        token_pair::mint_long_tokens(&e, &user, tokens_to_mint as i128);
        token_pair::mint_short_tokens(&e, &user, tokens_to_mint as i128);

        // Increment total collateral
        let total_collateral = crate::storage::get_total_collateral(&e);
        let new_total_collateral = total_collateral.safe_add(&e, collateral_used);
        crate::storage::set_total_collateral(&e, &new_total_collateral);

        Events::new(&e).mint(
            user,
            e.current_contract_address(),
            collateral_used,
            tokens_to_mint,
            current_time,
        );

        collateral_used
    }

    /// Redeems a pair of long and short tokens equal in number to tokens_to_redeem. Returns the commensurate
    /// amount of collateral to the caller for the pair of tokens, defined by the collateral_per_pair value.
    /// @param tokens_to_redeem number of long and short synthetic tokens to redeem.
    /// @return collateral_returned total collateral returned in exchange for the pair of synthetics.
    fn redeem(e: Env, user: Address, tokens_to_redeem: u128) -> u128 {
        user.require_auth();

        if tokens_to_redeem <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        if get_is_killed_redeem(&e) {
            panic_with_error!(&e, LongShortPairError::ActionPaused);
        }

        // Get the oracle price and store it. Also sets collateral_percent_long. Reverts if either:
        // a) the price request has not resolved (either a normal expiration call or early expiration call) or b) If the
        // the contract was attempted to be settled early but the price returned is the ignore oracle price.
        // Note that we use the bool receivedSettlementPrice over checking for price != 0 as 0 is a valid price.
        crate::utils::sync_collateral(&e);

        token_pair::burn_long_tokens(&e, &user, tokens_to_redeem);
        token_pair::burn_short_tokens(&e, &user, tokens_to_redeem);

        let collateral_returned = tokens_to_redeem.safe_fixed_mul_floor(
            &e,
            crate::storage::get_collateral_per_pair(&e),
            PRICE_PRECISION,
        );
        let total_collateral = crate::storage::get_total_collateral(&e);

        if collateral_returned > total_collateral {
            panic_with_error!(&e, LongShortPairError::InsufficientInventory);
        }

        SorobanTokenClient::new(&e, &crate::storage::get_collateral_token(&e)).transfer(
            &e.current_contract_address(),
            &user,
            &(collateral_returned as i128),
        );

        // Decrement total collateral
        let new_total_collateral = total_collateral.safe_sub(&e, collateral_returned);
        crate::storage::set_total_collateral(&e, &new_total_collateral);

        Events::new(&e).redemption(
            user,
            e.current_contract_address(),
            collateral_returned,
            tokens_to_redeem,
            e.ledger().timestamp(),
        );

        collateral_returned
    }

    fn redeem_one(e: Env, user: Address, token: Address, tokens_to_redeem: u128) -> u128 {
        user.require_auth();

        if tokens_to_redeem <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        if get_is_killed_redeem(&e) {
            panic_with_error!(&e, LongShortPairError::ActionPaused);
        }

        crate::utils::sync_collateral(&e);

        // Enable single token redemption during expiration
        if crate::storage::get_status(&e) != PairStatus::Expired {
            panic_with_error!(&e, LongShortPairError::InvalidStatus);
        }

        let long_token = token_pair::get_token_long(&e);
        let short_token = token_pair::get_token_short(&e);

        if token != long_token && token != short_token {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        if token == long_token {
            token_pair::burn_long_tokens(&e, &user, tokens_to_redeem);
        }

        if token == short_token {
            token_pair::burn_short_tokens(&e, &user, tokens_to_redeem);
        }

        let collateral_to_return = tokens_to_redeem.safe_fixed_mul_floor(
            &e,
            crate::storage::get_collateral_per_pair(&e),
            PRICE_PRECISION,
        );
        let total_collateral = crate::storage::get_total_collateral(&e);

        if collateral_to_return > total_collateral {
            panic_with_error!(&e, LongShortPairError::InsufficientInventory);
        }

        SorobanTokenClient::new(&e, &crate::storage::get_collateral_token(&e)).transfer(
            &e.current_contract_address(),
            &user,
            &(collateral_to_return as i128),
        );

        // Decrement total collateral
        let new_total_collateral = total_collateral.safe_sub(&e, collateral_to_return);
        crate::storage::set_total_collateral(&e, &new_total_collateral);

        Events::new(&e).redemption(
            user,
            e.current_contract_address(),
            collateral_to_return,
            tokens_to_redeem,
            e.ledger().timestamp(),
        );

        collateral_to_return
    }

    fn sync_collateral(e: Env) {
        crate::utils::sync_collateral(&e)
    }

    fn get_tokens(e: Env) -> Vec<Address> {
        Vec::from_array(
            &e,
            [
                token_pair::get_token_long(&e),
                token_pair::get_token_short(&e),
            ],
        )
    }

    fn get_price_bounds(e: Env) -> Vec<u128> {
        Vec::from_array(
            &e,
            [
                crate::storage::get_lower_bound(&e),
                crate::storage::get_upper_bound(&e),
            ],
        )
    }

    fn get_user_token_balances(e: Env, user: Address) -> Vec<u128> {
        Vec::from_array(
            &e,
            [
                token_pair::get_user_balance_long(&e, &user),
                token_pair::get_user_balance_short(&e, &user),
            ],
        )
    }

    fn get_total_token_supplies(e: Env) -> Vec<u128> {
        Vec::from_array(
            &e,
            [
                token_pair::get_total_long_shares(&e),
                token_pair::get_total_short_shares(&e),
            ],
        )
    }

    fn get_collateral_info(e: Env) -> CollateralInfo {
        CollateralInfo {
            collateral_token: crate::storage::get_collateral_token(&e),
            total_collateral: crate::storage::get_total_collateral(&e),
            collateral_per_pair: crate::storage::get_collateral_per_pair(&e),
            collateral_percent_long: crate::storage::get_collateral_percent_long(&e),
        }
    }

    fn get_pair_summary(e: Env) -> PairSummary {
        PairSummary {
            asset: crate::storage::get_asset(&e),
            status: crate::storage::get_status(&e),
            collateral: Self::get_collateral_info(e.clone()),
            long_token: token_pair::get_token_long(&e),
            short_token: token_pair::get_token_short(&e),
            price_bounds: (
                crate::storage::get_lower_bound(&e),
                crate::storage::get_upper_bound(&e),
            ),
            calculator: crate::storage::get_calculator(&e),
            oracle: crate::storage::get_oracle(&e),
        }
    }
}

#[contractimpl]
impl AdminInterfaceTrait for LongShortPair {
    fn set_calculator(e: Env, admin: Address, calculator: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_calculator(&e, &calculator);
    }

    fn set_oracle(e: Env, admin: Address, oracle: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_oracle(&e, &oracle);
    }

    // Stops the pair mints instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_mint(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_is_killed_mint(&e, &true);
        Events::new(&e).kill_mint();
    }

    // Stops the pair redemptions instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_redeem(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_is_killed_redeem(&e, &true);
        Events::new(&e).kill_redeem();
    }

    // Resumes the pair mints.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_mint(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_is_killed_mint(&e, &false);
        Events::new(&e).unkill_mint();
    }

    // Resumes the pair redemptions.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_redeem(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_is_killed_redeem(&e, &false);
        Events::new(&e).unkill_redeem();
    }

    // Get create killswitch status.
    fn get_is_killed_mint(e: Env) -> bool {
        crate::storage::get_is_killed_mint(&e)
    }

    // Get redeem killswitch status.
    fn get_is_killed_redeem(e: Env) -> bool {
        crate::storage::get_is_killed_redeem(&e)
    }
}

#[contractimpl]
impl OracleInterfaceTrait for LongShortPair {
    fn get_price(e: Env) -> u128 {
        crate::storage::get_collateral_percent_long(&e)
    }

    fn get_scaled_price(e: Env) -> u128 {
        let colleteral_percent_long = crate::storage::get_collateral_percent_long(&e);

        let lower_bound = crate::storage::get_lower_bound(&e);
        let upper_bound = crate::storage::get_upper_bound(&e);

        let price_bound_delta = upper_bound.safe_sub(&e, lower_bound);

        let scaled_price = price_bound_delta
            .safe_mul(&e, colleteral_percent_long)
            .safe_add(&e, lower_bound);

        scaled_price
    }
}

#[contractimpl]
impl UpgradeableContract for LongShortPair {
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
        Symbol::new(&e, "LongShortPair")
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
