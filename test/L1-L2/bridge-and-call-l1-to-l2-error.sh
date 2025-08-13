#!/bin/bash
# Error test cases for L1 to L2 bridge and call
# This script tests various failure scenarios for bridge and call operations

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
print_info "========== L1 TO L2 BRIDGE AND CALL ERROR TEST SUITE =========="
print_info "This suite tests error conditions for bridge and call operations"
echo ""

# Deploy receiver contract on L2 for testing
print_step "Deploying bridge-and-call receiver contract on L2"
DEPLOY_TX=$(forge create test/contracts/SimpleBridgeAndCallReceiver.sol:SimpleBridgeAndCallReceiver \
    --rpc-url $RPC_2 \
    --private-key $PRIVATE_KEY_1 \
    --json 2>/dev/null | jq -r '.deployedTo')

if [ -z "$DEPLOY_TX" ] || [ "$DEPLOY_TX" = "null" ]; then
    print_error "Failed to deploy receiver contract"
    exit 1
fi

RECEIVER_CONTRACT=$DEPLOY_TX
print_success "Receiver contract deployed at: $RECEIVER_CONTRACT"
echo ""

# Test 1: Bridge with invalid call data
print_test "Test 1: Bridge with malformed call data"
# Create invalid call data (not properly encoded)
INVALID_CALL_DATA="0x1234567890"  # Random hex, not a valid function call

# First approve the bridge extension
cast send $AGG_ERC20_L1 \
    "approve(address,uint256)" \
    $BRIDGE_EXTENSION_L1 10 \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 > /dev/null 2>&1

# Get L2 token address
L2_TOKEN=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
    "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
    1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
    --rpc-url $RPC_2)
L2_TOKEN="0x$(echo $L2_TOKEN | sed 's/0x0*//' | tail -c 41)"

RESULT=$(cast send $BRIDGE_EXTENSION_L1 \
    "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" \
    $AGG_ERC20_L1 \
    10 \
    $CHAIN_ID_AGGLAYER_1 \
    $RECEIVER_CONTRACT \
    $ACCOUNT_ADDRESS_1 \
    $INVALID_CALL_DATA \
    true \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json 2>&1 || echo "FAILED")

# Bridge should succeed even with invalid call data (validation happens on claim)
if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"revert"* ]]; then
    record_test_result "Bridge with malformed call data" false 1
else
    record_test_result "Bridge with malformed call data" false 0
fi
echo ""

# Test 2: Bridge without approval (with call data)
print_test "Test 2: Bridge and call without token approval"
CALL_DATA=$(cast abi-encode "receiveTokensWithMessage(address,uint256,string)" $L2_TOKEN_ADDRESS 5 "Test message")
UNAPPROVED_AMOUNT=999999999

# Get L2 token address
L2_TOKEN=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
    "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
    1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
    --rpc-url $RPC_2)
L2_TOKEN="0x$(echo $L2_TOKEN | sed 's/0x0*//' | tail -c 41)"

RESULT=$(cast send $BRIDGE_EXTENSION_L1 \
    "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" \
    $AGG_ERC20_L1 \
    $UNAPPROVED_AMOUNT \
    $CHAIN_ID_AGGLAYER_1 \
    $RECEIVER_CONTRACT \
    $ACCOUNT_ADDRESS_1 \
    $CALL_DATA \
    true \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json 2>&1 || echo "FAILED")

if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"revert"* ]] || [[ "$RESULT" == *"insufficient allowance"* ]]; then
    record_test_result "Bridge and call without approval" true 1
else
    record_test_result "Bridge and call without approval" true 0
fi
echo ""

# Test 3: Bridge with excessive call data
print_test "Test 3: Bridge with excessive call data size"
# Create very large call data (>65535 bytes)
LARGE_CALL_DATA=$(printf '0x%.0s00' {1..70000})

# Approve first
cast send $AGG_ERC20_L1 \
    "approve(address,uint256)" \
    $BRIDGE_EXTENSION_L1 1 \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 > /dev/null 2>&1

# Get L2 token address
L2_TOKEN=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
    "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
    1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
    --rpc-url $RPC_2)
L2_TOKEN="0x$(echo $L2_TOKEN | sed 's/0x0*//' | tail -c 41)"

RESULT=$(cast send $BRIDGE_EXTENSION_L1 \
    "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" \
    $AGG_ERC20_L1 \
    1 \
    $CHAIN_ID_AGGLAYER_1 \
    $RECEIVER_CONTRACT \
    $ACCOUNT_ADDRESS_1 \
    $LARGE_CALL_DATA \
    true \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json 2>&1 || echo "FAILED")

if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"MetadataTooLarge"* ]] || [[ "$RESULT" == *"revert"* ]]; then
    record_test_result "Excessive call data" true 1
