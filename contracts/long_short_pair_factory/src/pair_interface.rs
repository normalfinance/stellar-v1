use soroban_sdk::{Address, Env, Symbol};
use types::pair::Side;

pub trait PairInterfaceTrait {
    fn mint(
        e: Env,
        user: Address,
        asset: Symbol,
        collateral_token: Address,
        tokens_to_mint: u128,
    ) -> u128;

    fn redeem(
        e: Env,
        user: Address,
        asset: Symbol,
        collateral_token: Address,
        tokens_to_redeem: u128,
    ) -> u128;

    fn redeem_one(
        e: Env,
        user: Address,
        asset: Symbol,
        collateral_token: Address,
        side: Side,
        tokens_to_redeem: u128,
    ) -> u128;
}
