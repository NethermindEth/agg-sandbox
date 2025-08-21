#!/bin/bash
# Test script to verify the modular bridge library loads correctly
# This script tests that all modules are properly loaded and functions are available

set -e

echo "Testing modular bridge library..."

# Enable debug mode to see module loading
export DEBUG=1

# Source the main library
source "$(dirname "$0")/bridge_test_lib.sh"

echo ""
echo "=== Testing Core Functions ==="

# Test core logging functions
print_step "Testing logging functions"
print_info "This is an info message"
print_success "This is a success message"
print_warning "This is a warning message"
print_debug "This is a debug message"

echo ""
echo "=== Testing Module Function Availability ==="

# Test that functions from each module are available
declare -a test_functions=(
    # Bridge Asset Module
    "bridge_asset_modern"
    "execute_l1_to_l2_asset_bridge"
    "execute_l2_to_l1_asset_bridge"
    
    # Bridge Message Module
    "bridge_message_modern"
    "bridge_text_message"
    "execute_l1_to_l2_message_bridge"
    
    # Claim Asset Module
    "claim_asset_modern"
    "claim_asset_with_retry"
    "is_asset_claimed"
    
    # Claim Message Module
    "claim_message_modern"
    "claim_message_with_retry"
    "is_message_claimed"
    
    # Bridge and Call Module
    "bridge_and_call_modern"
    "deploy_bridge_call_receiver"
    "execute_l1_to_l2_bridge_and_call"
    
    # Claim Bridge and Call Module
    "claim_bridge_and_call_modern"
    "get_bridge_and_call_info"
    "wait_for_bridge_and_call_claimable"
    
    # Legacy compatibility
    "execute_l1_to_l2_bridge"
)

echo "Checking function availability..."

for func in "${test_functions[@]}"; do
    if declare -F "$func" > /dev/null; then
        print_success "✅ Function '$func' is available"
    else
        print_error "❌ Function '$func' is NOT available"
        exit 1
    fi
done

echo ""
echo "=== Testing Function Type Detection ==="

# Test that we can identify function types correctly
echo "Available bridge functions:"
declare -F | grep -E "(bridge_|claim_|execute_)" | head -10

echo ""
echo "=== Summary ==="
print_success "🎉 All modular library tests passed!"
print_info "The modular bridge library is working correctly"
print_info "All modules loaded successfully and functions are available"

echo ""
print_info "Available modules:"
print_info "  • Bridge Asset Module: ✅"
print_info "  • Bridge Message Module: ✅" 
print_info "  • Claim Asset Module: ✅"
print_info "  • Claim Message Module: ✅"
print_info "  • Bridge and Call Module: ✅"
print_info "  • Claim Bridge and Call Module: ✅"

echo ""
print_info "Usage example:"
echo "  source test/lib/bridge_test_lib.sh"
echo "  bridge_tx_hash=\$(bridge_asset_modern 0 1 100 \$TOKEN \$DEST \$PRIVATE_KEY)"
echo "  claim_tx_hash=\$(claim_asset_modern 1 \$bridge_tx_hash 0 \"\" \$PRIVATE_KEY)"
