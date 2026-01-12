use crate::errors::TreasuryError;
use crate::events::{Events, TreasuryEvents};
use crate::interface::{AdminInterfaceTrait, TradingTrait, TreasuryTrait};
use crate::storage::{
    get_fee_config, get_is_killed_deposit, get_is_killed_trade, get_is_killed_withdraw,
    get_pair_balances, get_pair_details, get_protocol_fees, get_total_shares, get_user_shares,
    set_fee_config, set_is_killed_deposit, set_is_killed_trade, set_is_killed_withdraw,
    set_pair_balances, set_pair_details, set_protocol_fees, set_total_shares, set_user_shares,
    TreasuryFeeConfig, TreasuryPairBalances, TreasuryPairDetails, TreasuryPairSummary,
    TreasuryUserPairSummary,
};
use soroban_sdk::auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation};
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, vec, Address, BytesN, Env, IntoVal,
    Symbol, Vec,
};
use types::pair::{Direction, Side};
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
    val = "A secondary market for Long Short Pair tokens with accurate price enforcement"
);

#[contract]
pub struct Treasury;

#[contractimpl]
impl Treasury {
    // __constructor
    // Initializes the treasury by setting the admin roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The address to be assigned the Admin role.
    pub fn __constructor(e: Env, admin: Address) {
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &admin);
        access_control.set_role_address(&Role::PauseAdmin, &admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &admin);
    }
}

#[contractimpl]
impl TreasuryTrait for Treasury {
    fn deposit(e: Env, user: Address, pair: Address, pairs_to_deposit: u128) -> u128 {
        user.require_auth();

        if pairs_to_deposit <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_deposit(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);

        let treasury = e.current_contract_address();

        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);
        let balances = get_pair_balances(&e, &pair);

        // Compute how many LP shares to mint
        let total_shares = get_total_shares(&e, &pair);
        let total_pairs =
            crate::lp::total_pairs(&e, &balances, collateral_info.collateral_per_pair);
        let shares_to_mint =
            crate::lp::shares_to_mint(&e, &pair, total_pairs, total_shares, pairs_to_deposit);

        // Transfer long and short tokens
        SorobanTokenClient::new(&e, &pair_details.token_long).transfer(
            &user,
            &treasury,
            &pairs_to_deposit.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &pair_details.token_short).transfer(
            &user,
            &treasury,
            &pairs_to_deposit.safe_to_i128(&e),
        );

