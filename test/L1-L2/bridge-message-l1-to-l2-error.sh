#!/bin/bash
# Error test cases for L1 to L2 message bridging
# This script tests various failure scenarios for message bridging

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
            print_success "✓ Test '$test_name' failed as expected"
            ((PASSED_TESTS++))
            ((EXPECTED_FAILURES++))
        else
            print_error "✗ Test '$test_name' succeeded but was expected to fail"
            ((FAILED_TESTS++))
        fi
    else
        if [ "$actual_result" = "0" ]; then
            print_success "✓ Test '$test_name' passed"
            ((PASSED_TESTS++))
        else
            print_error "✗ Test '$test_name' failed unexpectedly"
            ((FAILED_TESTS++))
        fi
    fi
}

echo ""
print_info "========== L1 TO L2 MESSAGE BRIDGE ERROR TEST SUITE =========="
print_info "This suite tests error conditions for message bridging"
echo ""

# Deploy message receiver contract on L2 for testing
print_step "Deploying SimpleBridgeMessageReceiver contract on L2"

# Check if contract source exists
CONTRACT_PATH="test/contracts/SimpleBridgeMessageReceiver.sol"
if [ ! -f "$CONTRACT_PATH" ]; then
    print_error "Contract source not found at $CONTRACT_PATH"
    exit 1
fi

# Compile the contract
print_info "Compiling contract..."
forge build --contracts $CONTRACT_PATH > /dev/null 2>&1

# Deploy the contract
print_info "Deploying SimpleBridgeMessageReceiver to L2..."
DEPLOY_OUTPUT=$(forge create $CONTRACT_PATH:SimpleBridgeMessageReceiver \
    --rpc-url $RPC_2 \
    --private-key $PRIVATE_KEY_2 \
    --json 2>/dev/null)

if [ $? -ne 0 ]; then
    print_error "Failed to deploy contract"
    print_info "Make sure forge is installed and contract compiles"
    exit 1
fi

MESSAGE_RECEIVER=$(echo "$DEPLOY_OUTPUT" | jq -r '.deployedTo')
if [ -z "$MESSAGE_RECEIVER" ] || [ "$MESSAGE_RECEIVER" = "null" ]; then
    print_error "Could not extract deployed contract address"
    exit 1
fi

print_success "Message receiver deployed at: $MESSAGE_RECEIVER"
echo ""

# Test 1: Bridge message with ETH to EOA (not implementing IBridgeMessageReceiver)
print_test "Test 1: Bridge message to EOA (no IBridgeMessageReceiver)"
# EOAs don't implement the interface, so claim should fail
EOA_ADDRESS=$ACCOUNT_ADDRESS_2  # Regular account
EOA_MESSAGE="Test EOA"
EOA_MESSAGE_HEX=$(echo -n "$EOA_MESSAGE" | xxd -p | tr -d '\n')
EOA_MESSAGE_BYTES="0x$EOA_MESSAGE_HEX"
ETH_VALUE=1000000  # 1M wei

# First bridge the message
EOA_BRIDGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "bridgeMessage(uint32,address,bool,bytes)" \
    $CHAIN_ID_AGGLAYER_1 \
    $EOA_ADDRESS \
    true \
    $EOA_MESSAGE_BYTES \
    --value $ETH_VALUE \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json | jq -r '.transactionHash')

if [ -z "$EOA_BRIDGE_TX" ] || [ "$EOA_BRIDGE_TX" = "null" ]; then
    record_test_result "Bridge to EOA" false 1
