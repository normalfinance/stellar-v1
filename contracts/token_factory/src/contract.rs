use crate::interface::TokenFactoryTrait;
use soroban_sdk::{contract, contractimpl, Address, Bytes, Env};

#[contract]
pub struct TokenFactory;

#[contractimpl]
impl TokenFactoryTrait for TokenFactory {
    /**
     * @notice Create a new token and return it to the caller.
     * @dev The caller will become the only minter and burner and the new owner capable of assigning the roles.
     * @param serialized_asset used to describe the new token.
     * @return sac_address an instance of the newly created token interface.
     */
    fn create_token(e: Env, serialized_asset: Bytes) -> Address {
        let deployer = e.deployer().with_stellar_asset(serialized_asset);
        let sac_address = deployer.deploy();

        sac_address
    }
}
