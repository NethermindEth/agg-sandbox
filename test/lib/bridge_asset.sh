#!/bin/bash
# Bridge Asset Module
# Functions for bridging assets using aggsandbox CLI
# Part of the modular bridge test library

# ============================================================================
# BRIDGE ASSET FUNCTIONS
# ============================================================================

# Bridge assets using the modern aggsandbox CLI
bridge_asset_modern() {
    local source_network="$1"
    local dest_network="$2"
    local amount="$3"
    local token_address="$4"
    local to_address="${5:-}" # Optional
    local private_key="${6:-}" # Optional
    
    print_step "Bridging $amount tokens from network $source_network to network $dest_network"
    print_info "Token: $token_address"
    print_info "To address: ${to_address:-auto-detected}"
    
    # Build the bridge command
    local cmd="aggsandbox bridge asset"
    cmd="$cmd --network $source_network"
    cmd="$cmd --destination-network $dest_network"
    cmd="$cmd --amount $amount"
    cmd="$cmd --token-address $token_address"
    
    if [ -n "$to_address" ]; then
        cmd="$cmd --to-address $to_address"
    fi
    
    if [ -n "$private_key" ]; then
        cmd="$cmd --private-key $private_key"
    fi
    
    if [ "${DEBUG:-0}" = "1" ]; then
        cmd="$cmd --verbose"
    fi
    
    print_debug "Executing: $cmd"
    
    # Execute the bridge command and capture output
    local output
    if output=$(eval "$cmd" 2>&1); then
        print_success "Bridge transaction initiated successfully"
        
        # Extract transaction hash from output
        local tx_hash
        tx_hash=$(echo "$output" | grep -o '0x[a-fA-F0-9]\{64\}' | head -1)
        
        if [ -n "$tx_hash" ]; then
            print_info "Bridge transaction hash: $tx_hash"
            echo "$tx_hash"
            return 0
        else
            print_warning "Could not extract transaction hash from output"
            print_debug "Output: $output"
            return 1
        fi
    else
        print_error "Bridge transaction failed"
        print_error "Error: $output"
        return 1
    fi
}

# Bridge asset with automatic approval (if needed)
bridge_asset_with_approval() {
    local source_network="$1"
    local dest_network="$2"
    local amount="$3"
    local token_address="$4"
    local to_address="${5:-}" # Optional
    local private_key="${6:-}" # Optional
    local bridge_contract="${7:-}" # Optional bridge contract address
    
    print_step "Bridging asset with automatic approval handling"
    
    # Check if approval is needed (for ERC20 tokens)
    if [ "$token_address" != "0x0000000000000000000000000000000000000000" ] && [ -n "$bridge_contract" ]; then
        print_info "Checking token approval..."
        
        # Get RPC URL based on source network
        local rpc_url
        case "$source_network" in
            0|"${NETWORK_ID_MAINNET:-0}")
                rpc_url="$RPC_1"
                ;;
            1|"${NETWORK_ID_AGGLAYER_1:-1}")
                rpc_url="$RPC_2"
                ;;
            *)
                print_error "Unknown source network ID: $source_network"
                return 1
                ;;
        esac
        
        # Check current allowance
        local account_address
        if [ -n "$private_key" ]; then
            account_address=$(cast wallet address "$private_key" 2>/dev/null || echo "")
        fi
        
        if [ -n "$account_address" ]; then
            local allowance
            allowance=$(cast call "$token_address" "allowance(address,address)" "$account_address" "$bridge_contract" --rpc-url "$rpc_url" 2>/dev/null || echo "0x0")
            local allowance_dec
            allowance_dec=$(cast to-dec "$allowance" 2>/dev/null || echo "0")
            
            if [ "$allowance_dec" -lt "$amount" ]; then
                print_info "Insufficient allowance ($allowance_dec < $amount), approving..."
                
                # Approve the bridge contract
                local approve_tx
                approve_tx=$(cast send "$token_address" \
                    "approve(address,uint256)" \
                    "$bridge_contract" "$amount" \
                    --private-key "$private_key" \
                    --rpc-url "$rpc_url" \
                    --json 2>/dev/null | jq -r '.transactionHash' || echo "")
                
                if [ -n "$approve_tx" ]; then
                    print_success "Approval transaction: $approve_tx"
                    sleep 2 # Wait for confirmation
                else
                    print_warning "Could not approve token, proceeding anyway..."
                fi
            else
                print_info "Sufficient allowance already exists"
            fi
        fi
    fi
    
    # Execute the bridge transaction
    bridge_asset_modern "$source_network" "$dest_network" "$amount" "$token_address" "$to_address" "$private_key"
}

