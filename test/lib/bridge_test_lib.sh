#!/bin/bash
# Bridge Test Library - Main Index
# Modular bridge testing library that sources all bridge operation modules
# Version: 3.0 - Modular architecture with separate modules for each function

set -e  # Exit on error

# Get the directory of this script for relative imports
BRIDGE_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ============================================================================
# CONFIGURATION AND CONSTANTS
# ============================================================================

# Default values (can be overridden by environment variables)
DEFAULT_BRIDGE_AMOUNT=${DEFAULT_BRIDGE_AMOUNT:-100}
DEFAULT_GAS_LIMIT=${DEFAULT_GAS_LIMIT:-3000000}
DEFAULT_WAIT_TIME=${DEFAULT_WAIT_TIME:-20}
DEFAULT_MAX_RETRIES=${DEFAULT_MAX_RETRIES:-10}
DEFAULT_RETRY_DELAY=${DEFAULT_RETRY_DELAY:-2}

# Colors for output
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly RED='\033[0;31m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# ============================================================================
# LOGGING FUNCTIONS
# ============================================================================

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
    if [ "${DEBUG:-0}" = "1" ]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

print_warning() {
    echo -e "${CYAN}[WARNING]${NC} $1"
}

# ============================================================================
# ENVIRONMENT VALIDATION
# ============================================================================

# Load environment variables if .env exists
load_environment() {
    if [ -f .env ]; then
        source .env
        print_info "Loaded environment variables from .env"
    else
        print_error ".env file not found. Please ensure you have the environment file."
        return 1
    fi
}

# Validate required environment variables for bridge operations
validate_bridge_environment() {
    local required_vars=(
        "PRIVATE_KEY_1" "PRIVATE_KEY_2" 
        "ACCOUNT_ADDRESS_1" "ACCOUNT_ADDRESS_2"
        "RPC_1" "RPC_2"
        "NETWORK_ID_MAINNET" "NETWORK_ID_AGGLAYER_1"
        "CHAIN_ID_AGGLAYER_1"
    )

    local missing_vars=()
    for var in "${required_vars[@]}"; do
        if [ -z "${!var}" ]; then
            missing_vars+=("$var")
        fi
    done

    if [ ${#missing_vars[@]} -gt 0 ]; then
        print_error "Missing required environment variables:"
        for var in "${missing_vars[@]}"; do
            print_error "  - $var"
        done
        return 1
    fi

    print_success "All required environment variables are set"
    return 0
}

# Validate aggsandbox is running
validate_sandbox_status() {
    print_step "Validating sandbox status"
    
    if ! command -v aggsandbox &> /dev/null; then
        print_error "aggsandbox CLI not found. Please install it first."
        return 1
    fi

    # Check if sandbox is running
    if ! aggsandbox status --quiet &> /dev/null; then
        print_error "Sandbox is not running. Please start it with: aggsandbox start --detach"
        return 1
    fi

    print_success "Sandbox is running and accessible"
    return 0
}

# ============================================================================
# NETWORK AND TOKEN UTILITIES
# ============================================================================

# Get token balance using aggsandbox or cast
get_token_balance() {
    local token_address="$1"
    local account_address="$2"
    local network_id="$3"
    local rpc_url=""

    # Determine RPC URL based on network ID
    case "$network_id" in
        0|"${NETWORK_ID_MAINNET:-0}")
            rpc_url="$RPC_1"
            ;;
        1|"${NETWORK_ID_AGGLAYER_1:-1}")
            rpc_url="$RPC_2"
            ;;
        2|"${NETWORK_ID_AGGLAYER_2:-2}")
            rpc_url="${RPC_3:-$RPC_2}"
            ;;
        *)
            print_error "Unknown network ID: $network_id"
            return 1
            ;;
    esac

    if [ "$token_address" = "0x0000000000000000000000000000000000000000" ]; then
        # ETH balance
        cast balance "$account_address" --rpc-url "$rpc_url"
    else
        # ERC20 balance
        cast call "$token_address" "balanceOf(address)" "$account_address" --rpc-url "$rpc_url"
    fi
}

# Get human-readable balance (convert from wei)
get_balance_decimal() {
    local hex_balance="$1"
    local decimals="${2:-18}"
    
    if [ -z "$hex_balance" ] || [ "$hex_balance" = "0x" ]; then
        echo "0"
        return
    fi
    
    cast to-dec "$hex_balance" 2>/dev/null || echo "0"
}

# Wait for transaction confirmation
wait_for_transaction() {
    local tx_hash="$1"
    local rpc_url="$2"
    local confirmations="${3:-1}"
    
    print_info "Waiting for transaction confirmation: $tx_hash"
    if ! cast receipt "$tx_hash" --rpc-url "$rpc_url" --confirmations "$confirmations" &> /dev/null; then
        print_error "Transaction failed or timed out: $tx_hash"
        return 1
    fi
    
    print_success "Transaction confirmed: $tx_hash"
    return 0
}

