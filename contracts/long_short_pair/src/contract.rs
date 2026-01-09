use crate::errors::LongShortPairError;
use crate::events::{Events, LongShortPairEvents};
use crate::interface::{AdminInterfaceTrait, LongShortPairTrait, OracleInterfaceTrait};
use access_control::access::{AccessControl, AccessControlTrait};

use access_control::management::SingleAddressManagementTrait;
use access_control::role::Role;
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{contract, contractimpl, contractmeta, panic_with_error, Address, Env, Vec};
use types::pair::{CollateralInfo, PairParams, PairStatus, PairSummary};
use utils::constant::PRICE_PRECISION;
use utils::math::safe_math::{PrecisionMath, SafeMath};

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
        crate::utils::sync_collateral(&e);

        token_pair::burn_long_tokens(&e, &user, tokens_to_redeem);
        token_pair::burn_short_tokens(&e, &user, tokens_to_redeem);

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
            current_time,
        );

        collateral_to_return
    }

    fn redeem_one(e: Env, user: Address, token: Address, tokens_to_redeem: u128) -> u128 {
        user.require_auth();

        if tokens_to_redeem <= 0 {
            panic_with_error!(&e, LongShortPairError::InvalidInput);
        }

        let current_time = e.ledger().timestamp();

        crate::utils::sync_collateral(&e);

        // Enable single token redemption during settlement
        if crate::storage::get_status(&e) != PairStatus::Settlement {
            panic_with_error!(&e, LongShortPairError::InvalidStatus);
        }

        let tokens = Self::get_tokens(e.clone());
        let long_token = tokens.get(0).unwrap();
        let short_token = tokens.get(1).unwrap();

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
            current_time,
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
        crate::storage::get_collateral_percent_long(&e) as u128
    }
}
