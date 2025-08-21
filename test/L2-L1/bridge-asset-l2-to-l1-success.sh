#!/bin/bash
# Test script for bridging assets from L2 to L1 using modern aggsandbox CLI
# This script demonstrates how to reuse the modular bridge library for different directions
# Version: 2.0 - Updated for modern aggsandbox CLI with modular design

set -e  # Exit on error

# ============================================================================
# SCRIPT CONFIGURATION
# ============================================================================

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Load the shared bridge test library
LIB_PATH="$SCRIPT_DIR/../lib/bridge_test_lib.sh"
if [ -f "$LIB_PATH" ]; then
    source "$LIB_PATH"
else
    echo "Error: Bridge test library not found at $LIB_PATH"
    echo "Please ensure the test library exists and is accessible."
    exit 1
fi

# ============================================================================
# SCRIPT-SPECIFIC CONFIGURATION
# ============================================================================

# Script metadata
readonly SCRIPT_NAME="L2-L1 Bridge Asset Test"
readonly SCRIPT_VERSION="2.0"
readonly SCRIPT_DESCRIPTION="Test bridging assets from L2 to L1 using modern aggsandbox CLI"

# Default configuration
BRIDGE_AMOUNT=${DEFAULT_BRIDGE_AMOUNT:-50}
TOKEN_ADDRESS=""
SHOW_EVENTS=false
VERBOSE=false
DRY_RUN=false

# ============================================================================
# L2-L1 SPECIFIC FUNCTIONS
# ============================================================================

