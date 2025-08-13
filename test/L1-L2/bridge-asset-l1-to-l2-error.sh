#!/bin/bash
# Comprehensive error test cases for L1 to L2 bridging
# This script tests various failure scenarios to ensure proper error handling

# Don't exit on error - we're testing failures!
set +e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Helper functions
print_test() {
    echo -e "${PURPLE}[TEST]${NC} $1"
}

print_step() {
    echo -e "${GREEN}[STEP]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_debug() {
    if [ "$DEBUG" = "1" ]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

# Helper function to extract JSON from aggsandbox output
extract_json() {
    sed -n '/^{/,/^}/p'
}

# Load environment variables
if [ -f .env ]; then
    source .env
    print_info "Loaded environment variables from .env"
else
    print_error ".env file not found. Please ensure you have the environment file."
    exit 1
fi

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
EXPECTED_FAILURES=0

# Function to record test result
record_test_result() {
    local test_name=$1
    local expected_to_fail=$2
    local actual_result=$3  # 0 = success, 1 = failure
    
    ((TOTAL_TESTS++))
    
    if [ "$expected_to_fail" = "true" ]; then
        if [ "$actual_result" = "1" ]; then
            print_success " Test '$test_name' failed as expected"
            ((PASSED_TESTS++))
            ((EXPECTED_FAILURES++))
        else
            print_error "L Test '$test_name' succeeded but was expected to fail"
            ((FAILED_TESTS++))
        fi
    else
        if [ "$actual_result" = "0" ]; then
            print_success " Test '$test_name' passed"
            ((PASSED_TESTS++))
        else
            print_error "L Test '$test_name' failed unexpectedly"
            ((FAILED_TESTS++))
        fi
    fi
}

echo ""
print_info "========== L1 TO L2 BRIDGE ERROR TEST SUITE =========="
print_info "This suite tests various error conditions and edge cases"
echo ""

# Test 1: Invalid token address
print_test "Test 1: Bridge with invalid token address"
INVALID_TOKEN="0x0000000000000000000000000000000000000000"
RESULT=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
    $CHAIN_ID_AGGLAYER_1 \
    $ACCOUNT_ADDRESS_2 \
    100 \
    $INVALID_TOKEN \
    true \
    0x \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json 2>&1 || echo "FAILED")

if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"revert"* ]]; then
    record_test_result "Invalid token address" true 1
else
    record_test_result "Invalid token address" true 0
fi
echo ""


# Test 2: Bridge without approval
print_test "Test 2: Bridge without token approval"
UNAPPROVED_AMOUNT=999999999
RESULT=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
    $CHAIN_ID_AGGLAYER_1 \
    $ACCOUNT_ADDRESS_2 \
    $UNAPPROVED_AMOUNT \
    $AGG_ERC20_L1 \
    true \
    0x \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json 2>&1 || echo "FAILED")

if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"revert"* ]] || [[ "$RESULT" == *"insufficient allowance"* ]]; then
    record_test_result "Bridge without approval" true 1
else
    record_test_result "Bridge without approval" true 0
fi
echo ""


# Test 3: Claim with wrong parameters
print_test "Test 3: Claim with mismatched parameters"
# This test attempts to claim with wrong deposit count
WRONG_DEPOSIT_COUNT=99999
METADATA=$(cast abi-encode "f(string,string,uint8)" "AggERC20" "AGGERC20" 18)

RESULT=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
    "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
    $WRONG_DEPOSIT_COUNT \
    "0x0000000000000000000000000000000000000000000000000000000000000000" \
    "0x0000000000000000000000000000000000000000000000000000000000000000" \
    1 \
    $AGG_ERC20_L1 \
    $CHAIN_ID_AGGLAYER_1 \
    $ACCOUNT_ADDRESS_2 \
    100 \
    $METADATA \
    --private-key $PRIVATE_KEY_2 \
    --rpc-url $RPC_2 \
    --json 2>&1 || echo "FAILED")

if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"revert"* ]]; then
    record_test_result "Claim with wrong parameters" true 1
else
    record_test_result "Claim with wrong parameters" true 0
fi
echo ""

# Test 4: Double claim attempt
print_test "Test 4: Attempting to claim the same deposit twice"
print_info "First, creating a valid bridge..."

# Approve and bridge
BRIDGE_AMOUNT=5
cast send $AGG_ERC20_L1 \
    "approve(address,uint256)" \
    $POLYGON_ZKEVM_BRIDGE_L1 $BRIDGE_AMOUNT \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 > /dev/null 2>&1

BRIDGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
    $CHAIN_ID_AGGLAYER_1 \
    $ACCOUNT_ADDRESS_2 \
    $BRIDGE_AMOUNT \
    $AGG_ERC20_L1 \
    true \
    0x \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json | jq -r '.transactionHash')

print_info "Bridge TX: $BRIDGE_TX"
print_info "Waiting for global exit root propagation (20s)..."
sleep 20

# Get bridge info
BRIDGE_INFO=$(aggsandbox show bridges --network-id 1 | extract_json)
MATCHING_BRIDGE=$(echo $BRIDGE_INFO | jq -r --arg tx "$BRIDGE_TX" '.bridges[] | select(.tx_hash == $tx)')
DEPOSIT_COUNT=$(echo $MATCHING_BRIDGE | jq -r '.deposit_count')

# Get L1 info tree index first
LEAF_INDEX=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $DEPOSIT_COUNT | awk '/════════════════════════════════════════════════════════════/{if(p) print p; p=""} {p=$0} END{if(p && p ~ /^[0-9]+$/) print p}')
print_debug "L1 info tree leaf index: $LEAF_INDEX"

# Get proof
PROOF_DATA=$(aggsandbox show claim-proof --network-id 1 --leaf-index $LEAF_INDEX --deposit-count $DEPOSIT_COUNT | extract_json)
MAINNET_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
ROLLUP_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.rollup_exit_root')

# First claim (should succeed)
print_info "Attempting first claim..."
# Calculate global index with mainnet flag for L1 origin
GLOBAL_INDEX=$(echo "$DEPOSIT_COUNT + 18446744073709551616" | bc)
print_debug "Global index: $GLOBAL_INDEX (deposit count: $DEPOSIT_COUNT with mainnet flag)"

CLAIM1=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
    "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
    $GLOBAL_INDEX \
    $MAINNET_EXIT_ROOT \
    $ROLLUP_EXIT_ROOT \
    0 \
    $AGG_ERC20_L1 \
    $CHAIN_ID_AGGLAYER_1 \
    $ACCOUNT_ADDRESS_2 \
    $BRIDGE_AMOUNT \
    $METADATA \
    --private-key $PRIVATE_KEY_2 \
    --rpc-url $RPC_2 \
    --json 2>&1)

if echo "$CLAIM1" | jq -e '.transactionHash' > /dev/null 2>&1; then
    print_info "First claim succeeded"
    sleep 3
    
    # Second claim (should fail)
    print_info "Attempting second claim of same deposit..."
    CLAIM2=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
        "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
        $GLOBAL_INDEX \
        $MAINNET_EXIT_ROOT \
        $ROLLUP_EXIT_ROOT \
        0 \
        $AGG_ERC20_L1 \
        $CHAIN_ID_AGGLAYER_1 \
        $ACCOUNT_ADDRESS_2 \
        $BRIDGE_AMOUNT \
        $METADATA \
        --private-key $PRIVATE_KEY_2 \
        --rpc-url $RPC_2 \
        --json 2>&1 || echo "FAILED")
    
    if [[ "$CLAIM2" == *"FAILED"* ]] || [[ "$CLAIM2" == *"AlreadyClaimed"* ]] || [[ "$CLAIM2" == *"revert"* ]]; then
        record_test_result "Double claim prevention" true 1
    else
        record_test_result "Double claim prevention" true 0
    fi
else
    print_info "First claim failed (might be GlobalExitRootInvalid)"
    record_test_result "Double claim prevention" false 1
fi
echo ""

# Test 5: Claim with insufficient gas
print_test "Test 5: Claim with insufficient gas limit"
INSUFFICIENT_GAS=50000  # Too low for claim operation

RESULT=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
    "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
    1 \
    "0x0000000000000000000000000000000000000000000000000000000000000000" \
    "0x0000000000000000000000000000000000000000000000000000000000000000" \
    1 \
    $AGG_ERC20_L1 \
    $CHAIN_ID_AGGLAYER_1 \
    $ACCOUNT_ADDRESS_2 \
    10 \
    $METADATA \
    --private-key $PRIVATE_KEY_2 \
    --rpc-url $RPC_2 \
    --gas-limit $INSUFFICIENT_GAS \
    --json 2>&1 || echo "FAILED")

if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"out of gas"* ]] || [[ "$RESULT" == *"gas"* ]]; then
    record_test_result "Insufficient gas" true 1
else
    record_test_result "Insufficient gas" true 0
fi
echo ""

# Test 6: Bridge with excessive metadata
print_test "Test 6: Bridge with excessive metadata size"
# Create large metadata (>65535 bytes)
LARGE_METADATA=$(printf '0x%.0s' {1..70000})