        // Transfer USDC
        let usdc_required = pairs_to_deposit.safe_fixed_mul_floor(
            &e,
            collateral_info.collateral_per_pair,
            PRICE_PRECISION,
        );
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &user,
            &treasury,
            &usdc_required.safe_to_i128(&e),
        );

        // Update total shares
        let new_total_shares = total_shares.safe_add(&e, shares_to_mint);
        set_total_shares(&e, &pair, new_total_shares);

        // Update user shares
        let user_shares = get_user_shares(&e, &pair, &user);
        let new_user_shares = user_shares.safe_add(&e, shares_to_mint);
        set_user_shares(&e, &pair, &user, new_user_shares);

        // Update balances
        set_pair_balances(
            &e,
            &pair,
            &(TreasuryPairBalances {
                token_quote: balances.token_quote.safe_add(&e, usdc_required),
                token_long: balances.token_long.safe_add(&e, pairs_to_deposit),
                token_short: balances.token_short.safe_add(&e, pairs_to_deposit),
            }),
        );

        Events::new(&e).deposit(
            user,
            pair,
            pairs_to_deposit,
            usdc_required,
            total_shares,
            new_total_shares,
            user_shares,
            new_user_shares,
            e.ledger().timestamp(),
        );

        shares_to_mint
    }

    fn withdraw(e: Env, user: Address, pair: Address, shares: u128) -> (u128, u128, u128) {
        user.require_auth();

        if shares <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_withdraw(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);

        let total_shares = get_total_shares(&e, &pair);
        let user_shares = get_user_shares(&e, &pair, &user);

        if shares > user_shares || total_shares <= 0 {
            panic_with_error!(&e, TreasuryError::InsufficientShares);
        }

        let treasury = e.current_contract_address();
        let balances = get_pair_balances(&e, &pair);
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        // Compute withdrawable pairs (inventory-based, not pro-rata token balances)
        let withdrawable_pairs =
            crate::lp::total_pairs(&e, &balances, collateral_info.collateral_per_pair);

        // Convert shares -> pairs_out pro-rata
        let pairs_out = crate::lp::pairs_for_withdraw(&e, withdrawable_pairs, total_shares, shares);

        // Never withdraw more pairs than inventory says is withdrawable
        if pairs_out > withdrawable_pairs {
            panic_with_error!(&e, TreasuryError::InsufficientInventory);
        }

        // Compute symmetric payouts
        let out_long = pairs_out;
        let out_short = pairs_out;
        let out_usdc = pairs_out.safe_fixed_mul_floor(
            &e,
            collateral_info.collateral_per_pair,
            PRICE_PRECISION,
        );

        // Burn shares
        let new_total_shares = total_shares.safe_sub(&e, shares);
        set_total_shares(&e, &pair, new_total_shares);

        let new_user_shares = user_shares.safe_sub(&e, shares);
        set_user_shares(&e, &pair, &user, new_user_shares);

        // Transfers
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &treasury,
            &user,
            &out_usdc.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &pair_details.token_long).transfer(
            &treasury,
            &user,
            &out_long.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &pair_details.token_short).transfer(
            &treasury,
            &user,
            &out_short.safe_to_i128(&e),
        );

        // Update balances
        set_pair_balances(
            &e,
            &pair,
            &(TreasuryPairBalances {
                token_quote: balances.token_quote.safe_sub(&e, out_usdc),
                token_long: balances.token_long.safe_sub(&e, out_long),
                token_short: balances.token_short.safe_sub(&e, out_short),
            }),
        );

        Events::new(&e).withdraw(
            user,
            pair,
            shares,
            out_long,
            out_short,
            out_usdc,
            total_shares,
            new_total_shares,
            user_shares,
            new_user_shares,
            e.ledger().timestamp(),
        );

        (out_usdc, out_long, out_short)
    }

    fn get_pair_details(e: Env, pair: Address) -> TreasuryPairDetails {
        crate::storage::get_pair_details(&e, &pair)
    }

    fn get_total_pairs(e: Env, pair: Address) -> u128 {
        let balances = get_pair_balances(&e, &pair);
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        crate::lp::total_pairs(&e, &balances, collateral_info.collateral_per_pair)
    }

    fn get_prices(e: Env, pair: Address) -> (u128, u128) {
        crate::price::get_prices(&e, &pair)
    }

    fn get_balances(e: Env, pair: Address) -> TreasuryPairBalances {
        crate::storage::get_pair_balances(&e, &pair)
    }

    fn get_total_shares(e: Env, pair: Address) -> u128 {
        crate::storage::get_total_shares(&e, &pair)
    }

    fn get_user_shares(e: Env, pair: Address, user: Address) -> u128 {
        crate::storage::get_user_shares(&e, &pair, &user)
    }

    fn get_pair_fee_config(e: Env, pair: Address) -> TreasuryFeeConfig {
        crate::storage::get_fee_config(&e, &pair)
    }

    fn get_pair_summary(e: Env, pair: Address) -> TreasuryPairSummary {
        let balances = crate::storage::get_pair_balances(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        TreasuryPairSummary {
            details: crate::storage::get_pair_details(&e, &pair),
            balances: crate::storage::get_pair_balances(&e, &pair),
            prices,
            total_pairs: crate::lp::total_pairs(&e, &balances, collateral_info.collateral_per_pair),
            total_shares: crate::storage::get_total_shares(&e, &pair),
            fee_config: crate::storage::get_fee_config(&e, &pair),
        }
    }

    fn get_user_with_pair_summary(e: Env, pair: Address, user: Address) -> TreasuryUserPairSummary {
        let balances = crate::storage::get_pair_balances(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        TreasuryUserPairSummary {
            pair_summary: TreasuryPairSummary {
                details: crate::storage::get_pair_details(&e, &pair),
                balances: crate::storage::get_pair_balances(&e, &pair),
                prices,
                total_pairs: crate::lp::total_pairs(
                    &e,
                    &balances,
                    collateral_info.collateral_per_pair,
                ),
                total_shares: crate::storage::get_total_shares(&e, &pair),
                fee_config: crate::storage::get_fee_config(&e, &pair),
            },
            user_shares: get_user_shares(&e, &pair, &user),
        }
    }
}

#[contractimpl]
impl TradingTrait for Treasury {
    fn estimate_trade(
        e: Env,
        pair: Address,
        direction: Direction,
        side: Side,
        amount_in: u128,
        taker_fee: bool,
    ) -> (u128, u128) {
        if amount_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        get_pair_details(&e, &pair); // call to ensure pair exists

        let fee_config = get_fee_config(&e, &pair);
        let fee = if taker_fee {
            fee_config.taker_fee
        } else {
            fee_config.maker_fee
        };

        let (long_price, short_price) = crate::price::get_prices(&e, &pair);

        if direction == Direction::Buy && side == Side::Long {
            return crate::price::quote_buy_token(&e, amount_in, long_price, fee);
        }

        if direction == Direction::Buy && side == Side::Short {
            return crate::price::quote_buy_token(&e, amount_in, short_price, fee);
        }

        if direction == Direction::Sell && side == Side::Long {
            return crate::price::quote_sell_token(&e, amount_in, long_price, fee);
        }

        if direction == Direction::Sell && side == Side::Short {
            return crate::price::quote_sell_token(&e, amount_in, short_price, fee);
        }

        panic_with_error!(&e, TreasuryError::InvalidInput)
    }

    fn mint_and_sell_short(e: Env, user: Address, pair: Address, usdc_in: u128) -> u128 {
        user.require_auth();

        if usdc_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);
        let treasury = e.current_contract_address();
        let fee_config = get_fee_config(&e, &pair);

        // Fetch collateral info
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);
        let long_price = collateral_info.collateral_percent_long;

        // Compute how many tokens to mint
        let (tokens_to_mint, usdc_fee) = crate::price::quote_buy_token(
            &e,
            usdc_in,
            long_price,
            fee_config.maker_fee, // note the lesser maker fee
        );

        // Transfer enough USDC from the user to the Treasury to mint
        let collateral_required = tokens_to_mint.safe_fixed_mul_floor(
            &e,
            collateral_info.collateral_per_pair,
            PRICE_PRECISION,
        );
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &user,
            &treasury,
            &collateral_required.safe_to_i128(&e),
        );

        // Mint tokens via the Long Short Pair
        e.authorize_as_current_contract(vec![
            &e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: pair_details.token_quote.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: (
                        e.current_contract_address(),
                        pair.clone(),
                        collateral_required.safe_to_i128(&e),
                    )
                        .into_val(&e),
                },
                sub_invocations: vec![&e],
            }),
        ]);
        let _collateral_used = crate::pair::mint_pair_as_treasury(&e, &pair, tokens_to_mint);

        // Transfer the newly minted long tokens to the user
        SorobanTokenClient::new(&e, &pair_details.token_long).transfer(
            &treasury,
            &user,
            &tokens_to_mint.safe_to_i128(&e),
        );

        // Then sell the short token by keeping it in the Treasury and returning it's value in USDC
        let usdc_to_return = collateral_required.safe_sub(&e, usdc_in);

        // Transfer the USDC proceeds from the "sale" of short token(s) back to the user
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &user,
            &treasury,
            &usdc_to_return.safe_to_i128(&e),
        );

        // Update balances
        let balances = get_pair_balances(&e, &pair);
        set_pair_balances(
            &e,
            &pair,
            &(TreasuryPairBalances {
                token_quote: balances
                    .token_quote
                    .safe_add(&e, usdc_in.safe_sub(&e, usdc_fee)),
                token_long: balances.token_long,
                token_short: balances.token_short.safe_add(&e, tokens_to_mint),
            }),
        );

        Events::new(&e).trade(
            user,
            pair,
            true,
            Side::Long,
            Direction::Buy,
            usdc_in,
            tokens_to_mint,
            long_price,
            fee_config.maker_fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        tokens_to_mint
    }

    fn mint_and_sell_long(e: Env, user: Address, pair: Address, usdc_in: u128) -> u128 {
        user.require_auth();

        if usdc_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);
        let treasury = e.current_contract_address();
        let fee_config = get_fee_config(&e, &pair);

        // Fetch collateral info
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);
        let short_price = PRICE_PRECISION.safe_sub(&e, collateral_info.collateral_percent_long);

        // Compute how many tokens to mint
        let (tokens_to_mint, usdc_fee) = crate::price::quote_buy_token(
            &e,
            usdc_in,
            short_price,
            fee_config.maker_fee, // note the lesser maker fee
        );

        // Transfer enough USDC from the user to the Treasury to mint
        let collateral_required = tokens_to_mint.safe_fixed_mul_floor(
            &e,
            collateral_info.collateral_per_pair,
            PRICE_PRECISION,
        );
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &user,
            &treasury,
            &collateral_required.safe_to_i128(&e),
        );

        // Mint tokens via the Long Short Pair
        e.authorize_as_current_contract(vec![
            &e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: pair_details.token_quote.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: (
                        e.current_contract_address(),
                        pair.clone(),
                        collateral_required.safe_to_i128(&e),
                    )
                        .into_val(&e),
                },
                sub_invocations: vec![&e],
            }),
        ]);

        crate::pair::mint_pair_as_treasury(&e, &pair, tokens_to_mint);

        // Transfer the newly minted short token(s) to the user
        SorobanTokenClient::new(&e, &pair_details.token_short).transfer(
            &treasury,
            &user,
            &tokens_to_mint.safe_to_i128(&e),
        );

        // Then sell the long token(s) by keeping them in the Treasury and returning their value in USDC
        let usdc_to_return = collateral_required.safe_sub(&e, usdc_in);

        // Transfer the USDC proceeds from the "sale" of long token(s) back to the user
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &user,
            &treasury,
            &usdc_to_return.safe_to_i128(&e),
        );

        // Update balances
        let balances = get_pair_balances(&e, &pair);
        set_pair_balances(
            &e,
            &pair,
            &(TreasuryPairBalances {
                token_quote: balances
                    .token_quote
                    .safe_add(&e, usdc_in.safe_sub(&e, usdc_fee)),
                token_long: balances.token_long.safe_add(&e, tokens_to_mint),
                token_short: balances.token_short,
            }),
        );

        Events::new(&e).trade(
            user,
            pair,
            true,
            Side::Short,
            Direction::Buy,
            usdc_in,
            tokens_to_mint,
            short_price,
            fee_config.maker_fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        tokens_to_mint
    }

    fn buy_long_and_redeem(e: Env, user: Address, pair: Address, short_in: u128) -> u128 {
        user.require_auth();

        if short_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);
        let treasury = e.current_contract_address();
        let fee_config = get_fee_config(&e, &pair);

        // Fetch collateral info
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        // Transfer USDC from user to Treasury
        let usdc_required =
            short_in.safe_fixed_mul_floor(&e, collateral_info.collateral_per_pair, PRICE_PRECISION);
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &user,
            &treasury,
            &usdc_required.safe_to_i128(&e),
        );

        // Transfer the short token(s) from the user to the Treasury
        SorobanTokenClient::new(&e, &pair_details.token_short).transfer(
            &user,
            &treasury,
            &short_in.safe_to_i128(&e),
        );

        // Redeem tokens via the Long Short Pair
        e.authorize_as_current_contract(vec![
            &e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: pair_details.token_long.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: (
                        e.current_contract_address(),
                        pair.clone(),
                        short_in.safe_to_i128(&e),
                    )
                        .into_val(&e),
                },
                sub_invocations: vec![&e],
            }),
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: pair_details.token_short.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: (
                        e.current_contract_address(),
                        pair.clone(),
                        short_in.safe_to_i128(&e),
                    )
                        .into_val(&e),
                },
                sub_invocations: vec![&e],
            }),
        ]);

        let collateral_returned = crate::pair::redeem_pair_as_treasury(&e, &pair, short_in);

        // Transfer the total amount of USDC back to the user
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &treasury,
            &user,
            &collateral_returned.safe_to_i128(&e),
        );

        // Update balances
        let balances = get_pair_balances(&e, &pair);
        set_pair_balances(
            &e,
            &pair,
            &(TreasuryPairBalances {
                token_quote: balances.token_quote.saturating_sub(collateral_returned), // saturated to avoid asserting collateral_returned <= balances.token_quote
                token_long: balances.token_long.saturating_sub(short_in),
                token_short: balances.token_short,
            }),
        );

        Events::new(&e).trade(
            user,
            pair,
            true,
            Side::Short,
            Direction::Sell,
            short_in,
            collateral_returned,
            PRICE_PRECISION.safe_sub(&e, collateral_info.collateral_percent_long), // short price
            fee_config.maker_fee,
            0,
            e.ledger().timestamp(),
        );

        collateral_returned
    }

    fn buy_short_and_redeem(e: Env, user: Address, pair: Address, long_in: u128) -> u128 {
        user.require_auth();

        if long_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);
        let treasury = e.current_contract_address();
        let fee_config = get_fee_config(&e, &pair);

        // Fetch collateral info
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        // Transfer USDC from user to Treasury
        let usdc_required =
            long_in.safe_fixed_mul_floor(&e, collateral_info.collateral_per_pair, PRICE_PRECISION);
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &user,
            &treasury,
            &usdc_required.safe_to_i128(&e),
        );

        // Transfer the long token(s) from the user to the Treasury
        SorobanTokenClient::new(&e, &pair_details.token_long).transfer(
            &user,
            &treasury,
            &long_in.safe_to_i128(&e),
        );

        // Redeem tokens via the Long Short Pair
        e.authorize_as_current_contract(vec![
            &e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: pair_details.token_long.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: (
                        e.current_contract_address(),
                        pair.clone(),
                        long_in.safe_to_i128(&e),
                    )
                        .into_val(&e),
                },
                sub_invocations: vec![&e],
            }),
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: pair_details.token_short.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: (
                        e.current_contract_address(),
                        pair.clone(),
                        long_in.safe_to_i128(&e),
                    )
                        .into_val(&e),
                },
                sub_invocations: vec![&e],
            }),
        ]);

        let collateral_returned = crate::pair::redeem_pair_as_treasury(&e, &pair, long_in);

        // Transfer the total amount of USDC back to the user
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &treasury,
            &user,
            &collateral_returned.safe_to_i128(&e),
        );

        // Update balances
        let balances = get_pair_balances(&e, &pair);
        set_pair_balances(
            &e,
            &pair,
            &(TreasuryPairBalances {
                token_quote: balances.token_quote.saturating_sub(collateral_returned), // saturated to avoid asserting collateral_returned <= balances.token_quote
                token_long: balances.token_long,
                token_short: balances.token_short.saturating_sub(long_in),
            }),
        );

        Events::new(&e).trade(
            user,
            pair,
            true,
            Side::Long,
            Direction::Sell,
            long_in,
            collateral_returned,
            collateral_info.collateral_percent_long, // long price
            fee_config.maker_fee,
            0,
            e.ledger().timestamp(),
        );

        collateral_returned
    }

    fn buy_long(e: Env, user: Address, pair: Address, usdc_in: u128, min_long_out: u128) -> u128 {
        user.require_auth();

        if usdc_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);

        let treasury = e.current_contract_address();
        let fee_config = get_fee_config(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        let (long_out, usdc_fee) =
            crate::price::quote_buy_token(&e, usdc_in, prices.0, fee_config.taker_fee);
        if long_out <= 0 || long_out < min_long_out {
            panic_with_error!(&e, TreasuryError::Slippage);
        }

        // Inventory check: treasury must have enough LONG to sell
        let balances = get_pair_balances(&e, &pair);
        if long_out > balances.token_long {
            panic_with_error!(&e, TreasuryError::InsufficientInventory);
        }

        // Move USDC in (fee stays in treasury because we net it inside quote)
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &user,
            &treasury,
            &usdc_in.safe_to_i128(&e),
        );

        // Move LONG out
        SorobanTokenClient::new(&e, &pair_details.token_long).transfer(
            &treasury,
            &user,
            &long_out.safe_to_i128(&e),
        );

        // Increment protocol fees
        let current_protocol_fees = get_protocol_fees(&e, &pair);
        set_protocol_fees(&e, &pair, &current_protocol_fees.safe_add(&e, usdc_fee));

        // Update balances
        set_pair_balances(
            &e,
            &pair,
            &&(TreasuryPairBalances {
                token_quote: balances
                    .token_quote
                    .safe_add(&e, usdc_in.safe_sub(&e, usdc_fee)),
                token_long: balances.token_long.safe_sub(&e, long_out),
                token_short: balances.token_short,
            }),
        );

        Events::new(&e).trade(
            user,
            pair,
            false,
            Side::Long,
            Direction::Buy,
            usdc_in,
            long_out,
            prices.0,
            fee_config.taker_fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        long_out
    }

    fn sell_long(e: Env, user: Address, pair: Address, long_in: u128, min_usdc_out: u128) -> u128 {
        user.require_auth();

        if long_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);

        let treasury = e.current_contract_address();
        let fee_config = get_fee_config(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        let (usdc_out, usdc_fee) =
            crate::price::quote_sell_token(&e, long_in, prices.0, fee_config.taker_fee);
        if usdc_out <= 0 || usdc_out < min_usdc_out {
            panic_with_error!(&e, TreasuryError::Slippage);
        }

        // Inventory check: treasury must have enough LONG to sell
        let balances = get_pair_balances(&e, &pair);
        if usdc_out > balances.token_quote {
            panic_with_error!(&e, TreasuryError::InsufficientInventory);
        }

        // Move LONG in
        SorobanTokenClient::new(&e, &pair_details.token_long).transfer(
            &user,
            &treasury,
            &long_in.safe_to_i128(&e),
        );

        // Pay USDC out
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &treasury,
            &user,
            &usdc_out.safe_to_i128(&e),
        );

        // Increment protocol fees
        let current_protocol_fees = get_protocol_fees(&e, &pair);
        set_protocol_fees(&e, &pair, &current_protocol_fees.safe_add(&e, usdc_fee));

        // Update balances
        set_pair_balances(
            &e,
            &pair,
            &&(TreasuryPairBalances {
                token_quote: balances.token_quote.safe_sub(&e, usdc_out),
                token_long: balances.token_long.safe_add(&e, long_in),
                token_short: balances.token_short,
            }),
        );

        Events::new(&e).trade(
            user,
            pair,
            false,
            Side::Long,
            Direction::Sell,
            long_in,
            usdc_out,
            prices.0,
            fee_config.taker_fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        usdc_out
    }

    fn buy_short(e: Env, user: Address, pair: Address, usdc_in: u128, min_short_out: u128) -> u128 {
        user.require_auth();

        if usdc_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);

        let treasury = e.current_contract_address();
        let fee_config = get_fee_config(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        let (short_out, usdc_fee) =
            crate::price::quote_buy_token(&e, usdc_in, prices.1, fee_config.taker_fee);
        if short_out <= 0 || short_out < min_short_out {
            panic_with_error!(&e, TreasuryError::Slippage);
        }

        // Inventory check: treasury must have enough SHORT to sell
        let balances = get_pair_balances(&e, &pair);
        if short_out > balances.token_short {
            panic_with_error!(&e, TreasuryError::InsufficientInventory);
        }

        // Move USDC in (fee stays in treasury because we net it inside quote)
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &user,
            &treasury,
            &usdc_in.safe_to_i128(&e),
        );

        // Move LONG out
        SorobanTokenClient::new(&e, &pair_details.token_short).transfer(
            &treasury,
            &user,
            &short_out.safe_to_i128(&e),
        );

        // Increment protocol fees
        let current_protocol_fees = get_protocol_fees(&e, &pair);
        set_protocol_fees(&e, &pair, &current_protocol_fees.safe_add(&e, usdc_fee));

        // Update balances
        set_pair_balances(
            &e,
            &pair,
            &&(TreasuryPairBalances {
                token_quote: balances
                    .token_quote
                    .safe_add(&e, usdc_in.safe_sub(&e, usdc_fee)),
                token_long: balances.token_long,
                token_short: balances.token_short.safe_sub(&e, short_out),
            }),
        );

        Events::new(&e).trade(
            user,
            pair,
            false,
            Side::Short,
            Direction::Buy,
            usdc_in,
            short_out,
            prices.1,
            fee_config.taker_fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        short_out
    }

    fn sell_short(
        e: Env,
        user: Address,
        pair: Address,
        short_in: u128,
        min_usdc_out: u128,
    ) -> u128 {
        user.require_auth();

        if short_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);

        let treasury = e.current_contract_address();
        let fee_config = get_fee_config(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        let (usdc_out, usdc_fee) =
            crate::price::quote_sell_token(&e, short_in, prices.1, fee_config.taker_fee);
        if usdc_out <= 0 || usdc_out < min_usdc_out {
            panic_with_error!(&e, TreasuryError::Slippage);
        }

        // Inventory check: treasury must have enough USDC to buy SHORT
        let balances = get_pair_balances(&e, &pair);
        if usdc_out > balances.token_quote {
            panic_with_error!(&e, TreasuryError::InsufficientInventory);
        }

        // Move SHORT in
        SorobanTokenClient::new(&e, &pair_details.token_short).transfer(
            &user,
            &treasury,
            &short_in.safe_to_i128(&e),
        );

        // Pay USDC out
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &treasury,
            &user,
            &usdc_out.safe_to_i128(&e),
        );

        // Increment protocol fees
        let current_protocol_fees = get_protocol_fees(&e, &pair);
        set_protocol_fees(&e, &pair, &current_protocol_fees.safe_add(&e, usdc_fee));

        // Update balances
        set_pair_balances(
            &e,
            &pair,
            &&(TreasuryPairBalances {
                token_quote: balances.token_quote.safe_sub(&e, usdc_out),
                token_long: balances.token_long,
                token_short: balances.token_short.safe_add(&e, short_in),
            }),
        );

        Events::new(&e).trade(
            user,
            pair,
            false,
            Side::Short,
            Direction::Sell,
            short_in,
            usdc_out,
            prices.1,
            fee_config.taker_fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        usdc_out
    }
}

