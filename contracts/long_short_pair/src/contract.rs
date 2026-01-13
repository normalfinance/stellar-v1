use crate::errors::LongShortPairError;
use crate::events::{Events, LongShortPairEvents};
use crate::interface::{AdminInterfaceTrait, LongShortPairTrait, OracleInterfaceTrait};
use crate::storage::{get_is_killed_mint, get_is_killed_redeem};
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, Address, BytesN, Env, Symbol, Vec,
};
use types::pair::{
    CollateralInfo, PairAmounts, PairParams, PairPriceBounds, PairStatus, PairSummary, PairTokens,
    Side,
};
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::{PrecisionMath, SafeConversion, SafeMath};

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
impl LongShortPair {
    /// Initializes the LongShortPair contract.
    ///
    /// This constructor is intended to be called exactly once at deploy time. It:
    /// - Sets core admin roles (`Admin`, `PauseAdmin`, `EmergencyAdmin`) to `params.admin`
    /// - Stores immutable-ish pair configuration such as the underlying `asset`
    /// - Stores collateral configuration (`collateral_token`, `collateral_per_pair`)
    /// - Stores token contract addresses for the LONG and SHORT tokens
    /// - Stores oracle and calculator addresses
    /// - Initializes price bounds (`lower_bound`, `upper_bound`)
    ///
    /// ### Reverts
    /// - [`LongShortPairError::AlreadyInitialized`] if the contract has already been initialized.
    ///
    /// ### Arguments
    /// - `e`: Soroban environment.
    /// - `params`: Full set of pair parameters used to bootstrap the contract.
    pub fn __constructor(e: Env, params: PairParams) {
        // NOTE: constructor auth is intentionally omitted because deployer auth patterns can vary.
        // If you want strict deploy-time auth, uncomment the line below:
        // params.admin.require_auth();

        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, LongShortPairError::AlreadyInitialized);
        }

        access_control.set_role_address(&Role::Admin, &params.admin);
        access_control.set_role_address(&Role::PauseAdmin, &params.admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &params.admin);

        crate::storage::set_asset(&e, &params.asset);
        crate::storage::set_status(&e, &PairStatus::Active);

        // Collateral configuration.
        crate::storage::set_collateral_token(&e, &params.collateral_token);
        crate::storage::set_collateral_per_pair(&e, &params.collateral_per_pair);

        // Token contract addresses for synthetic legs.
        token_pair::set_token_long(&e, &params.long_token);
        token_pair::set_token_short(&e, &params.short_token);

        // Oracle + calculator configuration.
        crate::storage::set_oracle(&e, &params.oracle);
        crate::storage::set_calculator(&e, &params.calculator);

        // Price boundaries for the linear settlement mapping.
        crate::storage::set_lower_bound(&e, &params.lower_bound);
        crate::storage::set_upper_bound(&e, &params.upper_bound);
    }
}

