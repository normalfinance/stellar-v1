use crate::errors::TreasuryError;
use crate::events::{Events, TreasuryEvents};
use crate::interface::{AdminInterfaceTrait, TreasuryTrait};
use crate::storage::{
    get_fee, get_is_killed_deposit, get_is_killed_trade, get_is_killed_withdraw, get_pair_balances,
    get_pair_details, get_protocol_fees, get_total_shares, get_user_shares, set_is_killed_deposit,
    set_is_killed_trade, set_is_killed_withdraw, set_pair_balances, set_pair_details,
    set_protocol_fees, set_total_shares, set_user_shares, TreasuryPairBalances,
    TreasuryPairDetails, TreasuryPairSummary, TreasuryUserPairSummary,
};
// Access
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::management::{MultipleAddressesManagementTrait, SingleAddressManagementTrait};
use access_control::role::Role;
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{contract, contractimpl, contractmeta, panic_with_error, Address, Env};
use utils::math::safe_math::{PrecisionMath, SafeMath};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "");

#[contract]
pub struct Treasury;

#[contractimpl]
impl TreasuryTrait for Treasury {
    fn initialize(e: Env, admin: Address) {
        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, TreasuryError::AlreadyInitialized);
        }
        access_control.set_role_address(&Role::Admin, &admin);
    }

    fn deposit(
        e: Env,
        user: Address,
        pair: Address,
        amt_long: u128,
        amt_short: u128,
        amt_usdc: u128,
    ) -> u128 {
        user.require_auth();

        if amt_long <= 0 && amt_usdc <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if amt_long != amt_short {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if get_is_killed_deposit(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let pair_details = get_pair_details(&e, &pair);

        let treasury = e.current_contract_address();

        let prices = crate::price::get_prices(&e, &pair);

        let balances = get_pair_balances(&e, &pair);

        let tvl_before = crate::price::tvl(&e, &balances, prices.0, prices.1);

        // Transfer tokens
        if amt_long > 0 {
            SorobanTokenClient::new(&e, &pair_details.token_long).transfer(
                &user,
                &treasury,
                &(amt_long as i128),
            );
            SorobanTokenClient::new(&e, &pair_details.token_short).transfer(
                &user,
                &treasury,
                &(amt_short as i128),
            );
        }
        if amt_usdc > 0 {
            SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
                &user,
                &treasury,
                &(amt_usdc as i128),
            );
        }

        // Update balances
        set_pair_balances(
            &e,
            &pair,
            &(TreasuryPairBalances {
                token_quote: balances.token_quote.safe_add(&e, amt_usdc),
                token_long: balances.token_long.safe_add(&e, amt_long),
                token_short: balances.token_short.safe_add(&e, amt_short),
            }),
        );

        let value_deposit = amt_long.safe_add(&e, amt_usdc);
        let total_shares = get_total_shares(&e, &pair);

        let shares_to_mint = if total_shares == 0 {
            value_deposit
        } else {
            if tvl_before <= 0 {
                panic_with_error!(&e, TreasuryError::ZeroTvl);
            }
            value_deposit
                .safe_mul(&e, total_shares)
                .safe_div(&e, tvl_before)
        };

        // Update total shares
        let new_total_shares = total_shares.safe_add(&e, shares_to_mint);
        set_total_shares(&e, &pair, new_total_shares);

        // Update user shares
        let user_shares = get_user_shares(&e, &pair, &user);
        let new_user_shares = user_shares.safe_add(&e, shares_to_mint);
        set_user_shares(&e, &pair, &user, new_user_shares);

        // Events::new(&e).deposit(current_time, user, pair, amt_long, new_total_shares);

        shares_to_mint
    }

    fn withdraw(
        e: Env,
        user: Address,
        pair: Address,
        shares: u128,
        min_usdc: u128,
        min_long: u128,
        min_short: u128,
    ) -> (u128, u128, u128) {
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

        // Get treasury balances
        let balances = get_pair_balances(&e, &pair);

        // Compute out amounts using share proportion
        let out_usdc = balances
            .token_quote
            .safe_mul(&e, shares)
            .safe_div(&e, total_shares);
        let out_long = balances
            .token_long
            .safe_mul(&e, shares)
            .safe_div(&e, total_shares);
        let out_short = balances
            .token_short
            .safe_mul(&e, shares)
            .safe_div(&e, total_shares);

        if out_usdc < min_usdc || out_long < min_long || out_short < min_short {
            panic_with_error!(&e, TreasuryError::Slippage);
        }

        // Update total shares
        let new_total_shares = total_shares.safe_sub(&e, shares);
        set_total_shares(&e, &pair, new_total_shares);

        // Updates user shares
        let new_user_shares = user_shares.safe_sub(&e, shares);
        set_user_shares(&e, &pair, &user, new_user_shares);

        // Move tokens
        if out_usdc > 0 {
            SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
                &treasury,
                &user,
                &(out_usdc as i128),
            );
        }
        if out_long > 0 {
            SorobanTokenClient::new(&e, &pair_details.token_long).transfer(
                &treasury,
                &user,
                &(out_long as i128),
            );
        }
        if out_short > 0 {
            SorobanTokenClient::new(&e, &pair_details.token_short).transfer(
                &treasury,
                &user,
                &(out_short as i128),
            );
        }

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

        (out_usdc, out_long, out_short)
    }

    /**
     * Trading
     */

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
        let fee = get_fee(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        let (long_out, usdc_fee) = crate::price::quote_buy_token(&e, usdc_in, prices.0, fee);
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
            &(usdc_in as i128),
        );

        // Move LONG out
        SorobanTokenClient::new(&e, &pair_details.token_long).transfer(
            &treasury,
            &user,
            &(long_out as i128),
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
        let fee = get_fee(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        let (usdc_out, usdc_fee) = crate::price::quote_sell_token(&e, long_in, prices.0, fee);
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
            &(long_in as i128),
        );

        // Pay USDC out
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &treasury,
            &user,
            &(usdc_out as i128),
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
        let fee = get_fee(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        let (short_out, usdc_fee) = crate::price::quote_buy_token(&e, usdc_in, prices.1, fee);
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
            &(usdc_in as i128),
        );

        // Move LONG out
        SorobanTokenClient::new(&e, &pair_details.token_short).transfer(
            &treasury,
            &user,
            &(short_out as i128),
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
        let fee = get_fee(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        let (usdc_out, usdc_fee) = crate::price::quote_sell_token(&e, short_in, prices.1, fee);
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
            &(short_in as i128),
        );

        // Pay USDC out
        SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
            &treasury,
            &user,
            &(usdc_out as i128),
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
        usdc_out
    }

    fn get_pair_details(e: Env, pair: Address) -> TreasuryPairDetails {
        crate::storage::get_pair_details(&e, &pair)
    }

    fn get_tvl(e: Env, pair: Address) -> u128 {
        let prices = crate::price::get_prices(&e, &pair);
        let balances = get_pair_balances(&e, &pair);
        crate::price::tvl(&e, &balances, prices.0, prices.1)
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

    fn get_pair_fee(e: Env, pair: Address) -> u128 {
        crate::storage::get_fee(&e, &pair)
    }

    fn get_pair_summary(e: Env, pair: Address) -> TreasuryPairSummary {
        let balances = crate::storage::get_pair_balances(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        TreasuryPairSummary {
            details: crate::storage::get_pair_details(&e, &pair),
            balances: crate::storage::get_pair_balances(&e, &pair),
            prices,
            tvl: crate::price::tvl(&e, &balances, prices.0, prices.1),
            total_shares: crate::storage::get_total_shares(&e, &pair),
            fee: crate::storage::get_fee(&e, &pair),
        }
    }

    fn get_user_with_pair_summary(e: Env, pair: Address, user: Address) -> TreasuryUserPairSummary {
        let balances = crate::storage::get_pair_balances(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        TreasuryUserPairSummary {
            pair_summary: TreasuryPairSummary {
                details: crate::storage::get_pair_details(&e, &pair),
                balances: crate::storage::get_pair_balances(&e, &pair),
                prices,
                tvl: crate::price::tvl(&e, &balances, prices.0, prices.1),
                total_shares: crate::storage::get_total_shares(&e, &pair),
                fee: crate::storage::get_fee(&e, &pair),
            },
            user_shares: get_user_shares(&e, &pair, &user),
        }
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
    ) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        // Init details
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

        // Init balances
        set_pair_balances(
            &e,
            &pair,
            &(TreasuryPairBalances {
                token_quote: 0,
                token_long: 0,
                token_short: 0,
            }),
        );
    }

    fn set_pair_fee(e: Env, admin: Address, pair: Address, fee: u128) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        // TODO: add validation
        crate::storage::set_fee(&e, &pair, fee);
    }

    // Returns the protocol fees accumulated in the pair.
    fn get_protocol_fees(e: Env, pair: Address) -> u128 {
        crate::storage::get_protocol_fees(&e, &pair)
    }

    // Claims the protocol fees accumulated in the pair.
    fn claim_protocol_fees(e: Env, admin: Address, pair: Address, destination: Address) -> u128 {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        let pair_details = get_pair_details(&e, &pair);

        let fee = get_protocol_fees(&e, &pair);

        if fee == 0 {
            return 0;
        }

        if fee > 0 {
            SorobanTokenClient::new(&e, &pair_details.token_quote).transfer(
                &e.current_contract_address(),
                &destination,
                &(fee as i128),
            );
            set_protocol_fees(&e, &pair, &0);
            // Events::new(&e).claim_protocol_fee(pair_details.token_quote, destination.clone(), fee);
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
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

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
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        set_is_killed_withdraw(&e, &true);
        Events::new(&e).kill_withdraw();
    }

    fn kill_trade(e: Env, admin: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

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
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

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
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

        set_is_killed_withdraw(&e, &false);
        Events::new(&e).unkill_withdraw();
    }

    fn unkill_trade(e: Env, admin: Address) {
        admin.require_auth();
        let access_control = AccessControl::new(&e);
        access_control.assert_address_has_role(&admin, &Role::Admin);

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
