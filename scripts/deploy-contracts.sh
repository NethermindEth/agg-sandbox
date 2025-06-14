#!/bin/bash

# Detect OS and set SED command accordingly
if [[ "$OSTYPE" == "darwin"* ]]; then
    # MacOS - needs special handling without using variables
    # We'll handle this directly in the function
    SED_IS_MAC=true
else
    # Linux and others
    SED_IS_MAC=false
fi

# Function to print timestamped messages
echo_ts() {
    local green="\e[32m"
    local end_color="\e[0m"
    local timestamp
    timestamp=$(date +"[%Y-%m-%d %H:%M:%S]")

    echo -e "$green$timestamp$end_color $1" >&2
}

# Function to update env file with a new variable
update_env_file() {
    local env_file="$1"
    local var_name="$2"
    local var_value="$3"
    
    # Check if variable already exists
    if grep -q "^$var_name=" "$env_file"; then
        # Update existing variable
        if [[ "$SED_IS_MAC" == "true" ]]; then
            # MacOS requires a different sed syntax
            sed -i '' "s/^$var_name=.*/$var_name=$var_value/" "$env_file"
        else
            # Linux
            sed -i "s/^$var_name=.*/$var_name=$var_value/" "$env_file"
        fi
    else
        # Append new variable
        echo "$var_name=$var_value" >> "$env_file"
    fi
}

# Handle env file argument
ENV_FILE="${1:-.env}"

# Source environment variables
if [ -f "$ENV_FILE" ]; then
    echo_ts "Loading environment variables from $ENV_FILE"
    set -a
    source "$ENV_FILE"
    set +a
else
    echo_ts "Error: $ENV_FILE file not found. Exiting."
    exit 1
fi

# Check if RPC URLs are set
if [[ -z "$RPC_URL_1" || -z "$RPC_URL_2" ]]; then
    echo_ts "RPC URLs not set. Using default values."
    RPC_URL_1="http://localhost:8545"
    RPC_URL_2="http://localhost:8546"
    
    update_env_file "$ENV_FILE" "RPC_URL_1" "$RPC_URL_1"
    update_env_file "$ENV_FILE" "RPC_URL_2" "$RPC_URL_2"
fi

# Check if private keys are set
if [[ -z "$PRIVATE_KEY_1" ]]; then
    echo_ts "Private key not set. Using default Anvil private key."
    # First default Anvil private key
    PRIVATE_KEY_1="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    update_env_file "$ENV_FILE" "PRIVATE_KEY_1" "$PRIVATE_KEY_1"
fi

if [[ -z "$PRIVATE_KEY_2" ]]; then
    echo_ts "Private key not set. Using default Anvil private key."
    # Using same key for simplicity
    PRIVATE_KEY_2="$PRIVATE_KEY_1"
    update_env_file "$ENV_FILE" "PRIVATE_KEY_2" "$PRIVATE_KEY_2"
fi

# Deploy L1 contracts
echo_ts "Deploying L1 contracts..."
rpc_url="$RPC_URL_1"
private_key="$PRIVATE_KEY_1"
suffix="L1"

# Get the script directory and navigate to the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Change to the agglayer-contracts directory
cd "$PROJECT_ROOT/agglayer-contracts" || { echo_ts "Error: agglayer-contracts directory not found at $PROJECT_ROOT/agglayer-contracts"; exit 1; }

output=$(forge script script/deployL1.s.sol:DeployContractsL1 --rpc-url "$rpc_url" --broadcast --private-key "$private_key" 2>&1)
echo "$output" > deploy_output_$suffix.log

# Parse and update env file for L1
while read -r line; do
    if [[ $line =~ FflonkVerifier: ]]; then
        addr=$(echo $line | awk '{print $2}')
        update_env_file "$ENV_FILE" "FFLONK_VERIFIER_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVM: ]]; then
        addr=$(echo $line | awk '{print $2}')
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVMBridgeV2: ]]; then
        addr=$(echo $line | awk '{print $2}')
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_BRIDGE_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVMTimelock: ]]; then
        addr=$(echo $line | awk '{print $2}')
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_TIMELOCK_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVMGlobalExitRootV2: ]]; then
        addr=$(echo $line | awk '{print $2}')
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_GLOBAL_EXIT_ROOT_$suffix" "$addr"
    elif [[ $line =~ PolygonRollupManager: ]]; then
        addr=$(echo $line | awk '{print $2}')
        update_env_file "$ENV_FILE" "POLYGON_ROLLUP_MANAGER_$suffix" "$addr"
    fi
done < <(echo "$output" | grep -E "FflonkVerifier:|PolygonZkEVM:|PolygonZkEVMBridgeV2:|PolygonZkEVMTimelock:|PolygonZkEVMGlobalExitRootV2:|PolygonRollupManager:")

# Deploy L2 contracts
echo_ts "Deploying L2 contracts..."
rpc_url="$RPC_URL_2"
private_key="$PRIVATE_KEY_2"
suffix="L2"

# Note: The class name in deployL2.s.sol is DeployContractsL1, not DeployContractsL2
output=$(forge script script/deployL2.s.sol:DeployContractsL1 --rpc-url "$rpc_url" --broadcast --private-key "$private_key" 2>&1)
echo "$output" > deploy_output_$suffix.log

# Parse and update env file for L2
while read -r line; do
    if [[ $line =~ PolygonZkEVMBridgeV2: ]]; then
        addr=$(echo $line | awk '{print $2}')
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_BRIDGE_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVMTimelock: ]]; then
        addr=$(echo $line | awk '{print $2}')
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_TIMELOCK_$suffix" "$addr"
    fi
done < <(echo "$output" | grep -E "PolygonZkEVMBridgeV2:|PolygonZkEVMTimelock:")

# Return to the original directory
cd "$SCRIPT_DIR"

echo_ts "Contract addresses have been saved to $ENV_FILE"