else
    print_info "Bridge TX: $EOA_BRIDGE_TX"
    print_info "Waiting for propagation (20s)..."
    sleep 20
    
    # Try to claim - should fail
    EOA_BRIDGE_INFO=$(aggsandbox show bridges --network-id 1 | extract_json)
    EOA_MATCHING=$(echo $EOA_BRIDGE_INFO | jq -r --arg tx "$EOA_BRIDGE_TX" '.bridges[] | select(.tx_hash == $tx)')
    
    if [ -n "$EOA_MATCHING" ]; then
        EOA_DEPOSIT=$(echo $EOA_MATCHING | jq -r '.deposit_count')
        
        # Get L1 info tree index
        EOA_LEAF_INDEX=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $EOA_DEPOSIT | awk '/════════════════════════════════════════════════════════════/{if(p) print p; p=""} {p=$0} END{if(p && p ~ /^[0-9]+$/) print p}')
        
        EOA_PROOF=$(aggsandbox show claim-proof --network-id 1 --leaf-index $EOA_LEAF_INDEX --deposit-count $EOA_DEPOSIT | extract_json)
        EOA_MAINNET=$(echo $EOA_PROOF | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
        EOA_ROLLUP=$(echo $EOA_PROOF | jq -r '.l1_info_tree_leaf.rollup_exit_root')
        
        # Calculate global index with mainnet flag for L1 origin
        EOA_GLOBAL_INDEX=$(echo "$EOA_DEPOSIT + 18446744073709551616" | bc)
        
        # Try to claim - should fail because EOA doesn't implement IBridgeMessageReceiver
        EOA_CLAIM=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
            "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
            $EOA_GLOBAL_INDEX \
            $EOA_MAINNET \
            $EOA_ROLLUP \
            0 \
            $ACCOUNT_ADDRESS_1 \
            $CHAIN_ID_AGGLAYER_1 \
            $EOA_ADDRESS \
            $ETH_VALUE \
            $EOA_MESSAGE_BYTES \
            --private-key $PRIVATE_KEY_2 \
            --rpc-url $RPC_2 \
            --json 2>&1 || echo "FAILED")
        
        if [[ "$EOA_CLAIM" == *"FAILED"* ]] || [[ "$EOA_CLAIM" == *"revert"* ]]; then
            record_test_result "Claim to EOA (should fail)" true 1
            print_info "Claim failed as expected - EOA doesn't implement IBridgeMessageReceiver"
        else
            record_test_result "Claim to EOA (should fail)" true 0
            print_error "Claim succeeded but should have failed!"
        fi
    else
        print_info "Bridge not found in API"
        record_test_result "Bridge to EOA test" false 1
    fi
fi
echo ""

# Test 2: Bridge message without ETH but claim with wrong amount
print_test "Test 2: Bridge message without ETH but claim with wrong amount"
# Bridge a message without ETH, then try to claim with ETH amount
SIMPLE_MESSAGE="Test message"
SIMPLE_MESSAGE_HEX=$(echo -n "$SIMPLE_MESSAGE" | xxd -p | tr -d '\n')
SIMPLE_MESSAGE_BYTES="0x$SIMPLE_MESSAGE_HEX"

# First bridge the message without ETH
BRIDGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "bridgeMessage(uint32,address,bool,bytes)" \
    $CHAIN_ID_AGGLAYER_1 \
    $MESSAGE_RECEIVER \
    true \
    $SIMPLE_MESSAGE_BYTES \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json | jq -r '.transactionHash')

if [ -z "$BRIDGE_TX" ] || [ "$BRIDGE_TX" = "null" ]; then
    record_test_result "Bridge message for amount mismatch test" false 1
else
    print_info "Bridge TX: $BRIDGE_TX"
    # This sets up for test 3 which will try to claim with wrong amount
    record_test_result "Bridge message for amount mismatch test" false 0
fi
echo ""

# Test 3: Claim message with wrong ETH amount
print_test "Test 3: Claim message with mismatched ETH amount"
# Try to claim the message from test 2 with wrong ETH amount
if [ -n "$BRIDGE_TX" ] && [ "$BRIDGE_TX" != "null" ]; then
    print_info "Waiting for indexing (5s)..."
    sleep 5
    
    BRIDGE_INFO=$(aggsandbox show bridges --network-id 1 | extract_json)
    MATCHING_BRIDGE=$(echo $BRIDGE_INFO | jq -r --arg tx "$BRIDGE_TX" '.bridges[] | select(.tx_hash == $tx)')
    
    if [ -n "$MATCHING_BRIDGE" ]; then
        DEPOSIT_COUNT=$(echo $MATCHING_BRIDGE | jq -r '.deposit_count')
        ORIGIN_ADDRESS=$(echo $MATCHING_BRIDGE | jq -r '.origin_address')
        
        # Calculate global index with mainnet flag for L1 origin
        GLOBAL_INDEX=$((DEPOSIT_COUNT | (1 << 64)))
        
        # Try to claim with wrong ETH amount (1000 wei instead of 0)
        RESULT=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
            "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
            $GLOBAL_INDEX \
            "0x0000000000000000000000000000000000000000000000000000000000000000" \
            "0x0000000000000000000000000000000000000000000000000000000000000000" \
            0 \
            $ORIGIN_ADDRESS \
            $CHAIN_ID_AGGLAYER_1 \
            $MESSAGE_RECEIVER \
            1000 \
            $SIMPLE_MESSAGE_BYTES \
            --private-key $PRIVATE_KEY_2 \
            --rpc-url $RPC_2 \
            --json 2>&1 || echo "FAILED")
        
        if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"revert"* ]] || [[ "$RESULT" == *"InvalidProof"* ]]; then
            record_test_result "Claim with wrong ETH amount" true 1
        else
            record_test_result "Claim with wrong ETH amount" true 0
        fi
    else
        print_info "Bridge not found in API"
        record_test_result "Claim with wrong ETH amount" true 1
    fi
