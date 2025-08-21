#!/bin/bash
# Claim Bridge and Call Module
# Functions for claiming bridge and call transactions using aggsandbox CLI
# Part of the modular bridge test library

# ============================================================================
# CLAIM BRIDGE AND CALL FUNCTIONS
# ============================================================================

# Claim bridge and call using the modern aggsandbox CLI
claim_bridge_and_call_modern() {
    local dest_network="$1"
    local tx_hash="$2"
    local source_network="$3"
    local deposit_count="${4:-}" # Optional
    local private_key="${5:-}" # Optional
    
    print_step "Claiming bridge and call on network $dest_network"
    print_info "Source transaction: $tx_hash"
    print_info "Source network: $source_network"
    print_info "This will claim both the asset and execute the call"
    
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
        print_success "Bridge and call claim completed successfully"
        
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
        print_error "Bridge and call claim failed"
        print_error "Error: $output"
        
        # Check for common error patterns
        if echo "$output" | grep -q "AlreadyClaimed"; then
            print_warning "Bridge and call was already claimed"
            return 2  # Special return code for already claimed
        elif echo "$output" | grep -q "GlobalExitRootInvalid"; then
            print_warning "Global exit root invalid - may need to wait longer"
            return 3  # Special return code for GER issues
        elif echo "$output" | grep -q "UnclaimedAsset"; then
            print_warning "Asset must be claimed before message"
            return 4  # Special return code for unclaimed asset
        fi
        
        return 1
    fi
}

# Claim bridge and call with retry logic
claim_bridge_and_call_with_retry() {
    local dest_network="$1"
    local tx_hash="$2"
    local source_network="$3"
    local deposit_count="${4:-}" # Optional
    local private_key="${5:-}" # Optional
    local max_retries="${6:-3}"
    local retry_delay="${7:-10}"
    
    print_step "Claiming bridge and call with retry logic (max $max_retries attempts)"
    
    local attempt=1
    local claim_result
    
    while [ $attempt -le $max_retries ]; do
        print_info "Claim attempt $attempt/$max_retries"
        
        claim_result=$(claim_bridge_and_call_modern "$dest_network" "$tx_hash" "$source_network" "$deposit_count" "$private_key")
        local exit_code=$?
        
        case $exit_code in
            0)
                # Success
                print_success "Bridge and call claimed successfully on attempt $attempt"
                echo "$claim_result"
                return 0
                ;;
            2)
                # Already claimed
                print_info "Bridge and call was already claimed"
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
            4)
                # Unclaimed asset - this shouldn't happen with modern aggsandbox
                print_error "Unclaimed asset error - this may indicate a problem with the bridge and call flow"
                return 1
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

# Get bridge and call information from the bridge service
get_bridge_and_call_info() {
    local network_id="$1"
    local tx_hash="$2"
    
    print_step "Getting bridge and call information"
    print_info "Network: $network_id"
    print_info "Transaction: $tx_hash"
    
    # Get bridge information
    local bridge_info
    if bridge_info=$(get_bridge_info "$network_id" "$tx_hash"); then
        print_success "Bridge information retrieved"
        
        if [ "${DEBUG:-0}" = "1" ]; then
            print_debug "Bridge info: $bridge_info"
        fi
        
        # Parse the bridge info to extract asset and message deposits
        local asset_deposits
        local message_deposits
        
        # Look for asset bridges (leaf_type = 0)
        asset_deposits=$(echo "$bridge_info" | grep -E "leaf_type.*0" | wc -l || echo "0")
        
        # Look for message bridges (leaf_type = 1)  
        message_deposits=$(echo "$bridge_info" | grep -E "leaf_type.*1" | wc -l || echo "0")
        
        print_info "Asset deposits found: $asset_deposits"
        print_info "Message deposits found: $message_deposits"
        
        if [ "$asset_deposits" -gt "0" ] && [ "$message_deposits" -gt "0" ]; then
            print_success "✅ Bridge and call transaction found (has both asset and message)"
        elif [ "$asset_deposits" -gt "0" ]; then
            print_warning "⚠️ Only asset deposit found (missing message)"
        elif [ "$message_deposits" -gt "0" ]; then
            print_warning "⚠️ Only message deposit found (missing asset)"
        else
            print_warning "⚠️ No deposits found for this transaction"
        fi
        
        echo "$bridge_info"
        return 0
    else
        print_error "Failed to retrieve bridge information"
        return 1
    fi
}

