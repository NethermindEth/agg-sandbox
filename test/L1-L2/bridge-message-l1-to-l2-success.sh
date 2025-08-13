#!/bin/bash
# Test script for bridging messages from L1 to L2
# This script demonstrates bridging arbitrary messages (no token transfer)

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

echo ""
print_info "========== L1 TO L2 MESSAGE BRIDGE TEST =========="
print_info "This test bridges a message without token transfer"
echo ""

# Step 0: Deploy message receiver contract on L2
print_step "0. Deploying SimpleBridgeMessageReceiver contract on L2"

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

# Step 1: Prepare the message
print_step "1. Preparing message to bridge"

# Create a message payload
# The message will be used to call onMessageReceived on the destination
MESSAGE="Hello from L1!"
MESSAGE_HEX=$(echo -n "$MESSAGE" | xxd -p | tr -d '\n')
MESSAGE_BYTES="0x$MESSAGE_HEX"

# Parse optional ETH amount
ETH_AMOUNT=${1:-0}  # Default to 0 ETH if not specified

print_info "Message: $MESSAGE"
print_info "Message (hex): $MESSAGE_BYTES"
print_info "ETH amount: $ETH_AMOUNT wei"
print_debug "Message length: ${#MESSAGE} characters"

echo ""

# Step 2: Bridge the message
print_step "2. Sending message from L1 to L2"

# For message bridging, we use bridgeMessage function
# bridgeMessage(uint32 destinationNetwork, address destinationAddress, bool forceUpdateGlobalExitRoot, bytes calldata metadata)
DESTINATION_ADDRESS=$MESSAGE_RECEIVER  # Our deployed receiver contract

print_info "Destination network: $CHAIN_ID_AGGLAYER_1"
print_info "Destination address: $DESTINATION_ADDRESS"

if [ "$ETH_AMOUNT" -gt 0 ]; then
    print_info "Sending message with $ETH_AMOUNT wei"
    BRIDGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
        "bridgeMessage(uint32,address,bool,bytes)" \
        $CHAIN_ID_AGGLAYER_1 \
        $DESTINATION_ADDRESS \
        true \
        $MESSAGE_BYTES \
        --value $ETH_AMOUNT \
        --private-key $PRIVATE_KEY_1 \
        --rpc-url $RPC_1 \
        --json | jq -r '.transactionHash')
else
    print_info "Sending message without ETH"
    BRIDGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
        "bridgeMessage(uint32,address,bool,bytes)" \
        $CHAIN_ID_AGGLAYER_1 \
        $DESTINATION_ADDRESS \
        true \
        $MESSAGE_BYTES \
        --private-key $PRIVATE_KEY_1 \
        --rpc-url $RPC_1 \
        --json | jq -r '.transactionHash')
fi

if [ -z "$BRIDGE_TX" ] || [ "$BRIDGE_TX" = "null" ]; then
    print_error "Failed to send message bridge transaction"
    exit 1
fi

print_success "Message bridge TX: $BRIDGE_TX"
print_info "Waiting for confirmation..."
sleep 2

echo ""

# Step 3: Get bridge event details
print_step "3. Getting bridge event details"

# Get the transaction receipt
RECEIPT=$(cast receipt $BRIDGE_TX --rpc-url $RPC_1 --json)

if [ "$DEBUG" = "1" ]; then
    print_debug "Transaction receipt:"
    echo "$RECEIPT" | jq '.'
fi

# Check if transaction was successful
TX_STATUS=$(echo "$RECEIPT" | jq -r '.status')
if [ "$TX_STATUS" != "0x1" ]; then
    print_error "Bridge transaction failed!"
    exit 1
fi

print_info "Bridge transaction confirmed"

echo ""

# Step 4: Wait for global exit root propagation
print_step "4. Waiting for global exit root to propagate"
print_info "This typically takes 15-20 seconds..."

for i in {1..20}; do
    echo -n "."
    sleep 1
done
echo ""

echo ""

# Step 5: Get bridge information from API
print_step "5. Getting bridge information from bridge service"

BRIDGE_INFO=$(aggsandbox show bridges --network-id 1 | extract_json)

if [ "$DEBUG" = "1" ]; then
    print_debug "Bridge API response:"
    echo "$BRIDGE_INFO" | jq '.'
