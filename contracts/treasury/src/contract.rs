use crate::errors::TreasuryError;
use crate::events::{Events, TreasuryEvents};
use crate::interface::{AdminInterfaceTrait, TradingTrait, TreasuryTrait};
use crate::storage::{
    PairConfig, TreasuryFeeConfig, TreasuryRiskParameters, TreasurySummary, TreasuryUserSummary,
};
use soroban_sdk::token::TokenClient as SorobanTokenClient;
use soroban_sdk::{
    contract, contractimpl, contractmeta, panic_with_error, Address, BytesN, Env, Symbol, Vec,
};
use types::pair::{Direction, PairAmountsWithUSDC, Side};
use utils::constant::{MAX_BASE_FEE, MAX_BOUND_POWER, PRICE_PRECISION};
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
    val = "A primary market for Long Short Pair tokens with accurate price enforcement and LP protections"
);

#[contract]
pub struct Treasury;

#[contractimpl]
impl Treasury {
    /// Initializes the Treasury contract.
    ///
    /// This constructor is intended to be called exactly once at deploy time. It:
    /// - Sets up core admin roles (`Admin`, `PauseAdmin`, `EmergencyAdmin`) to the provided `admin`
    /// - Stores the canonical USDC oracle used by the Treasury
    ///
    /// ### Reverts
    /// - [`TreasuryError::AlreadyInitialized`] if the contract has already been initialized.
    ///
    /// ### Arguments
    /// - `e`: Soroban environment.
    /// - `admin`: Address to assign administrative roles to.
    /// - `oracle`: Address of the Normal Oracle contract used for USDC pricing.
    pub fn __constructor(e: Env, admin: Address, oracle: Address) {
        // NOTE: constructor auth is intentionally omitted because deployer auth patterns can vary.
        // If you want strict deploy-time auth, uncomment the line below:
        // admin.require_auth();

        let access_control = AccessControl::new(&e);
        if access_control.get_role_safe(&Role::Admin).is_some() {
            panic_with_error!(&e, TreasuryError::AlreadyInitialized);
        }

        access_control.set_role_address(&Role::Admin, &admin);
        access_control.set_role_address(&Role::PauseAdmin, &admin);
        access_control.set_role_address(&Role::EmergencyAdmin, &admin);

        crate::storage::set_oracle(&e, &oracle);
    }
}

