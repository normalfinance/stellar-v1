use crate::errors::LongShortPairError;
use crate::events::{Events, LongShortPairEvents};
use crate::interface::{
    AdminInterfaceTrait, LongShortPairTrait, OracleInterfaceTrait, UpgradeableContract,
};
use crate::storage::{
    get_calculator, get_collateral_per_pair, get_collateral_percent_long,
    get_cumulative_funding_index_long, get_cumulative_funding_index_short, get_funding_period,
    get_is_killed_create, get_is_killed_redeem, get_is_killed_update_funding,
    get_last_24h_avg_funding_rate, get_last_funding_rate, get_last_funding_rate_ts,
    get_normal_oracle, get_sanitize_clamp_denominator, get_user_funding_checkpoint, set_calculator,
    set_collateral_percent_long, set_cumulative_funding_index_long,
    set_cumulative_funding_index_short, set_funding_period, set_is_killed_create,
    set_is_killed_redeem, set_is_killed_update_funding, set_last_funding_rate_ts,
    set_normal_oracle, set_pool, CollateralInfo, FundingInfo,
};
use crate::storage::{
    get_token_long, get_token_short, put_token_collateral, put_token_long, put_token_short,
};
use crate::token::burn_token_short_from;
use crate::token::{burn_token_long_from, get_token_long_balance_of, get_token_short_balance_of};
use crate::token::{mint_token_long, mint_token_short, transfer_token_collateral};
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
    require_operations_admin_or_owner, require_pause_admin_or_owner,
    require_pause_or_emergency_pause_admin_or_owner,
};
use oracle::get_oracle_price;
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, Address, BytesN, Env, IntoVal,
    Map, Symbol, Vec,
};
use types::pair::PairParams;
use upgrade::events::Events as UpgradeEvents;
use upgrade::{apply_upgrade, commit_upgrade, revert_upgrade};
use utils::constant::PRICE_PRECISION_I128;
use utils::math::safe_math::SafeMath;

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "");

#[contract]
pub struct LongShortPair;

