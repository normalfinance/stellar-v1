use soroban_sdk::{Address, BytesN, Env, Symbol};
use types::pair::{Direction, PairAmountsWithUSDC, Side};

use crate::storage::{PairConfig, TreasuryFeeConfig, TreasurySummary, TreasuryUserSummary};

pub trait TreasuryTrait {
    fn deposit(e: Env, user: Address, pair: Address, pairs_to_deposit: u128) -> u128;

    fn withdraw(e: Env, user: Address, pair: Address, shares: u128) -> PairAmountsWithUSDC;

    fn get_config(e: Env, pair: Address) -> PairConfig;

    fn get_prices(e: Env, pair: Address) -> PairAmountsWithUSDC;

    fn get_balances(e: Env, pair: Address) -> PairAmountsWithUSDC;

    fn get_total_shares(e: Env, pair: Address) -> u128;

    fn get_user_shares(e: Env, pair: Address, user: Address) -> u128;

    fn get_fee_config(e: Env, pair: Address) -> TreasuryFeeConfig;

    fn get_summary(e: Env, pair: Address) -> TreasurySummary;

    fn get_user_with_summary(e: Env, pair: Address, user: Address) -> TreasuryUserSummary;
}

pub trait TradingTrait {
    fn estimate_trade(
        e: Env,
        pair: Address,
        direction: Direction,
        side: Side,
        amount_in: u128,
    ) -> (u128, u128);

    fn buy_long(e: Env, user: Address, pair: Address, usdc_in: u128, min_long_out: u128) -> u128;
    fn sell_long(e: Env, user: Address, pair: Address, long_in: u128, min_usdc_out: u128) -> u128;
    fn buy_short(e: Env, user: Address, pair: Address, usdc_in: u128, min_short_out: u128) -> u128;
    fn sell_short(e: Env, user: Address, pair: Address, short_in: u128, min_usdc_out: u128)
        -> u128;
}

pub trait AdminInterfaceTrait {
    fn add_pair(e: Env, admin: Address, pair: Address, fee_config: TreasuryFeeConfig);

    fn set_fee_config(e: Env, admin: Address, pair: Address, config: TreasuryFeeConfig);

    fn set_usdc_floor(e: Env, admin: Address, floor_fraction: u128);

    fn get_protocol_fees(e: Env, pair: Address) -> u128;

    fn claim_protocol_fees(e: Env, admin: Address, pair: Address, destination: Address) -> u128;

    // Stop pair instantly
    fn kill_deposit(e: Env, admin: Address);
    fn kill_withdraw(e: Env, admin: Address);
    fn kill_trade(e: Env, admin: Address);

    // Resume pair
    fn unkill_deposit(e: Env, admin: Address);
    fn unkill_withdraw(e: Env, admin: Address);
    fn unkill_trade(e: Env, admin: Address);

    // Get killswitch status
    fn get_is_killed_deposit(e: Env) -> bool;
    fn get_is_killed_withdraw(e: Env) -> bool;
    fn get_is_killed_trade(e: Env) -> bool;
}

pub trait UpgradeableContract {
    // Get contract version
    fn version() -> u32;

    // Get contract type symbolic name
    fn contract_name(e: Env) -> Symbol;

    // Upgrade contract with new wasm code
    fn commit_upgrade(e: Env, admin: Address, new_wasm_hash: BytesN<32>);
    fn apply_upgrade(e: Env, admin: Address) -> BytesN<32>;
    fn revert_upgrade(e: Env, admin: Address);

    // Emergency mode - bypass upgrade deadline
    fn set_emergency_mode(e: Env, admin: Address, value: bool);
    fn get_emergency_mode(e: Env) -> bool;
}
