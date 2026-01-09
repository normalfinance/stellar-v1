# Ensure the script exits on any errors
set -e

# Check if the arguments are provided
# Required: identity_string, network
if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <identity_string> <network>"
    exit 1
fi

IDENTITY_STRING=$1
NETWORK=$2

# Load env vars dynamically
source "$(dirname "${BASH_SOURCE[0]}")/load-env.sh" "$NETWORK"

# Fetch the admin's address
ADMIN_ADDRESS=$(soroban keys address $IDENTITY_STRING)

# echo "Deploy pair via factory..."

PAIR_ADDR=$(stellar contract invoke \
    --id $FACTORY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE \
    -- \
    deploy_pair_contract \
    --admin $ADMIN_ADDRESS \
    --asset ETH
)

echo "Pair deployed: $PAIR_ADDR"

# get tokens

echo "Adding pair to Treasury..."

stellar contract invoke \
    --id $TREASURY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE \
    -- \
    add_pair \
    --admin $ADMIN_ADDRESS \
    --pair $PAIR_ADDR \
    --quote_token $USDC_ADDRESS \
    --long_token $PAIR_LONG_ADDR \
    --short_token $PAIR_SHORT_ADDR


echo "#############################"

echo "Initialization complete!"

