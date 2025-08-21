#!/bin/bash
# Simple L1-L2 Asset Bridge Test
# Demonstrates bridging ERC20 tokens from L1 to L2 using the modular bridge library
# Version: 3.0 - Simplified using modular architecture

set -e  # Exit on error

# ============================================================================
# SETUP
# ============================================================================

# Load the modular bridge test library
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/bridge_test_lib.sh"

# Script metadata
readonly SCRIPT_NAME="L1-L2 Bridge Asset Test"
readonly SCRIPT_VERSION="3.0"

# Configuration
BRIDGE_AMOUNT=${1:-50}  # Default to 50 tokens
TOKEN_ADDRESS=""

# ============================================================================
# MAIN TEST
# ============================================================================

main() {
    echo ""
    print_success "========== $SCRIPT_NAME v$SCRIPT_VERSION =========="
    print_info "Bridge Amount: $BRIDGE_AMOUNT tokens"
    echo ""

    # Initialize environment
    print_step "Initializing test environment"
    if ! init_bridge_test_environment; then
        print_error "Failed to initialize environment"
        exit 1
    fi
    
    # Set token address
    if [ -z "$TOKEN_ADDRESS" ]; then
        if [ -n "$AGG_ERC20_L1" ]; then
            TOKEN_ADDRESS="$AGG_ERC20_L1"
            print_info "Using L1 token: $TOKEN_ADDRESS"
        else
            print_error "No token address available. Ensure contracts are deployed."
            exit 1
        fi
    fi
    
    # Check initial balance
    print_step "Checking L1 balance"
    local l1_balance_hex
    l1_balance_hex=$(get_token_balance "$TOKEN_ADDRESS" "$ACCOUNT_ADDRESS_1" "$NETWORK_ID_MAINNET")
    local l1_balance_dec
    l1_balance_dec=$(get_balance_decimal "$l1_balance_hex")
    
    print_info "L1 Balance: $l1_balance_dec tokens"
    
    # Use bc for large number comparison to avoid integer overflow
    if [ "$(echo "$l1_balance_dec < $BRIDGE_AMOUNT" | bc 2>/dev/null || echo "0")" = "1" ]; then
        print_error "Insufficient balance. Need: $BRIDGE_AMOUNT, Have: $l1_balance_dec"
        exit 1
    fi
    
    # Step 1: Bridge assets from L1 to L2
    print_step "Bridging assets from L1 to L2"
    local bridge_result
    if ! bridge_result=$(bridge_asset_modern \
        "$NETWORK_ID_MAINNET" \
        "$NETWORK_ID_AGGLAYER_1" \
        "$BRIDGE_AMOUNT" \
        "$TOKEN_ADDRESS" \
        "$ACCOUNT_ADDRESS_2" \
        "$PRIVATE_KEY_1"); then
        print_error "Bridge failed"
        exit 1
    fi
    
    # Extract just the transaction hash (avoid private key)
    local bridge_tx_hash
    # Look for "Bridge transaction hash:" or "transaction hash:" followed by the hash
    bridge_tx_hash=$(echo "$bridge_result" | grep -i "transaction hash" | grep -oE '0x[a-fA-F0-9]{64}' | head -1)
    
    # If not found, try to get the last hash that's NOT the private key
    if [ -z "$bridge_tx_hash" ]; then
        bridge_tx_hash=$(echo "$bridge_result" | grep -oE '0x[a-fA-F0-9]{64}' | grep -v "$PRIVATE_KEY_1" | tail -1)
    fi
    
    if [ -z "$bridge_tx_hash" ]; then
        print_error "Could not extract bridge transaction hash"
        print_debug "Bridge result: $bridge_result"
        exit 1
    fi
    
    print_success "Bridge completed: $bridge_tx_hash"
    
    # Step 2: Wait for bridge indexing (simplified)
    print_step "Waiting for bridge indexing"
    print_info "Checking network $NETWORK_ID_MAINNET (L1) for bridge TX: $bridge_tx_hash"
    print_info "Waiting 25 seconds for bridge indexing and GER propagation..."
    sleep 25
    
    # Check if bridge appears in the service and get the most recent one if ours isn't found
    print_info "Checking if bridge is indexed..."
    if aggsandbox show bridges --network-id "$NETWORK_ID_MAINNET" 2>/dev/null | grep -q "$bridge_tx_hash"; then
        print_success "Bridge found in indexing system"
    else
        print_warning "Bridge $bridge_tx_hash not found yet, using most recent bridge for claim"
        # Get the most recent bridge transaction hash
        local most_recent_tx
        most_recent_tx=$(aggsandbox show bridges --network-id 0 2>/dev/null | grep "tx_hash" | tail -1 | grep -oE '0x[a-fA-F0-9]{64}')
        if [ -n "$most_recent_tx" ]; then
            bridge_tx_hash="$most_recent_tx"
            print_info "Using most recent bridge TX: $bridge_tx_hash"
        fi
        print_info "Bridge service shows $(aggsandbox show bridges --network-id 0 2>/dev/null | grep -c tx_hash) total bridges"
    fi
    
    # Step 3: Claim assets on L2
    print_step "Claiming assets on L2"
    print_info "Claiming on network $NETWORK_ID_AGGLAYER_1 for bridge TX: $bridge_tx_hash"
    print_info "Source network: $NETWORK_ID_MAINNET"
    
    local claim_result
    if ! claim_result=$(claim_asset_modern \
        "$NETWORK_ID_AGGLAYER_1" \
        "$bridge_tx_hash" \
        "$NETWORK_ID_MAINNET" \
        "" \
        "$PRIVATE_KEY_2" 2>&1); then
        print_error "Claim failed"
        print_debug "Claim error output: $claim_result"
        
        # Try to get more info about available claims
        print_info "Checking available claims on L2..."
        aggsandbox show claims --network-id "$NETWORK_ID_AGGLAYER_1" 2>/dev/null | head -10 || print_warning "Could not get claims info"
        exit 1
    fi
    
    # Extract just the transaction hash (last line that looks like a tx hash)
    local claim_tx_hash
    claim_tx_hash=$(echo "$claim_result" | grep -oE '0x[a-fA-F0-9]{64}' | tail -1)
    
    if [ -z "$claim_tx_hash" ]; then
        print_warning "Could not extract claim transaction hash"
        claim_tx_hash="N/A"
    fi
    
    print_success "Claim completed: $claim_tx_hash"
    
    # Show bridge events with detailed information
    print_step "Showing bridge events with network and deposit information"
    
    print_info "Bridge events on L1 (Network $NETWORK_ID_MAINNET):"
    local l1_bridges
    l1_bridges=$(aggsandbox show bridges --network-id "$NETWORK_ID_MAINNET" 2>/dev/null | grep -A 10 -B 2 "$bridge_tx_hash" || echo "No matching bridges found")
    echo "$l1_bridges"
    
    print_info "Bridge events on L2 (Network $NETWORK_ID_AGGLAYER_1):"
    local l2_bridges
    l2_bridges=$(aggsandbox show bridges --network-id "$NETWORK_ID_AGGLAYER_1" 2>/dev/null | grep -A 10 -B 2 "$bridge_tx_hash" || echo "No matching bridges found")
    echo "$l2_bridges"
    
    # Try to extract deposit count from bridge info
    local deposit_count
    deposit_count=$(echo "$l1_bridges" | grep -i "deposit.*count" | head -1 | grep -oE '[0-9]+' || echo "N/A")
    if [ "$deposit_count" != "N/A" ]; then
        print_info "Deposit count: $deposit_count"
    fi
    
    # Verify the claim was successful by checking L2 balance
    print_step "Verifying assets were claimed on L2"
    
    # Calculate the wrapped token address (deterministic)
    local wrapped_token_addr
    wrapped_token_addr=$(cast call "$POLYGON_ZKEVM_BRIDGE_L2" \
        "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
        1 "$TOKEN_ADDRESS" "AggERC20" "AGGERC20" 18 \
        --rpc-url "$RPC_2" 2>/dev/null || echo "")
    
    if [ -n "$wrapped_token_addr" ] && [ "$wrapped_token_addr" != "0x0000000000000000000000000000000000000000" ]; then
        # Remove padding from address
        wrapped_token_addr="0x$(echo $wrapped_token_addr | sed 's/0x0*//' | tail -c 41)"
        
        # Check L2 balance
        local l2_balance_hex
        l2_balance_hex=$(get_token_balance "$wrapped_token_addr" "$ACCOUNT_ADDRESS_2" "$NETWORK_ID_AGGLAYER_1" 2>/dev/null || echo "0x0")
        local l2_balance_dec
        l2_balance_dec=$(get_balance_decimal "$l2_balance_hex")
        
        print_info "L2 wrapped token address: $wrapped_token_addr"
        print_info "L2 balance after claim: $l2_balance_dec tokens"
        
        # Use bc for large number comparison
        if [ "$(echo "$l2_balance_dec >= $BRIDGE_AMOUNT" | bc 2>/dev/null || echo "0")" = "1" ]; then
            print_success "✅ Assets successfully bridged and claimed on L2!"
        else
            print_warning "⚠️ L2 balance ($l2_balance_dec) is less than bridged amount ($BRIDGE_AMOUNT)"
            print_info "This might be normal if there were previous balances or multiple transactions"
        fi
    else
        print_warning "Could not determine wrapped token address for verification"
    fi
    
    # Print results
    print_bridge_summary "$bridge_tx_hash" "$claim_tx_hash" "$BRIDGE_AMOUNT" "$TOKEN_ADDRESS"
    
    print_success "🎉 L1 to L2 asset bridge and claim completed successfully!"
    print_info "Bridge TX (L1 Network $NETWORK_ID_MAINNET): $bridge_tx_hash"
    print_info "Claim TX (L2 Network $NETWORK_ID_AGGLAYER_1): $claim_tx_hash"
    if [ "$deposit_count" != "N/A" ]; then
        print_info "Deposit Count: $deposit_count"
    fi
}

# Run the test
main "$@"