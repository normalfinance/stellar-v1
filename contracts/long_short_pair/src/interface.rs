use soroban_sdk::{Address, Env, Vec};
use types::pair::{CollateralInfo, PairParams, PairSummary};

pub trait LongShortPairTrait {
    fn initialize(e: Env, params: PairParams);

    /// Creates a pair of long and short tokens equal in number to tokens_to_mint. Pulls the required collateral
    /// amount into this contract, defined by the collateral_per_pair value.
    /// @param tokens_to_mint number of long and short synthetic tokens to create.
    /// @return collateral_used total collateral used to mint the synthetics.
    fn mint(e: Env, user: Address, tokens_to_mint: u128) -> u128;

    /// Redeems a pair of long and short tokens equal in number to tokens_to_redeem. Returns the commensurate
    /// amount of collateral to the caller for the pair of tokens, defined by the collateral_per_pair value.
    /// and short tokens are burned, rather than transferred, from the caller.
    /// @dev This method can be called either pre or post expiration.
    /// @param tokens_to_redeem number of long and short synthetic tokens to redeem.
    /// @return collateral_returned total collateral returned in exchange for the pair of synthetics.
    fn redeem(e: Env, user: Address, tokens_to_redeem: u128) -> u128;

    fn redeem_one(e: Env, user: Address, token: Address, tokens_to_redeem: u128) -> u128;

    fn sync_collateral(e: Env);

    fn get_tokens(e: Env) -> Vec<Address>;

    fn get_price_bounds(e: Env) -> Vec<u128>;

    fn get_user_token_balances(e: Env, user: Address) -> Vec<u128>;

    fn get_total_token_supplies(e: Env) -> Vec<u128>;

    fn get_collateral_info(e: Env) -> CollateralInfo;

    fn get_pair_summary(e: Env) -> PairSummary;
}

pub trait AdminInterfaceTrait {
    fn set_oracle(e: Env, admin: Address, oracle: Address);
    fn set_calculator(e: Env, admin: Address, calculator: Address);

    // Stop pair instantly
    fn kill_mint(e: Env, admin: Address);
    fn kill_redeem(e: Env, admin: Address);

    // Resume pair
    fn unkill_mint(e: Env, admin: Address);
    fn unkill_redeem(e: Env, admin: Address);

    // Get killswitch status
    fn get_is_killed_mint(e: Env) -> bool;
    fn get_is_killed_redeem(e: Env) -> bool;
}

pub trait OracleInterfaceTrait {
    fn get_price(e: Env) -> u128;
    fn get_scaled_price(e: Env) -> u128;
}