# ============================================================================
# MODULE IMPORTS
# ============================================================================

# Import all bridge operation modules
print_debug "Loading bridge operation modules..."

# Load bridge asset operations
if [ -f "$BRIDGE_LIB_DIR/bridge_asset.sh" ]; then
    source "$BRIDGE_LIB_DIR/bridge_asset.sh"
else
    print_error "Bridge asset module not found: $BRIDGE_LIB_DIR/bridge_asset.sh"
    exit 1
fi

# Load bridge message operations  
if [ -f "$BRIDGE_LIB_DIR/bridge_message.sh" ]; then
    source "$BRIDGE_LIB_DIR/bridge_message.sh"
else
    print_error "Bridge message module not found: $BRIDGE_LIB_DIR/bridge_message.sh"
    exit 1
fi

# Load claim asset operations
if [ -f "$BRIDGE_LIB_DIR/claim_asset.sh" ]; then
    source "$BRIDGE_LIB_DIR/claim_asset.sh"
else
    print_error "Claim asset module not found: $BRIDGE_LIB_DIR/claim_asset.sh"
    exit 1
fi

# Load claim message operations
if [ -f "$BRIDGE_LIB_DIR/claim_message.sh" ]; then
    source "$BRIDGE_LIB_DIR/claim_message.sh"
else
    print_error "Claim message module not found: $BRIDGE_LIB_DIR/claim_message.sh"
    exit 1
fi

# Load bridge and call operations
if [ -f "$BRIDGE_LIB_DIR/bridge_and_call.sh" ]; then
    source "$BRIDGE_LIB_DIR/bridge_and_call.sh"
else
    print_error "Bridge and call module not found: $BRIDGE_LIB_DIR/bridge_and_call.sh"
    exit 1
fi

# Load claim bridge and call operations
if [ -f "$BRIDGE_LIB_DIR/claim_bridge_and_call.sh" ]; then
    source "$BRIDGE_LIB_DIR/claim_bridge_and_call.sh"
else
    print_error "Claim bridge and call module not found: $BRIDGE_LIB_DIR/claim_bridge_and_call.sh"
    exit 1
fi

print_debug "All bridge operation modules loaded successfully"

# ============================================================================
# BRIDGE INFORMATION QUERIES
# ============================================================================

# Get bridge information for a transaction
get_bridge_info() {
    local network_id="$1"
    local tx_hash="${2:-}" # Optional filter by tx_hash
    
    print_debug "Getting bridge information for network $network_id"
    
    local output
    if output=$(aggsandbox show bridges --network-id "$network_id" 2>&1); then
        if [ -n "$tx_hash" ]; then
            # Filter for specific transaction hash
            echo "$output" | grep -A 20 -B 5 "$tx_hash" || echo ""
        else
            echo "$output"
        fi
        return 0
    else
        print_debug "Failed to get bridge info: $output"
        return 1
    fi
}

# Wait for bridge indexing with retry logic
wait_for_bridge_indexing() {
    local network_id="$1"
    local tx_hash="$2"
    local max_retries="${3:-$DEFAULT_MAX_RETRIES}"
    local retry_delay="${4:-$DEFAULT_RETRY_DELAY}"
    local wait_time="${5:-$DEFAULT_WAIT_TIME}"
    
    print_step "Waiting for bridge indexing and global exit root propagation"
    print_info "This typically takes $wait_time-$(($wait_time + 10)) seconds in sandbox mode"
    
    # Initial wait for GER propagation
    sleep "$wait_time"
    
    # Retry logic for bridge indexing
    local retry_count=0
    local bridge_found=false
    
    while [ $retry_count -lt $max_retries ]; do
        print_debug "Checking bridge indexing (attempt $((retry_count + 1))/$max_retries)"
        
        local bridge_info
        if bridge_info=$(get_bridge_info "$network_id" "$tx_hash"); then
            if echo "$bridge_info" | grep -q "$tx_hash"; then
                print_success "Bridge found in indexing system"
                bridge_found=true
                break
            fi
        fi
        
        retry_count=$((retry_count + 1))
        if [ $retry_count -lt $max_retries ]; then
            print_info "Bridge not indexed yet, waiting... (attempt $retry_count/$max_retries)"
            sleep "$retry_delay"
        fi
    done
    
    if [ "$bridge_found" != "true" ]; then
        print_error "Bridge with TX $tx_hash not found after $max_retries attempts"
        return 1
    fi
    
    return 0
}

# Get claims information
get_claims_info() {
    local network_id="$1"
    
    print_debug "Getting claims information for network $network_id"
    
    local output
    if output=$(aggsandbox show claims --network-id "$network_id" 2>&1); then
        echo "$output"
        return 0
    else
        print_debug "Failed to get claims info: $output"
        return 1
    fi
}

# ============================================================================
# LEGACY COMPATIBILITY FUNCTIONS
# ============================================================================