# Wait for bridge and call to be claimable
wait_for_bridge_and_call_claimable() {
    local network_id="$1"
    local tx_hash="$2"
    local max_wait_time="${3:-90}" # Longer wait for bridge and call
    local check_interval="${4:-5}"
    
    print_step "Waiting for bridge and call to become claimable"
    print_info "Checking every ${check_interval}s for up to ${max_wait_time}s"
    print_info "Bridge and call requires both asset and message to be indexed"
    
    local elapsed_time=0
    local asset_found=false
    local message_found=false
    
    while [ $elapsed_time -lt $max_wait_time ]; do
        # Check if bridge is indexed and has both asset and message
        local bridge_info
        if bridge_info=$(get_bridge_info "$network_id" "$tx_hash" 2>/dev/null); then
            if echo "$bridge_info" | grep -q "$tx_hash"; then
                # Check for both asset and message deposits
                local asset_count
                local message_count
                
                asset_count=$(echo "$bridge_info" | grep -E "leaf_type.*0" | wc -l || echo "0")
                message_count=$(echo "$bridge_info" | grep -E "leaf_type.*1" | wc -l || echo "0")
                
                if [ "$asset_count" -gt "0" ]; then
                    asset_found=true
                fi
                
                if [ "$message_count" -gt "0" ]; then
                    message_found=true
                fi
                
                print_debug "Asset deposits: $asset_count, Message deposits: $message_count"
                
                if [ "$asset_found" = "true" ] && [ "$message_found" = "true" ]; then
                    print_success "Bridge and call is now claimable (both asset and message indexed)"
                    return 0
                fi
            fi
        fi
        
        local status_msg="Waiting..."
        if [ "$asset_found" = "true" ]; then
            status_msg="$status_msg Asset indexed."
        fi
        if [ "$message_found" = "true" ]; then
            status_msg="$status_msg Message indexed."
        fi
        
        print_debug "$status_msg"
        sleep $check_interval
        elapsed_time=$((elapsed_time + check_interval))
    done
    
    print_warning "Timeout waiting for bridge and call to become claimable"
    if [ "$asset_found" = "false" ]; then
        print_warning "Asset deposit not found"
    fi
    if [ "$message_found" = "false" ]; then
        print_warning "Message deposit not found"
    fi
    
    return 1
}

