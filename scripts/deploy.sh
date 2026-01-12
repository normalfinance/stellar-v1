# Ensure the script exits on any errors
set -e

# Check if the argument is provided
if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <identity_string> <network>"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK=$2

# Load env vars dynamically
source "$(dirname "${BASH_SOURCE[0]}")/load-env.sh" "$NETWORK"

echo $STELLAR_RPC_URL
echo "$STELLAR_NETWORK_PASSPHRASE"

echo "Build and optimize the contracts..."

task build
cd target/wasm32v1-none/release

echo "Contracts compiled."
echo "Optimize contracts..."

soroban contract optimize --wasm soroban_token_contract.wasm

soroban contract optimize --wasm normal_oracle.wasm
soroban contract optimize --wasm treasury.wasm
soroban contract optimize --wasm long_short_pair_calculator.wasm
soroban contract optimize --wasm long_short_pair.wasm
soroban contract optimize --wasm long_short_pair_factory.wasm

echo "Contracts optimized."

# Fetch the admin's address
ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)

echo "Install the pair contract..."

PAIR_WASM_HASH=$(soroban contract upload \
    --wasm long_short_pair.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE
)

# echo "Pair contract deployed."

# Treasury
TREASURY_ADDR=$(soroban contract deploy \
    --wasm treasury.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE
)

# echo "Initialize Pair Factory..."

CALCULATOR_ADDR=$(soroban contract deploy \
    --wasm long_short_pair_calculator.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE
)

ORACLE_ADDR=$(soroban contract deploy \
    --wasm normal_oracle.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE
)

FACTORY_ADDR=$(soroban contract deploy \
    --wasm long_short_pair_factory.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE \
    -- --admin $ADMIN_ADDRESS --pair_contract_wasm $PAIR_WASM_HASH
)

echo "Contracts deployed."

echo "#############################"

echo "Initialization complete!"

echo "Treasury Contract address: $TREASURY_ADDR"
echo "Calculator Contract address: $CALCULATOR_ADDR"
echo "Pair Factory Contract address: $FACTORY_ADDR"
echo "Normal Oracle Contract address: $ORACLE_ADDR"

echo "Pair wasm hash: $PAIR_WASM_HASH"