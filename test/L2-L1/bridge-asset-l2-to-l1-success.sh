#!/bin/bash
# Test script for successful L2 to L1 bridging
# This tests the complete flow of bridging assets from L2 back to L1

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
BRIDGE_AMOUNT=${1:-10}  # Default to 10 tokens if not specified

echo ""
print_info "========== L2 TO L1 BRIDGE TEST =========="
print_info "Bridge Amount: $BRIDGE_AMOUNT AGG tokens"
echo ""

# Step 1: Check initial balances
print_step "1. Checking initial balances"

# Get L1 balance
L1_BALANCE_BEFORE=$(cast call $AGG_ERC20_L1 \
    "balanceOf(address)" \
    $ACCOUNT_ADDRESS_1 \
    --rpc-url $RPC_1 | sed 's/0x//' | tr '[:lower:]' '[:upper:]')
L1_BALANCE_BEFORE_DEC=$((16#$L1_BALANCE_BEFORE))

# Get L2 balance
L2_BALANCE_BEFORE=$(cast call $AGG_ERC20_L2 \
    "balanceOf(address)" \
    $ACCOUNT_ADDRESS_1 \
    --rpc-url $RPC_2 | sed 's/0x//' | tr '[:lower:]' '[:upper:]')
L2_BALANCE_BEFORE_DEC=$((16#$L2_BALANCE_BEFORE))

print_info "Account: $ACCOUNT_ADDRESS_1"
print_info "L1 Balance: $L1_BALANCE_BEFORE_DEC AGG tokens"
print_info "L2 Balance: $L2_BALANCE_BEFORE_DEC AGG tokens"

# Check if we have enough balance on L2
if [ $L2_BALANCE_BEFORE_DEC -lt $BRIDGE_AMOUNT ]; then
    print_error "Insufficient L2 balance. Need $BRIDGE_AMOUNT but have $L2_BALANCE_BEFORE_DEC"
    print_info "Please ensure you have bridged tokens from L1 to L2 first"
    exit 1
fi

echo ""

# Step 2: Approve the bridge contract on L2
print_step "2. Approving bridge contract on L2"

APPROVE_TX=$(cast send $AGG_ERC20_L2 \
    "approve(address,uint256)" \
    $POLYGON_ZKEVM_BRIDGE_L2 $BRIDGE_AMOUNT \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_2 \
    --json | jq -r '.transactionHash')

print_info "Approval TX: $APPROVE_TX"
print_debug "Waiting for approval confirmation..."
sleep 2

# Check approval
ALLOWANCE=$(cast call $AGG_ERC20_L2 \
    "allowance(address,address)" \
    $ACCOUNT_ADDRESS_1 $POLYGON_ZKEVM_BRIDGE_L2 \
    --rpc-url $RPC_2 | sed 's/0x//' | tr '[:lower:]' '[:upper:]')
ALLOWANCE_DEC=$((16#$ALLOWANCE))

print_info "Approved allowance: $ALLOWANCE_DEC"
echo ""

# Step 3: Bridge from L2 to L1
print_step "3. Bridging $BRIDGE_AMOUNT tokens from L2 to L1"

# Destination network for L2->L1 is always 0 (mainnet)
DESTINATION_NETWORK=0

BRIDGE_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L2 \
    "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
    $DESTINATION_NETWORK \
    $ACCOUNT_ADDRESS_1 \
    $BRIDGE_AMOUNT \
    $AGG_ERC20_L2 \
    true \
    0x \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_2 \
    --json | jq -r '.transactionHash')

if [ -z "$BRIDGE_TX" ] || [ "$BRIDGE_TX" = "null" ]; then
    print_error "Failed to get bridge transaction hash"
    exit 1
fi

print_success "Bridge TX: $BRIDGE_TX"
echo ""

# Step 4: Get bridge event details
print_step "4. Getting bridge event details"

# Wait a bit for event indexing
sleep 3

# Get the bridge receipt
print_debug "Getting transaction receipt..."
RECEIPT=$(cast receipt $BRIDGE_TX --rpc-url $RPC_2 --json)

if [ "$DEBUG" = "1" ]; then
    echo "$RECEIPT" | jq '.'
fi

# Extract deposit count from logs (should be in the BridgeEvent)
DEPOSIT_COUNT=$(echo "$RECEIPT" | jq -r '.logs[] | select(.topics[0] == "0x501781209a1f8899323b96b4ef08b168df93e0a90c673d1e4cce39366cb62f9b") | .topics[3]' | sed 's/^0x//')
DEPOSIT_COUNT_DEC=$((16#$DEPOSIT_COUNT))

print_info "Deposit count: $DEPOSIT_COUNT_DEC"

# Get block number
BLOCK_NUMBER=$(echo "$RECEIPT" | jq -r '.blockNumber')
print_info "Block number: $BLOCK_NUMBER"

echo ""

# Step 5: Wait for global exit root propagation
print_step "5. Waiting for global exit root to propagate to L1"
print_info "This typically takes 15-20 seconds..."

for i in {1..20}; do
    echo -n "."
    sleep 1
done
echo ""

echo ""

# Step 6: Get bridge information from API
print_step "6. Getting bridge information from bridge service"

# Query bridges for L2 network (network_id 1101)
BRIDGE_INFO=$(aggsandbox show bridges --network-id 1101 | extract_json)

if [ "$DEBUG" = "1" ]; then
    print_debug "Bridge API response:"
    echo "$BRIDGE_INFO" | jq '.'
fi

# Find our bridge transaction
MATCHING_BRIDGE=$(echo $BRIDGE_INFO | jq -r --arg tx "$BRIDGE_TX" '.bridges[] | select(.tx_hash == $tx)')

if [ -z "$MATCHING_BRIDGE" ] || [ "$MATCHING_BRIDGE" = "null" ]; then
    print_error "Could not find bridge transaction in API response"
    print_info "This might be a timing issue. Try running the script again."
    exit 1
fi

print_info "Found bridge in API:"
echo "$MATCHING_BRIDGE" | jq '.'

# Extract necessary values
API_DEPOSIT_COUNT=$(echo $MATCHING_BRIDGE | jq -r '.deposit_count')
CLAIM_TX_HASH=$(echo $MATCHING_BRIDGE | jq -r '.claim_tx_hash')
READY_FOR_CLAIM=$(echo $MATCHING_BRIDGE | jq -r '.ready_for_claim')

print_info "API Deposit count: $API_DEPOSIT_COUNT"
print_info "Ready for claim: $READY_FOR_CLAIM"
print_info "Claim TX hash: $CLAIM_TX_HASH"

echo ""

# Step 7: Get claim proof
print_step "7. Getting claim proof for L1"

# For L2->L1, we need proof from network 1101 with the deposit count
PROOF_DATA=$(aggsandbox show claim-proof --network-id 1101 --leaf-index $API_DEPOSIT_COUNT --deposit-count $API_DEPOSIT_COUNT | extract_json)

if [ "$DEBUG" = "1" ]; then
    print_debug "Proof data:"
    echo "$PROOF_DATA" | jq '.'
fi

# Extract proof components
MAINNET_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
ROLLUP_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.rollup_exit_root')

print_info "Mainnet exit root: $MAINNET_EXIT_ROOT"
print_info "Rollup exit root: $ROLLUP_EXIT_ROOT"

echo ""

# Step 8: Claim on L1
print_step "8. Claiming tokens on L1"

# Encode metadata for the wrapped token
METADATA=$(cast abi-encode "f(string,string,uint8)" "AggERC20" "AGGERC20" 18)
print_debug "Metadata: $METADATA"

# The origin network for L2->L1 is the L2 chain ID
ORIGIN_NETWORK=$CHAIN_ID_AGGLAYER_1

print_info "Submitting claim transaction on L1..."
CLAIM_TX=$(cast send $POLYGON_ZKEVM_BRIDGE_L1 \
    "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
    $API_DEPOSIT_COUNT \
    $MAINNET_EXIT_ROOT \
    $ROLLUP_EXIT_ROOT \
    $ORIGIN_NETWORK \
    $AGG_ERC20_L2 \
    $DESTINATION_NETWORK \
    $ACCOUNT_ADDRESS_1 \
    $BRIDGE_AMOUNT \
    $METADATA \
    --private-key $PRIVATE_KEY_1 \
    --rpc-url $RPC_1 \
    --json 2>&1)

# Check if claim was successful
if echo "$CLAIM_TX" | jq -e '.transactionHash' > /dev/null 2>&1; then
    CLAIM_TX_HASH=$(echo "$CLAIM_TX" | jq -r '.transactionHash')
    print_success "Claim TX: $CLAIM_TX_HASH"
    echo ""
    
    # Wait for claim confirmation
    print_info "Waiting for claim confirmation..."
    sleep 3
    
    # Step 9: Verify final balances
    print_step "9. Verifying final balances"
    
    # Get L1 balance after
    L1_BALANCE_AFTER=$(cast call $AGG_ERC20_L1 \
        "balanceOf(address)" \
        $ACCOUNT_ADDRESS_1 \
        --rpc-url $RPC_1 | sed 's/0x//' | tr '[:lower:]' '[:upper:]')
    L1_BALANCE_AFTER_DEC=$((16#$L1_BALANCE_AFTER))
    
    # Get L2 balance after
    L2_BALANCE_AFTER=$(cast call $AGG_ERC20_L2 \
        "balanceOf(address)" \
        $ACCOUNT_ADDRESS_1 \
        --rpc-url $RPC_2 | sed 's/0x//' | tr '[:lower:]' '[:upper:]')
    L2_BALANCE_AFTER_DEC=$((16#$L2_BALANCE_AFTER))
    
    print_info "Final L1 Balance: $L1_BALANCE_AFTER_DEC AGG tokens"
    print_info "Final L2 Balance: $L2_BALANCE_AFTER_DEC AGG tokens"
    
    # Calculate differences
    L1_DIFF=$((L1_BALANCE_AFTER_DEC - L1_BALANCE_BEFORE_DEC))
    L2_DIFF=$((L2_BALANCE_AFTER_DEC - L2_BALANCE_BEFORE_DEC))
    
    print_info "L1 Balance change: +$L1_DIFF"
    print_info "L2 Balance change: $L2_DIFF"
    
    echo ""
    
    # Verify the bridge worked correctly
    if [ $L1_DIFF -eq $BRIDGE_AMOUNT ] && [ $L2_DIFF -eq -$BRIDGE_AMOUNT ]; then
        print_success " L2 to L1 bridge completed successfully!"
        print_success "Successfully moved $BRIDGE_AMOUNT tokens from L2 to L1"
    else
        print_error "Balance changes don't match expected amounts"
        print_error "Expected L1: +$BRIDGE_AMOUNT, Got: +$L1_DIFF"
        print_error "Expected L2: -$BRIDGE_AMOUNT, Got: $L2_DIFF"
        exit 1
    fi
    
else
    # Claim failed
    print_error "Claim transaction failed!"
    
    # Check for common errors
    if echo "$CLAIM_TX" | grep -q "0x002f6fad"; then
        print_error "Error: GlobalExitRootInvalid (0x002f6fad)"
        print_info "The global exit root has not been synchronized to L1 yet"
        print_info "Try waiting longer before claiming, or run the script again"
    elif echo "$CLAIM_TX" | grep -q "AlreadyClaimed"; then
        print_error "Error: This deposit has already been claimed"
    else
        print_error "Error details:"
        echo "$CLAIM_TX"
    fi
    
    exit 1
fi

echo ""
print_info "========== TEST COMPLETED =========="
print_info "Summary:"
print_info "- Bridged $BRIDGE_AMOUNT tokens from L2 to L1"
print_info "- Bridge TX: $BRIDGE_TX"
print_info "- Claim TX: $CLAIM_TX_HASH"
print_info "- Final L1 balance: $L1_BALANCE_AFTER_DEC tokens"
print_info "- Final L2 balance: $L2_BALANCE_AFTER_DEC tokens"
echo ""