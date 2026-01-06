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

echo "Setup Treasury..."

stellar contract invoke \
    --id $TREASURY_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS


echo "Setup oracle..."


stellar contract invoke \
    --id $ORACLE_ADDR \
    --source $IDENTITY_STRING \
    --network $NETWORK \
    --rpc-url $STELLAR_RPC_URL \
    --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" \
    --fee $STELLAR_BASE_FEE \
    -- \
    initialize \
    --admin $ADMIN_ADDRESS \
    --asset $POOL_PLANE_ADDR \
    --oracle_source \
    --oracle_addr $

echo "Tokens and pool router deployed."

echo "#############################"

echo "Initialization complete!"

