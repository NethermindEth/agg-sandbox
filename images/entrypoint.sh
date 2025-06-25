#!/bin/bash

# Debug: Print environment variables
echo "DEBUG: ENABLE_FORK_MODE='$ENABLE_FORK_MODE'"
echo "DEBUG: FORK_URL_MAINNET='$FORK_URL_MAINNET'"
echo "DEBUG: FORK_URL_AGGLAYER_1='$FORK_URL_AGGLAYER_1'"
echo "DEBUG: FORK_URL_AGGLAYER_2='$FORK_URL_AGGLAYER_2'"

# Build the anvil command
ANVIL_CMD="anvil --host 0.0.0.0"

# Only use fork parameters if fork mode is explicitly enabled
if [ "$ENABLE_FORK_MODE" = "true" ]; then
    echo "Fork mode enabled, checking for fork URLs..."
    
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

    # Support for second L2 chain
    if [ -n "$FORK_URL_AGGLAYER_2" ]; then
        ANVIL_CMD="$ANVIL_CMD --fork-url $FORK_URL_AGGLAYER_2"
        if [ -n "$CHAIN_ID_AGGLAYER_2" ]; then
            ANVIL_CMD="$ANVIL_CMD --chain-id $CHAIN_ID_AGGLAYER_2"
        fi
    fi
else
    echo "Local mode enabled, ignoring fork URLs if present"
    # In local mode, only set chain IDs if they're provided, but don't use fork URLs
    if [ -n "$CHAIN_ID_MAINNET" ]; then
        ANVIL_CMD="$ANVIL_CMD --chain-id $CHAIN_ID_MAINNET"
    fi
    if [ -n "$CHAIN_ID_AGGLAYER_1" ]; then
        ANVIL_CMD="$ANVIL_CMD --chain-id $CHAIN_ID_AGGLAYER_1"
    fi
    if [ -n "$CHAIN_ID_AGGLAYER_2" ]; then
        ANVIL_CMD="$ANVIL_CMD --chain-id $CHAIN_ID_AGGLAYER_2"
    fi
fi

# Execute the command
echo "Starting anvil with: $ANVIL_CMD"
exec $ANVIL_CMD 