# Legacy wrapper for execute_l1_to_l2_asset_bridge (for backward compatibility)
execute_l1_to_l2_bridge() {
    print_debug "Using legacy wrapper - consider updating to execute_l1_to_l2_asset_bridge"
    execute_l1_to_l2_asset_bridge "$@"
}

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

# Print summary of bridge operation
print_bridge_summary() {
    local bridge_tx_hash="$1"
    local claim_tx_hash="${2:-N/A}"
    local amount="${3:-Unknown}"
    local token_address="${4:-Unknown}"
    
    echo ""
    print_info "========== BRIDGE OPERATION SUMMARY =========="
    print_success "Bridge Transaction:"
    print_info "  ✅ Amount: $amount tokens"
    print_info "  ✅ Token: $token_address"
    print_info "  ✅ Bridge TX: $bridge_tx_hash"
    
    if [ "$claim_tx_hash" != "N/A" ] && [ -n "$claim_tx_hash" ]; then
        print_success "Claim Transaction:"
        print_info "  ✅ Claim TX: $claim_tx_hash"
    else
        print_info "  ⚠️  Claim transaction: Auto-executed or not required"
    fi
    
    print_info "============================================="
    echo ""
}

# Print test configuration
print_test_config() {
    echo ""
    print_info "========== TEST CONFIGURATION =========="
    print_info "Networks:"
    print_info "  • L1 (Network ${NETWORK_ID_MAINNET:-0}): $RPC_1"
    print_info "  • L2 (Network ${NETWORK_ID_AGGLAYER_1:-1}): $RPC_2"
    print_info "Accounts:"
    print_info "  • Account 1: ${ACCOUNT_ADDRESS_1:-Not set}"
    print_info "  • Account 2: ${ACCOUNT_ADDRESS_2:-Not set}"
    print_info "Tokens:"
    print_info "  • L1 Token: ${AGG_ERC20_L1:-Not deployed}"
    print_info "  • L2 Token: ${AGG_ERC20_L2:-Not deployed}"
    print_info "========================================"
    echo ""
}

# Show recent events (optional utility)
show_recent_events() {
    local chain="${1:-anvil-l2}"
    local blocks="${2:-5}"
    
    print_step "Recent $chain Events (last $blocks blocks)"
    if command -v aggsandbox &> /dev/null; then
        aggsandbox events --chain "$chain" --blocks "$blocks" 2>/dev/null || print_info "Events not available"
    else
        print_info "aggsandbox CLI not available for events"
    fi
}

# ============================================================================
# INITIALIZATION
# ============================================================================

# Initialize the test environment
init_bridge_test_environment() {
    print_step "Initializing bridge test environment"
    
    # Load environment variables
    if ! load_environment; then
        return 1
    fi
    
    # Validate environment
    if ! validate_bridge_environment; then
        return 1
    fi
    
    # Validate sandbox status
    if ! validate_sandbox_status; then
        return 1
    fi
    
    # Print configuration
    if [ "${VERBOSE:-0}" = "1" ] || [ "${DEBUG:-0}" = "1" ]; then
        print_test_config
    fi
    
    print_success "Bridge test environment initialized successfully"
    return 0
}

# ============================================================================
# EXPORTS
# ============================================================================

# Export core functions for use in other scripts
export -f print_step print_info print_error print_success print_debug print_warning
export -f load_environment validate_bridge_environment validate_sandbox_status
export -f get_token_balance get_balance_decimal wait_for_transaction
export -f get_bridge_info wait_for_bridge_indexing get_claims_info
export -f print_bridge_summary print_test_config show_recent_events
export -f init_bridge_test_environment

# Export legacy compatibility functions
export -f execute_l1_to_l2_bridge

# Note: Module functions are exported by their respective modules
# Bridge Asset Module: bridge_asset_modern, bridge_asset_with_approval, execute_l1_to_l2_asset_bridge, execute_l2_to_l1_asset_bridge
# Bridge Message Module: bridge_message_modern, bridge_text_message, bridge_function_call_message, execute_l1_to_l2_message_bridge, execute_l2_to_l1_message_bridge
# Claim Asset Module: claim_asset_modern, claim_asset_with_retry, get_claim_proof, is_asset_claimed, wait_for_asset_claimable, verify_claim_transaction
# Claim Message Module: claim_message_modern, claim_message_with_retry, is_message_claimed, get_message_proof, wait_for_message_claimable, verify_message_claim_transaction, decode_message_data
# Bridge and Call Module: bridge_and_call_modern, bridge_and_call_function, deploy_bridge_call_receiver, execute_l1_to_l2_bridge_and_call, execute_l2_to_l1_bridge_and_call, verify_bridge_and_call_execution
# Claim Bridge and Call Module: claim_bridge_and_call_modern, claim_bridge_and_call_with_retry, get_bridge_and_call_info, wait_for_bridge_and_call_claimable, verify_bridge_and_call_claim, extract_bridge_and_call_details

print_debug "Bridge test library v3.0 loaded successfully (modular architecture)" 