else
    record_test_result "Excessive call data" true 0
fi
echo ""

# Test 4: Call to dangerous function
print_test "Test 4: Bridge with call to self-destruct"
# Encode a call to selfdestruct (if contract had it)
# selfdestruct(address)
DANGEROUS_CALL=$(cast abi-encode "selfdestruct(address)" $ACCOUNT_ADDRESS_1)

# Approve first
cast send $AGG_ERC20_L1 \
    "approve(address,uint256)" \
    $BRIDGE_EXTENSION_L1 5 \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 > /dev/null 2>&1

# Get L2 token address
L2_TOKEN=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
    "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
    1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
    --rpc-url $RPC_2)
L2_TOKEN="0x$(echo $L2_TOKEN | sed 's/0x0*//' | tail -c 41)"

RESULT=$(cast send $BRIDGE_EXTENSION_L1 \
    "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" \
    $AGG_ERC20_L1 \
    5 \
    $CHAIN_ID_AGGLAYER_1 \
    $RECEIVER_CONTRACT \
    $ACCOUNT_ADDRESS_1 \
    $DANGEROUS_CALL \
    true \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json 2>&1 || echo "FAILED")

# Bridge should succeed (call execution validation happens on L2)
if [[ "$RESULT" == *"FAILED"* ]] || [[ "$RESULT" == *"revert"* ]]; then
    record_test_result "Bridge with dangerous call" false 1
else
    record_test_result "Bridge with dangerous call" false 0
fi
echo ""

# Test 5: Double claim with call data
print_test "Test 5: Attempting to claim the same bridge-and-call twice"
print_info "Creating a valid bridge and call..."

# Approve and bridge with call data
BRIDGE_AMOUNT=8
CALL_DATA=$(cast abi-encode "receiveTokensWithMessage(address,uint256,string)" $L2_TOKEN_ADDRESS 2 "Double claim test")

cast send $AGG_ERC20_L1 \
    "approve(address,uint256)" \
    $BRIDGE_EXTENSION_L1 $BRIDGE_AMOUNT \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 > /dev/null 2>&1

# Get L2 token address
L2_TOKEN=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
    "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
    1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
    --rpc-url $RPC_2)
L2_TOKEN="0x$(echo $L2_TOKEN | sed 's/0x0*//' | tail -c 41)"

BRIDGE_TX=$(cast send $BRIDGE_EXTENSION_L1 \
    "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" \
    $AGG_ERC20_L1 \
    $BRIDGE_AMOUNT \
    $CHAIN_ID_AGGLAYER_1 \
    $RECEIVER_CONTRACT \
    $ACCOUNT_ADDRESS_1 \
    $CALL_DATA \
    true \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json | jq -r '.transactionHash')

print_info "Bridge TX: $BRIDGE_TX"

# Verify bridge events were created
print_debug "Checking bridge events..."
BRIDGE_EVENTS=$(aggsandbox events --network-id 0 --bridge $POLYGON_ZKEVM_BRIDGE_L1 | extract_json)
EVENT_COUNT=$(echo "$BRIDGE_EVENTS" | jq --arg tx "$BRIDGE_TX" '[.events[] | select(.transaction_hash == $tx)] | length')
if [ $EVENT_COUNT -eq 2 ]; then
    print_debug "Found $EVENT_COUNT bridge events (asset + message) as expected"
fi

print_info "Waiting for global exit root propagation (20s)..."
sleep 20

# Get bridge info
BRIDGE_INFO=$(aggsandbox show bridges --network-id 1 | extract_json)

# For bridge and call, we need to find both the asset bridge and the message bridge
# The BridgeExtension creates two deposits: one for assets, one for the message
print_info "Looking for bridge transactions created by our bridge and call..."

# Find the asset bridge (leaf_type = 0)
ASSET_BRIDGE=$(echo $BRIDGE_INFO | jq -r '.bridges[] | select(.leaf_type == 0)' | jq -s 'sort_by(.deposit_count) | .[-1]')

# Find the message bridge (leaf_type = 1)
MESSAGE_BRIDGE=$(echo $BRIDGE_INFO | jq -r '.bridges[] | select(.leaf_type == 1)' | jq -s 'sort_by(.deposit_count) | .[-1]')

if [ -z "$MESSAGE_BRIDGE" ] || [ "$MESSAGE_BRIDGE" = "null" ]; then
    print_error "Could not find message bridge transaction in API"
    record_test_result "Double bridge-and-call claim prevention" false 1
    echo ""
