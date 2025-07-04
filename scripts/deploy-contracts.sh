#!/bin/bash

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
    
    echo_ts "Updating $var_name=$var_value in $env_file"
    
    # Create a temporary file
    local temp_file="${env_file}.tmp.$$"
    
    # If the env file doesn't exist, create it
    if [[ ! -f "$env_file" ]]; then
        touch "$env_file"
    fi
    
    # Process the file line by line
    local var_found=false
    while IFS= read -r line || [[ -n "$line" ]]; do
        # Check if line starts with the variable name (uncommented)
        if [[ "$line" == "${var_name}="* ]]; then
            # Replace existing variable
            echo "$var_name=$var_value" >> "$temp_file"
            var_found=true
            echo_ts "Found and replacing existing variable: $var_name"
        # Check if line starts with commented version
        elif [[ "$line" == "#"*"${var_name}="* ]] || [[ "$line" == "# ${var_name}="* ]]; then
            # Replace existing commented variable
            echo "$var_name=$var_value" >> "$temp_file"
            var_found=true
            echo_ts "Found and replacing commented variable: $var_name"
        else
            # Keep existing line
            echo "$line" >> "$temp_file"
        fi
    done < "$env_file"
    
    # If variable wasn't found, append it
    if [[ "$var_found" == "false" ]]; then
        echo "$var_name=$var_value" >> "$temp_file"
        echo_ts "Adding new variable: $var_name"
    fi
    
    # Replace the original file with the temporary file
    if mv "$temp_file" "$env_file"; then
        echo_ts "Successfully updated $var_name in $env_file"
    else
        echo_ts "Error: Failed to update $env_file"
        # Clean up temp file if move failed
        rm -f "$temp_file"
        return 1
    fi
}

# Handle env file argument
ENV_FILE="${1:-.env}"

# Make ENV_FILE an absolute path if it's not already
if [[ ! "$ENV_FILE" = /* ]]; then
    # Get the script directory and navigate to the project root
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
    ENV_FILE="$PROJECT_ROOT/$ENV_FILE"
fi

echo_ts "Using environment file: $ENV_FILE"

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
    RPC_URL_1="http://anvil-l1:8545"
    RPC_URL_2="http://anvil-l2:8545"
    
    update_env_file "$ENV_FILE" "RPC_URL_1" "$RPC_URL_1"
    update_env_file "$ENV_FILE" "RPC_URL_2" "$RPC_URL_2"
fi

# Handle RPC_URL_3 for multi-L2 mode if it exists
if [[ ! -z "$RPC_URL_3" ]]; then
    echo_ts "RPC_URL_3 is set. Using for third chain deployment."
    update_env_file "$ENV_FILE" "RPC_URL_3" "$RPC_URL_3"
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

echo_ts "L1 deployment output:"
echo "$output"

# Parse and update env file for L1
while read -r line; do
    if [[ $line =~ FflonkVerifier:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "FFLONK_VERIFIER_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVM:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVMBridgeV2:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_BRIDGE_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVMTimelock:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_TIMELOCK_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVMGlobalExitRootV2:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_GLOBAL_EXIT_ROOT_$suffix" "$addr"
    elif [[ $line =~ PolygonRollupManager:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "POLYGON_ROLLUP_MANAGER_$suffix" "$addr"
    elif [[ $line =~ AggERC20:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "AGG_ERC20_$suffix" "$addr"
    elif [[ $line =~ BridgeExtension:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "BRIDGE_EXTENSION_$suffix" "$addr"
    fi
done < <(echo "$output")

# Deploy L2 contracts
echo_ts "Deploying L2 contracts..."
rpc_url="$RPC_URL_2"
private_key="$PRIVATE_KEY_2"
suffix="L2"

# Note: The class name in deployL2.s.sol is DeployContractsL1, not DeployContractsL2
output=$(forge script script/deployL2.s.sol:DeployContractsL2 --rpc-url "$rpc_url" --broadcast --private-key "$private_key" 2>&1)
echo "$output" > deploy_output_$suffix.log

echo_ts "L2 deployment output:"
echo "$output"

# Parse and update env file for L2
while read -r line; do
    if [[ $line =~ PolygonZkEVMBridgeV2:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_BRIDGE_$suffix" "$addr"
    elif [[ $line =~ PolygonZkEVMTimelock:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "POLYGON_ZKEVM_TIMELOCK_$suffix" "$addr"
    elif [[ $line =~ GlobalExitRootManagerL2SovereignChain:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "GLOBAL_EXIT_ROOT_MANAGER_$suffix" "$addr"
    elif [[ $line =~ AggERC20:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "AGG_ERC20_$suffix" "$addr"
    elif [[ $line =~ BridgeExtension:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
        addr="${BASH_REMATCH[1]}"
        update_env_file "$ENV_FILE" "BRIDGE_EXTENSION_$suffix" "$addr"
    fi
done < <(echo "$output")

# === NEW SECTION: Deploy L3 contracts if RPC_URL_3 is provided ===
if [[ ! -z "$RPC_URL_3" ]]; then
    echo_ts "Deploying L3 contracts..."
    rpc_url="$RPC_URL_3"
    # Re-use PRIVATE_KEY_2 unless a specific key is provided
    private_key="${PRIVATE_KEY_3:-$PRIVATE_KEY_2}"
    suffix="L3"

    # Deploy contracts on the third chain (re-using the L2 deployment script)
    output=$(forge script script/deployL3.s.sol:DeployContractsL2 --rpc-url "$rpc_url" --broadcast --private-key "$private_key" 2>&1)
    echo "$output" > deploy_output_$suffix.log

    echo_ts "L3 deployment output:"
    echo "$output"

    # Parse and update env file for L3
    while read -r line; do
        if [[ $line =~ PolygonZkEVMBridgeV2:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
            addr="${BASH_REMATCH[1]}"
            update_env_file "$ENV_FILE" "POLYGON_ZKEVM_BRIDGE_$suffix" "$addr"
        elif [[ $line =~ PolygonZkEVMTimelock:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
            addr="${BASH_REMATCH[1]}"
            update_env_file "$ENV_FILE" "POLYGON_ZKEVM_TIMELOCK_$suffix" "$addr"
        elif [[ $line =~ GlobalExitRootManagerL2SovereignChain:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
            addr="${BASH_REMATCH[1]}"
            update_env_file "$ENV_FILE" "GLOBAL_EXIT_ROOT_MANAGER_$suffix" "$addr"
        elif [[ $line =~ AggERC20:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
            addr="${BASH_REMATCH[1]}"
            update_env_file "$ENV_FILE" "AGG_ERC20_$suffix" "$addr"
        elif [[ $line =~ BridgeExtension:[[:space:]]+([0-9a-fA-Fx]+) ]]; then
            addr="${BASH_REMATCH[1]}"
            update_env_file "$ENV_FILE" "BRIDGE_EXTENSION_$suffix" "$addr"
        fi
    done < <(echo "$output")

    echo_ts "L3 contract deployment complete. Addresses stored with *_L3 suffix in $ENV_FILE"
fi

# Return to the original directory
cd "$SCRIPT_DIR"

echo_ts "Contract addresses have been saved to $ENV_FILE"