use soroban_sdk::{Address, Env, Symbol};

pub trait PairInterfaceTrait {
    fn mint(e: Env, user: Address, asset: Symbol, tokens_to_mint: u128) -> u128;

    fn redeem(e: Env, user: Address, asset: Symbol, tokens_to_redeem: u128) -> u128;

    fn redeem_one(
        e: Env,
        user: Address,
        asset: Symbol,
        token: Address,
        tokens_to_redeem: u128,
    ) -> u128;
}