# Execute complete L1 to L2 asset bridge flow
execute_l1_to_l2_asset_bridge() {
    local amount="${1:-$DEFAULT_BRIDGE_AMOUNT}"
    local token_address="${2:-$AGG_ERC20_L1}"
    local source_account="${3:-$ACCOUNT_ADDRESS_1}"
    local dest_account="${4:-$ACCOUNT_ADDRESS_2}"
    local source_private_key="${5:-$PRIVATE_KEY_1}"
    local dest_private_key="${6:-$PRIVATE_KEY_2}"
    
    print_step "Executing complete L1 to L2 asset bridge flow"
    print_info "Amount: $amount tokens"
    print_info "Token: $token_address"
    print_info "From: $source_account (L1)"
    print_info "To: $dest_account (L2)"
    
    # Step 1: Bridge assets from L1 to L2
    local bridge_tx_hash
    if ! bridge_tx_hash=$(bridge_asset_modern \
        "$NETWORK_ID_MAINNET" \
        "$NETWORK_ID_AGGLAYER_1" \
        "$amount" \
        "$token_address" \
        "$dest_account" \
        "$source_private_key"); then
        print_error "Failed to bridge assets from L1 to L2"
        return 1
    fi
    
    print_success "Bridge transaction completed: $bridge_tx_hash"
    
    # Step 2: Wait for bridge indexing
    if ! wait_for_bridge_indexing "$NETWORK_ID_AGGLAYER_1" "$bridge_tx_hash"; then
        print_error "Bridge indexing failed or timed out"
        return 1
    fi
    
    # Step 3: Claim assets on L2
    local claim_tx_hash
    if ! claim_tx_hash=$(claim_asset_modern \
        "$NETWORK_ID_AGGLAYER_1" \
        "$bridge_tx_hash" \
        "$NETWORK_ID_MAINNET" \
        "" \
        "$dest_private_key"); then
        print_error "Failed to claim assets on L2"
        return 1
    fi
    
    print_success "Claim transaction completed: ${claim_tx_hash:-N/A}"
    
    # Return bridge transaction hash for further processing
    echo "$bridge_tx_hash"
    return 0
}

# Execute complete L2 to L1 asset bridge flow
execute_l2_to_l1_asset_bridge() {
    local amount="${1:-$DEFAULT_BRIDGE_AMOUNT}"
    local token_address="${2:-}" # L2 wrapped token address
    local source_account="${3:-$ACCOUNT_ADDRESS_2}"
    local dest_account="${4:-$ACCOUNT_ADDRESS_1}"
    local source_private_key="${5:-$PRIVATE_KEY_2}"
    local dest_private_key="${6:-$PRIVATE_KEY_1}"
    
    print_step "Executing complete L2 to L1 asset bridge flow"
    print_info "Amount: $amount tokens"
    print_info "Token: $token_address"
    print_info "From: $source_account (L2)"
    print_info "To: $dest_account (L1)"
    
    # Step 1: Bridge assets from L2 to L1
    local bridge_tx_hash
    if ! bridge_tx_hash=$(bridge_asset_modern \
        "$NETWORK_ID_AGGLAYER_1" \
        "$NETWORK_ID_MAINNET" \
        "$amount" \
        "$token_address" \
        "$dest_account" \
        "$source_private_key"); then
        print_error "Failed to bridge assets from L2 to L1"
        return 1
    fi
    
    print_success "Bridge transaction completed: $bridge_tx_hash"
    
    # Step 2: Wait for bridge indexing (L1 network for L2->L1 bridges)
    if ! wait_for_bridge_indexing "$NETWORK_ID_MAINNET" "$bridge_tx_hash"; then
        print_error "Bridge indexing failed or timed out"
        return 1
    fi
    
    # Step 3: Claim assets on L1
    local claim_tx_hash
    if ! claim_tx_hash=$(claim_asset_modern \
        "$NETWORK_ID_MAINNET" \
        "$bridge_tx_hash" \
        "$NETWORK_ID_AGGLAYER_1" \
        "" \
        "$dest_private_key"); then
        print_error "Failed to claim assets on L1"
        return 1
    fi
    
    print_success "Claim transaction completed: ${claim_tx_hash:-N/A}"
    
    # Return bridge transaction hash for further processing
    echo "$bridge_tx_hash"
    return 0
}

# ============================================================================
# EXPORTS
# ============================================================================

# Export functions for use in other scripts
export -f bridge_asset_modern bridge_asset_with_approval
export -f execute_l1_to_l2_asset_bridge execute_l2_to_l1_asset_bridge

print_debug "Bridge asset module loaded successfully"