#[contractimpl]
impl LongShortPairTrait for LongShortPair {
    // Initializes the long short pair.
    fn initialize(e: Env, params: PairParams) {
        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, LongShortPairError::AlreadyInitialized);
        }
        access_control.set_role_address(&Role::Admin, &params.admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &params.privileged_addrs.0);
        access_control.set_role_address(&Role::RewardsAdmin, &params.privileged_addrs.1);
        access_control.set_role_address(&Role::OperationsAdmin, &params.privileged_addrs.2);
        access_control.set_role_address(&Role::PauseAdmin, &params.privileged_addrs.3);
        access_control.set_role_addresses(&Role::EmergencyPauseAdmin, &params.privileged_addrs.4);
        access_control.set_role_address(&Role::SystemFeeAdmin, &params.privileged_addrs.5);

        set_normal_oracle(&e, &params.oracle);
        set_calculator(&e, &params.pair_calculator);
        set_pool(&e, &params.pool);

        if params.tokens.len() != 3 {
            panic_with_error!(&e, LongShortPairError::WrongInputVecSize);
        }

        let token_long = params.tokens.get(0).unwrap();
        let token_short = params.tokens.get(1).unwrap();
        let token_collateral = params.tokens.get(2).unwrap();

        put_token_long(&e, token_long);
        put_token_short(&e, token_short);
        put_token_collateral(&e, token_collateral);

        // Add the new pair's parameters to the Calculator
        e.invoke_contract(
            &params.pair_calculator,
            &Symbol::new(&e, "set_parameters"),
            Vec::from_array(
                &e,
                [
                    e.current_contract_address().into_val(&e),
                    params.lower_bound.into_val(&e),
                    params.upper_bound.into_val(&e),
                ],
            ),
        )
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

        let collateral_used = tokens_to_mint.safe_mul(&e, get_collateral_per_pair(&e));

        transfer_token_collateral(&e, &user, &e.current_contract_address(), collateral_used);

        mint_token_long(&e, &user, &(tokens_to_mint as i128));
        mint_token_short(&e, &user, &(tokens_to_mint as i128));

        let mut checkpoint = get_user_funding_checkpoint(&e, &user);
        checkpoint.mint(&e, tokens_to_mint);

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
        Self::update_oracle_price(e.clone());

        burn_token_long_from(&e, &user, &(tokens_to_redeem as i128));
        burn_token_short_from(&e, &user, &(tokens_to_redeem as i128));

        let mut checkpoint = get_user_funding_checkpoint(&e, &user);
        let net_funding_delta = checkpoint.net_funding_delta(&e);

        let collateral = tokens_to_redeem.safe_mul(&e, get_collateral_per_pair(&e));
        let multiplier = PRICE_PRECISION_I128.safe_add(&e, net_funding_delta as i128);

        let collateral_adjusted = collateral.safe_mul(&e, multiplier as u128);

        checkpoint.redeem(&e, tokens_to_redeem);

        transfer_token_collateral(
            &e,
            &user,
            &e.current_contract_address(),
            collateral_adjusted,
        );

        Events::new(&e).tokens_redeemed(
            current_time,
            user,
            collateral,
            collateral_adjusted,
            tokens_to_redeem,
        );

        collateral_adjusted
    }

    fn checkpoint_funding(
        e: Env,
        token_contract: Address,
        from: Address,
        to: Address,
        transfer_amount: u128,
    ) {
        token_contract.require_auth();

        let token_long = get_token_long(&e);
        if token_contract != token_long && token_contract != get_token_short(&e) {
            panic_with_error!(&e, AccessControlError::Unauthorized);
        }

        // checkpoint users
        let mut from_checkpoint = get_user_funding_checkpoint(&e, &from);
        let mut to_checkpoint = get_user_funding_checkpoint(&e, &to);

        if token_contract == token_long {
            let cumulative_funding_index_long = get_cumulative_funding_index_long(&e);

            from_checkpoint.long_balance =
                from_checkpoint.long_balance.safe_sub(&e, transfer_amount);
            // apply funding PnL on transfer if needed
            from_checkpoint.long_index = cumulative_funding_index_long;

            // receiver inherits current index
            to_checkpoint.long_index = cumulative_funding_index_long;
            to_checkpoint.long_balance = to_checkpoint.long_balance.safe_add(&e, transfer_amount);
        } else {
            let cumulative_funding_index_short = get_cumulative_funding_index_short(&e);

            from_checkpoint.short_balance =
                from_checkpoint.short_balance.safe_sub(&e, transfer_amount);
            from_checkpoint.short_index = cumulative_funding_index_short;

            to_checkpoint.short_index = cumulative_funding_index_short;
            to_checkpoint.short_balance = to_checkpoint.short_balance.safe_add(&e, transfer_amount);
        }

        from_checkpoint.save(&e);
        to_checkpoint.save(&e);
    }

    fn update_oracle_price(e: Env) {
        let oracle_price_data = get_oracle_price(&e, &get_normal_oracle(&e));

        match e.try_invoke_contract::<u64, soroban_sdk::Error>(
            &get_calculator(&e),
            &Symbol::new(&e, "percent_long_collateral"),
            Vec::from_array(
                &e,
                [
                    e.current_contract_address().into_val(&e),
                    oracle_price_data.price.into_val(&e),
                ],
            ),
        ) {
            Ok(Err(_)) | Err(_) => {
                panic_with_error!(e, LongShortPairError::FailedToGetCalculatorPercent)
            }
            Ok(Ok(new_collateral_percent_long)) => {
                set_collateral_percent_long(&e, &new_collateral_percent_long.min(1_u64));
            }
        }
    }

    fn get_tokens(e: Env) -> Vec<Address> {
        Vec::from_array(&e, [get_token_long(&e), get_token_short(&e)])
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

    fn get_collateral_info(e: Env) -> CollateralInfo {
        CollateralInfo {
            collateral_per_pair: get_collateral_per_pair(&e),
            collateral_percent_long: get_collateral_percent_long(&e),
        }
    }

    fn get_funding_info(e: Env) -> FundingInfo {
        FundingInfo {
            sanitize_clamp_denominator: get_sanitize_clamp_denominator(&e),
            cumulative_funding_index_long: get_cumulative_funding_index_long(&e),
            cumulative_funding_index_short: get_cumulative_funding_index_short(&e),
            last_funding_rate: get_last_funding_rate(&e),
            last24h_avg_funding_rate: get_last_24h_avg_funding_rate(&e),
            last_funding_rate_ts: get_last_funding_rate_ts(&e),
            funding_period: get_funding_period(&e),
        }
    }
}