#[contractimpl]
impl LongShortPairTrait for LongShortPair {
    /// Mints equal amounts of LONG and SHORT tokens by depositing collateral.
    ///
    /// For `tokens_to_mint`, the contract:
    /// 1. Calculates `collateral_used = tokens_to_mint * collateral_per_pair`
    /// 2. Transfers `collateral_used` of collateral from `user` into the Pair contract
    /// 3. Mints `tokens_to_mint` LONG and `tokens_to_mint` SHORT to the user
    /// 4. Increments the tracked `total_collateral`
    ///
    /// This is the primary entry mechanism for creating new synthetic exposure.
    ///
    /// ### Reverts
    /// - [`LongShortPairError::InvalidInput`] if `tokens_to_mint == 0`.
    /// - [`LongShortPairError::ActionPaused`] if minting is paused.
    /// - [`LongShortPairError::MintingDisabled`] if the pair is not [`PairStatus::Active`].
    ///
    /// ### Returns
    /// Returns the amount of collateral transferred in (`collateral_used`).
    fn mint(e: Env, user: Address, tokens_to_mint: u128) -> u128 {
        user.require_auth();

        if tokens_to_mint <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        if get_is_killed_mint(&e) {
            panic_with_error!(&e, LongShortPairError::ActionPaused);
        }

        // Do not allow minting unless the pair is active.
        if crate::storage::get_status(&e) != PairStatus::Active {
            panic_with_error!(&e, LongShortPairError::MintingDisabled);
        }

        let collateral_used = tokens_to_mint.safe_fixed_mul_floor(
            &e,
            crate::storage::get_collateral_per_pair(&e),
            PRICE_PRECISION,
        );

        // Pull collateral into the Pair contract first. Storage updates happen after successful transfer.
        SorobanTokenClient::new(&e, &crate::storage::get_collateral_token(&e)).transfer(
            &user,
            &e.current_contract_address(),
            &collateral_used.safe_to_i128(&e),
        );

        // Mint both legs 1:1 to the user.
        token_pair::mint_long_tokens(&e, &user, tokens_to_mint);
        token_pair::mint_short_tokens(&e, &user, tokens_to_mint);

        // Increment total collateral accounting.
        let total_collateral = crate::storage::get_total_collateral(&e);
        let new_total_collateral = total_collateral.safe_add(&e, collateral_used);
        crate::storage::set_total_collateral(&e, &new_total_collateral);

        Events::new(&e).mint(
            user,
            e.current_contract_address(),
            collateral_used,
            tokens_to_mint,
            e.ledger().timestamp(),
        );

        collateral_used
    }

