# Create bindings directory
mkdir -p bindings

# Build all
task build

# Generate bindings for each contract
soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/soroban_token_contract.wasm --output-dir bindings/soroban_token_contract

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/normal_oracle.wasm --output-dir bindings/normal_oracle

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/token_lsp.wasm --output-dir bindings/token_lsp

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/calculator.wasm --output-dir bindings/calculator

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/token_factory.wasm --output-dir bindings/token_factory

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/long_short_pair.wasm --output-dir bindings/long_short_pair

soroban contract bindings typescript --overwrite --wasm target/wasm32v1-none/release/long_short_pair_factory.wasm --output-dir bindings/long_short_pair_factory