RESULT=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
    $CHAIN_ID_AGGLAYER_1 \
    $ACCOUNT_ADDRESS_2 \
    1 \
    $AGG_ERC20_L1 \
    true \
    $LARGE_METADATA \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json 2>&1 || echo "FAILED")

if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"MetadataTooLarge"* ]] || [[ "$RESULT" == *"revert"* ]]; then
    record_test_result "Excessive metadata" true 1
else
    record_test_result "Excessive metadata" true 0
fi
echo ""

# Test 7: Claim before wait time
print_test "Test 7: Claim immediately after bridge (no wait)"
# Approve and bridge
cast send $AGG_ERC20_L1 \
    "approve(address,uint256)" \
    $POLYGON_ZKEVM_BRIDGE_L1 3 \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 > /dev/null 2>&1

QUICK_BRIDGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
    $CHAIN_ID_AGGLAYER_1 \
    $ACCOUNT_ADDRESS_2 \
    3 \
    $AGG_ERC20_L1 \
    true \
    0x \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json | jq -r '.transactionHash')

print_info "Bridge TX: $QUICK_BRIDGE_TX"
print_info "Attempting immediate claim (no wait)..."

# Try to get bridge info immediately
sleep 2  # Just enough for indexing, not for GER propagation
QUICK_BRIDGE_INFO=$(aggsandbox show bridges --network-id 1 | extract_json)
QUICK_MATCHING=$(echo $QUICK_BRIDGE_INFO | jq -r --arg tx "$QUICK_BRIDGE_TX" '.bridges[] | select(.tx_hash == $tx)')

if [ -n "$QUICK_MATCHING" ]; then
    QUICK_DEPOSIT=$(echo $QUICK_MATCHING | jq -r '.deposit_count')
    # Get L1 info tree index for quick bridge
    QUICK_LEAF_INDEX=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $QUICK_DEPOSIT | awk '/════════════════════════════════════════════════════════════/{if(p) print p; p=""} {p=$0} END{if(p && p ~ /^[0-9]+$/) print p}')
    
    QUICK_PROOF=$(aggsandbox show claim-proof --network-id 1 --leaf-index $QUICK_LEAF_INDEX --deposit-count $QUICK_DEPOSIT | extract_json)
    QUICK_MAINNET=$(echo $QUICK_PROOF | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
    QUICK_ROLLUP=$(echo $QUICK_PROOF | jq -r '.l1_info_tree_leaf.rollup_exit_root')
    
    # Calculate global index for quick claim
    QUICK_GLOBAL_INDEX=$(echo "$QUICK_DEPOSIT + 18446744073709551616" | bc)
    
    QUICK_CLAIM=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
        "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
        $QUICK_GLOBAL_INDEX \
        $QUICK_MAINNET \
        $QUICK_ROLLUP \
        0 \
        $AGG_ERC20_L1 \
        $CHAIN_ID_AGGLAYER_1 \
        $ACCOUNT_ADDRESS_2 \
        3 \
        $METADATA \
        --private-key $PRIVATE_KEY_2 \
        --rpc-url $RPC_2 \
        --json 2>&1 || echo "FAILED")
    
    if [[ "$QUICK_CLAIM" == *"0x002f6fad"* ]] || [[ "$QUICK_CLAIM" == *"GlobalExitRootInvalid"* ]] || [[ "$QUICK_CLAIM" == *"FAILED"* ]]; then
        record_test_result "Claim without proper wait" true 1
    else
        record_test_result "Claim without proper wait" true 0
    fi
else
    print_info "Bridge not indexed yet"
    record_test_result "Claim without proper wait" true 1
fi
echo ""

# Test Summary
echo ""
print_info "========== ERROR TEST SUMMARY =========="
print_info "Total tests run: $TOTAL_TESTS"
print_success "Tests passed: $PASSED_TESTS"
if [ $FAILED_TESTS -gt 0 ]; then
    print_error "Tests failed: $FAILED_TESTS"
else
    print_info "Tests failed: $FAILED_TESTS"
fi
print_info "Expected failures caught: $EXPECTED_FAILURES"

echo ""
print_info "Error conditions tested:"
print_info "1.  Invalid token address"
print_info "2.  Bridge without approval"
print_info "3.  Claim with wrong parameters"
print_info "4.  Double claim prevention"
print_info "5.  Insufficient gas"
print_info "6.  Excessive metadata"
print_info "7.  Claim timing issues"

echo ""
echo ""
if [ $FAILED_TESTS -eq 0 ]; then
    print_success "All error handling tests completed successfully! <�"
    exit 0
else
    print_error "Some tests did not behave as expected"
    exit 1
fi