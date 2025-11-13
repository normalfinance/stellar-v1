use soroban_sdk::{Address, BytesN, Env, Map, Symbol, Vec};

use crate::storage::{CollateralInfo, FundingInfo};

pub trait LongShortPairTrait {
    // Initialize lsp
    fn initialize(
        e: Env,
        admin: Address,
        privileged_addrs: (Address, Address, Address, Address, Vec<Address>, Address),
        tokens: Vec<Address>,
        oracle: Address,
        calculator: Address,
    );

    /**
     * @notice Creates a pair of long and short tokens equal in number to tokensToCreate. Pulls the required collateral
     * amount into this contract, defined by the collateralPerPair value.
     * @dev The caller must approve this contract to transfer `tokensToCreate * collateralPerPair` amount of collateral.
     * @param tokensToCreate number of long and short synthetic tokens to create.
     * @return collateralUsed total collateral used to mint the synthetics.
     */
    fn create(e: Env, user: Address, tokens_to_create: u128) -> u128;

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
    fn redeem(e: Env, user: Address, tokens_to_redeem: u128) -> u128;

    fn checkpoint_funding(
        e: Env,
        token_contract: Address,
        from: Address,
        to: Address,
        transfer_amount: u128,
    );

    fn update_oracle_price(e: Env);

    fn get_tokens(e: Env) -> Vec<Address>;

    fn get_position_tokens(e: Env, user: Address) -> Vec<u128>;

    fn get_collateral_info(e: Env) -> CollateralInfo;

    fn get_funding_info(e: Env) -> FundingInfo;
}

pub trait AdminInterfaceTrait {
    // Funding
    fn update_funding_period(e: Env, admin: Address, funding_period: u64);

    fn update_funding_rate(e: Env, admin: Address);

    // Set privileged addresses
    fn set_privileged_addrs(
        e: Env,
        admin: Address,
        rewards_admin: Address,
        operations_admin: Address,
        pause_admin: Address,
        emergency_pause_admins: Vec<Address>,
        system_fee_admin: Address,
    );

    // Get map of privileged roles
    fn get_privileged_addrs(e: Env) -> Map<Symbol, Vec<Address>>;

    fn set_oracle(e: Env, admin: Address, oracle: Address);

    // Stop LSP instantly
    fn kill_create(e: Env, admin: Address);
    fn kill_redeem(e: Env, admin: Address);
    fn kill_update_funding(e: Env, admin: Address);

    // Resume LSP
    fn unkill_create(e: Env, admin: Address);
    fn unkill_redeem(e: Env, admin: Address);
    fn unkill_update_funding(e: Env, admin: Address);

    // Get killswitch status
    fn get_is_killed_create(e: Env) -> bool;
    fn get_is_killed_redeem(e: Env) -> bool;
    fn get_is_killed_update_funding(e: Env) -> bool;
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
