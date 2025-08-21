#!/bin/bash
# Bridge Message Module
# Functions for bridging messages using aggsandbox CLI
# Part of the modular bridge test library

# ============================================================================
# BRIDGE MESSAGE FUNCTIONS
# ============================================================================

# Bridge a message using the modern aggsandbox CLI
bridge_message_modern() {
    local source_network="$1"
    local dest_network="$2"
    local to_address="$3"
    local message_data="$4"
    local private_key="${5:-}" # Optional
    local force_update_ger="${6:-true}" # Optional, defaults to true
    
    print_step "Bridging message from network $source_network to network $dest_network"
    print_info "To address: $to_address"
    print_info "Message data: ${message_data:0:66}..." # Show first 66 chars
    
    # Build the bridge message command
    local cmd="aggsandbox bridge message"
    cmd="$cmd --network $source_network"
    cmd="$cmd --destination-network $dest_network"
    cmd="$cmd --to-address $to_address"
    cmd="$cmd --message-data $message_data"
    
    if [ "$force_update_ger" = "true" ]; then
        cmd="$cmd --force-update-global-exit-root"
    fi
    
    if [ -n "$private_key" ]; then
        cmd="$cmd --private-key $private_key"
    fi
    
    if [ "${DEBUG:-0}" = "1" ]; then
        cmd="$cmd --verbose"
    fi
    
    print_debug "Executing: $cmd"
    
    # Execute the bridge message command and capture output
    local output
    if output=$(eval "$cmd" 2>&1); then
        print_success "Bridge message transaction initiated successfully"
        
        # Extract transaction hash from output
        local tx_hash
        tx_hash=$(echo "$output" | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
        
        if [ -n "$tx_hash" ]; then
            print_info "Bridge message transaction hash: $tx_hash"
            echo "$tx_hash"
            return 0
        else
            print_warning "Could not extract transaction hash from output"
            print_debug "Output: $output"
            return 1
        fi
    else
        print_error "Bridge message transaction failed"
        print_error "Error: $output"
        return 1
    fi
}

# Bridge a simple text message
bridge_text_message() {
    local source_network="$1"
    local dest_network="$2"
    local to_address="$3"
    local text_message="$4"
    local private_key="${5:-}" # Optional
    
    print_step "Bridging text message: '$text_message'"
    
    # Encode the text message as bytes
    local message_data
    message_data=$(cast abi-encode "f(string)" "$text_message" 2>/dev/null)
    
    if [ -z "$message_data" ]; then
        print_error "Failed to encode text message"
        return 1
    fi
    
    print_debug "Encoded message data: $message_data"
    
    # Bridge the encoded message
    bridge_message_modern "$source_network" "$dest_network" "$to_address" "$message_data" "$private_key"
}

# Bridge a function call message
bridge_function_call_message() {
    local source_network="$1"
    local dest_network="$2"
    local to_address="$3"
    local function_signature="$4"
    local private_key="${5:-}" # Optional
    shift 5 # Remove first 5 arguments, rest are function parameters
    
    print_step "Bridging function call message"
    print_info "Function: $function_signature"
    print_info "Parameters: $*"
    
    # Encode the function call
    local message_data
    if [ $# -gt 0 ]; then
        message_data=$(cast abi-encode "$function_signature" "$@" 2>/dev/null)
    else
        message_data=$(cast abi-encode "$function_signature" 2>/dev/null)
    fi
    
    if [ -z "$message_data" ]; then
        print_error "Failed to encode function call"
        return 1
    fi
    
    print_debug "Encoded function call: $message_data"
    
    # Bridge the encoded function call
    bridge_message_modern "$source_network" "$dest_network" "$to_address" "$message_data" "$private_key"
}

# Execute complete L1 to L2 message bridge flow
execute_l1_to_l2_message_bridge() {
    local to_address="$1"
    local message_data="$2"
    local source_account="${3:-$ACCOUNT_ADDRESS_1}"
    local dest_account="${4:-$ACCOUNT_ADDRESS_2}"
    local source_private_key="${5:-$PRIVATE_KEY_1}"
    local dest_private_key="${6:-$PRIVATE_KEY_2}"
    
    print_step "Executing complete L1 to L2 message bridge flow"
    print_info "To address: $to_address"
    print_info "Message data: ${message_data:0:66}..."
    print_info "From: $source_account (L1)"
    print_info "Claiming account: $dest_account (L2)"
    
    # Step 1: Bridge message from L1 to L2
    local bridge_tx_hash
    if ! bridge_tx_hash=$(bridge_message_modern \
        "$NETWORK_ID_MAINNET" \
        "$NETWORK_ID_AGGLAYER_1" \
        "$to_address" \
        "$message_data" \
        "$source_private_key"); then
        print_error "Failed to bridge message from L1 to L2"
        return 1
    fi
    
    print_success "Bridge message transaction completed: $bridge_tx_hash"
    
    # Step 2: Wait for bridge indexing
    if ! wait_for_bridge_indexing "$NETWORK_ID_AGGLAYER_1" "$bridge_tx_hash"; then
        print_error "Bridge indexing failed or timed out"
        return 1
    fi
    
    # Step 3: Claim message on L2
    local claim_tx_hash
    if ! claim_tx_hash=$(claim_message_modern \
        "$NETWORK_ID_AGGLAYER_1" \
        "$bridge_tx_hash" \
        "$NETWORK_ID_MAINNET" \
        "" \
        "$dest_private_key"); then
        print_error "Failed to claim message on L2"
        return 1
    fi
    
    print_success "Claim message transaction completed: ${claim_tx_hash:-N/A}"
    
    # Return bridge transaction hash for further processing
    echo "$bridge_tx_hash"
    return 0
}

# Execute complete L2 to L1 message bridge flow
execute_l2_to_l1_message_bridge() {
    local to_address="$1"
    local message_data="$2"
    local source_account="${3:-$ACCOUNT_ADDRESS_2}"
    local dest_account="${4:-$ACCOUNT_ADDRESS_1}"
    local source_private_key="${5:-$PRIVATE_KEY_2}"
    local dest_private_key="${6:-$PRIVATE_KEY_1}"
    
    print_step "Executing complete L2 to L1 message bridge flow"
    print_info "To address: $to_address"
    print_info "Message data: ${message_data:0:66}..."
    print_info "From: $source_account (L2)"
    print_info "Claiming account: $dest_account (L1)"
    
    # Step 1: Bridge message from L2 to L1
    local bridge_tx_hash
    if ! bridge_tx_hash=$(bridge_message_modern \
        "$NETWORK_ID_AGGLAYER_1" \
        "$NETWORK_ID_MAINNET" \
        "$to_address" \
        "$message_data" \
        "$source_private_key"); then
        print_error "Failed to bridge message from L2 to L1"
        return 1
    fi
    
    print_success "Bridge message transaction completed: $bridge_tx_hash"
    
    # Step 2: Wait for bridge indexing (L1 network for L2->L1 bridges)
    if ! wait_for_bridge_indexing "$NETWORK_ID_MAINNET" "$bridge_tx_hash"; then
        print_error "Bridge indexing failed or timed out"
        return 1
    fi
    
    # Step 3: Claim message on L1
    local claim_tx_hash
    if ! claim_tx_hash=$(claim_message_modern \
        "$NETWORK_ID_MAINNET" \
        "$bridge_tx_hash" \
        "$NETWORK_ID_AGGLAYER_1" \
        "" \
        "$dest_private_key"); then
        print_error "Failed to claim message on L1"
        return 1
    fi
    
    print_success "Claim message transaction completed: ${claim_tx_hash:-N/A}"
    
    # Return bridge transaction hash for further processing
    echo "$bridge_tx_hash"
    return 0
}

# ============================================================================
# EXPORTS
# ============================================================================

# Export functions for use in other scripts
export -f bridge_message_modern bridge_text_message bridge_function_call_message
export -f execute_l1_to_l2_message_bridge execute_l2_to_l1_message_bridge

print_debug "Bridge message module loaded successfully"