#[contractimpl]
impl AdminInterfaceTrait for LongShortPair {
    fn update_funding_period(e: Env, admin: Address, funding_period: u64) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        if funding_period <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        set_funding_period(&e, &funding_period);
    }

    fn update_funding_rate(e: Env, admin: Address) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        let current_time = e.ledger().timestamp();
        let funding_paused = get_is_killed_update_funding(&e);

        let is_updated = crate::funding::update_funding_rate(&e, funding_paused, current_time);

        if !is_updated {
            let last_update_ts = get_last_funding_rate_ts(&e);
            let funding_period = get_funding_period(&e);

            let time_until_next_update = crate::funding::on_the_hour_update(
                &e,
                current_time,
                last_update_ts,
                funding_period,
            );
            log!(
                &e,
                "time_until_next_update = {:?} seconds",
                time_until_next_update
            );

            panic_with_error!(&e, LongShortPairError::FundingWasNotUpdated)
        }
    }

    fn migrate(e: Env, admin: Address, lower_bound: u128, upper_bound: u128) {
        admin.require_auth();
        require_operations_admin_or_owner(&e, &admin);

        let current_time = e.ledger().timestamp();

        match e.try_invoke_contract::<u64, soroban_sdk::Error>(
            &get_calculator(&e),
            &Symbol::new(&e, "set_parameters"),
            Vec::from_array(
                &e,
                [
                    e.current_contract_address().into_val(&e),
                    lower_bound.into_val(&e),
                    upper_bound.into_val(&e),
                ],
            ),
        ) {
            Ok(Err(_)) | Err(_) => {
                panic_with_error!(e, LongShortPairError::FailedToGetCalculatorPercent)
            }
            Ok(Ok(new_collateral_percent_long)) => {
                set_collateral_percent_long(&e, &new_collateral_percent_long);

                // Reset funding
                set_last_funding_rate_ts(&e, &current_time);
                set_cumulative_funding_index_long(&e, &1_i64);
                set_cumulative_funding_index_short(&e, &1_i64);

                Events::new(&e).migration(current_time, lower_bound, upper_bound);
            }
        }

        //  event
    }

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

        set_normal_oracle(&e, &oracle);
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
    fn kill_update_funding(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_or_emergency_pause_admin_or_owner(&e, &admin);

        set_is_killed_update_funding(&e, &true);
        Events::new(&e).kill_update_funding();
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
    fn unkill_update_funding(e: Env, admin: Address) {
        admin.require_auth();
        require_pause_admin_or_owner(&e, &admin);

        set_is_killed_update_funding(&e, &false);
        Events::new(&e).unkill_update_funding();
    }

    // Get create killswitch status.
    fn get_is_killed_create(e: Env) -> bool {
        get_is_killed_create(&e)
    }

    // Get redeem killswitch status.
    fn get_is_killed_redeem(e: Env) -> bool {
        get_is_killed_redeem(&e)
    }

    // Get update_funding killswitch status.
    fn get_is_killed_update_funding(e: Env) -> bool {
        get_is_killed_update_funding(&e)
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
