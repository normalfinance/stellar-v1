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

soroban contract optimize --wasm token_factory.wasm
soroban contract optimize --wasm calculator.wasm
soroban contract optimize --wasm long_short_pair.wasm
soroban contract optimize --wasm long_short_pair_factory.wasm

echo "Contracts optimized."

# Fetch the admin's address
ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)

echo "Install the pool and pool elastic contract..."

LSP_WASM_HASH=$(soroban contract upload \
    --wasm long_short_pair.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE
)

echo "LSP contracts deployed."

#   _______     ______    ____  ____  ___________  _______   _______
#  /"      \   /    " \  ("  _||_ " |("     _   ")/"     "| /"      \
# |:        | // ____  \ |   (  ) : | )__/  \\__/(: ______)|:        |
# |_____/   )/  /    ) :)(:  |  | . )    \\_ /    \/    |  |_____/   )
#  //      /(: (____/ //  \\ \__/ //     |.  |    // ___)_  //      /
# |:  __   \ \        /   /\\ __ //\     \:  |   (:      "||:  __   \
# |__|  \___) \"_____/   (__________)     \__|    \_______)|__|  \___)

echo "Initialize LSP Factory..."

TOKEN_FACTORY_ADDR=$(soroban contract deploy \
    --wasm token_factory.optimized.wasm \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE
)

CALCULATOR_ADDR=$(soroban contract deploy \
    --wasm calculator.optimized.wasm \
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
    -- --admin $ADMIN_ADDRESS --emergency_admin $ADMIN_ADDRESS --token_factory $TOKEN_FACTORY_ADDR --lsp_contract_wasm $LSP_WASM_HASH
)

echo "Contracts deployed."

echo "#############################"

echo "Initialization complete!"

echo "Token Factory Contract address: $TOKEN_FACTORY_ADDR"
echo "Calculator Contract address: $CALCULATOR_ADDR"
echo "LSP Factory Contract address: $FACTORY_ADDR"

echo "LSP wasm hash: $LSP_WASM_HASH"