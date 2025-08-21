#!/bin/bash
# Claim Asset Module
# Functions for claiming bridged assets using aggsandbox CLI
# Part of the modular bridge test library

# ============================================================================
# CLAIM ASSET FUNCTIONS
# ============================================================================

# Claim bridged assets using the modern aggsandbox CLI
claim_asset_modern() {
    local dest_network="$1"
    local tx_hash="$2"
    local source_network="$3"
    local deposit_count="${4:-}" # Optional
    local private_key="${5:-}" # Optional
    
    print_step "Claiming bridged assets on network $dest_network"
    print_info "Source transaction: $tx_hash"
    print_info "Source network: $source_network"
    
    # Build the claim command
    local cmd="aggsandbox bridge claim"
    cmd="$cmd --network $dest_network"
    cmd="$cmd --tx-hash $tx_hash"
    cmd="$cmd --source-network $source_network"
    
    if [ -n "$deposit_count" ]; then
        cmd="$cmd --deposit-count $deposit_count"
    fi
    
    if [ -n "$private_key" ]; then
        cmd="$cmd --private-key $private_key"
    fi
    
    if [ "${DEBUG:-0}" = "1" ]; then
        cmd="$cmd --verbose"
    fi
    
    print_debug "Executing: $cmd"
    
    # Execute the claim command
    local output
    if output=$(eval "$cmd" 2>&1); then
        print_success "Claim transaction completed successfully"
        
        # Extract transaction hash from output
        local claim_tx_hash
        claim_tx_hash=$(echo "$output" | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
        
        if [ -n "$claim_tx_hash" ]; then
            print_info "Claim transaction hash: $claim_tx_hash"
            echo "$claim_tx_hash"
            return 0
        else
            print_warning "Could not extract claim transaction hash"
            print_debug "Output: $output"
            return 0  # Still success even if we can't extract the hash
        fi
    else
        print_error "Claim transaction failed"
        print_error "Error: $output"
        
        # Check for common error patterns
        if echo "$output" | grep -q "AlreadyClaimed"; then
            print_warning "Asset was already claimed"
            return 2  # Special return code for already claimed
        elif echo "$output" | grep -q "GlobalExitRootInvalid"; then
            print_warning "Global exit root invalid - may need to wait longer"
            return 3  # Special return code for GER issues
        fi
        
        return 1
    fi
}

# Claim asset with retry logic
claim_asset_with_retry() {
    local dest_network="$1"
    local tx_hash="$2"
    local source_network="$3"
    local deposit_count="${4:-}" # Optional
    local private_key="${5:-}" # Optional
    local max_retries="${6:-3}"
    local retry_delay="${7:-10}"
    
    print_step "Claiming asset with retry logic (max $max_retries attempts)"
    
    local attempt=1
    local claim_result
    
    while [ $attempt -le $max_retries ]; do
        print_info "Claim attempt $attempt/$max_retries"
        
        claim_result=$(claim_asset_modern "$dest_network" "$tx_hash" "$source_network" "$deposit_count" "$private_key")
        local exit_code=$?
        
        case $exit_code in
            0)
                # Success
                print_success "Asset claimed successfully on attempt $attempt"
                echo "$claim_result"
                return 0
                ;;
            2)
                # Already claimed
                print_info "Asset was already claimed"
                return 0
                ;;
            3)
                # GER invalid - retry after delay
                if [ $attempt -lt $max_retries ]; then
                    print_warning "Global exit root invalid, retrying in ${retry_delay}s..."
                    sleep $retry_delay
                else
                    print_error "Max retries reached, global exit root still invalid"
                    return 1
                fi
                ;;
            *)
                # Other error - retry with shorter delay
                if [ $attempt -lt $max_retries ]; then
                    print_warning "Claim failed, retrying in 5s..."
                    sleep 5
                else
                    print_error "Max retries reached, claim failed"
                    return 1
                fi
                ;;
        esac
        
        attempt=$((attempt + 1))
    done
    
    return 1
}

# Get claim proof information for manual claiming
get_claim_proof() {
    local network_id="$1"
    local deposit_count="$2"
    local leaf_index="${3:-}" # Optional, will use deposit_count if not provided
    
    print_step "Getting claim proof for network $network_id"
    print_info "Deposit count: $deposit_count"
    print_info "Leaf index: ${leaf_index:-$deposit_count}"
    
    # Use leaf index if provided, otherwise use deposit count
    local index_to_use="${leaf_index:-$deposit_count}"
    
    # Get proof using aggsandbox
    local proof_output
    if proof_output=$(aggsandbox show claim-proof \
        --network-id "$network_id" \
        --leaf-index "$index_to_use" \
        --deposit-count "$deposit_count" 2>&1); then
        
        print_success "Claim proof retrieved successfully"
        
        # Extract JSON from the output
        local proof_json
        proof_json=$(echo "$proof_output" | sed -n '/^{/,/^}/p')
        
        if [ -n "$proof_json" ]; then
            print_debug "Proof JSON: $proof_json"
            echo "$proof_json"
            return 0
        else
            print_error "Could not extract JSON from proof output"
            print_debug "Raw output: $proof_output"
            return 1
        fi
    else
        print_error "Failed to get claim proof"
        print_error "Error: $proof_output"
        return 1
    fi
}