# Execute L2 to L1 bridge flow (reverse direction)
execute_l2_to_l1_bridge() {
    local amount="${1:-$DEFAULT_BRIDGE_AMOUNT}"
    local token_address="${2:-$AGG_ERC20_L2}"
    local source_account="${3:-$ACCOUNT_ADDRESS_2}"
    local dest_account="${4:-$ACCOUNT_ADDRESS_1}"
    local source_private_key="${5:-$PRIVATE_KEY_2}"
    local dest_private_key="${6:-$PRIVATE_KEY_1}"
    
    print_step "Executing complete L2 to L1 bridge flow"
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

# Validate L2-L1 specific prerequisites
validate_l2_prerequisites() {
    print_step "Validating L2-L1 bridge prerequisites"
    
    # Check if we're in the right directory
    if [ ! -f "$PROJECT_ROOT/.env" ] && [ ! -f "$PROJECT_ROOT/.env.example" ]; then
        print_error "Not in project root directory. Please run from the aggsandbox project root."
        return 1
    fi
    
    # Change to project root if not already there
    cd "$PROJECT_ROOT"
    
    # Initialize environment
    if ! init_bridge_test_environment; then
        return 1
    fi
    
    # Set default token address if not specified (use L2 wrapped token)
    if [ -z "$TOKEN_ADDRESS" ]; then
        # For L2->L1, we typically bridge wrapped tokens back
        # This would be the wrapped token created from previous L1->L2 bridge
        TOKEN_ADDRESS="0x19e2b7738a026883d08c3642984ab6d7510ca238"  # Common wrapped token address
        print_info "Using default L2 wrapped token: $TOKEN_ADDRESS"
        print_info "Note: This assumes you have previously bridged tokens from L1 to L2"
    fi
    
    # Validate token address format
    if [[ ! "$TOKEN_ADDRESS" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
        print_error "Invalid token address format: $TOKEN_ADDRESS"
        return 1
    fi
    
    print_success "L2-L1 bridge prerequisites validated"
    return 0
}

# Check L2 initial balances
check_l2_initial_balances() {
    print_step "Checking initial L2 token balances"
    
    # L2 balance for source account
    local l2_balance_hex
    l2_balance_hex=$(get_token_balance "$TOKEN_ADDRESS" "$ACCOUNT_ADDRESS_2" "$NETWORK_ID_AGGLAYER_1")
    local l2_balance_dec
    l2_balance_dec=$(get_balance_decimal "$l2_balance_hex")
    
    print_info "L2 Balance ($ACCOUNT_ADDRESS_2): $l2_balance_dec tokens"
    
    # Check if sufficient balance for bridge
    if [ "$l2_balance_dec" -lt "$BRIDGE_AMOUNT" ]; then
        print_error "Insufficient L2 balance. Have: $l2_balance_dec, Need: $BRIDGE_AMOUNT"
        print_info "Tip: Run the L1-L2 bridge test first to get tokens on L2"
        return 1
    fi
    
    print_success "L2 balance check passed"
    return 0
}

# ============================================================================
# MAIN EXECUTION FLOW
# ============================================================================

main() {
    # Print script header
    echo ""
    print_success "==================== $SCRIPT_NAME v$SCRIPT_VERSION ===================="
    print_info "$SCRIPT_DESCRIPTION"
    echo ""
    
    # Parse command line arguments (reuse from L1-L2 script pattern)
    while [[ $# -gt 0 ]]; do
        case $1 in
            --token-address)
                TOKEN_ADDRESS="$2"
                shift 2
                ;;
            --show-events)
                SHOW_EVENTS=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                export VERBOSE=1
                shift
                ;;
            --debug)
                DEBUG=true
                export DEBUG=1
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help|-h)
                cat << EOF
$SCRIPT_NAME v$SCRIPT_VERSION
$SCRIPT_DESCRIPTION

Usage: $0 [OPTIONS] [AMOUNT]

Arguments:
  AMOUNT                 Amount of tokens to bridge from L2 to L1 (default: $BRIDGE_AMOUNT)

Options:
  --token-address ADDR   L2 token contract address to bridge back to L1
  --show-events         Show recent blockchain events after the test
  --verbose             Enable verbose output
  --debug               Enable debug mode with detailed logging
  --dry-run             Validate environment without executing transactions
  --help, -h            Show this help message

Examples:
  $0                                    # Bridge default amount using default wrapped token
  $0 25                                 # Bridge 25 wrapped tokens back to L1
  $0 --token-address 0x123... 50        # Bridge 50 of specific L2 token back to L1

Note: This script bridges tokens FROM L2 TO L1. Run the L1-L2 bridge test first
      to ensure you have wrapped tokens on L2 to bridge back.
EOF
                exit 0
                ;;
            -*)
                print_error "Unknown option: $1"
                exit 1
                ;;
            *)
                if [[ "$1" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
                    BRIDGE_AMOUNT="$1"
                else
                    print_error "Invalid argument: $1"
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # Set debug/verbose flags
    if [ "$DEBUG" = "true" ]; then
        export DEBUG=1
    fi
    if [ "$VERBOSE" = "true" ]; then
        export VERBOSE=1
    fi
    
    # Validate prerequisites
    if ! validate_l2_prerequisites; then
        print_error "Prerequisites validation failed"
        exit 1
    fi
    
    # Print test configuration
    if [ "$VERBOSE" = "true" ] || [ "$DEBUG" = "true" ]; then
        print_test_config
    fi
    
    # Check if this is a dry run
    if [ "$DRY_RUN" = "true" ]; then
        print_success "✅ Dry run completed - environment is properly configured"
        print_info "Would bridge $BRIDGE_AMOUNT tokens of $TOKEN_ADDRESS from L2 to L1"
        exit 0
    fi
    
    # Check initial L2 balances
    if ! check_l2_initial_balances; then
        print_error "L2 balance check failed"
        exit 1
    fi
    
    # Execute the L2-L1 bridge test
    print_info "Bridge amount: $BRIDGE_AMOUNT tokens"
    print_info "Token address: $TOKEN_ADDRESS"
    print_info "From: $ACCOUNT_ADDRESS_2 (L2)"
    print_info "To: $ACCOUNT_ADDRESS_1 (L1)"
    
    local bridge_tx_hash
    if ! bridge_tx_hash=$(execute_l2_to_l1_bridge \
        "$BRIDGE_AMOUNT" \
        "$TOKEN_ADDRESS" \
        "$ACCOUNT_ADDRESS_2" \
        "$ACCOUNT_ADDRESS_1" \
        "$PRIVATE_KEY_2" \
        "$PRIVATE_KEY_1"); then
        print_error "L2-L1 bridge test failed"
        exit 1
    fi
    
    # Print final summary
    print_bridge_summary "$bridge_tx_hash" "" "$BRIDGE_AMOUNT" "$TOKEN_ADDRESS"
    
    # Show recent events if requested
    if [ "$SHOW_EVENTS" = "true" ]; then
        show_recent_events "anvil-l1" 5
    fi
    
    print_success "🎉 L2 to L1 bridge test completed successfully!"
    
    echo ""
    print_info "💡 Next steps:"
    print_info "  • Check L1 balance: cast call $AGG_ERC20_L1 \"balanceOf(address)\" $ACCOUNT_ADDRESS_1 --rpc-url $RPC_1"
    print_info "  • View bridge status: aggsandbox show bridges --network-id $NETWORK_ID_MAINNET"
    print_info "  • Check claims: aggsandbox show claims --network-id $NETWORK_ID_MAINNET"
}

# Handle script execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
else
    export -f execute_l2_to_l1_bridge validate_l2_prerequisites check_l2_initial_balances
    print_debug "L2-L1 bridge test script loaded as library"
fi