fi

# Find our bridge transaction
MATCHING_BRIDGE=$(echo $BRIDGE_INFO | jq -r --arg tx "$BRIDGE_TX" '.bridges[] | select(.tx_hash == $tx)')

if [ -z "$MATCHING_BRIDGE" ] || [ "$MATCHING_BRIDGE" = "null" ]; then
    print_error "Could not find bridge transaction in API"
    print_info "This might be a timing issue. Try running the script again."
    exit 1
fi

print_info "Found bridge in API:"
echo "$MATCHING_BRIDGE" | jq '.'

# Extract values
LEAF_TYPE=$(echo $MATCHING_BRIDGE | jq -r '.leaf_type')
READY_FOR_CLAIM=$(echo $MATCHING_BRIDGE | jq -r '.ready_for_claim')
DEPOSIT_COUNT=$(echo $MATCHING_BRIDGE | jq -r '.deposit_count')
ORIGIN_ADDRESS=$(echo $MATCHING_BRIDGE | jq -r '.origin_address')

print_info "Leaf type: $LEAF_TYPE (1 = message)"
print_info "Ready for claim: $READY_FOR_CLAIM"
print_info "Deposit count: $DEPOSIT_COUNT"
print_info "Origin address: $ORIGIN_ADDRESS"

echo ""

# Step 6: Get claim proof
print_step "6. Getting claim proof"

# Get L1 info tree index first
LEAF_INDEX=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $DEPOSIT_COUNT | awk '/════════════════════════════════════════════════════════════/{if(p) print p; p=""} {p=$0} END{if(p && p ~ /^[0-9]+$/) print p}')
print_info "L1 info tree leaf index: $LEAF_INDEX"

PROOF_DATA=$(aggsandbox show claim-proof --network-id 1 --leaf-index $LEAF_INDEX --deposit-count $DEPOSIT_COUNT | extract_json)

if [ "$DEBUG" = "1" ]; then
    print_debug "Proof data:"
    echo "$PROOF_DATA" | jq '.'
fi

MAINNET_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
ROLLUP_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.rollup_exit_root')

print_info "Mainnet exit root: $MAINNET_EXIT_ROOT"
print_info "Rollup exit root: $ROLLUP_EXIT_ROOT"

echo ""

# Step 7: Claim the message on L2
print_step "7. Claiming message on L2"

# For messages, we use claimMessage function
# claimMessage(uint256 globalIndex, bytes32 mainnetExitRoot, bytes32 rollupExitRoot, uint32 originNetwork, address originAddress, uint32 destinationNetwork, address destinationAddress, uint256 amount, bytes calldata metadata)
print_info "Submitting claim transaction..."

# The globalIndex for L1 origin has the mainnet flag set (bit 64)
# For mainnet (network 0), the global index is just the leaf index with the mainnet flag
GLOBAL_INDEX=$(echo "$DEPOSIT_COUNT + 18446744073709551616" | bc)
print_debug "Global index: $GLOBAL_INDEX (deposit count: $DEPOSIT_COUNT with mainnet flag)"

CLAIM_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
    "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
    $GLOBAL_INDEX \
    $MAINNET_EXIT_ROOT \
    $ROLLUP_EXIT_ROOT \
    0 \
    $ORIGIN_ADDRESS \
    $CHAIN_ID_AGGLAYER_1 \
    $DESTINATION_ADDRESS \
    $ETH_AMOUNT \
    $MESSAGE_BYTES \
    --private-key $PRIVATE_KEY_2 \
    --rpc-url $RPC_2 \
    --json 2>&1)