#[contractimpl]
impl TreasuryTrait for Treasury {
    /// Deposits LP collateral into the Treasury in exchange for newly-minted LP shares.
    ///
    /// A deposit consists of:
    /// - `pairs_to_deposit` LONG tokens
    /// - `pairs_to_deposit` SHORT tokens
    /// - `pairs_to_deposit * collateral_per_pair` USDC
    ///
    /// Shares are minted proportional to the deposit's NAV relative to the Treasury's total NAV.
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if `pairs_to_deposit == 0`.
    /// - [`TreasuryError::ActionPaused`] if deposits are paused.
    ///
    /// ### Returns
    /// Returns the number of LP shares minted for the depositor.
    fn deposit(e: Env, user: Address, pair: Address, pairs_to_deposit: u128) -> u128 {
        user.require_auth();

        if pairs_to_deposit <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if crate::storage::get_is_killed_deposit(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let config = crate::storage::get_config(&e, &pair);

        // Pull collateral parameters (e.g., collateral_per_pair) from the Pair contract.
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        // Snapshot current state for pricing/share math.
        let balances = crate::storage::get_balances(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);
        let nav = crate::lp::nav(&e, &balances, &prices);

        // Convert deposit into NAV, then NAV into newly minted shares.
        let total_shares = crate::storage::get_total_shares(&e, &pair);
        let deposit_nav = crate::lp::pairs_to_nav(
            &e,
            pairs_to_deposit,
            collateral_info.collateral_per_pair,
            &prices,
        );
        let shares_to_mint = crate::lp::nav_to_shares(&e, &pair, nav, deposit_nav, total_shares);

        // Transfers happen before state writes so we only mutate storage after successful token movement.
        let treasury = e.current_contract_address();

        // Move LONG and SHORT into the Treasury.
        SorobanTokenClient::new(&e, &config.long).transfer(
            &user,
            &treasury,
            &pairs_to_deposit.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &config.short).transfer(
            &user,
            &treasury,
            &pairs_to_deposit.safe_to_i128(&e),
        );

        // Compute and transfer the required USDC collateral.
        let required_usdc = pairs_to_deposit.safe_fixed_mul_floor(
            &e,
            collateral_info.collateral_per_pair,
            PRICE_PRECISION,
        );
        SorobanTokenClient::new(&e, &config.usdc).transfer(
            &user,
            &treasury,
            &required_usdc.safe_to_i128(&e),
        );

        // Update share accounting.
        let new_total_shares = total_shares.safe_add(&e, shares_to_mint);
        crate::storage::set_total_shares(&e, &pair, new_total_shares);

        let user_shares = crate::storage::get_user_shares(&e, &pair, &user, true);
        let new_user_shares = user_shares.safe_add(&e, shares_to_mint);
        crate::storage::set_user_shares(&e, &pair, &user, new_user_shares);

        // Update Treasury balances to reflect assets received.
        crate::storage::set_balances(
            &e,
            &pair,
            &(PairAmountsWithUSDC {
                long: balances.long.safe_add(&e, pairs_to_deposit),
                short: balances.short.safe_add(&e, pairs_to_deposit),
                usdc: balances.usdc.safe_add(&e, required_usdc),
            }),
        );

        Events::new(&e).deposit(
            user,
            pair,
            pairs_to_deposit,
            required_usdc,
            total_shares,
            new_total_shares,
            user_shares,
            new_user_shares,
            e.ledger().timestamp(),
        );

        shares_to_mint
    }

    /// Withdraws LP collateral from the Treasury by burning LP shares.
    ///
    /// The caller specifies the number of shares to burn. The Treasury computes the
    /// proportional amount of LONG/SHORT/USDC owed, validates safety constraints, and
    /// transfers the tokens to the user.
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if `shares == 0`.
    /// - [`TreasuryError::ActionPaused`] if withdrawals are paused.
    /// - [`TreasuryError::InsufficientShares`] if `shares > user_shares` or `total_shares == 0`.
    ///
    /// ### Returns
    /// Returns the token amounts withdrawn: `{ long, short, usdc }`.
    fn withdraw(e: Env, user: Address, pair: Address, shares: u128) -> PairAmountsWithUSDC {
        user.require_auth();

        if shares <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if crate::storage::get_is_killed_withdraw(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let config = crate::storage::get_config(&e, &pair);

        let total_shares = crate::storage::get_total_shares(&e, &pair);
        // return_zero = false so we revert if the user never initialized shares for this pair.
        let user_shares = crate::storage::get_user_shares(&e, &pair, &user, false);

        if shares > user_shares || total_shares <= 0 {
            panic_with_error!(&e, TreasuryError::InsufficientShares);
        }

        // Convert shares into token amounts, validating that the withdrawal is safe for LPs.
        let balances = crate::storage::get_balances(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);
        let tokens_out =
            crate::lp::validate_lp_withdrawal(&e, &balances, &prices, total_shares, shares);

        // Burn shares in storage.
        let new_total_shares = total_shares.safe_sub(&e, shares);
        crate::storage::set_total_shares(&e, &pair, new_total_shares);

        let new_user_shares = user_shares.safe_sub(&e, shares);
        crate::storage::set_user_shares(&e, &pair, &user, new_user_shares);

        // Transfers happen after checks but before balance writes for correctness.
        let treasury = e.current_contract_address();

        SorobanTokenClient::new(&e, &config.usdc).transfer(
            &treasury,
            &user,
            &tokens_out.usdc.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &config.long).transfer(
            &treasury,
            &user,
            &tokens_out.long.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &config.short).transfer(
            &treasury,
            &user,
            &tokens_out.short.safe_to_i128(&e),
        );

        // Update stored balances to reflect assets paid out.
        crate::storage::set_balances(
            &e,
            &pair,
            &(PairAmountsWithUSDC {
                long: balances.long.safe_sub(&e, tokens_out.long),
                short: balances.short.safe_sub(&e, tokens_out.short),
                usdc: balances.usdc.safe_sub(&e, tokens_out.usdc),
            }),
        );

        Events::new(&e).withdraw(
            user,
            pair,
            shares,
            tokens_out.long,
            tokens_out.short,
            tokens_out.usdc,
            total_shares,
            new_total_shares,
            user_shares,
            new_user_shares,
            e.ledger().timestamp(),
        );

        tokens_out
    }

    /// Returns the stored token configuration for a given Pair.
    ///
    /// This includes the contract addresses for the Pair, LONG, SHORT, and collateral (USDC) tokens.
    fn get_config(e: Env, pair: Address) -> PairConfig {
        crate::storage::get_config(&e, &pair)
    }

    /// Returns the current oracle prices for the given Pair.
    ///
    /// Prices are returned in the same precision as `PRICE_PRECISION`.
    fn get_prices(e: Env, pair: Address) -> PairAmountsWithUSDC {
        crate::price::get_prices(&e, &pair)
    }

    /// Returns the Treasury’s internal accounting balances for the given Pair.
    ///
    /// These balances represent the Treasury’s tracked inventory and may be used for quoting,
    /// risk checks, and LP accounting.
    fn get_balances(e: Env, pair: Address) -> PairAmountsWithUSDC {
        crate::storage::get_balances(&e, &pair)
    }

    /// Returns the total LP shares outstanding for the given Pair.
    fn get_total_shares(e: Env, pair: Address) -> u128 {
        crate::storage::get_total_shares(&e, &pair)
    }

    /// Returns the LP share balance for `user` for the given Pair.
    ///
    /// Missing records default to zero for read-only UX.
    fn get_user_shares(e: Env, pair: Address, user: Address) -> u128 {
        crate::storage::get_user_shares(&e, &pair, &user, true)
    }

    /// Returns the current fee configuration for the given Pair.
    fn get_fee_config(e: Env, pair: Address) -> TreasuryFeeConfig {
        crate::storage::get_fee_config(&e, &pair)
    }

    /// Returns a snapshot of the Treasury state for the given Pair.
    ///
    /// This is a convenience method for frontends/indexers to avoid multiple round-trips.
    fn get_summary(e: Env, pair: Address) -> TreasurySummary {
        TreasurySummary {
            config: crate::storage::get_config(&e, &pair),
            balances: crate::storage::get_balances(&e, &pair),
            prices: crate::price::get_prices(&e, &pair),
            total_shares: crate::storage::get_total_shares(&e, &pair),
            fee_config: crate::storage::get_fee_config(&e, &pair),
        }
    }

    /// Returns a snapshot of the Treasury state for a Pair plus the user's share balance.
    ///
    /// This is a convenience method for frontends/indexers to fetch summary + user position in one call.
    fn get_user_with_summary(e: Env, pair: Address, user: Address) -> TreasuryUserSummary {
        TreasuryUserSummary {
            summary: TreasurySummary {
                config: crate::storage::get_config(&e, &pair),
                balances: crate::storage::get_balances(&e, &pair),
                prices: crate::price::get_prices(&e, &pair),
                total_shares: crate::storage::get_total_shares(&e, &pair),
                fee_config: crate::storage::get_fee_config(&e, &pair),
            },
            user_shares: crate::storage::get_user_shares(&e, &pair, &user, true),
        }
    }
}

#[contractimpl]
impl TradingTrait for Treasury {
    /// Estimates a trade output and fee without executing any token transfers.
    ///
    /// This helper is intended for quoting in UIs and off-chain routing. It uses the same
    /// pricing + fee model as the on-chain trade methods.
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if `amount_in == 0`.
    /// - Reverts if `pair` is not configured (via `get_config`).
    ///
    /// ### Returns
    /// Returns `(amount_out, usdc_fee)`.
    fn estimate_trade(
        e: Env,
        pair: Address,
        direction: Direction,
        side: Side,
        amount_in: u128,
    ) -> (u128, u128) {
        if amount_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        // Ensure the pair exists / is configured. The return value isn't needed here.
        crate::storage::get_config(&e, &pair);

        let fee_config = crate::storage::get_fee_config(&e, &pair);
        let balances = crate::storage::get_balances(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);
        let fee = crate::fees::calculate_fee(
            &e,
            Side::Long,
            &fee_config,
            &balances,
            &prices,
            collateral_info.collateral_percent_long,
        );

        if direction == Direction::Buy && side == Side::Long {
            return crate::price::quote_buy_token(&e, amount_in, prices.long, fee);
        }

        if direction == Direction::Buy && side == Side::Short {
            return crate::price::quote_buy_token(&e, amount_in, prices.short, fee);
        }

        if direction == Direction::Sell && side == Side::Long {
            return crate::price::quote_sell_token(&e, amount_in, prices.long, fee);
        }

        if direction == Direction::Sell && side == Side::Short {
            return crate::price::quote_sell_token(&e, amount_in, prices.short, fee);
        }

        panic_with_error!(&e, TreasuryError::InvalidInput)
    }

    /// Buys **LONG** tokens from the Treasury using USDC.
    ///
    /// The caller supplies `usdc_in` and receives `long_out` quoted from the current
    /// oracle price and fee model. This trade is executed **against Treasury inventory**:
    /// the Treasury must already hold sufficient LONG to deliver the trade.
    ///
    /// ### Pricing & Fees
    /// - Uses `prices.long` from the oracle.
    /// - Computes a dynamic taker fee via `calculate_fee(...)`.
    /// - `quote_buy_token` returns `(long_out, usdc_fee)`; `usdc_fee` is retained and tracked.
    ///
    /// ### Slippage Protection
    /// Reverts if `long_out < min_long_out`.
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if `usdc_in == 0`.
    /// - [`TreasuryError::ActionPaused`] if trading is paused.
    /// - [`TreasuryError::Slippage`] if output is zero or below `min_long_out`.
    /// - [`TreasuryError::InsufficientInventory`] if the Treasury lacks LONG inventory.
    ///
    /// ### Returns
    /// Returns the amount of LONG transferred to the user.
    fn buy_long(e: Env, user: Address, pair: Address, usdc_in: u128, min_long_out: u128) -> u128 {
        user.require_auth();

        if usdc_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if crate::storage::get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let config = crate::storage::get_config(&e, &pair);
        let treasury = e.current_contract_address();

        // Snapshot state used for pricing + fee calculation.
        let fee_config = crate::storage::get_fee_config(&e, &pair);
        let balances = crate::storage::get_balances(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        let fee = crate::fees::calculate_fee(
            &e,
            Side::Long,
            &fee_config,
            &balances,
            &prices,
            collateral_info.collateral_percent_long,
        );

        let (long_out, usdc_fee) = crate::price::quote_buy_token(&e, usdc_in, prices.long, fee);

        // Ensure the Treasury has enough LONG to fulfill this purchase.
        if long_out <= 0 || long_out < min_long_out {
            panic_with_error!(&e, TreasuryError::Slippage);
        }

        // Inventory check: treasury must have enough LONG to sell
        if long_out > balances.long {
            panic_with_error!(&e, TreasuryError::InsufficientInventory);
        }

        // Token movements first (atomic execution ensures revert on failure).
        SorobanTokenClient::new(&e, &config.usdc).transfer(
            &user,
            &treasury,
            &usdc_in.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &config.long).transfer(
            &treasury,
            &user,
            &long_out.safe_to_i128(&e),
        );

        // Track protocol fee in USDC.
        let current_protocol_fees = crate::storage::get_protocol_fees(&e, &pair);
        crate::storage::set_protocol_fees(&e, &pair, &current_protocol_fees.safe_add(&e, usdc_fee));

        // Persist updated accounting balances.
        crate::storage::set_balances(
            &e,
            &pair,
            &&(PairAmountsWithUSDC {
                long: balances.long.safe_sub(&e, long_out),
                short: balances.short,
                usdc: balances.usdc.safe_add(&e, usdc_in.safe_sub(&e, usdc_fee)),
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
            prices.long,
            fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        long_out
    }

    /// Sells **LONG** tokens to the Treasury in exchange for USDC.
    ///
    /// The caller supplies `long_in` and receives `usdc_out` quoted from the current
    /// oracle price and fee model. This trade is executed **against Treasury USDC inventory**:
    /// the Treasury must have sufficient USDC to pay out.
    ///
    /// Additional safety checks apply to protect LPs:
    /// - LONG-side trades may be blocked as "toxic"
    /// - USDC floor constraints prevent draining too much collateral in a single trade
    ///
    /// ### Slippage Protection
    /// Reverts if `usdc_out < min_usdc_out`.
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if `long_in == 0`.
    /// - [`TreasuryError::ActionPaused`] if trading is paused.
    /// - [`TreasuryError::ToxicSideNotAccepted`] if risk logic blocks LONG sells.
    /// - [`TreasuryError::Slippage`] if output is zero or below `min_usdc_out`.
    /// - [`TreasuryError::InsufficientInventory`] if the Treasury lacks USDC to pay out.
    /// - [`TreasuryError::CannotPassFloor`] if paying out violates the USDC floor.
    ///
    /// ### Returns
    /// Returns the amount of USDC transferred to the user.
    fn sell_long(e: Env, user: Address, pair: Address, long_in: u128, min_usdc_out: u128) -> u128 {
        user.require_auth();

        if long_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if crate::storage::get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let config = crate::storage::get_config(&e, &pair);
        let treasury = e.current_contract_address();

        // Pull oracle prices early for toxicity checks.
        let fee_config = crate::storage::get_fee_config(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        // Block the trade if LONG is currently considered toxic.
        crate::risk::block_toxic_trades(&e, &pair, Side::Long, prices.long);

        // Snapshot state used for pricing + fee calculation.
        let balances = crate::storage::get_balances(&e, &pair);
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        let fee = crate::fees::calculate_fee(
            &e,
            Side::Long,
            &fee_config,
            &balances,
            &prices,
            collateral_info.collateral_percent_long,
        );

        let (usdc_out, usdc_fee) = crate::price::quote_sell_token(&e, long_in, prices.long, fee);

        if usdc_out <= 0 || usdc_out < min_usdc_out {
            panic_with_error!(&e, TreasuryError::Slippage);
        }

        // Ensure the Treasury has enough USDC to pay out.
        if usdc_out > balances.usdc {
            panic_with_error!(&e, TreasuryError::InsufficientInventory);
        }

        // Prevent a single trade from pushing the Treasury below its configured USDC floor.
        let nav = crate::lp::nav(&e, &balances, &prices);
        crate::risk::validate_usdc_floor(&e, balances.usdc, usdc_out, nav, prices.usdc);

        // Token movements: receive LONG, pay USDC.
        SorobanTokenClient::new(&e, &config.long).transfer(
            &user,
            &treasury,
            &long_in.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &config.usdc).transfer(
            &treasury,
            &user,
            &usdc_out.safe_to_i128(&e),
        );

        // Track protocol fee in USDC.
        let current_protocol_fees = crate::storage::get_protocol_fees(&e, &pair);
        crate::storage::set_protocol_fees(&e, &pair, &current_protocol_fees.safe_add(&e, usdc_fee));

        // Persist updated accounting balances.
        crate::storage::set_balances(
            &e,
            &pair,
            &&(PairAmountsWithUSDC {
                long: balances.long.safe_add(&e, long_in),
                short: balances.short,
                usdc: balances.usdc.safe_sub(&e, usdc_out),
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
            prices.long,
            fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        usdc_out
    }

    /// Buys **SHORT** tokens from the Treasury using USDC.
    ///
    /// The caller supplies `usdc_in` and receives `short_out` quoted from the current
    /// oracle price and fee model. This trade is executed **against Treasury inventory**:
    /// the Treasury must already hold sufficient SHORT to deliver the trade.
    ///
    /// ### Pricing & Fees
    /// - Uses `prices.short` from the oracle.
    /// - Computes a dynamic taker fee via `calculate_fee(...)`.
    /// - `quote_buy_token` returns `(short_out, usdc_fee)`; `usdc_fee` is retained and tracked.
    ///
    /// ### Slippage Protection
    /// Reverts if `short_out < min_short_out`.
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if `usdc_in == 0`.
    /// - [`TreasuryError::ActionPaused`] if trading is paused.
    /// - [`TreasuryError::Slippage`] if output is zero or below `min_short_out`.
    /// - [`TreasuryError::InsufficientInventory`] if the Treasury lacks SHORT inventory.
    ///
    /// ### Returns
    /// Returns the amount of SHORT transferred to the user.
    fn buy_short(e: Env, user: Address, pair: Address, usdc_in: u128, min_short_out: u128) -> u128 {
        user.require_auth();

        if usdc_in <= 0 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if crate::storage::get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let config = crate::storage::get_config(&e, &pair);
        let treasury = e.current_contract_address();

        // Snapshot state used for pricing + fee calculation.
        let fee_config = crate::storage::get_fee_config(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);
        let balances = crate::storage::get_balances(&e, &pair);
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        let fee = crate::fees::calculate_fee(
            &e,
            Side::Long,
            &fee_config,
            &balances,
            &prices,
            collateral_info.collateral_percent_long,
        );

        let (short_out, usdc_fee) = crate::price::quote_buy_token(&e, usdc_in, prices.short, fee);

        if short_out <= 0 || short_out < min_short_out {
            panic_with_error!(&e, TreasuryError::Slippage);
        }

        // Ensure the Treasury has enough SHORT to fulfill this purchase.
        if short_out > balances.short {
            panic_with_error!(&e, TreasuryError::InsufficientInventory);
        }

        // Token movements first.
        SorobanTokenClient::new(&e, &config.usdc).transfer(
            &user,
            &treasury,
            &usdc_in.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &config.short).transfer(
            &treasury,
            &user,
            &short_out.safe_to_i128(&e),
        );

        // Track protocol fee in USDC.
        let current_protocol_fees = crate::storage::get_protocol_fees(&e, &pair);
        crate::storage::set_protocol_fees(&e, &pair, &current_protocol_fees.safe_add(&e, usdc_fee));

        // Persist updated accounting balances.
        crate::storage::set_balances(
            &e,
            &pair,
            &&(PairAmountsWithUSDC {
                long: balances.long,
                short: balances.short.safe_sub(&e, short_out),
                usdc: balances.usdc.safe_add(&e, usdc_in.safe_sub(&e, usdc_fee)),
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
            prices.short,
            fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        short_out
    }

    /// Sells **SHORT** tokens to the Treasury in exchange for USDC.
    ///
    /// The caller supplies `short_in` and receives `usdc_out` quoted from the current
    /// oracle price and fee model. This trade is executed **against Treasury USDC inventory**:
    /// the Treasury must have sufficient USDC to pay out.
    ///
    /// Additional safety checks apply to protect LPs:
    /// - SHORT-side trades may be blocked as "toxic"
    /// - USDC floor constraints prevent draining too much collateral in a single trade
    ///
    /// ### Slippage Protection
    /// Reverts if `usdc_out < min_usdc_out`.
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if `short_in == 0`.
    /// - [`TreasuryError::ActionPaused`] if trading is paused.
    /// - [`TreasuryError::ToxicSideNotAccepted`] if risk logic blocks SHORT sells.
    /// - [`TreasuryError::Slippage`] if output is zero or below `min_usdc_out`.
    /// - [`TreasuryError::InsufficientInventory`] if the Treasury lacks USDC to pay out.
    /// - [`TreasuryError::CannotPassFloor`] if paying out violates the USDC floor.
    ///
    /// ### Returns
    /// Returns the amount of USDC transferred to the user.
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

        if crate::storage::get_is_killed_trade(&e) {
            panic_with_error!(&e, TreasuryError::ActionPaused);
        }

        let config = crate::storage::get_config(&e, &pair);
        let treasury = e.current_contract_address();

        let fee_config = crate::storage::get_fee_config(&e, &pair);
        let prices = crate::price::get_prices(&e, &pair);

        // Block the trade if SHORT is currently considered toxic.
        crate::risk::block_toxic_trades(&e, &pair, Side::Short, prices.short);

        // Snapshot state used for pricing + fee calculation.
        let balances = crate::storage::get_balances(&e, &pair);
        let collateral_info = crate::pair::get_pair_collateral_info(&e, &pair);

        let fee = crate::fees::calculate_fee(
            &e,
            Side::Long,
            &fee_config,
            &balances,
            &prices,
            collateral_info.collateral_percent_long,
        );

        let (usdc_out, usdc_fee) = crate::price::quote_sell_token(&e, short_in, prices.short, fee);

        if usdc_out <= 0 || usdc_out < min_usdc_out {
            panic_with_error!(&e, TreasuryError::Slippage);
        }

        // Ensure the Treasury has enough USDC to pay out.
        if usdc_out > balances.usdc {
            panic_with_error!(&e, TreasuryError::InsufficientInventory);
        }

        // Prevent a single trade from pushing the Treasury below its configured USDC floor.
        let nav = crate::lp::nav(&e, &balances, &prices);
        crate::risk::validate_usdc_floor(&e, balances.usdc, usdc_out, nav, prices.usdc);

        // Token movements: receive SHORT, pay USDC.
        SorobanTokenClient::new(&e, &config.short).transfer(
            &user,
            &treasury,
            &short_in.safe_to_i128(&e),
        );
        SorobanTokenClient::new(&e, &config.usdc).transfer(
            &treasury,
            &user,
            &usdc_out.safe_to_i128(&e),
        );

        // Track protocol fee in USDC.
        let current_protocol_fees = crate::storage::get_protocol_fees(&e, &pair);
        crate::storage::set_protocol_fees(&e, &pair, &current_protocol_fees.safe_add(&e, usdc_fee));

        // Persist updated accounting balances.
        crate::storage::set_balances(
            &e,
            &pair,
            &&(PairAmountsWithUSDC {
                long: balances.long,
                short: balances.short.safe_add(&e, short_in),
                usdc: balances.usdc.safe_sub(&e, usdc_out),
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
            prices.short,
            fee,
            usdc_fee,
            e.ledger().timestamp(),
        );

        usdc_out
    }
}

#[contractimpl]
impl AdminInterfaceTrait for Treasury {
    /// Registers a new Pair with the Treasury and initializes its state.
    ///
    /// This call:
    /// - Stores the token addresses for LONG/SHORT/USDC from the Pair contract
    /// - Initializes balances, shares, risk parameters, fee config, and protocol fee counters
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if fee fields are out of range.
    fn add_pair(e: Env, admin: Address, pair: Address, fee_config: TreasuryFeeConfig) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        if fee_config.maker_base_fee > PRICE_PRECISION
            || fee_config.taker_base_fee > PRICE_PRECISION
        {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        let tokens = crate::pair::get_pair_tokens(&e, &pair);

        crate::storage::set_total_shares(&e, &pair, 0);

        crate::storage::set_config(
            &e,
            &pair,
            &(PairConfig {
                pair: pair.clone(),
                long: tokens.long,
                short: tokens.short,
                usdc: tokens.collateral,
            }),
        );

        crate::storage::set_balances(
            &e,
            &pair,
            &(PairAmountsWithUSDC {
                long: 0,
                short: 0,
                usdc: 0,
            }),
        );

        crate::storage::set_risk_parameters(
            &e,
            &pair,
            &(TreasuryRiskParameters {
                toxic_threshold: PRICE_PRECISION / 10, // 10%,
            }),
        );

        crate::storage::set_fee_config(&e, &pair, &fee_config);
        crate::storage::set_protocol_fees(&e, &pair, &0);
    }

    /// Updates the fee configuration for an existing Pair.
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if any configured fee exceeds its maximum bound.
    fn set_fee_config(e: Env, admin: Address, pair: Address, config: TreasuryFeeConfig) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        if config.maker_base_fee > MAX_BASE_FEE || config.taker_base_fee > MAX_BASE_FEE {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        if config.bound_power > MAX_BOUND_POWER {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        crate::storage::set_fee_config(&e, &pair, &config);
    }

    /// Sets the global USDC floor fraction used by risk checks.
    ///
    /// The floor fraction is expressed in `PRICE_PRECISION` units (e.g. `0.10e7` for 10%).
    ///
    /// ### Reverts
    /// - [`TreasuryError::InvalidInput`] if `floor_fraction` is zero or below the minimum allowed.
    fn set_usdc_floor(e: Env, admin: Address, floor_fraction: u128) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        if 0 == floor_fraction || floor_fraction < PRICE_PRECISION / 10 {
            panic_with_error!(&e, TreasuryError::InvalidInput);
        }

        crate::storage::set_usdc_floor_fraction(&e, &floor_fraction);
    }

    /// Returns the protocol fees accumulated for the given Pair (denominated in USDC).
    fn get_protocol_fees(e: Env, pair: Address) -> u128 {
        crate::storage::get_protocol_fees(&e, &pair)
    }

    /// Transfers accumulated protocol fees for `pair` to `destination`.
    ///
    /// ### Reverts
    /// - Reverts if `admin` is not authorized.
    ///
    /// ### Returns
    /// Returns the amount of fees transferred.
    fn claim_protocol_fees(e: Env, admin: Address, pair: Address, destination: Address) -> u128 {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let config = crate::storage::get_config(&e, &pair);
        let fee = crate::storage::get_protocol_fees(&e, &pair);

        if fee == 0 {
            return 0;
        }

        SorobanTokenClient::new(&e, &config.usdc).transfer(
            &e.current_contract_address(),
            &destination,
            &fee.safe_to_i128(&e),
        );

        crate::storage::set_protocol_fees(&e, &pair, &0);
        Events::new(&e).claim_protocol_fee(pair, config.usdc, destination.clone(), fee);

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

        crate::storage::set_is_killed_deposit(&e, &true);
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

        crate::storage::set_is_killed_withdraw(&e, &true);
        Events::new(&e).kill_withdraw();
    }

    fn kill_trade(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_is_killed_trade(&e, &true);
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

        crate::storage::set_is_killed_deposit(&e, &false);
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

        crate::storage::set_is_killed_withdraw(&e, &false);
        Events::new(&e).unkill_withdraw();
    }

    fn unkill_trade(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_is_killed_trade(&e, &false);
        Events::new(&e).unkill_trade();
    }

    // Get create killswitch status.
    fn get_is_killed_deposit(e: Env) -> bool {
        crate::storage::get_is_killed_deposit(&e)
    }

    // Get withdraw killswitch status.
    fn get_is_killed_withdraw(e: Env) -> bool {
        crate::storage::get_is_killed_withdraw(&e)
    }

    // Get trade killswitch status.
    fn get_is_killed_trade(e: Env) -> bool {
        crate::storage::get_is_killed_trade(&e)
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