# Verify bridge and call claim was successful and call was executed
verify_bridge_and_call_claim() {
    local claim_tx_hash="$1"
    local network_id="$2"
    local expected_call_target="${3:-}" # Optional expected call target
    
    print_step "Verifying bridge and call claim execution"
    print_info "Claim transaction: $claim_tx_hash"
    print_info "Network: $network_id"
    
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
    if receipt=$(cast receipt "$claim_tx_hash" --rpc-url "$rpc_url" --json 2>/dev/null); then
        local status
        status=$(echo "$receipt" | jq -r '.status')
        
        if [ "$status" = "0x1" ]; then
            print_success "Bridge and call claim transaction was successful"
            
            # Analyze logs for bridge and call execution
            local logs
            logs=$(echo "$receipt" | jq '.logs')
            
            if [ "${DEBUG:-0}" = "1" ]; then
                print_debug "Transaction logs: $logs"
            fi
            
            # Count different types of events
            local total_logs
            total_logs=$(echo "$logs" | jq 'length')
            print_info "Total log entries: $total_logs"
            
            # Look for ClaimEvent logs (asset and message claims)
            local claim_events
            claim_events=$(echo "$logs" | jq -r '[.[] | select(.topics[0] == "0x25308c93ceeed775b33ab0a7fa6302fc6f1e36a6c5a8b3ad44b22e2d960529b6")] | length')
            print_info "Claim events found: $claim_events"
            
            # Check if expected call target was invoked
            if [ -n "$expected_call_target" ]; then
                local target_logs
                target_logs=$(echo "$logs" | jq -r --arg target "$expected_call_target" '[.[] | select(.address == $target)] | length')
                
                if [ "$target_logs" -gt "0" ]; then
                    print_success "✅ Call target $expected_call_target was invoked ($target_logs events)"
                else
                    print_warning "⚠️ Call target $expected_call_target was not invoked"
                fi
            fi
            
            # Look for any contract interactions beyond the bridge
            local unique_addresses
            unique_addresses=$(echo "$logs" | jq -r '[.[].address] | unique | length')
            print_info "Unique contract addresses in logs: $unique_addresses"
            
            if [ "$unique_addresses" -gt "1" ]; then
                print_success "✅ Multiple contracts involved - likely indicates call execution"
            fi
            
            return 0
        else
            print_error "Bridge and call claim transaction failed (status: $status)"
            
            # Try to get revert reason
            local revert_reason
            revert_reason=$(echo "$receipt" | jq -r '.revertReason // empty' 2>/dev/null || echo "")
            if [ -n "$revert_reason" ]; then
                print_error "Revert reason: $revert_reason"
            fi
            
            return 1
        fi
    else
        print_error "Could not retrieve claim transaction receipt"
        return 1
    fi
}

# Extract bridge and call details from transaction logs
extract_bridge_and_call_details() {
    local tx_hash="$1"
    local network_id="$2"
    
    print_step "Extracting bridge and call details from transaction"
    
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
        local logs
        logs=$(echo "$receipt" | jq '.logs')
        
        # Extract BridgeEvent logs (topic0: keccak256("BridgeEvent(uint8,uint32,address,uint32,address,uint256,bytes,uint32)"))
        local bridge_events
        bridge_events=$(echo "$logs" | jq -r '[.[] | select(.topics[0] == "0x3e54d0825ed78523037d00a81759237eb436ce774bd546993ee67a1b67b6e766")]')
        
        local bridge_event_count
        bridge_event_count=$(echo "$bridge_events" | jq 'length')
        
        print_info "Bridge events found: $bridge_event_count"
        
        if [ "$bridge_event_count" -eq "2" ]; then
            print_success "✅ Found expected 2 bridge events (asset + message)"
            
            # Extract details from each event
            local asset_event
            local message_event
            
            # Parse events to identify asset vs message
            for i in $(seq 0 $((bridge_event_count - 1))); do
                local event
                event=$(echo "$bridge_events" | jq ".[$i]")
                
                # Decode the leaf_type from the event data
                local data
                data=$(echo "$event" | jq -r '.data')
                
                # First 32 bytes of data is leaf_type
                local leaf_type_hex
                leaf_type_hex=$(echo "$data" | cut -c3-66)  # Remove 0x and take first 32 bytes
                local leaf_type
                leaf_type=$(printf "%d" "0x$leaf_type_hex" 2>/dev/null || echo "unknown")
                
                if [ "$leaf_type" = "0" ]; then
                    asset_event="$event"
                    print_info "Asset event found (leaf_type: 0)"
                elif [ "$leaf_type" = "1" ]; then
                    message_event="$event"
                    print_info "Message event found (leaf_type: 1)"
                fi
            done
            
            # Return structured information
            echo "{\"asset_event\": $asset_event, \"message_event\": $message_event, \"total_events\": $bridge_event_count}"
            return 0
        else
            print_warning "Unexpected number of bridge events: $bridge_event_count"
            echo "{\"total_events\": $bridge_event_count, \"events\": $bridge_events}"
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
export -f claim_bridge_and_call_modern claim_bridge_and_call_with_retry
export -f get_bridge_and_call_info wait_for_bridge_and_call_claimable
export -f verify_bridge_and_call_claim extract_bridge_and_call_details

print_debug "Claim bridge and call module loaded successfully"