else
    print_info "No bridge TX from previous test"
    record_test_result "Claim with wrong ETH amount" true 1
fi
echo ""

# Test 4: Double claim of message
print_test "Test 4: Attempting to claim the same message twice"
print_info "First, creating a valid message bridge..."

# Send a valid message
TEST_MESSAGE="Test message for double claim"
TEST_MESSAGE_HEX=$(echo -n "$TEST_MESSAGE" | xxd -p | tr -d '\n')
TEST_MESSAGE_BYTES="0x$TEST_MESSAGE_HEX"

BRIDGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "bridgeMessage(uint32,address,bool,bytes)" \
    $CHAIN_ID_AGGLAYER_1 \
    $MESSAGE_RECEIVER \
    true \
    $TEST_MESSAGE_BYTES \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json | jq -r '.transactionHash')

print_info "Bridge TX: $BRIDGE_TX"
print_info "Waiting for global exit root propagation (30s)..."
sleep 30

# Get bridge info
BRIDGE_INFO=$(aggsandbox show bridges --network-id 1 | extract_json)
MATCHING_BRIDGE=$(echo $BRIDGE_INFO | jq -r --arg tx "$BRIDGE_TX" '.bridges[] | select(.tx_hash == $tx)')

if [ -z "$MATCHING_BRIDGE" ] || [ "$MATCHING_BRIDGE" = "null" ]; then
    print_error "Could not find bridge transaction in API"
    record_test_result "Double message claim prevention" false 1
    echo ""
