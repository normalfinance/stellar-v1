use crate::events::{Events, FactoryConfigEvents, FactoryEvents};
use crate::factory_interface::{AdminInterface, LongShortPairFactoryTrait};
use crate::pair_interface::PairInterfaceTrait;
use soroban_sdk::{
    contract, contractimpl, contractmeta, contracttype, Address, BytesN, Env, Symbol, Vec,
};
use soroban_sdk::{symbol_short, IntoVal};

// Access control
use access_control::access::{AccessControl, AccessControlTrait};
use access_control::management::SingleAddressManagementTrait;
use access_control::role::Role;

contractmeta!(key = "Description", val = "");

#[contract]
pub struct LongShortPairFactory;

// Factory configuration struct for query methods
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactoryConfig {
    pub pair_contract_wasm: BytesN<32>,
}

#[contractimpl]
impl LongShortPairFactory {
    // __constructor
    // Initializes the factory by setting the admin roles and storing critical parameters.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The address to be assigned the Admin role.
    //   - pair_contract_wasm: The WASM hash (BytesN<32>) for the long short pair contract.
    pub fn __constructor(e: Env, admin: Address, pair_contract_wasm: BytesN<32>) {
        let access_control = AccessControl::new(&e);
        access_control.set_role_address(&Role::Admin, &admin);

        crate::storage::set_pair_contract_wasm(&e, &pair_contract_wasm);
    }
}

#[contractimpl]
impl LongShortPairFactoryTrait for LongShortPairFactory {
    /**
     * @notice Creates a longShortPair contract and associated long and short tokens.
     * @param params Constructor params used to initialize the LSP. Key-valued object with the following structure:
     *     - `pairName`: Name of the long short pair contract.
     *     - `collateralPerPair`: How many units of collateral are required to mint one pair of synthetic tokens.
     *     - `collateralToken`: ERC20 token used as collateral in the LSP.
     *     - `calculator`: Contract providing settlement payout logic.
     * @return lspAddress the deployed address of the new long short pair contract.
     * @notice Created LSP is not registered within the registry as the LSP uses the Optimistic Oracle for settlement.
     * @notice The LSP constructor does a number of validations on input params. These are not repeated here.
     */
    fn deploy_pair_contract(e: Env, admin: Address, asset: Symbol) -> Address {
        admin.require_auth();

        // Deploy the pair contract
        let salt = crate::pair_utils::get_pair_salt(&e, &asset);

        let pair_address = e
            .deployer()
            .with_current_contract(salt.clone())
            .deploy_v2(crate::storage::get_pair_contract_wasm(&e), ());

        // Add to pair registry
        crate::storage::add_deployed_pair(&e, &pair_address);
        crate::storage::put_pair(&e, salt, &pair_address);

        Events::new(&e).pair_deployed(e.ledger().timestamp(), admin.clone(), pair_address.clone());

        pair_address
    }
}

#[contractimpl]
impl PairInterfaceTrait for LongShortPairFactory {
    fn mint(e: Env, user: Address, asset: Symbol, tokens_to_mint: u128) -> u128 {
        user.require_auth();

        let salt = crate::pair_utils::get_pair_salt(&e, &asset);
        let pair_address = crate::storage::get_pair(&e, salt);

        let minted_tokens: u128 = e.invoke_contract(
            &pair_address,
            &symbol_short!("mint"),
            Vec::from_array(&e, [user.clone().into_val(&e), tokens_to_mint.into_val(&e)]),
        );

        Events::new(&e).mint(
            user,
            asset,
            pair_address,
            0,
            tokens_to_mint,
            e.ledger().timestamp(),
        );

        minted_tokens
    }

    fn redeem(e: Env, user: Address, asset: Symbol, tokens_to_redeem: u128) -> u128 {
        user.require_auth();

        let salt = crate::pair_utils::get_pair_salt(&e, &asset);
        let pair_address = crate::storage::get_pair(&e, salt);

        let collateral: u128 = e.invoke_contract(
            &pair_address,
            &symbol_short!("redeem"),
            Vec::from_array(
                &e,
                [user.clone().into_val(&e), tokens_to_redeem.into_val(&e)],
            ),
        );

        Events::new(&e).redeem(
            user,
            asset,
            pair_address,
            collateral,
            e.ledger().timestamp(),
        );

        collateral
    }