#[contractimpl]
impl AdminInterfaceTrait for Treasury {
    fn add_pair(
        e: Env,
        admin: Address,
        pair: Address,
        quote_token: Address,
        long_token: Address,
        short_token: Address,
        maker_fee: u128,
        taker_fee: u128,
    ) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        if maker_fee > PRICE_PRECISION || taker_fee > PRICE_PRECISION {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        // Initialize pair details, balances, and fees
        set_pair_details(
            &e,
            &pair,
            &(TreasuryPairDetails {
                pair: pair.clone(),
                token_quote: quote_token,
                token_long: long_token,
                token_short: short_token,
            }),
        );
        set_pair_balances(
            &e,
            &pair,
            &(TreasuryPairBalances {
                token_quote: 0,
                token_long: 0,
                token_short: 0,
            }),
        );
        set_fee_config(
            &e,
            &pair,
            TreasuryFeeConfig {
                maker_fee,
                taker_fee,
            },
        );
    }

    fn set_fee_config(e: Env, admin: Address, pair: Address, maker_fee: u128, taker_fee: u128) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        if maker_fee > PRICE_PRECISION || taker_fee > PRICE_PRECISION {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        crate::storage::set_fee_config(
            &e,
            &pair,
            TreasuryFeeConfig {
                maker_fee,
                taker_fee,
            },
        );
    }