# Check if claim was successful
if echo "$CLAIM_TX" | jq -e '.transactionHash' > /dev/null 2>&1; then
    CLAIM_TX_HASH=$(echo "$CLAIM_TX" | jq -r '.transactionHash')
    print_success "Claim TX: $CLAIM_TX_HASH"
    
    # Wait for confirmation
    sleep 2
    
    # Get claim receipt
    CLAIM_RECEIPT=$(cast receipt $CLAIM_TX_HASH --rpc-url $RPC_2 --json)
    CLAIM_STATUS=$(echo "$CLAIM_RECEIPT" | jq -r '.status')
    
    if [ "$CLAIM_STATUS" = "0x1" ]; then
        print_success "Message claim successful!"
        
        # Check if the destination received the message
        print_info "Message delivered to $DESTINATION_ADDRESS"
        
        # Check the receiver contract state
        print_info "Checking message receiver state..."
        
        # Get last message details from the contract
        print_info "Retrieving last message from receiver contract..."
        LAST_MESSAGE_RAW=$(cast call $MESSAGE_RECEIVER "getLastMessage()" --rpc-url $RPC_2)
        
        # Decode the response (returns: address, uint32, bytes, uint256)
        # The response is ABI encoded, we need to parse it
        print_debug "Raw response: $LAST_MESSAGE_RAW"
        
        # Get individual fields
        RECEIVED_ORIGIN=$(cast call $MESSAGE_RECEIVER "lastOriginAddress()" --rpc-url $RPC_2)
        RECEIVED_NETWORK=$(cast call $MESSAGE_RECEIVER "lastOriginNetwork()" --rpc-url $RPC_2)
        RECEIVED_DATA=$(cast call $MESSAGE_RECEIVER "lastMessageData()" --rpc-url $RPC_2)
        RECEIVED_VALUE=$(cast call $MESSAGE_RECEIVER "lastMessageValue()" --rpc-url $RPC_2)
        
        # Convert values for comparison
        # Remove 0x prefix and any leading zeros for addresses
        RECEIVED_ORIGIN_CLEAN=$(echo $RECEIVED_ORIGIN | sed 's/^0x//' | sed 's/^0*//')
        # Ensure it's 40 characters (20 bytes)
        if [ ${#RECEIVED_ORIGIN_CLEAN} -lt 40 ]; then
            RECEIVED_ORIGIN_CLEAN=$(printf "%040s" $RECEIVED_ORIGIN_CLEAN | tr ' ' '0')
        fi
        RECEIVED_ORIGIN_ADDR="0x$RECEIVED_ORIGIN_CLEAN"
        
        # Convert network and value
        RECEIVED_NETWORK_DEC=$(printf "%d" $RECEIVED_NETWORK 2>/dev/null || echo "0")
        RECEIVED_VALUE_DEC=$(printf "%d" $RECEIVED_VALUE 2>/dev/null || echo "0")
        
        print_info "Received message details:"
        print_info "  Origin address: $RECEIVED_ORIGIN_ADDR"
        print_info "  Origin network: $RECEIVED_NETWORK_DEC"
        print_info "  Message value: $RECEIVED_VALUE_DEC wei"
        print_info "  Message data: $RECEIVED_DATA"
        
        # Verify the message matches what we sent
        print_step "Verifying message integrity..."
        
        # Check origin address (should be the original sender)
        # Normalize the expected address too
        EXPECTED_ORIGIN_CLEAN=$(echo $ORIGIN_ADDRESS | sed 's/^0x//' | tr '[:upper:]' '[:lower:]')
        RECEIVED_ORIGIN_LOWER=$(echo $RECEIVED_ORIGIN_ADDR | sed 's/^0x//' | tr '[:upper:]' '[:lower:]')
        
        if [ "$RECEIVED_ORIGIN_LOWER" = "$EXPECTED_ORIGIN_CLEAN" ]; then
            print_success "✓ Origin address matches: $ORIGIN_ADDRESS"
        else
            print_error "✗ Origin address mismatch! Expected: $ORIGIN_ADDRESS, Got: $RECEIVED_ORIGIN_ADDR"
        fi
        
        # Check origin network (should be 0 for L1)
        if [ "$RECEIVED_NETWORK_DEC" = "0" ]; then
            print_success "✓ Origin network correct: 0 (L1)"
        else
            print_error "✗ Origin network mismatch! Expected: 0, Got: $RECEIVED_NETWORK_DEC"
        fi
        
        # Check ETH amount
        if [ "$RECEIVED_VALUE_DEC" = "$ETH_AMOUNT" ]; then
            print_success "✓ ETH amount matches: $ETH_AMOUNT wei"
        else
            print_error "✗ ETH amount mismatch! Expected: $ETH_AMOUNT, Got: $RECEIVED_VALUE_DEC"
        fi
        
        # Check message data
        # Remove 0x prefix for comparison
        SENT_DATA_NO_PREFIX=${MESSAGE_BYTES#0x}
        RECEIVED_DATA_NO_PREFIX=${RECEIVED_DATA#0x}
        
        # The received data might be padded, so we need to extract the actual data
        # First 64 chars (32 bytes) = offset, next 64 chars = length, then the actual data
        if [[ ${#RECEIVED_DATA_NO_PREFIX} -gt 128 ]]; then
            # Skip offset (64 chars) and length (64 chars)
            DATA_LENGTH_HEX=${RECEIVED_DATA_NO_PREFIX:64:64}
            DATA_LENGTH=$((16#$DATA_LENGTH_HEX))
            # Each byte is 2 hex chars
            DATA_HEX_LENGTH=$((DATA_LENGTH * 2))
            ACTUAL_DATA=${RECEIVED_DATA_NO_PREFIX:128:$DATA_HEX_LENGTH}
            
            if [ "$ACTUAL_DATA" = "$SENT_DATA_NO_PREFIX" ]; then
                print_success "✓ Message data matches!"
                # Decode the hex back to string for display
                DECODED_MESSAGE=$(echo -n "$ACTUAL_DATA" | xxd -r -p)
                print_info "  Decoded message: '$DECODED_MESSAGE'"
            else
                print_error "✗ Message data mismatch!"
                print_info "  Expected: $SENT_DATA_NO_PREFIX"
                print_info "  Received: $ACTUAL_DATA"
            fi
        else
            print_error "✗ Unexpected data format"
        fi
        
        # Get total messages received
        TOTAL_MESSAGES=$(cast call $MESSAGE_RECEIVER "totalMessagesReceived()" --rpc-url $RPC_2)
        TOTAL_MESSAGES_DEC=$(cast to-dec $TOTAL_MESSAGES)
        print_info "Total messages received by contract: $TOTAL_MESSAGES_DEC"
        
        # If ETH was sent, verify contract balance
        if [ "$ETH_AMOUNT" -gt 0 ]; then
            CONTRACT_BALANCE=$(cast balance $MESSAGE_RECEIVER --rpc-url $RPC_2)
            print_info "Receiver contract balance: $CONTRACT_BALANCE wei"
            
            TOTAL_ETH=$(cast call $MESSAGE_RECEIVER "totalEthReceived()" --rpc-url $RPC_2)
            TOTAL_ETH_DEC=$(cast to-dec $TOTAL_ETH)
            print_info "Total ETH received by contract: $TOTAL_ETH_DEC wei"
            
            if [ "$TOTAL_ETH_DEC" -ge "$ETH_AMOUNT" ]; then
                print_success "✓ Contract received the ETH successfully"
            else
                print_error "✗ ETH receipt issue"
            fi
        fi
        
        # Try to decode any events from the claim
        if [ "$DEBUG" = "1" ]; then
            print_debug "Claim transaction logs:"
            echo "$CLAIM_RECEIPT" | jq '.logs'
        fi
    else
        print_error "Claim transaction reverted"
        exit 1
    fi
else
    print_error "Failed to send claim transaction"
    
    if echo "$CLAIM_TX" | grep -q "0x002f6fad"; then
        print_info "GlobalExitRootInvalid error - synchronization issue"
        print_info "Try waiting longer before claiming"
    elif echo "$CLAIM_TX" | grep -q "AlreadyClaimed"; then
        print_info "This message was already claimed"
    elif echo "$CLAIM_TX" | grep -q "DestinationNetworkInvalid"; then
        print_info "Destination network is invalid"
    fi
    
    exit 1
fi

echo ""

# Summary
print_info "========== MESSAGE BRIDGE TEST SUMMARY =========="
print_success "Message Bridge Completed Successfully!"
print_info "  ✅ Message: $MESSAGE"
print_info "  ✅ Bridge TX: $BRIDGE_TX"
print_info "  ✅ Deposit Count: $DEPOSIT_COUNT"
print_info "  ✅ Claim TX: $CLAIM_TX_HASH"
if [ "$ETH_AMOUNT" -gt 0 ]; then
    print_info "  ✅ ETH Amount: $ETH_AMOUNT wei"
fi
print_info ""
print_info "Note: Messages are delivered to contracts implementing IBridgeMessageReceiver"
print_info "The SimpleBridgeMessageReceiver contract stores message details for verification"
print_info "========================================="
echo ""