    fn redeem_one(
        e: Env,
        user: Address,
        asset: Symbol,
        token: Address,
        tokens_to_redeem: u128,
    ) -> u128 {
        user.require_auth();

        let salt = crate::pair_utils::get_pair_salt(&e, &asset);
        let pair_address = crate::storage::get_pair(&e, salt);

        let collateral: u128 = e.invoke_contract(
            &pair_address,
            &Symbol::new(&e, "redeem_one"),
            Vec::from_array(
                &e,
                [
                    user.clone().into_val(&e),
                    token.into_val(&e),
                    tokens_to_redeem.into_val(&e),
                ],
            ),
        );

        Events::new(&e).redeem_one(
            user,
            asset,
            pair_address,
            token,
            tokens_to_redeem,
            e.ledger().timestamp(),
        );

        collateral
    }
}

#[contractimpl]
impl AdminInterface for LongShortPairFactory {
    //   _______    _______  ___________  ___________  _______   _______    ________
    //  /" _   "|  /"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (: ( \___) (: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \/ \       \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //  //  \ ___  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    // (:   _(  _|(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    //  \_______)  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    fn get_factory_config(e: Env) -> FactoryConfig {
        FactoryConfig {
            pair_contract_wasm: crate::storage::get_pair_contract_wasm(&e),
        }
    }

    fn get_pair_contract_wasm(e: Env) -> BytesN<32> {
        crate::storage::get_pair_contract_wasm(&e)
    }

    fn get_all_deployed_pairs(e: Env) -> Vec<Address> {
        crate::storage::get_all_deployed_pairs(&e)
    }

    fn get_total_pair_count(e: Env) -> u32 {
        let all_pairs = crate::storage::get_all_deployed_pairs(&e);
        all_pairs.len()
    }

    fn get_pair_by_asset(e: Env, asset: Symbol) -> Address {
        let salt = crate::pair_utils::get_pair_salt(&e, &asset);
        crate::storage::get_pair(&e, salt)
    }

    //   ________  _______  ___________  ___________  _______   _______    ________
    //  /"       )/"     "|("     _   ")("     _   ")/"     "| /"      \  /"       )
    // (:   \___/(: ______) )__/  \\__/  )__/  \\__/(: ______)|:        |(:   \___/
    //  \___  \   \/    |      \\_ /        \\_ /    \/    |  |_____/   ) \___  \
    //   __/  \\  // ___)_     |.  |        |.  |    // ___)_  //      /   __/  \\
    //  /" \   :)(:      "|    \:  |        \:  |   (:      "||:  __   \  /" \   :)
    // (_______/  \_______)     \__|         \__|    \_______)|__|  \___)(_______/

    // set_pair_contract_wasm
    // Updates the WASM hash for the long short pair contract.
    //
    // Arguments:
    //   - e: The Soroban environment.
    //   - admin: The admin address (must be authorized).
    //   - pair_contract_wasm: The new WASM hash (BytesN<32>) for the swap fee contract.
    fn set_pair_contract_wasm(e: Env, admin: Address, pair_contract_wasm: BytesN<32>) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        let old_wasm = crate::storage::get_pair_contract_wasm(&e);
        crate::storage::set_pair_contract_wasm(&e, &pair_contract_wasm);

        let current_time = e.ledger().timestamp();
        Events::new(&e).pair_wasm_updated(
            current_time,
            admin.clone(),
            old_wasm.clone(),
            pair_contract_wasm.clone(),
            1,
        );
    }

    //    _______     __       ____  ____   ________  _______  ________
    //   |   __ "\   /""\     ("  _||_ " | /"       )/"     "||"      "\
    //   (. |__) :) /    \    |   (  ) : |(:   \___/(: ______)(.  ___  :)
    //   |:  ____/ /' /\  \   (:  |  | . ) \___  \   \/    |  |: \   ) ||
    //   (|  /    //  __'  \   \\ \__/ //   __/  \\  // ___)_ (| (___\ ||
    //  /|__/ \  /   /  \\  \  /\\ __ //\  /" \   :)(:      "||:       :)
    // (_______)(___/    \___)(__________)(_______/  \_______)(________/

    fn kill_create(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_is_killed_create(&e, &true);
        Events::new(&e).factory_paused(e.ledger().timestamp(), admin);
    }

    fn unkill_create(e: Env, admin: Address) {
        admin.require_auth();
        AccessControl::new(&e).assert_address_has_role(&admin, &Role::Admin);

        crate::storage::set_is_killed_create(&e, &false);
        Events::new(&e).factory_unpaused(e.ledger().timestamp(), admin);
    }

    fn get_is_killed_create(e: Env) -> bool {
        crate::storage::get_is_killed_create(&e)
    }
}