    // Returns the protocol fees accumulated in the pair.
    fn get_protocol_fees(e: Env, pair: Address) -> u128 {
        crate::storage::get_protocol_fees(&e, &pair)
    }

    // Claims the protocol fees accumulated in the pair.
    fn claim_protocol_fees(e: Env, admin: Address, pair: Address, destination: Address) -> u128 {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let pair_details = get_pair_details(&e, &pair);

        let fee = get_protocol_fees(&e, &pair);

        if fee == 0 {
            return 0;
        }

        if fee > 0 {
            SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
                &e.current_contract_address(),
                &destination,
                &fee.safe_to_i128(&e),
            );
            set_protocol_fees(&e, &pair, &0);
            Events::new(&e).claim_protocol_fee(
                pair,
                pair_details.token_quote,
                destination.clone(),
                fee,
            );
        }

        fee
    }

    // Stops the pair deposits instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_deposit(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        set_is_killed_deposit(&e, &true);
        Events::new(&e).kill_deposit();
    }

    // Stops the pair redemptions instantly.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn kill_withdraw(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        set_is_killed_withdraw(&e, &true);
        Events::new(&e).kill_withdraw();
    }

    fn kill_trade(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        set_is_killed_trade(&e, &true);
        Events::new(&e).kill_trade();
    }

    // Resumes the pair deposits.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_deposit(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        set_is_killed_deposit(&e, &false);
        Events::new(&e).unkill_deposit();
    }

    // Resumes the pair redemptions.
    //
    // # Arguments
    //
    // * `admin` - The address of the admin.
    fn unkill_withdraw(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        set_is_killed_withdraw(&e, &false);
        Events::new(&e).unkill_withdraw();
    }

    fn unkill_trade(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        set_is_killed_trade(&e, &false);
        Events::new(&e).unkill_trade();
    }

    // Get create killswitch status.
    fn get_is_killed_deposit(e: Env) -> bool {
        get_is_killed_deposit(&e)
    }

    // Get withdraw killswitch status.
    fn get_is_killed_withdraw(e: Env) -> bool {
        get_is_killed_withdraw(&e)
    }

    // Get trade killswitch status.
    fn get_is_killed_trade(e: Env) -> bool {
        get_is_killed_trade(&e)
    }
}

#[contractimpl]
impl UpgradeableContract for Treasury {
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
        Symbol::new(&e, "Treasury")
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
impl TransferableContract for Treasury {
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
