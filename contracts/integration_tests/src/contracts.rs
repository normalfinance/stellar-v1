// Main

pub(crate) mod constant_product_pool {
    soroban_sdk::contractimport!(file = "../../wasm/liquidity_pool.wasm");
}
pub(crate) mod stableswap_pool {
    soroban_sdk::contractimport!(file = "../../wasm/liquidity_pool_stableswap.wasm");
}
pub(crate) mod synthetic_pool {
    soroban_sdk::contractimport!(file = "../../wasm/liquidity_pool_synthetic.wasm");
}
pub(crate) mod liquidity_calculator {
    soroban_sdk::contractimport!(file = "../../wasm/liquidity_pool_liquidity_calculator.wasm");
}
pub(crate) mod pool_plane {
    soroban_sdk::contractimport!(file = "../../wasm/liquidity_pool_plane.wasm");
}
pub(crate) mod router {
    soroban_sdk::contractimport!(file = "../../wasm/liquidity_pool_router.wasm");
}
pub(crate) mod token_pool {
    soroban_sdk::contractimport!(file = "../../wasm/token_pool.wasm");
}
pub(crate) mod config_storage {
    soroban_sdk::contractimport!(file = "../../wasm/config_storage.wasm");
}
pub(crate) mod rewards_gauge {
    soroban_sdk::contractimport!(file = "../../wasm/rewards_gauge.wasm");
}

// Long Short Pair
pub(crate) mod normal_oracle {
    soroban_sdk::contractimport!(file = "../../wasm/normal_oracle.wasm");
}
pub(crate) mod long_short_pair_calculator {
    soroban_sdk::contractimport!(file = "../../wasm/long_short_pair_calculator.wasm");
}
pub(crate) mod long_short_pair {
    soroban_sdk::contractimport!(file = "../../wasm/long_short_pair.wasm");
}
pub(crate) mod long_short_pair_factory {
    soroban_sdk::contractimport!(file = "../../wasm/long_short_pair_factory.wasm");
}
