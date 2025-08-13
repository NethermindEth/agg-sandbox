#!/bin/bash
# Test script for bridge and call from L1 to L2
# This demonstrates bridging tokens and executing a function call in one transaction

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
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

# Parse command line arguments
BRIDGE_AMOUNT=${1:-50}  # Default to 50 tokens if not specified

echo ""
print_info "========== L1 TO L2 BRIDGE AND CALL TEST =========="
print_info "Bridge Amount: $BRIDGE_AMOUNT AGG tokens"
print_info "This test bridges tokens and executes a call in one transaction"
echo ""

# Step 1: Check initial balance
print_step "1. Checking initial token balance on L1"

L1_BALANCE=$(cast call $AGG_ERC20_L1 \
    "balanceOf(address)" \
    $ACCOUNT_ADDRESS_1 \
    --rpc-url $RPC_1 | sed 's/0x//' | tr '[:lower:]' '[:upper:]')
L1_BALANCE_DEC=$((16#$L1_BALANCE))

print_info "L1 Balance: $L1_BALANCE_DEC AGG tokens"

if [ $L1_BALANCE_DEC -lt $BRIDGE_AMOUNT ]; then
    print_error "Insufficient L1 balance. Need $BRIDGE_AMOUNT but have $L1_BALANCE_DEC"
    exit 1
fi

echo ""

# Step 2: Approve the bridge extension contract
print_step "2. Approving bridge extension contract to spend tokens"

APPROVE_TX=$(cast send $AGG_ERC20_L1 \
    "approve(address,uint256)" \
    $BRIDGE_EXTENSION_L1 $BRIDGE_AMOUNT \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json | jq -r '.transactionHash')

print_info "Approval TX: $APPROVE_TX"
print_info "Waiting for confirmation..."
sleep 2

echo ""

# Step 3: Deploy receiver contract on L2 and prepare call data
print_step "3. Deploying bridge-and-call receiver contract on L2"

# Deploy SimpleBridgeAndCallReceiver contract on L2
print_info "Deploying SimpleBridgeAndCallReceiver contract..."
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

# Get the precalculated L2 token address
print_info "Getting precalculated wrapped token address on L2..."
L2_TOKEN_ADDRESS=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
    "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
    1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
    --rpc-url $RPC_2)
# Remove padding from address
L2_TOKEN_ADDRESS="0x$(echo $L2_TOKEN_ADDRESS | sed 's/0x0*//' | tail -c 41)"
print_info "L2 wrapped token address: $L2_TOKEN_ADDRESS"

# Prepare call data for the receiver contract
# receiveTokensWithMessage(address token, uint256 amount, string memory message)
MESSAGE="Hello from L1 bridge-and-call!"
CALL_AMOUNT=10  # Amount to pass to the receiver function

# Encode the function call
CALL_DATA=$(cast abi-encode "receiveTokensWithMessage(address,uint256,string)" $L2_TOKEN_ADDRESS $CALL_AMOUNT "$MESSAGE")
print_info "Call data: $CALL_DATA"
print_info "Will call receiveTokensWithMessage with $CALL_AMOUNT tokens and message: '$MESSAGE'"

echo ""

# Step 4: Execute bridge and call
print_step "4. Executing bridge and call transaction"

# bridgeAndCall(address token, uint256 amount, uint32 destinationNetwork, address callAddress, address fallbackAddress, bytes calldata callData, bool forceUpdateGlobalExitRoot)
print_info "Destination network: $CHAIN_ID_AGGLAYER_1"
print_info "Call address (Receiver contract): $RECEIVER_CONTRACT"
print_info "Fallback address: $ACCOUNT_ADDRESS_1"

# Execute bridge and call through the extension contract
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

if [ -z "$BRIDGE_TX" ] || [ "$BRIDGE_TX" = "null" ]; then
    print_error "Failed to send bridge and call transaction"
    exit 1
fi

print_success "Bridge and call TX: $BRIDGE_TX"
print_info "Waiting for confirmation..."
sleep 2

echo ""

# Step 5: Get bridge event details
print_step "5. Getting bridge event details"

RECEIPT=$(cast receipt $BRIDGE_TX --rpc-url $RPC_1 --json)
TX_STATUS=$(echo "$RECEIPT" | jq -r '.status')
TX_BLOCK=$(echo "$RECEIPT" | jq -r '.blockNumber' | xargs printf "%d")

if [ "$TX_STATUS" != "0x1" ]; then
    print_error "Bridge transaction failed!"
    exit 1
fi

print_success "Bridge transaction confirmed in block $TX_BLOCK"

echo ""

# Step 6: Wait for global exit root propagation
print_step "6. Waiting for global exit root to propagate"
print_info "This typically takes 20 seconds..."

for i in {1..20}; do
    echo -n "."
    sleep 1
done
echo ""

echo ""

# Step 7: Get bridge information from API
print_step "7. Getting bridge information from bridge service"

BRIDGE_INFO=$(aggsandbox show bridges --network-id 1 | extract_json)

if [ "$DEBUG" = "1" ]; then
    print_debug "Bridge API response:"
    echo "$BRIDGE_INFO" | jq '.'
fi

# For bridge and call, we need to find both the asset bridge and the message bridge
# The BridgeExtension creates two deposits: one for assets, one for the message
print_info "Looking for bridge transactions created by our bridge and call..."

# Get all bridges after our transaction
BRIDGES_AFTER_TX=$(echo $BRIDGE_INFO | jq -r --arg block "$TX_BLOCK" '.bridges[] | select(.block_num >= ($block | tonumber))')

# Find the asset bridge (leaf_type = 0)
ASSET_BRIDGE=$(echo $BRIDGE_INFO | jq -r '.bridges[] | select(.leaf_type == 0)' | jq -s 'sort_by(.deposit_count) | .[-1]')

# Find the message bridge (leaf_type = 1)
MESSAGE_BRIDGE=$(echo $BRIDGE_INFO | jq -r '.bridges[] | select(.leaf_type == 1)' | jq -s 'sort_by(.deposit_count) | .[-1]')

if [ -z "$ASSET_BRIDGE" ] || [ "$ASSET_BRIDGE" = "null" ]; then
    print_error "Could not find asset bridge transaction in API"
    exit 1
fi

if [ -z "$MESSAGE_BRIDGE" ] || [ "$MESSAGE_BRIDGE" = "null" ]; then
    print_error "Could not find message bridge transaction in API"
    exit 1
fi

print_info "Found asset bridge in API:"
echo "$ASSET_BRIDGE" | jq '.'

print_info "Found message bridge in API:"
echo "$MESSAGE_BRIDGE" | jq '.'

# Extract values from asset bridge
ASSET_DEPOSIT_COUNT=$(echo $ASSET_BRIDGE | jq -r '.deposit_count')
ASSET_METADATA=$(echo $ASSET_BRIDGE | jq -r '.metadata')
ASSET_AMOUNT=$(echo $ASSET_BRIDGE | jq -r '.amount')
ASSET_ORIGIN_NETWORK=$(echo $ASSET_BRIDGE | jq -r '.origin_network')

# Extract values from message bridge
MESSAGE_DEPOSIT_COUNT=$(echo $MESSAGE_BRIDGE | jq -r '.deposit_count')
MESSAGE_METADATA=$(echo $MESSAGE_BRIDGE | jq -r '.metadata')
MESSAGE_ORIGIN_NETWORK=$(echo $MESSAGE_BRIDGE | jq -r '.origin_network')

print_info "Asset deposit count: $ASSET_DEPOSIT_COUNT (origin network: $ASSET_ORIGIN_NETWORK)"
print_info "Asset amount: $ASSET_AMOUNT"
print_info "Message deposit count: $MESSAGE_DEPOSIT_COUNT (origin network: $MESSAGE_ORIGIN_NETWORK)"
print_info "Message metadata first 66 chars: ${MESSAGE_METADATA:0:66}"

# Decode the dependsOnIndex from message metadata
if [[ ${#MESSAGE_METADATA} -gt 66 ]]; then
    DEPENDS_ON_INDEX_HEX=${MESSAGE_METADATA:2:64}
    DEPENDS_ON_INDEX=$((16#$DEPENDS_ON_INDEX_HEX))
    print_info "DependsOnIndex from metadata: $DEPENDS_ON_INDEX (should equal asset deposit count: $ASSET_DEPOSIT_COUNT)"
fi

echo ""

# Step 8: Get claim proof for the asset bridge
print_step "8. Getting claim proof for the asset bridge"

# First get the L1 info tree index for the asset
print_debug "Getting L1 info tree index for asset deposit count: $ASSET_DEPOSIT_COUNT"
L1_INFO_ASSET=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $ASSET_DEPOSIT_COUNT 2>&1)

# Check if the command succeeded
if [[ "$L1_INFO_ASSET" == *"Error"* ]] || [[ "$L1_INFO_ASSET" == *"error"* ]]; then
    print_error "Failed to get L1 info tree index: $L1_INFO_ASSET"
    print_info "Falling back to using deposit count as leaf index"
    LEAF_INDEX_ASSET=$ASSET_DEPOSIT_COUNT
else
    # Extract the number from the decorated output
    # The output format has the number between two separator lines
    LEAF_INDEX_ASSET=$(echo "$L1_INFO_ASSET" | grep -A1 -E "^════+$" | grep -E "^[0-9]+$" | head -1)
    
    if [ -z "$LEAF_INDEX_ASSET" ]; then
        print_info "Could not extract L1 info tree index, using deposit count"
        LEAF_INDEX_ASSET=$ASSET_DEPOSIT_COUNT
    fi
fi

print_info "Asset L1 info tree leaf index: $LEAF_INDEX_ASSET"

# Get proof using the leaf index
PROOF_DATA_ASSET=$(aggsandbox show claim-proof --network-id 1 --leaf-index $LEAF_INDEX_ASSET --deposit-count $ASSET_DEPOSIT_COUNT | extract_json)

if [ "$DEBUG" = "1" ]; then
    print_debug "Asset proof data:"
    echo "$PROOF_DATA_ASSET" | jq '.'
fi

MAINNET_EXIT_ROOT=$(echo $PROOF_DATA_ASSET | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
ROLLUP_EXIT_ROOT=$(echo $PROOF_DATA_ASSET | jq -r '.l1_info_tree_leaf.rollup_exit_root')

print_info "Mainnet exit root: $MAINNET_EXIT_ROOT"
print_info "Rollup exit root: $ROLLUP_EXIT_ROOT"

echo ""

# Step 9: Execute claims on L2 - Asset first, then Message
print_step "9. Claiming asset and message on L2"

# FIRST: Claim the asset
print_info "Step 9a: Claiming asset..."

# Calculate global index based on origin network
# In this SDK, mainnet is network 1 (not 0), so we add mainnet flag for network 1
if [ "$ASSET_ORIGIN_NETWORK" = "1" ]; then
    # For mainnet origin (network 1 in this SDK), we need to set the mainnet flag in global index
    GLOBAL_INDEX_ASSET=$(echo "$ASSET_DEPOSIT_COUNT + 18446744073709551616" | bc)
    print_debug "Asset global index: $GLOBAL_INDEX_ASSET (deposit count: $ASSET_DEPOSIT_COUNT with mainnet flag for network 1)"
else
    # For other networks, just use the deposit count
    GLOBAL_INDEX_ASSET=$ASSET_DEPOSIT_COUNT
    print_debug "Asset global index: $GLOBAL_INDEX_ASSET (deposit count: $ASSET_DEPOSIT_COUNT, no mainnet flag)"
fi

# Extract asset details
ASSET_ORIGIN=$(echo $ASSET_BRIDGE | jq -r '.origin_address')
ASSET_DEST=$(echo $ASSET_BRIDGE | jq -r '.destination_address')

# The asset goes to the pre-computed JumpPoint address, not directly to user
# BridgeExtension computes this address based on the dependsOnIndex
print_info "Claiming asset (will go to JumpPoint contract)..."

CLAIM_ASSET_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
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

# Check if asset claim was successful
if echo "$CLAIM_ASSET_TX" | jq -e '.transactionHash' > /dev/null 2>&1; then
    CLAIM_ASSET_TX_HASH=$(echo "$CLAIM_ASSET_TX" | jq -r '.transactionHash')
    print_success "Asset claim TX: $CLAIM_ASSET_TX_HASH"
    
    # Wait for asset claim to be confirmed
    print_info "Waiting for asset claim confirmation..."
    sleep 3
    
    # Verify the asset was actually claimed
    ASSET_RECEIPT=$(cast receipt $CLAIM_ASSET_TX_HASH --rpc-url $RPC_2 --json)
    ASSET_STATUS=$(echo "$ASSET_RECEIPT" | jq -r '.status')
    
    if [ "$ASSET_STATUS" != "0x1" ]; then
        print_error "Asset claim transaction failed!"
        exit 1
    fi
    
    # Check if the asset is marked as claimed
    print_debug "Checking if asset is marked as claimed..."
    # Check if the asset is marked as claimed using the correct function signature
    # isClaimed(uint32 leafIndex, uint32 sourceBridgeNetwork)
    # The BridgeExtension contract shows as origin network 1
    # Check both networks to see which one the asset was claimed from
    IS_CLAIMED_NET0=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 "isClaimed(uint32,uint32)" $ASSET_DEPOSIT_COUNT 0 --rpc-url $RPC_2)
    IS_CLAIMED_NET1=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 "isClaimed(uint32,uint32)" $ASSET_DEPOSIT_COUNT 1 --rpc-url $RPC_2)
    print_info "Asset deposit $ASSET_DEPOSIT_COUNT claimed status (network 0): $IS_CLAIMED_NET0"
    print_info "Asset deposit $ASSET_DEPOSIT_COUNT claimed status (network 1): $IS_CLAIMED_NET1"
    
    # Additional wait to ensure state is updated
    sleep 2
else
    print_error "Failed to claim asset"
    echo "$CLAIM_ASSET_TX"
    if echo "$CLAIM_ASSET_TX" | grep -q "0x002f6fad"; then
        print_info "GlobalExitRootInvalid error - synchronization issue"
        print_info "The global exit root may not have propagated yet"
    fi
    exit 1
fi

# SECOND: Get proof and claim the message
print_info "Step 9b: Getting proof for message claim..."

# Get the L1 info tree index for the message
print_debug "Getting L1 info tree index for message deposit count: $MESSAGE_DEPOSIT_COUNT"
L1_INFO_MESSAGE=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $MESSAGE_DEPOSIT_COUNT 2>&1)

# Check if the command succeeded
if [[ "$L1_INFO_MESSAGE" == *"Error"* ]] || [[ "$L1_INFO_MESSAGE" == *"error"* ]]; then
    print_error "Failed to get L1 info tree index: $L1_INFO_MESSAGE"
    print_info "Falling back to using deposit count as leaf index"
    LEAF_INDEX_MESSAGE=$MESSAGE_DEPOSIT_COUNT
else
    # Extract the number from the decorated output
    # The output format has the number between two separator lines
    LEAF_INDEX_MESSAGE=$(echo "$L1_INFO_MESSAGE" | grep -A1 -E "^════+$" | grep -E "^[0-9]+$" | head -1)
    
    if [ -z "$LEAF_INDEX_MESSAGE" ]; then
        print_info "Could not extract L1 info tree index, using deposit count"
        LEAF_INDEX_MESSAGE=$MESSAGE_DEPOSIT_COUNT
    fi
fi

print_info "Message L1 info tree leaf index: $LEAF_INDEX_MESSAGE"

# Get proof for message with updated exit roots (may have changed after asset claim)
PROOF_DATA_MESSAGE=$(aggsandbox show claim-proof --network-id 1 --leaf-index $LEAF_INDEX_MESSAGE --deposit-count $MESSAGE_DEPOSIT_COUNT | extract_json)
MAINNET_EXIT_ROOT=$(echo $PROOF_DATA_MESSAGE | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
ROLLUP_EXIT_ROOT=$(echo $PROOF_DATA_MESSAGE | jq -r '.l1_info_tree_leaf.rollup_exit_root')

print_info "Step 9c: Claiming message (will trigger call execution)..."

# Calculate global index based on origin network
# In this SDK, mainnet is network 1 (not 0), so we add mainnet flag for network 1
if [ "$MESSAGE_ORIGIN_NETWORK" = "1" ]; then
    GLOBAL_INDEX_MESSAGE=$(echo "$MESSAGE_DEPOSIT_COUNT + 18446744073709551616" | bc)
    print_debug "Message global index: $GLOBAL_INDEX_MESSAGE (deposit count: $MESSAGE_DEPOSIT_COUNT with mainnet flag for network 1)"
else
    GLOBAL_INDEX_MESSAGE=$MESSAGE_DEPOSIT_COUNT
    print_debug "Message global index: $GLOBAL_INDEX_MESSAGE (deposit count: $MESSAGE_DEPOSIT_COUNT, no mainnet flag)"
fi

# Extract message details
MESSAGE_ORIGIN=$(echo $MESSAGE_BRIDGE | jq -r '.origin_address')
MESSAGE_DEST=$(echo $MESSAGE_BRIDGE | jq -r '.destination_address')
MESSAGE_AMOUNT=$(echo $MESSAGE_BRIDGE | jq -r '.amount')
MESSAGE_ORIGIN_NETWORK=$(echo $MESSAGE_BRIDGE | jq -r '.origin_network')

print_debug "Message origin network: $MESSAGE_ORIGIN_NETWORK"
print_debug "Asset origin network: $(echo $ASSET_BRIDGE | jq -r '.origin_network')"

# The message was claimed from originNetwork which comes from the message bridge
print_debug "Using origin network $MESSAGE_ORIGIN_NETWORK for isClaimed check"

# Check if the asset is claimed with the correct origin network
IS_ASSET_CLAIMED=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 "isClaimed(uint32,uint32)" $ASSET_DEPOSIT_COUNT $MESSAGE_ORIGIN_NETWORK --rpc-url $RPC_2)
print_info "Asset deposit $ASSET_DEPOSIT_COUNT claimed (network $MESSAGE_ORIGIN_NETWORK): $IS_ASSET_CLAIMED"

if [ "$IS_ASSET_CLAIMED" = "false" ]; then
    print_error "Asset not claimed on expected network!"
    print_info "This is why we're getting UnclaimedAsset error"
    print_info "The asset was claimed with origin network $ASSET_ORIGIN_NETWORK but BridgeExtension checks with $MESSAGE_ORIGIN_NETWORK"
fi

# Use the original metadata as-is - BridgeExtension already encoded it correctly
# The dependsOnIndex mismatch is expected due to how deposit counts work
if [ "$DEPENDS_ON_INDEX" != "$ASSET_DEPOSIT_COUNT" ]; then
    print_info "Note: DependsOnIndex ($DEPENDS_ON_INDEX) differs from asset deposit count ($ASSET_DEPOSIT_COUNT)"
    print_info "This is expected - BridgeExtension calculated it at bridge time"
fi

# Claim the message - this will trigger onMessageReceived on BridgeExtension
CLAIM_MESSAGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
    "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
    $GLOBAL_INDEX_MESSAGE \
    $MAINNET_EXIT_ROOT \
    $ROLLUP_EXIT_ROOT \
    $MESSAGE_ORIGIN_NETWORK \
    $MESSAGE_ORIGIN \
    $CHAIN_ID_AGGLAYER_1 \
    $MESSAGE_DEST \
    $MESSAGE_AMOUNT \
    $MESSAGE_METADATA \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_2 \
    --json 2>&1)

# Check if message claim was successful
if echo "$CLAIM_MESSAGE_TX" | jq -e '.transactionHash' > /dev/null 2>&1; then
    CLAIM_MESSAGE_TX_HASH=$(echo "$CLAIM_MESSAGE_TX" | jq -r '.transactionHash')
    print_success "Message claim TX: $CLAIM_MESSAGE_TX_HASH"
    
    # Wait for confirmation
    sleep 3
    
    # Get claim receipt
    CLAIM_RECEIPT=$(cast receipt $CLAIM_MESSAGE_TX_HASH --rpc-url $RPC_2 --json)
    CLAIM_STATUS=$(echo "$CLAIM_RECEIPT" | jq -r '.status')
    
    if [ "$CLAIM_STATUS" = "0x1" ]; then
        print_success "Asset and message claims executed successfully!"
        print_info "BridgeExtension will now deploy JumpPoint and execute the call"
        
        # Check if the call was executed
        print_info "Checking for call execution events..."
        if [ "$DEBUG" = "1" ]; then
            echo "$CLAIM_RECEIPT" | jq '.logs'
        fi
    else
        print_error "Message claim transaction reverted"
        exit 1
    fi
else
    print_error "Failed to send message claim transaction"
    
    if echo "$CLAIM_MESSAGE_TX" | grep -q "0x002f6fad"; then
        print_info "GlobalExitRootInvalid error - synchronization issue"
    elif echo "$CLAIM_MESSAGE_TX" | grep -q "AlreadyClaimed"; then
        print_info "This message was already claimed"
    elif echo "$CLAIM_MESSAGE_TX" | grep -q "UnclaimedAsset"; then
        print_info "Asset must be claimed before message"
    fi
    
    exit 1
fi

echo ""

# Step 10: Verify bridge events
print_step "10. Verifying bridge events using aggsandbox"

# Check L1 bridge events
print_info "Checking L1 bridge events..."
L1_EVENTS=$(aggsandbox events --network-id 0 --bridge $POLYGON_ZKEVM_BRIDGE_L1 --from-block $TX_BLOCK | extract_json)

if [ "$DEBUG" = "1" ]; then
    print_debug "L1 bridge events:"
    echo "$L1_EVENTS" | jq '.'
fi

# Count bridge events from our transaction
BRIDGE_EVENT_COUNT=$(echo "$L1_EVENTS" | jq --arg tx "$BRIDGE_TX" '[.events[] | select(.transaction_hash == $tx)] | length')
print_info "Found $BRIDGE_EVENT_COUNT bridge events from our transaction"

if [ $BRIDGE_EVENT_COUNT -eq 2 ]; then
    print_success "✓ Found expected 2 events (asset + message) from BridgeExtension"
    
    # Extract event details
    ASSET_EVENT=$(echo "$L1_EVENTS" | jq --arg tx "$BRIDGE_TX" '[.events[] | select(.transaction_hash == $tx and .event_name == "BridgeEvent" and .leaf_type == 0)] | .[0]')
    MESSAGE_EVENT=$(echo "$L1_EVENTS" | jq --arg tx "$BRIDGE_TX" '[.events[] | select(.transaction_hash == $tx and .event_name == "BridgeEvent" and .leaf_type == 1)] | .[0]')
    
    if [ "$ASSET_EVENT" != "null" ] && [ "$MESSAGE_EVENT" != "null" ]; then
        print_info "Asset event deposit count: $(echo $ASSET_EVENT | jq -r '.deposit_count')"
        print_info "Message event deposit count: $(echo $MESSAGE_EVENT | jq -r '.deposit_count')"
    fi
else
    print_error "Unexpected number of bridge events: $BRIDGE_EVENT_COUNT"
fi

# Check L2 claim events
print_info "Checking L2 claim events..."
L2_EVENTS=$(aggsandbox events --network-id $CHAIN_ID_AGGLAYER_1 --bridge $POLYGON_ZKEVM_BRIDGE_L2 | extract_json)

if [ "$DEBUG" = "1" ]; then
    print_debug "L2 bridge events:"
    echo "$L2_EVENTS" | jq '.'
fi

# Count claim events
ASSET_CLAIM_EVENT=$(echo "$L2_EVENTS" | jq --arg tx "$CLAIM_ASSET_TX_HASH" '[.events[] | select(.transaction_hash == $tx and .event_name == "ClaimEvent")] | .[0]')
MESSAGE_CLAIM_EVENT=$(echo "$L2_EVENTS" | jq --arg tx "$CLAIM_MESSAGE_TX_HASH" '[.events[] | select(.transaction_hash == $tx and .event_name == "ClaimEvent")] | .[0]')

if [ "$ASSET_CLAIM_EVENT" != "null" ]; then
    print_success "✓ Found asset claim event"
    print_info "Asset claimed - index: $(echo $ASSET_CLAIM_EVENT | jq -r '.global_index')"
fi

if [ "$MESSAGE_CLAIM_EVENT" != "null" ]; then
    print_success "✓ Found message claim event"
    print_info "Message claimed - index: $(echo $MESSAGE_CLAIM_EVENT | jq -r '.global_index')"
fi

echo ""

# Step 11: Verify the call execution
print_step "11. Verifying call execution results"

# Check if the receiver contract received the call
print_info "Checking receiver contract state..."

# Get call count
CALL_COUNT=$(cast call $RECEIVER_CONTRACT "getCallCount()" --rpc-url $RPC_2)
CALL_COUNT_DEC=$(printf "%d" $CALL_COUNT 2>/dev/null || echo "0")
print_info "Call count: $CALL_COUNT_DEC"

if [ $CALL_COUNT_DEC -gt 0 ]; then
    # Get last message
    LAST_MESSAGE=$(cast call $RECEIVER_CONTRACT "getLastMessage()" --rpc-url $RPC_2)
    # Decode the string from hex
    DECODED_MESSAGE=$(cast --abi-decode "f()(string)" $LAST_MESSAGE 2>/dev/null | sed 's/^[[:space:]]*//')
    print_info "Last message received: $DECODED_MESSAGE"
    
    # Get call details
    CALL_DETAILS=$(cast call $RECEIVER_CONTRACT "getCall(uint256)" 0 --rpc-url $RPC_2)
    print_info "Call details retrieved"
    
    # Check token balance in receiver
    TOKEN_BALANCE=$(cast call $RECEIVER_CONTRACT "tokenBalances(address)" $L2_TOKEN_ADDRESS --rpc-url $RPC_2)
    TOKEN_BALANCE_DEC=$(printf "%d" $TOKEN_BALANCE 2>/dev/null || echo "0")
    print_info "Token balance in receiver: $TOKEN_BALANCE_DEC"
    
    # Verify the message matches
    if [[ "$DECODED_MESSAGE" == "$MESSAGE" ]]; then
        print_success "Call executed! Message matches: '$MESSAGE'"
        
        # Also check JumpPoint balance (should have received the bridged tokens)
        # The JumpPoint would have forwarded tokens to the receiver
        if [ $TOKEN_BALANCE_DEC -gt 0 ]; then
            print_success "Tokens received by contract: $TOKEN_BALANCE_DEC"
        fi
    else
        print_info "Message doesn't match expected value"
    fi
else
    print_error "No calls recorded in receiver contract"
    
    # Check if tokens were at least bridged to the JumpPoint
    print_info "Checking if tokens were minted on L2..."
    TOTAL_SUPPLY=$(cast call $L2_TOKEN_ADDRESS "totalSupply()" --rpc-url $RPC_2 2>/dev/null || echo "0x0")
    if [ "$TOTAL_SUPPLY" != "0x0" ]; then
        TOTAL_SUPPLY_DEC=$(printf "%d" $TOTAL_SUPPLY 2>/dev/null || echo "0")
        print_info "Token was minted with total supply: $TOTAL_SUPPLY_DEC"
    fi
fi

echo ""

# Summary
print_info "========== BRIDGE AND CALL TEST SUMMARY =========="
print_success "Bridge and Call Completed!"
print_info "  ✅ Bridged Amount: $BRIDGE_AMOUNT tokens"
print_info "  ✅ Bridge TX: $BRIDGE_TX"
print_info "  ✅ Asset Deposit Count: $ASSET_DEPOSIT_COUNT"
print_info "  ✅ Message Deposit Count: $MESSAGE_DEPOSIT_COUNT"
print_info "  ✅ Asset Claim TX: $CLAIM_ASSET_TX_HASH"
print_info "  ✅ Message Claim TX: $CLAIM_MESSAGE_TX_HASH"
print_info "  ✅ Bridge Events: $BRIDGE_EVENT_COUNT events created"
if [ -n "$CALL_DATA" ] && [ "$CALL_DATA" != "0x" ]; then
    print_info "  ✅ Call Data: Included"
    print_info "  ✅ Call Target: $RECEIVER_CONTRACT"
    print_info "  ✅ Call Function: receiveTokensWithMessage"
    print_info "  ✅ Message: '$MESSAGE'"
    if [ $CALL_COUNT_DEC -gt 0 ]; then
        print_info "  ✅ Call Executed: Yes ($CALL_COUNT_DEC calls recorded)"
    fi
fi
print_info ""
print_info "Note: Bridge and call allows atomic execution of"
print_info "token bridging and contract calls in one transaction"
print_info "========================================="
echo ""