# Check if an asset is already claimed
is_asset_claimed() {
    local network_id="$1"
    local deposit_count="$2"
    local source_network="${3:-0}" # Default to network 0
    
    print_debug "Checking if asset is claimed: deposit_count=$deposit_count, source_network=$source_network"
    
    # Get the bridge contract address for the network
    local bridge_contract
    case "$network_id" in
        0|"${NETWORK_ID_MAINNET:-0}")
            bridge_contract="$POLYGON_ZKEVM_BRIDGE_L1"
            ;;
        1|"${NETWORK_ID_AGGLAYER_1:-1}")
            bridge_contract="$POLYGON_ZKEVM_BRIDGE_L2"
            ;;
        *)
            print_error "Unknown network ID for claim check: $network_id"
            return 1
            ;;
    esac
    
    if [ -z "$bridge_contract" ]; then
        print_warning "Bridge contract not set for network $network_id"
        return 1
    fi
    
    # Get RPC URL
    local rpc_url
    case "$network_id" in
        0|"${NETWORK_ID_MAINNET:-0}")
            rpc_url="$RPC_1"
            ;;
        1|"${NETWORK_ID_AGGLAYER_1:-1}")
            rpc_url="$RPC_2"
            ;;
        *)
            print_error "Unknown network ID for RPC: $network_id"
            return 1
            ;;
    esac
    
    # Check if claimed using the bridge contract
    local is_claimed
    is_claimed=$(cast call "$bridge_contract" \
        "isClaimed(uint32,uint32)" \
        "$deposit_count" "$source_network" \
        --rpc-url "$rpc_url" 2>/dev/null || echo "false")
    
    print_debug "Asset claimed status: $is_claimed"
    
    if [ "$is_claimed" = "true" ]; then
        return 0  # Already claimed
    else
        return 1  # Not claimed
    fi
}

# Wait for asset to be claimable (GER propagation)
wait_for_asset_claimable() {
    local network_id="$1"
    local tx_hash="$2"
    local max_wait_time="${3:-60}" # Maximum wait time in seconds
    local check_interval="${4:-5}" # Check interval in seconds
    
    print_step "Waiting for asset to become claimable"
    print_info "Checking every ${check_interval}s for up to ${max_wait_time}s"
    
    local elapsed_time=0
    
    while [ $elapsed_time -lt $max_wait_time ]; do
        # Check if bridge is indexed
        local bridge_info
        if bridge_info=$(get_bridge_info "$network_id" "$tx_hash" 2>/dev/null); then
            if echo "$bridge_info" | grep -q "$tx_hash"; then
                print_success "Asset is now claimable"
                return 0
            fi
        fi
        
        print_debug "Asset not yet claimable, waiting..."
        sleep $check_interval
        elapsed_time=$((elapsed_time + check_interval))
    done
    
    print_warning "Timeout waiting for asset to become claimable"
    return 1
}

# Verify claim transaction was successful
verify_claim_transaction() {
    local tx_hash="$1"
    local network_id="$2"
    
    print_step "Verifying claim transaction: $tx_hash"
    
    # Get RPC URL
    local rpc_url
    case "$network_id" in
        0|"${NETWORK_ID_MAINNET:-0}")
            rpc_url="$RPC_1"
            ;;
        1|"${NETWORK_ID_AGGLAYER_1:-1}")
            rpc_url="$RPC_2"
            ;;
        *)
            print_error "Unknown network ID: $network_id"
            return 1
            ;;
    esac
    
    # Get transaction receipt
    local receipt
    if receipt=$(cast receipt "$tx_hash" --rpc-url "$rpc_url" --json 2>/dev/null); then
        local status
        status=$(echo "$receipt" | jq -r '.status')
        
        if [ "$status" = "0x1" ]; then
            print_success "Claim transaction was successful"
            
            # Extract logs for debugging
            if [ "${DEBUG:-0}" = "1" ]; then
                local logs
                logs=$(echo "$receipt" | jq '.logs')
                print_debug "Transaction logs: $logs"
            fi
            
            return 0
        else
            print_error "Claim transaction failed (status: $status)"
            return 1
        fi
    else
        print_error "Could not retrieve transaction receipt"
        return 1
    fi
}

# ============================================================================
# EXPORTS
# ============================================================================

# Export functions for use in other scripts
export -f claim_asset_modern claim_asset_with_retry
export -f get_claim_proof is_asset_claimed wait_for_asset_claimable verify_claim_transaction

print_debug "Claim asset module loaded successfully"