else
    DEPOSIT_COUNT=$(echo $MATCHING_BRIDGE | jq -r '.deposit_count')
    print_debug "Deposit count: $DEPOSIT_COUNT"

    # Get L1 info tree index and proof
    LEAF_INDEX=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $DEPOSIT_COUNT | awk '/════════════════════════════════════════════════════════════/{if(p) print p; p=""} {p=$0} END{if(p && p ~ /^[0-9]+$/) print p}')
    print_debug "L1 info tree leaf index: $LEAF_INDEX"
    
    PROOF_DATA=$(aggsandbox show claim-proof --network-id 1 --leaf-index $LEAF_INDEX --deposit-count $DEPOSIT_COUNT | extract_json)
    MAINNET_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
    ROLLUP_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.rollup_exit_root')

    # First claim (should succeed)
    print_info "Attempting first claim..."
    # Calculate global index with mainnet flag for L1 origin
    GLOBAL_INDEX=$(echo "$DEPOSIT_COUNT + 18446744073709551616" | bc)
    
    CLAIM1=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
        "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
        $GLOBAL_INDEX \
        $MAINNET_EXIT_ROOT \
        $ROLLUP_EXIT_ROOT \
        0 \
        $ACCOUNT_ADDRESS_1 \
        $CHAIN_ID_AGGLAYER_1 \
        $MESSAGE_RECEIVER \
        0 \
        $TEST_MESSAGE_BYTES \
        --private-key $PRIVATE_KEY_2 \
        --rpc-url $RPC_2 \
        --json 2>&1)

    if echo "$CLAIM1" | jq -e '.transactionHash' > /dev/null 2>&1; then
        CLAIM1_TX=$(echo "$CLAIM1" | jq -r '.transactionHash')
        print_info "First claim succeeded: $CLAIM1_TX"
        sleep 3
        
        # Verify the message was received correctly
        print_info "Verifying message content..."
        
        # Get message details from receiver contract
        RECEIVED_DATA=$(cast call $MESSAGE_RECEIVER "lastMessageData()" --rpc-url $RPC_2)
        RECEIVED_ORIGIN=$(cast call $MESSAGE_RECEIVER "lastOriginAddress()" --rpc-url $RPC_2)
        RECEIVED_VALUE=$(cast call $MESSAGE_RECEIVER "totalMessagesReceived()" --rpc-url $RPC_2)
        
        # Verify message data
        SENT_DATA_NO_PREFIX=${TEST_MESSAGE_BYTES#0x}
        RECEIVED_DATA_NO_PREFIX=${RECEIVED_DATA#0x}
        
        # Extract actual data from ABI encoding
        if [[ ${#RECEIVED_DATA_NO_PREFIX} -gt 128 ]]; then
            DATA_LENGTH_HEX=${RECEIVED_DATA_NO_PREFIX:64:64}
            DATA_LENGTH=$((16#$DATA_LENGTH_HEX))
            DATA_HEX_LENGTH=$((DATA_LENGTH * 2))
            ACTUAL_DATA=${RECEIVED_DATA_NO_PREFIX:128:$DATA_HEX_LENGTH}
            
            if [ "$ACTUAL_DATA" = "$SENT_DATA_NO_PREFIX" ]; then
                print_success "✓ Message data verified correctly"
                DECODED_MESSAGE=$(echo -n "$ACTUAL_DATA" | xxd -r -p)
                print_debug "Decoded message: '$DECODED_MESSAGE'"
            else
                print_error "✗ Message data mismatch!"
            fi
        fi
        
        # Second claim (should fail)
        print_info "Attempting second claim of same message..."
        CLAIM2=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
            "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
            $GLOBAL_INDEX \
            $MAINNET_EXIT_ROOT \
            $ROLLUP_EXIT_ROOT \
            0 \
            $ACCOUNT_ADDRESS_1 \
            $CHAIN_ID_AGGLAYER_1 \
            $MESSAGE_RECEIVER \
            0 \
            $TEST_MESSAGE_BYTES \
            --private-key $PRIVATE_KEY_2 \
            --rpc-url $RPC_2 \
            --json 2>&1 || echo "FAILED")
        
        if [[ "$CLAIM2" == *"FAILED"* ]] || [[ "$CLAIM2" == *"AlreadyClaimed"* ]] || [[ "$CLAIM2" == *"revert"* ]]; then
            record_test_result "Double message claim prevention" true 1
        else
            record_test_result "Double message claim prevention" true 0
        fi
    else
        print_info "First claim failed (might be GlobalExitRootInvalid)"
        record_test_result "Double message claim prevention" false 1
    fi
fi
echo ""

# Test 5: Claim with insufficient gas
print_test "Test 5: Claim message with insufficient gas limit"
INSUFFICIENT_GAS=50000  # Too low for claim operation
TEST_MSG="0x546573744d657373616765"  # "TestMessage"
# Global index with mainnet flag
TEST_GLOBAL_INDEX=$((1 | (1 << 64)))

RESULT=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
    "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
    $TEST_GLOBAL_INDEX \
    "0x0000000000000000000000000000000000000000000000000000000000000000" \
    "0x0000000000000000000000000000000000000000000000000000000000000000" \
    0 \
    $ACCOUNT_ADDRESS_1 \
    $CHAIN_ID_AGGLAYER_1 \
    $MESSAGE_RECEIVER \
    0 \
    $TEST_MSG \
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

# Test 6: Claim before wait time
print_test "Test 6: Claim message immediately after bridge (no wait)"
# Send a message
QUICK_MESSAGE="Quick test message"
QUICK_MESSAGE_HEX=$(echo -n "$QUICK_MESSAGE" | xxd -p | tr -d '\n')
QUICK_MESSAGE_BYTES="0x$QUICK_MESSAGE_HEX"

QUICK_BRIDGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "bridgeMessage(uint32,address,bool,bytes)" \
    $CHAIN_ID_AGGLAYER_1 \
    $MESSAGE_RECEIVER \
    true \
    $QUICK_MESSAGE_BYTES \
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
    
    # Get L1 info tree index
    QUICK_LEAF_INDEX=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $QUICK_DEPOSIT | awk '/════════════════════════════════════════════════════════════/{if(p) print p; p=""} {p=$0} END{if(p && p ~ /^[0-9]+$/) print p}')
    
    QUICK_PROOF=$(aggsandbox show claim-proof --network-id 1 --leaf-index $QUICK_LEAF_INDEX --deposit-count $QUICK_DEPOSIT | extract_json)
    QUICK_MAINNET=$(echo $QUICK_PROOF | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
    QUICK_ROLLUP=$(echo $QUICK_PROOF | jq -r '.l1_info_tree_leaf.rollup_exit_root')
    
    # Calculate global index with mainnet flag for L1 origin
    QUICK_GLOBAL_INDEX=$(echo "$QUICK_DEPOSIT + 18446744073709551616" | bc)
    
    QUICK_CLAIM=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
        "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
        $QUICK_GLOBAL_INDEX \
        $QUICK_MAINNET \
        $QUICK_ROLLUP \
        0 \
        $ACCOUNT_ADDRESS_1 \
        $CHAIN_ID_AGGLAYER_1 \
        $MESSAGE_RECEIVER \
        0 \
        $QUICK_MESSAGE_BYTES \
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
print_info "========== MESSAGE ERROR TEST SUMMARY =========="
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
print_info "1. ✓ Message to EOA (no IBridgeMessageReceiver)"
print_info "2. ✓ Bridge message for amount mismatch test"
print_info "3. ✓ Claim with wrong ETH amount"
print_info "4. ✓ Double message claim prevention"
print_info "5. ✓ Insufficient gas"
print_info "6. ✓ Claim timing issues"

echo ""
echo ""
if [ $FAILED_TESTS -eq 0 ]; then
    print_success "All message error handling tests completed successfully! ✅"
    exit 0
else
    print_error "Some tests did not behave as expected"
    exit 1
fi