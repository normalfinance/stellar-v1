# Create bindings directory
mkdir -p bindings

# Build all
task build

# Generate bindings for each contract
soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/soroban_token_contract.wasm --output-dir bindings/soroban_token_contract

# DEX
soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/token_pool.wasm --output-dir bindings/token_pool

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/config_storage.wasm --output-dir bindings/config_storage

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/rewards_gauge.wasm --output-dir bindings/rewards_gauge

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/liquidity_pool_plane.wasm --output-dir bindings/liquidity_pool_plane

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/liquidity_pool_liquidity_calculator.wasm --output-dir bindings/liquidity_pool_liquidity_calculator

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/liquidity_pool.wasm --output-dir bindings/liquidity_pool

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/liquidity_pool_stableswap.wasm --output-dir bindings/liquidity_pool_stableswap

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/liquidity_pool_synthetic.wasm --output-dir bindings/liquidity_pool_synthetic

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/liquidity_pool_router.wasm --output-dir bindings/liquidity_pool_router

# LSP
soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/normal_oracle.wasm --output-dir bindings/normal_oracle

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/token_long_short_pair.wasm --output-dir bindings/token_long_short_pair

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/long_short_pair_calculator.wasm --output-dir bindings/long_short_pair_calculator

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/long_short_pair.wasm --output-dir bindings/long_short_pair

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/long_short_pair_factory.wasm --output-dir bindings/long_short_pair_factory