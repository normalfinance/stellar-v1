use paste::paste;
use soroban_sdk::{contracttype, panic_with_error, Address, BytesN, Env, Symbol, Vec};
use utils::bump::{bump_instance, bump_persistent};
use utils::{
    errors::storage_errors::StorageError, generate_instance_storage_getter,
    generate_instance_storage_getter_and_setter,
    generate_instance_storage_getter_and_setter_with_default,
    generate_instance_storage_getter_with_default, generate_instance_storage_setter,
};

use crate::errors::LongShortPairFactoryError;

/********** Storage Key Types **********/
// Instance-scoped killswitch for pair creation (global to this factory instance).
const KEY_IS_KILLED_CREATE: &str = "IsKilledCreate";

// Instance-scoped WASM hash used when deploying new Pair contracts.
const KEY_PAIR_CONTRACT_WASM: &str = "PairContractWASM";

/// Persistent storage keys for factory-managed data.
///
/// We store:
/// - a mapping from an asset-specific salt → deployed pair address
/// - a global list of every deployed pair address (for discovery / indexing)
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Map: `salt` → `pair_address`.
    ///
    /// `salt` is typically derived from asset metadata so that:
    /// - pair deployments are deterministic / idempotent per asset
    /// - callers can look up the canonical pair for a given asset
    AssetPair(BytesN<32>),

    /// Global append-only list of all pairs deployed by this factory.
    ///
    /// Used for discovery (frontends), analytics, and indexers.
    AllDeployedPairs, // Vec<Address>
}

/********** Storage **********/

// WASM hash (BytesN<32>) for the Pair contract deployed by this factory.
//
// The factory reads this value during `create_pair(...)` to deploy a new
// [`LongShortPair`] instance.
generate_instance_storage_getter_and_setter!(
    pair_contract_wasm,
    KEY_PAIR_CONTRACT_WASM,
    BytesN<32>
);

// Killswitch for creating new pairs.
//
// When `true`, the factory should reject pair creation requests (but existing
// pairs remain unaffected).
generate_instance_storage_getter_and_setter_with_default!(
    is_killed_create,
    KEY_IS_KILLED_CREATE,
    bool,
    false
);

/// Fetches the deployed pair address for a given `salt`.
///
/// ### Arguments
/// - `salt`: A deterministic identifier for the asset/pair (commonly a hash).
///
/// ### Returns
/// The deployed Pair contract [`Address`].
///
/// ### Reverts
/// - [`LongShortPairFactoryError::PairNotFound`] if no pair has been registered for `salt`.
///
/// ### Notes
/// - This value is stored in persistent storage and TTL-bumped on access.
pub fn get_pair(e: &Env, salt: BytesN<32>) -> Address {
    let key = DataKey::AssetPair(salt);
    match e.storage().persistent().get(&key) {
        Some(address) => {
            bump_persistent(e, &key);
            address
        }
        None => panic_with_error!(&e, LongShortPairFactoryError::PairNotFound),
    }
}

/// Registers a deployed pair address for a given `salt`.
///
/// ### Arguments
/// - `salt`: A deterministic identifier for the asset/pair (commonly a hash).
/// - `pair`: The deployed Pair contract address.
///
/// ### Notes
/// - Overwriting an existing mapping is allowed by this function; callers should
///   enforce uniqueness/idempotence at the factory method level if desired.
/// - Stored in persistent storage and TTL-bumped immediately.
pub fn put_pair(e: &Env, salt: BytesN<32>, pair: &Address) {
    let key = DataKey::AssetPair(salt);
    e.storage().persistent().set(&key, pair);
    bump_persistent(e, &key);
}

/// Appends `pair_address` to the global list of all deployed pairs.
///
/// This is intended for discovery and indexing. It does **not** validate that the
/// address is unique, nor does it verify that the address is a valid pair contract.
///
/// ### Notes
/// - This list is append-only (unless manually rewritten by an upgrade).
/// - If you need uniqueness, consider:
///   - maintaining an additional `Set`-like key (`has_pair(address)`)
///   - or checking the salt→pair mapping before pushing
///
/// TODO: consider addressing potential unbounded growth / storage costs:
/// https://app.almanax.ai/scan/13ca3512-fbc7-4909-929a-53855e07d7af/findings/897d84fc-0d49-4569-afc8-7704bd534a5c
pub fn add_deployed_pair(env: &Env, pair_address: &Address) {
    // Load the current list (or initialize empty on first use).
    let global_key = DataKey::AllDeployedPairs;
    let mut all_pairs: Vec<Address> = match env.storage().persistent().get(&global_key) {
        Some(pairs) => pairs,
        None => Vec::new(env),
    };

    // Append the new pair address.
    all_pairs.push_back(pair_address.clone());

    // Persist updated list and TTL-bump.
    env.storage().persistent().set(&global_key, &all_pairs);
    bump_persistent(env, &global_key);
}

/// Returns the global list of all pair addresses deployed by this factory.
///
/// ### Returns
/// A `Vec<Address>` containing deployed pair contract addresses, ordered by insertion.
///
/// ### Notes
/// - If none have been deployed yet, returns an empty vector.
/// - Stored in persistent storage and TTL-bumped on access.
pub fn get_all_deployed_pairs(env: &Env) -> Vec<Address> {
    let key = DataKey::AllDeployedPairs;
    match env.storage().persistent().get(&key) {
        Some(pairs) => {
            bump_persistent(env, &key);
            pairs
        }
        None => Vec::new(env),
    }
}
