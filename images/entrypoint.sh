#!/bin/bash

# Build the anvil command
ANVIL_CMD="anvil --host 0.0.0.0"

# Add fork parameters if fork URL is provided
if [ -n "$FORK_URL_MAINNET" ]; then
    ANVIL_CMD="$ANVIL_CMD --fork-url $FORK_URL_MAINNET"
    if [ -n "$CHAIN_ID_MAINNET" ]; then
        ANVIL_CMD="$ANVIL_CMD --chain-id $CHAIN_ID_MAINNET"
    fi
fi

if [ -n "$FORK_URL_AGGLAYER_1" ]; then
    ANVIL_CMD="$ANVIL_CMD --fork-url $FORK_URL_AGGLAYER_1"
    if [ -n "$CHAIN_ID_AGGLAYER_1" ]; then
        ANVIL_CMD="$ANVIL_CMD --chain-id $CHAIN_ID_AGGLAYER_1"
    fi
fi

# Execute the command
echo "Starting anvil with: $ANVIL_CMD"
exec $ANVIL_CMD 