else
    # Extract values from both bridges
    # Asset bridge details
    ASSET_DEPOSIT_COUNT=$(echo $ASSET_BRIDGE | jq -r '.deposit_count')
    ASSET_METADATA=$(echo $ASSET_BRIDGE | jq -r '.metadata')
    
    # Message bridge details  
    MESSAGE_DEPOSIT_COUNT=$(echo $MESSAGE_BRIDGE | jq -r '.deposit_count')
    MESSAGE_METADATA=$(echo $MESSAGE_BRIDGE | jq -r '.metadata')
    MESSAGE_ORIGIN=$(echo $MESSAGE_BRIDGE | jq -r '.origin_address')
    MESSAGE_DEST=$(echo $MESSAGE_BRIDGE | jq -r '.destination_address')
    MESSAGE_AMOUNT=$(echo $MESSAGE_BRIDGE | jq -r '.amount')
    
    # Get L1 info tree index and proof for the asset first
    LEAF_INDEX_ASSET=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $ASSET_DEPOSIT_COUNT | awk '/════════════════════════════════════════════════════════════/{if(p) print p; p=""} {p=$0} END{if(p && p ~ /^[0-9]+$/) print p}')
    print_debug "Asset L1 info tree leaf index: $LEAF_INDEX_ASSET"
    
    PROOF_DATA_ASSET=$(aggsandbox show claim-proof --network-id 1 --leaf-index $LEAF_INDEX_ASSET --deposit-count $ASSET_DEPOSIT_COUNT | extract_json)
    MAINNET_EXIT_ROOT=$(echo $PROOF_DATA_ASSET | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
    ROLLUP_EXIT_ROOT=$(echo $PROOF_DATA_ASSET | jq -r '.l1_info_tree_leaf.rollup_exit_root')
    
    # First claim the asset
    print_info "Attempting first claim - Step 1: Asset..."
    # Calculate global index with mainnet flag for network 1 (mainnet in this SDK)
    GLOBAL_INDEX_ASSET=$(echo "$ASSET_DEPOSIT_COUNT + 18446744073709551616" | bc)
    print_debug "Asset global index: $GLOBAL_INDEX_ASSET (deposit count: $ASSET_DEPOSIT_COUNT with mainnet flag)"
    
    # Extract asset details
    ASSET_ORIGIN=$(echo $ASSET_BRIDGE | jq -r '.origin_address')
    ASSET_DEST=$(echo $ASSET_BRIDGE | jq -r '.destination_address')  
    ASSET_AMOUNT=$(echo $ASSET_BRIDGE | jq -r '.amount')
    
    CLAIM_ASSET=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
        "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
        $GLOBAL_INDEX_ASSET \
        $MAINNET_EXIT_ROOT \
        $ROLLUP_EXIT_ROOT \
        1 \
        $AGG_ERC20_L1 \
        $CHAIN_ID_AGGLAYER_1 \
        $ASSET_DEST \
        $ASSET_AMOUNT \
        $ASSET_METADATA \
        --private-key $PRIVATE_KEY_1 \
        --rpc-url $RPC_2 \
        --json 2>&1)
    
    if ! echo "$CLAIM_ASSET" | jq -e '.transactionHash' > /dev/null 2>&1; then
        print_error "Failed to claim asset"
        record_test_result "Double bridge-and-call claim prevention" false 1
        echo ""
    else
        CLAIM_ASSET_TX=$(echo "$CLAIM_ASSET" | jq -r '.transactionHash')
        print_info "Asset claim TX: $CLAIM_ASSET_TX"
        sleep 2
        
        # Now claim the message
        print_info "Attempting first claim - Step 2: Message..."
        
        # Get fresh proof for message
        LEAF_INDEX_MESSAGE=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $MESSAGE_DEPOSIT_COUNT | awk '/════════════════════════════════════════════════════════════/{if(p) print p; p=""} {p=$0} END{if(p && p ~ /^[0-9]+$/) print p}')
        print_debug "Message L1 info tree leaf index: $LEAF_INDEX_MESSAGE"
        
        PROOF_DATA_MESSAGE=$(aggsandbox show claim-proof --network-id 1 --leaf-index $LEAF_INDEX_MESSAGE --deposit-count $MESSAGE_DEPOSIT_COUNT | extract_json)
        MAINNET_EXIT_ROOT=$(echo $PROOF_DATA_MESSAGE | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
        ROLLUP_EXIT_ROOT=$(echo $PROOF_DATA_MESSAGE | jq -r '.l1_info_tree_leaf.rollup_exit_root')
        
        GLOBAL_INDEX_MESSAGE=$(echo "$MESSAGE_DEPOSIT_COUNT + 18446744073709551616" | bc)
        print_debug "Message global index: $GLOBAL_INDEX_MESSAGE (deposit count: $MESSAGE_DEPOSIT_COUNT with mainnet flag)"
        
        CLAIM1=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
            "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
            $GLOBAL_INDEX_MESSAGE \
            $MAINNET_EXIT_ROOT \
            $ROLLUP_EXIT_ROOT \
            0 \
            $MESSAGE_ORIGIN \
            $CHAIN_ID_AGGLAYER_1 \
            $MESSAGE_DEST \
            $MESSAGE_AMOUNT \
            $MESSAGE_METADATA \
            --private-key $PRIVATE_KEY_1 \
            --rpc-url $RPC_2 \
            --json 2>&1)
    
    if echo "$CLAIM1" | jq -e '.transactionHash' > /dev/null 2>&1; then
            CLAIM1_TX=$(echo "$CLAIM1" | jq -r '.transactionHash')
            print_info "First message claim succeeded: $CLAIM1_TX"
            sleep 3
        
        # Verify the bridge and call execution
        print_debug "Verifying first claim execution..."
        CLAIM1_RECEIPT=$(cast receipt $CLAIM1_TX --rpc-url $RPC_2 --json)
        CLAIM1_STATUS=$(echo "$CLAIM1_RECEIPT" | jq -r '.status')
        
        if [ "$CLAIM1_STATUS" = "0x1" ]; then
            print_success "✓ First claim executed successfully"
            
            # Verify claim events
            print_debug "Verifying claim events..."
            L2_EVENTS=$(aggsandbox events --network-id $CHAIN_ID_AGGLAYER_1 --bridge $POLYGON_ZKEVM_BRIDGE_L2 | extract_json)
            ASSET_CLAIM=$(echo "$L2_EVENTS" | jq --arg tx "$CLAIM_ASSET_TX" '[.events[] | select(.transaction_hash == $tx and .event_name == "ClaimEvent")] | length')
            MESSAGE_CLAIM=$(echo "$L2_EVENTS" | jq --arg tx "$CLAIM1_TX" '[.events[] | select(.transaction_hash == $tx and .event_name == "ClaimEvent")] | length')
            
            if [ $ASSET_CLAIM -gt 0 ] && [ $MESSAGE_CLAIM -gt 0 ]; then
                print_debug "Found claim events: $ASSET_CLAIM asset claim(s), $MESSAGE_CLAIM message claim(s)"
            fi
            
            # Check if call data was executed
            if [ -n "$CALL_DATA" ] && [ "$CALL_DATA" != "0x" ] && [ "$CALL_DATA" != "0x00" ]; then
                print_debug "Call data was included, checking execution..."
                # Check receiver contract
                CALL_COUNT=$(cast call $RECEIVER_CONTRACT "getCallCount()" --rpc-url $RPC_2)
                CALL_COUNT_DEC=$(printf "%d" $CALL_COUNT 2>/dev/null || echo "0")
                if [ $CALL_COUNT_DEC -gt 0 ]; then
                    print_info "Receiver contract recorded $CALL_COUNT_DEC call(s)"
                fi
            fi
        else
            print_error "First claim reverted unexpectedly"
        fi
        
        # Second claim (should fail) - try to claim the message again
            print_info "Attempting second claim of same message..."
            CLAIM2=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
                "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
                $GLOBAL_INDEX_MESSAGE \
                $MAINNET_EXIT_ROOT \
                $ROLLUP_EXIT_ROOT \
                0 \
                $MESSAGE_ORIGIN \
                $CHAIN_ID_AGGLAYER_1 \
                $MESSAGE_DEST \
                $MESSAGE_AMOUNT \
                $MESSAGE_METADATA \
                --private-key $PRIVATE_KEY_1 \
                --rpc-url $RPC_2 \
                --json 2>&1 || echo "FAILED")
        
        if [[ "$CLAIM2" == *"FAILED"* ]] || [[ "$CLAIM2" == *"AlreadyClaimed"* ]] || [[ "$CLAIM2" == *"revert"* ]]; then
            record_test_result "Double bridge-and-call claim prevention" true 1
        else
            record_test_result "Double bridge-and-call claim prevention" true 0
        fi
    else
            print_info "First message claim failed (might be GlobalExitRootInvalid)"
            record_test_result "Double bridge-and-call claim prevention" false 1
        fi
    fi
fi
echo ""

# Test Summary
echo ""
print_info "========== BRIDGE AND CALL ERROR TEST SUMMARY =========="
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
print_info "1. ✓ Bridge with malformed call data"
print_info "2. ✓ Bridge and call without approval"
print_info "3. ✓ Excessive call data"
print_info "4. ✓ Bridge with dangerous call"
print_info "5. ✓ Double bridge-and-call claim prevention"

echo ""
echo ""
if [ $FAILED_TESTS -eq 0 ]; then
    print_success "All bridge and call error handling tests completed successfully! ✅"
    exit 0
else
    print_error "Some tests did not behave as expected"
    exit 1
fi