    /// Redeems equal amounts of LONG and SHORT tokens for collateral.
    ///
    /// For `tokens_to_redeem`, the contract:
    /// 1. Synchronizes internal collateral accounting (`sync_collateral`)
    /// 2. Burns `tokens_to_redeem` LONG and `tokens_to_redeem` SHORT from the user
    /// 3. Calculates `collateral_returned = tokens_to_redeem * collateral_per_pair`
    /// 4. Transfers `collateral_returned` of collateral back to the user
    /// 5. Decrements the tracked `total_collateral`
    ///
    /// This redemption path requires burning *both* legs (LONG and SHORT) in equal quantity.
    ///
    /// ### Reverts
    /// - [`LongShortPairError::InvalidInput`] if `tokens_to_redeem == 0`.
    /// - [`LongShortPairError::ActionPaused`] if redeeming is paused.
    /// - [`LongShortPairError::InsufficientInventory`] if the contract does not have enough collateral.
    ///
    /// ### Returns
    /// Returns the amount of collateral returned (`collateral_returned`).
    fn redeem(e: Env, user: Address, tokens_to_redeem: u128) -> u128 {
        user.require_auth();

        if tokens_to_redeem <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        if get_is_killed_redeem(&e) {
            panic_with_error!(&e, LongShortPairError::ActionPaused);
        }

        // Keep `total_collateral` consistent with the actual collateral token balance (if applicable).
        crate::utils::sync_collateral(&e);

        // Burn both legs 1:1 from the redeemer.
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

        // Pay collateral out to the user.
        SorobanTokenClient::new(&e, &crate::storage::get_collateral_token(&e)).transfer(
            &e.current_contract_address(),
            &user,
            &collateral_returned.safe_to_i128(&e),
        );

        // Decrement total collateral accounting.
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

    /// Redeems a single side (LONG *or* SHORT) for collateral after expiration.
    ///
    /// This method is only enabled when the pair status is [`PairStatus::Expired`]. At expiry,
    /// settlement determines the collateral split between LONG and SHORT using
    /// `collateral_percent_long` (and `1 - collateral_percent_long` for SHORT).
    ///
    /// The contract:
    /// 1. Synchronizes internal collateral accounting (`sync_collateral`)
    /// 2. Requires the pair to be expired
    /// 3. Determines the payout percent for `side`
    /// 4. Burns `tokens_to_redeem` of the chosen `side`
    /// 5. Pays out `tokens_to_redeem * collateral_per_pair * side_pct`
    /// 6. Decrements tracked `total_collateral`
    ///
    /// ### Notes
    /// - Redeeming a worthless side is forbidden (e.g., side percent is 0).
    /// - A computed payout of 0 is rejected (e.g., tiny redemption amount).
    ///
    /// ### Reverts
    /// - [`LongShortPairError::InvalidInput`] if `tokens_to_redeem == 0` or payout would be 0.
    /// - [`LongShortPairError::ActionPaused`] if redeeming is paused.
    /// - [`LongShortPairError::InvalidStatus`] if the pair is not expired.
    /// - [`LongShortPairError::InsufficientInventory`] if the chosen side is worthless or collateral is insufficient.
    ///
    /// ### Returns
    /// Returns the amount of collateral returned (`collateral_to_return`).
    fn redeem_one(e: Env, user: Address, side: Side, tokens_to_redeem: u128) -> u128 {
        user.require_auth();

        if tokens_to_redeem <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        if get_is_killed_redeem(&e) {
            panic_with_error!(&e, LongShortPairError::ActionPaused);
        }

        crate::utils::sync_collateral(&e);

        // Enable single token redemption only during expiration.
        if crate::storage::get_status(&e) != PairStatus::Expired {
            panic_with_error!(&e, LongShortPairError::InvalidStatus);
        }

        // Settlement allocation
        let collateral_percent_long = crate::storage::get_collateral_percent_long(&e);
        let collateral_percent_short = PRICE_PRECISION.safe_sub(&e, collateral_percent_long);

        let side_pct = if side == Side::Long {
            collateral_percent_long
        } else {
            collateral_percent_short
        };

        // Forbid redeeming a worthless side (prevents "burn for 0").
        if side_pct == 0 {
            panic_with_error!(&e, LongShortPairError::InsufficientInventory);
        }

        // Burn only the redeemed side.
        if side == Side::Long {
            token_pair::burn_long_tokens(&e, &user, tokens_to_redeem);
        } else {
            token_pair::burn_short_tokens(&e, &user, tokens_to_redeem);
        }

        // Base collateral per full pair, then apply settlement percent for the chosen side.
        let base = tokens_to_redeem.safe_fixed_mul_floor(
            &e,
            crate::storage::get_collateral_per_pair(&e),
            PRICE_PRECISION,
        );
        let collateral_to_return = base.safe_fixed_mul_floor(&e, side_pct, PRICE_PRECISION);

        // Forbid 0 payout (covers side_pct>0 but tiny amounts).
        if collateral_to_return == 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        let total_collateral = crate::storage::get_total_collateral(&e);

        if collateral_to_return > total_collateral {
            panic_with_error!(&e, LongShortPairError::InsufficientInventory);
        }

        // Pay collateral out to the user.
        SorobanTokenClient::new(&e, &crate::storage::get_collateral_token(&e)).transfer(
            &e.current_contract_address(),
            &user,
            &collateral_to_return.safe_to_i128(&e),
        );

        // Decrement total collateral accounting.
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

    /// Updates the Pair's settlement allocation (`collateral_percent_long`) using the oracle TWAP.
    ///
    /// This is a passthrough to `crate::utils::sync_collateral(&e)`.
    ///
    /// Despite the name, this does **not** transfer collateral or reconcile balances. It only:
    /// - Queries the oracle TWAP
    /// - Computes a new `collateral_percent_long` via the calculator
    /// - Updates `last_update_ts` and `collateral_percent_long`
    /// - Potentially marks the Pair as [`PairStatus::Expired`] and sets `expiration_ts`
    ///   if the TWAP reaches or crosses the configured bounds.
    ///
    /// This function is useful for keepers/frontends that want to “poke” the Pair so that
    /// settlement state is fresh before mint/redeem flows or UI display.
    fn sync_collateral(e: Env) {
        crate::utils::sync_collateral(&e)
    }

    /// Returns the token contract addresses for the Pair: LONG, SHORT, and collateral.
    fn get_tokens(e: Env) -> PairTokens {
        PairTokens {
            long: token_pair::get_token_long(&e),
            short: token_pair::get_token_short(&e),
            collateral: crate::storage::get_collateral_token(&e),
        }
    }

    /// Returns the current configured price bounds for the Pair.
    ///
    /// These bounds define the linear mapping used to convert settlement percent into a scaled price.
    fn get_price_bounds(e: Env) -> PairPriceBounds {
        PairPriceBounds {
            lower: crate::storage::get_lower_bound(&e),
            upper: crate::storage::get_upper_bound(&e),
        }
    }

    /// Returns the user's balances of LONG and SHORT tokens.
    fn get_user_token_balances(e: Env, user: Address) -> PairAmounts {
        PairAmounts {
            long: token_pair::get_user_balance_long(&e, &user),
            short: token_pair::get_user_balance_short(&e, &user),
        }
    }

    /// Returns the total supply of LONG and SHORT tokens.
    fn get_total_token_supplies(e: Env) -> PairAmounts {
        PairAmounts {
            long: token_pair::get_total_long_shares(&e),
            short: token_pair::get_total_short_shares(&e),
        }
    }

    /// Returns current collateral configuration and settlement information.
    ///
    /// `collateral_percent_long` is the settlement allocation to LONG in `PRICE_PRECISION` units.
    /// SHORT receives `PRICE_PRECISION - collateral_percent_long`.
    fn get_collateral_info(e: Env) -> CollateralInfo {
        CollateralInfo {
            collateral_token: crate::storage::get_collateral_token(&e),
            total_collateral: crate::storage::get_total_collateral(&e),
            collateral_per_pair: crate::storage::get_collateral_per_pair(&e),
            collateral_percent_long: crate::storage::get_collateral_percent_long(&e),
        }
    }

    /// Returns an aggregated snapshot of the Pair state.
    ///
    /// This is a convenience method for frontends/indexers to avoid multiple round-trips.
    fn get_summary(e: Env) -> PairSummary {
        PairSummary {
            asset: crate::storage::get_asset(&e),
            status: crate::storage::get_status(&e),
            collateral: Self::get_collateral_info(e.clone()),
            tokens: PairTokens {
                long: token_pair::get_token_long(&e),
                short: token_pair::get_token_short(&e),
                collateral: crate::storage::get_collateral_token(&e),
            },
            price_bounds: PairPriceBounds {
                lower: crate::storage::get_lower_bound(&e),
                upper: crate::storage::get_upper_bound(&e),
            },
            calculator: crate::storage::get_calculator(&e),
            oracle: crate::storage::get_oracle(&e),
        }
    }

    /// Returns the current pair status.
    fn get_status(e: Env) -> PairStatus {
        crate::storage::get_status(&e)
    }
}

#[contractimpl]
impl AdminInterfaceTrait for LongShortPair {
    /// Updates the calculator address used by this Pair.
    ///
    /// ### Reverts
    /// - Reverts if `admin` is not authorized.
    fn set_calculator(e: Env, admin: Address, calculator: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_calculator(&e, &calculator);
    }

    /// Updates the oracle address used by this Pair.
    ///
    /// ### Reverts
    /// - Reverts if `admin` is not authorized.
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
    /// Returns the current settlement allocation to LONG in `PRICE_PRECISION` units.
    ///
    /// This is not a spot price. It is the settlement fraction of collateral assigned to LONG.
    fn get_price(e: Env) -> u128 {
        crate::storage::get_collateral_percent_long(&e)
    }

    /// Returns the "scaled price" implied by the current settlement allocation.
    ///
    /// This maps `collateral_percent_long` linearly into the configured `[lower_bound, upper_bound]` range:
    ///
    /// ```text
    /// scaled_price = lower_bound + (upper_bound - lower_bound) * collateral_percent_long / PRICE_PRECISION
    /// ```
    ///
    /// ### Returns
    /// Returns the scaled price in the same units as the configured bounds.
    fn get_scaled_price(e: Env) -> u128 {
        let colleteral_percent_long = crate::storage::get_collateral_percent_long(&e);

        let lower_bound = crate::storage::get_lower_bound(&e);
        let upper_bound = crate::storage::get_upper_bound(&e);

        let price_bound_delta = upper_bound.safe_sub(&e, lower_bound);

        let scaled_price = price_bound_delta
            .safe_mul(&e, colleteral_percent_long)
            .safe_div(&e, PRICE_PRECISION)
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
