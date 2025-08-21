#!/bin/bash
# Claim Message Module
# Functions for claiming bridged messages using aggsandbox CLI
# Part of the modular bridge test library

# ============================================================================
# CLAIM MESSAGE FUNCTIONS
# ============================================================================

# Claim bridged messages using the modern aggsandbox CLI
claim_message_modern() {
    local dest_network="$1"
    local tx_hash="$2"
    local source_network="$3"
    local deposit_count="${4:-}" # Optional
    local private_key="${5:-}" # Optional
    
    print_step "Claiming bridged message on network $dest_network"
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
        print_success "Claim message transaction completed successfully"
        
        # Extract transaction hash from output
        local claim_tx_hash
        claim_tx_hash=$(echo "$output" | grep -oE '0x[a-fA-F0-9]{64}' | head -1)
        
        if [ -n "$claim_tx_hash" ]; then
            print_info "Claim message transaction hash: $claim_tx_hash"
            echo "$claim_tx_hash"
            return 0
        else
            print_warning "Could not extract claim transaction hash"
            print_debug "Output: $output"
            return 0  # Still success even if we can't extract the hash
        fi
    else
        print_error "Claim message transaction failed"
        print_error "Error: $output"
        
        # Check for common error patterns
        if echo "$output" | grep -q "AlreadyClaimed"; then
            print_warning "Message was already claimed"
            return 2  # Special return code for already claimed
        elif echo "$output" | grep -q "GlobalExitRootInvalid"; then
            print_warning "Global exit root invalid - may need to wait longer"
            return 3  # Special return code for GER issues
        fi
        
        return 1
    fi
}

# Claim message with retry logic
claim_message_with_retry() {
    local dest_network="$1"
    local tx_hash="$2"
    local source_network="$3"
    local deposit_count="${4:-}" # Optional
    local private_key="${5:-}" # Optional
    local max_retries="${6:-3}"
    local retry_delay="${7:-10}"
    
    print_step "Claiming message with retry logic (max $max_retries attempts)"
    
    local attempt=1
    local claim_result
    
    while [ $attempt -le $max_retries ]; do
        print_info "Claim attempt $attempt/$max_retries"
        
        claim_result=$(claim_message_modern "$dest_network" "$tx_hash" "$source_network" "$deposit_count" "$private_key")
        local exit_code=$?
        
        case $exit_code in
            0)
                # Success
                print_success "Message claimed successfully on attempt $attempt"
                echo "$claim_result"
                return 0
                ;;
            2)
                # Already claimed
                print_info "Message was already claimed"
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

# Check if a message is already claimed
is_message_claimed() {
    local network_id="$1"
    local deposit_count="$2"
    local source_network="${3:-0}" # Default to network 0
    
    print_debug "Checking if message is claimed: deposit_count=$deposit_count, source_network=$source_network"
    
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
    
    print_debug "Message claimed status: $is_claimed"
    
    if [ "$is_claimed" = "true" ]; then
        return 0  # Already claimed
    else
        return 1  # Not claimed
    fi
}

# Get message proof information for manual claiming
get_message_proof() {
    local network_id="$1"
    local deposit_count="$2"
    local leaf_index="${3:-}" # Optional, will use deposit_count if not provided
    
    print_step "Getting message proof for network $network_id"
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
        
        print_success "Message proof retrieved successfully"
        
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
        print_error "Failed to get message proof"
        print_error "Error: $proof_output"
        return 1
    fi
}

# Wait for message to be claimable (GER propagation)
wait_for_message_claimable() {
    local network_id="$1"
    local tx_hash="$2"
    local max_wait_time="${3:-60}" # Maximum wait time in seconds
    local check_interval="${4:-5}" # Check interval in seconds
    
    print_step "Waiting for message to become claimable"
    print_info "Checking every ${check_interval}s for up to ${max_wait_time}s"
    
    local elapsed_time=0
    
    while [ $elapsed_time -lt $max_wait_time ]; do
        # Check if bridge is indexed
        local bridge_info
        if bridge_info=$(get_bridge_info "$network_id" "$tx_hash" 2>/dev/null); then
            if echo "$bridge_info" | grep -q "$tx_hash"; then
                print_success "Message is now claimable"
                return 0
            fi
        fi
        
        print_debug "Message not yet claimable, waiting..."
        sleep $check_interval
        elapsed_time=$((elapsed_time + check_interval))
    done
    
    print_warning "Timeout waiting for message to become claimable"
    return 1
}

# Verify message claim transaction was successful and extract events
verify_message_claim_transaction() {
    local tx_hash="$1"
    local network_id="$2"
    local expected_target="${3:-}" # Optional: expected target contract
    
    print_step "Verifying message claim transaction: $tx_hash"
    
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
            print_success "Message claim transaction was successful"
            
            # Extract and analyze logs
            if [ "${DEBUG:-0}" = "1" ]; then
                local logs
                logs=$(echo "$receipt" | jq '.logs')
                print_debug "Transaction logs: $logs"
                
                # Look for ClaimEvent logs
                local claim_events
                claim_events=$(echo "$receipt" | jq -r '.logs[] | select(.topics[0] == "0x25308c93ceeed775b33ab0a7fa6302fc6f1e36a6c5a8b3ad44b22e2d960529b6") | .data')
                if [ -n "$claim_events" ] && [ "$claim_events" != "null" ]; then
                    print_debug "Found ClaimEvent: $claim_events"
                fi
            fi
            
            # Check if expected target was called (if provided)
            if [ -n "$expected_target" ]; then
                local target_called=false
                local logs
                logs=$(echo "$receipt" | jq -r '.logs[].address')
                
                while IFS= read -r log_address; do
                    if [ "$log_address" = "$expected_target" ]; then
                        target_called=true
                        break
                    fi
                done <<< "$logs"
                
                if [ "$target_called" = "true" ]; then
                    print_success "Target contract $expected_target was called"
                else
                    print_warning "Target contract $expected_target was not called (may be normal)"
                fi
            fi
            
            return 0
        else
            print_error "Message claim transaction failed (status: $status)"
            return 1
        fi
    else
        print_error "Could not retrieve transaction receipt"
        return 1
    fi
}

# Decode message data for debugging
decode_message_data() {
    local message_data="$1"
    local function_signature="${2:-}" # Optional function signature for decoding
    
    print_step "Decoding message data"
    print_info "Raw data: $message_data"
    
    if [ -n "$function_signature" ]; then
        # Try to decode with provided function signature
        local decoded
        if decoded=$(cast abi-decode "$function_signature" "$message_data" 2>/dev/null); then
            print_success "Decoded with signature '$function_signature': $decoded"
            echo "$decoded"
            return 0
        else
            print_warning "Could not decode with provided signature"
        fi
    fi
    
    # Try common decodings
    # Try as string
    local as_string
    if as_string=$(cast abi-decode "f(string)" "$message_data" 2>/dev/null); then
        print_info "As string: $as_string"
    fi
    
    # Try as bytes
    local as_bytes
    if as_bytes=$(cast abi-decode "f(bytes)" "$message_data" 2>/dev/null); then
        print_info "As bytes: $as_bytes"
    fi
    
    # Show hex representation
    print_info "Hex data: $message_data"
    
    return 0
}

# ============================================================================
# EXPORTS
# ============================================================================

# Export functions for use in other scripts
export -f claim_message_modern claim_message_with_retry
export -f is_message_claimed get_message_proof wait_for_message_claimable
export -f verify_message_claim_transaction decode_message_data

print_debug "Claim message module loaded successfully"
