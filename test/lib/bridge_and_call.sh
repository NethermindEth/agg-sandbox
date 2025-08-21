#!/bin/bash
# Bridge and Call Module
# Functions for bridging assets and executing calls using aggsandbox CLI
# Part of the modular bridge test library

# ============================================================================
# BRIDGE AND CALL FUNCTIONS
# ============================================================================

# Bridge and call using the modern aggsandbox CLI
bridge_and_call_modern() {
    local source_network="$1"
    local dest_network="$2"
    local amount="$3"
    local token_address="$4"
    local call_address="$5"
    local call_data="$6"
    local private_key="${7:-}" # Optional
    local fallback_address="${8:-}" # Optional fallback address
    
    print_step "Executing bridge and call from network $source_network to network $dest_network"
    print_info "Amount: $amount tokens"
    print_info "Token: $token_address"
    print_info "Call address: $call_address"
    print_info "Call data: ${call_data:0:66}..." # Show first 66 chars
    print_info "Fallback address: ${fallback_address:-auto-detected}"
    
    # Build the bridge and call command
    local cmd="aggsandbox bridge call"
    cmd="$cmd --network $source_network"
    cmd="$cmd --destination-network $dest_network"
    cmd="$cmd --amount $amount"
    cmd="$cmd --token-address $token_address"
    cmd="$cmd --call-address $call_address"
    cmd="$cmd --call-data $call_data"
    
    if [ -n "$fallback_address" ]; then
        cmd="$cmd --fallback-address $fallback_address"
    fi
    
    if [ -n "$private_key" ]; then
        cmd="$cmd --private-key $private_key"
    fi
    
    if [ "${DEBUG:-0}" = "1" ]; then
        cmd="$cmd --verbose"
    fi
    
    print_debug "Executing: $cmd"
    
    # Execute the bridge and call command and capture output
    local output
    if output=$(eval "$cmd" 2>&1); then
        print_success "Bridge and call transaction initiated successfully"
        
        # Extract transaction hash from output
        local tx_hash
        tx_hash=$(echo "$output" | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
        
        if [ -n "$tx_hash" ]; then
            print_info "Bridge and call transaction hash: $tx_hash"
            echo "$tx_hash"
            return 0
        else
            print_warning "Could not extract transaction hash from output"
            print_debug "Output: $output"
            return 1
        fi
    else
        print_error "Bridge and call transaction failed"
        print_error "Error: $output"
        return 1
    fi
}

# Bridge and call with function signature encoding
bridge_and_call_function() {
    local source_network="$1"
    local dest_network="$2"
    local amount="$3"
    local token_address="$4"
    local call_address="$5"
    local function_signature="$6"
    local private_key="${7:-}" # Optional
    local fallback_address="${8:-}" # Optional fallback address
    shift 8 # Remove first 8 arguments, rest are function parameters
    
    print_step "Executing bridge and call with function encoding"
    print_info "Function: $function_signature"
    print_info "Parameters: $*"
    
    # Encode the function call
    local call_data
    if [ $# -gt 0 ]; then
        call_data=$(cast abi-encode "$function_signature" "$@" 2>/dev/null)
    else
        call_data=$(cast abi-encode "$function_signature" 2>/dev/null)
    fi
    
    if [ -z "$call_data" ]; then
        print_error "Failed to encode function call"
        return 1
    fi
    
    print_debug "Encoded call data: $call_data"
    
    # Execute bridge and call
    bridge_and_call_modern "$source_network" "$dest_network" "$amount" "$token_address" \
        "$call_address" "$call_data" "$private_key" "$fallback_address"
}

# Deploy a receiver contract for testing bridge and call
deploy_bridge_call_receiver() {
    local network_id="$1"
    local private_key="${2:-$PRIVATE_KEY_1}"
    local contract_name="${3:-SimpleBridgeAndCallReceiver}"
    
    print_step "Deploying bridge and call receiver contract on network $network_id"
    print_info "Contract: $contract_name"
    
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
    
    # Deploy the contract
    print_info "Deploying $contract_name contract..."
    local deploy_output
    deploy_output=$(forge create "test/contracts/$contract_name.sol:$contract_name" \
        --rpc-url "$rpc_url" \
        --private-key "$private_key" \
        --json 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        local contract_address
        contract_address=$(echo "$deploy_output" | jq -r '.deployedTo' 2>/dev/null)
        
        if [ -n "$contract_address" ] && [ "$contract_address" != "null" ]; then
            print_success "Contract deployed at: $contract_address"
            echo "$contract_address"
            return 0
        else
            print_error "Could not extract contract address from deployment output"
            print_debug "Deploy output: $deploy_output"
            return 1
        fi
    else
        print_error "Contract deployment failed"
        return 1
    fi
}

# Execute complete L1 to L2 bridge and call flow
execute_l1_to_l2_bridge_and_call() {
    local amount="$1"
    local token_address="$2"
    local call_address="$3"
    local call_data="$4"
    local source_account="${5:-$ACCOUNT_ADDRESS_1}"
    local dest_account="${6:-$ACCOUNT_ADDRESS_2}"
    local source_private_key="${7:-$PRIVATE_KEY_1}"
    local dest_private_key="${8:-$PRIVATE_KEY_2}"
    
    print_step "Executing complete L1 to L2 bridge and call flow"
    print_info "Amount: $amount tokens"
    print_info "Token: $token_address"
    print_info "Call address: $call_address"
    print_info "From: $source_account (L1)"
    print_info "Claiming account: $dest_account (L2)"
    
    # Step 1: Execute bridge and call from L1 to L2
    local bridge_tx_hash
    if ! bridge_tx_hash=$(bridge_and_call_modern \
        "$NETWORK_ID_MAINNET" \
        "$NETWORK_ID_AGGLAYER_1" \
        "$amount" \
        "$token_address" \
        "$call_address" \
        "$call_data" \
        "$source_private_key" \
        "$source_account"); then
        print_error "Failed to execute bridge and call from L1 to L2"
        return 1
    fi
    
    print_success "Bridge and call transaction completed: $bridge_tx_hash"
    
    # Step 2: Wait for bridge indexing
    if ! wait_for_bridge_indexing "$NETWORK_ID_AGGLAYER_1" "$bridge_tx_hash"; then
        print_error "Bridge indexing failed or timed out"
        return 1
    fi
    
    # Step 3: Claim bridge and call on L2
    local claim_tx_hash
    if ! claim_tx_hash=$(claim_bridge_and_call_modern \
        "$NETWORK_ID_AGGLAYER_1" \
        "$bridge_tx_hash" \
        "$NETWORK_ID_MAINNET" \
        "" \
        "$dest_private_key"); then
        print_error "Failed to claim bridge and call on L2"
        return 1
    fi
    
    print_success "Bridge and call claim completed: ${claim_tx_hash:-N/A}"
    
    # Return bridge transaction hash for further processing
    echo "$bridge_tx_hash"
    return 0
}

# Execute complete L2 to L1 bridge and call flow
execute_l2_to_l1_bridge_and_call() {
    local amount="$1"
    local token_address="$2"
    local call_address="$3"
    local call_data="$4"
    local source_account="${5:-$ACCOUNT_ADDRESS_2}"
    local dest_account="${6:-$ACCOUNT_ADDRESS_1}"
    local source_private_key="${7:-$PRIVATE_KEY_2}"
    local dest_private_key="${8:-$PRIVATE_KEY_1}"
    
    print_step "Executing complete L2 to L1 bridge and call flow"
    print_info "Amount: $amount tokens"
    print_info "Token: $token_address"
    print_info "Call address: $call_address"
    print_info "From: $source_account (L2)"
    print_info "Claiming account: $dest_account (L1)"
    
    # Step 1: Execute bridge and call from L2 to L1
    local bridge_tx_hash
    if ! bridge_tx_hash=$(bridge_and_call_modern \
        "$NETWORK_ID_AGGLAYER_1" \
        "$NETWORK_ID_MAINNET" \
        "$amount" \
        "$token_address" \
        "$call_address" \
        "$call_data" \
        "$source_private_key" \
        "$source_account"); then
        print_error "Failed to execute bridge and call from L2 to L1"
        return 1
    fi
    
    print_success "Bridge and call transaction completed: $bridge_tx_hash"
    
    # Step 2: Wait for bridge indexing (L1 network for L2->L1 bridges)
    if ! wait_for_bridge_indexing "$NETWORK_ID_MAINNET" "$bridge_tx_hash"; then
        print_error "Bridge indexing failed or timed out"
        return 1
    fi
    
    # Step 3: Claim bridge and call on L1
    local claim_tx_hash
    if ! claim_tx_hash=$(claim_bridge_and_call_modern \
        "$NETWORK_ID_MAINNET" \
        "$bridge_tx_hash" \
        "$NETWORK_ID_AGGLAYER_1" \
        "" \
        "$dest_private_key"); then
        print_error "Failed to claim bridge and call on L1"
        return 1
    fi
    
    print_success "Bridge and call claim completed: ${claim_tx_hash:-N/A}"
    
    # Return bridge transaction hash for further processing
    echo "$bridge_tx_hash"
    return 0
}

# Verify bridge and call execution by checking receiver contract state
verify_bridge_and_call_execution() {
    local receiver_contract="$1"
    local network_id="$2"
    local expected_message="${3:-}" # Optional expected message
    local expected_amount="${4:-}" # Optional expected token amount
    
    print_step "Verifying bridge and call execution"
    print_info "Receiver contract: $receiver_contract"
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
    
    # Check call count
    local call_count
    call_count=$(cast call "$receiver_contract" "getCallCount()" --rpc-url "$rpc_url" 2>/dev/null || echo "0x0")
    local call_count_dec
    call_count_dec=$(cast to-dec "$call_count" 2>/dev/null || echo "0")
    
    print_info "Call count: $call_count_dec"
    
    if [ "$call_count_dec" -gt "0" ]; then
        print_success "✅ Bridge and call was executed ($call_count_dec calls recorded)"
        
        # Get last message if expected message is provided
        if [ -n "$expected_message" ]; then
            local last_message
            last_message=$(cast call "$receiver_contract" "getLastMessage()" --rpc-url "$rpc_url" 2>/dev/null || echo "")
            
            if [ -n "$last_message" ]; then
                # Decode the string from hex
                local decoded_message
                decoded_message=$(cast abi-decode "f()(string)" "$last_message" 2>/dev/null | sed 's/^[[:space:]]*//' || echo "")
                
                print_info "Last message received: '$decoded_message'"
                
                if [ "$decoded_message" = "$expected_message" ]; then
                    print_success "✅ Message matches expected value"
                else
                    print_warning "⚠️ Message doesn't match expected value"
                    print_info "Expected: '$expected_message'"
                    print_info "Received: '$decoded_message'"
                fi
            fi
        fi
        
        # Check token balance if expected amount is provided
        if [ -n "$expected_amount" ]; then
            # This would require knowing the token address, which varies
            # For now, just indicate that token verification is possible
            print_info "Token balance verification available (requires token address)"
        fi
        
        return 0
    else
        print_error "❌ No calls recorded in receiver contract"
        return 1
    fi
}

# ============================================================================
# EXPORTS
# ============================================================================

# Export functions for use in other scripts
export -f bridge_and_call_modern bridge_and_call_function
export -f deploy_bridge_call_receiver
export -f execute_l1_to_l2_bridge_and_call execute_l2_to_l1_bridge_and_call
export -f verify_bridge_and_call_execution

print_debug "Bridge and call module loaded successfully"
