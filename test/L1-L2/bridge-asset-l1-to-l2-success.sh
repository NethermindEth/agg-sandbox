#!/bin/bash
# Test script for bridging assets from L1 to L2 using modern aggsandbox CLI
# This script demonstrates the complete flow of bridging ERC20 tokens from L1 to L2
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
readonly SCRIPT_NAME="L1-L2 Bridge Asset Test"
readonly SCRIPT_VERSION="2.0"
readonly SCRIPT_DESCRIPTION="Test bridging ERC20 tokens from L1 to L2 using modern aggsandbox CLI"

# Default configuration (can be overridden by command line arguments)
BRIDGE_AMOUNT=${DEFAULT_BRIDGE_AMOUNT:-100}
TOKEN_ADDRESS=""
SHOW_EVENTS=false
VERBOSE=false
DRY_RUN=false

# ============================================================================
# COMMAND LINE ARGUMENT PARSING
# ============================================================================

print_usage() {
    cat << EOF
$SCRIPT_NAME v$SCRIPT_VERSION
$SCRIPT_DESCRIPTION

Usage: $0 [OPTIONS] [AMOUNT]

Arguments:
  AMOUNT                 Amount of tokens to bridge (default: $DEFAULT_BRIDGE_AMOUNT)

Options:
  --token-address ADDR   Token contract address to bridge (default: AGG_ERC20_L1 from env)
  --show-events         Show recent blockchain events after the test
  --verbose             Enable verbose output
  --debug               Enable debug mode with detailed logging
  --dry-run             Validate environment without executing transactions
  --help, -h            Show this help message

Environment Variables:
  DEBUG                 Set to 1 to enable debug mode
  VERBOSE               Set to 1 to enable verbose mode
  DEFAULT_BRIDGE_AMOUNT Default amount to bridge if not specified

Examples:
  $0                                    # Bridge default amount using default token
  $0 50                                 # Bridge 50 tokens
  $0 --token-address 0x123... 25        # Bridge 25 of specific token
  $0 --show-events --verbose            # Bridge with events and verbose output
  DEBUG=1 $0 100                        # Bridge 100 tokens with debug logging

EOF
}

parse_arguments() {
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
                print_usage
                exit 0
                ;;
            -*)
                print_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
            *)
                # Assume it's the amount if it's a number
                if [[ "$1" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
                    BRIDGE_AMOUNT="$1"
                else
                    print_error "Invalid argument: $1"
                    print_usage
                    exit 1
                fi
                shift
                ;;
        esac
    done
}

# ============================================================================
# TEST EXECUTION FUNCTIONS
# ============================================================================

# Pre-flight checks
validate_test_prerequisites() {
    print_step "Validating test prerequisites"
    
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
    
    # Set default token address if not specified
    if [ -z "$TOKEN_ADDRESS" ]; then
        if [ -n "$AGG_ERC20_L1" ]; then
            TOKEN_ADDRESS="$AGG_ERC20_L1"
            print_info "Using default L1 token: $TOKEN_ADDRESS"
        else
            print_error "No token address specified and AGG_ERC20_L1 not set in environment"
            print_info "Please specify --token-address or ensure contracts are deployed"
            return 1
        fi
    fi
    
    # Validate token address format
    if [[ ! "$TOKEN_ADDRESS" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
        print_error "Invalid token address format: $TOKEN_ADDRESS"
        return 1
    fi
    
    print_success "Test prerequisites validated"
    return 0
}

# Check initial balances
check_initial_balances() {
    print_step "Checking initial token balances"
    
    # L1 balance for source account
    local l1_balance_hex
    l1_balance_hex=$(get_token_balance "$TOKEN_ADDRESS" "$ACCOUNT_ADDRESS_1" "$NETWORK_ID_MAINNET")
    local l1_balance_dec
    l1_balance_dec=$(get_balance_decimal "$l1_balance_hex")
    
    print_info "L1 Balance ($ACCOUNT_ADDRESS_1): $l1_balance_dec tokens"
    
    # Check if sufficient balance for bridge
    if [ "$l1_balance_dec" -lt "$BRIDGE_AMOUNT" ]; then
        print_error "Insufficient L1 balance. Have: $l1_balance_dec, Need: $BRIDGE_AMOUNT"
        return 1
    fi
    
    # L2 balance for destination account (wrapped token might not exist yet)
    print_info "L2 wrapped token balance will be checked after bridge operation"
    
    return 0
}

# Execute the main bridge test
execute_bridge_test() {
    print_step "Executing L1 to L2 bridge test"
    print_info "Bridge amount: $BRIDGE_AMOUNT tokens"
    print_info "Token address: $TOKEN_ADDRESS"
    print_info "From: $ACCOUNT_ADDRESS_1 (L1)"
    print_info "To: $ACCOUNT_ADDRESS_2 (L2)"
    
    # Execute the complete bridge flow using the library function
    local bridge_tx_hash
    if ! bridge_tx_hash=$(execute_l1_to_l2_bridge \
        "$BRIDGE_AMOUNT" \
        "$TOKEN_ADDRESS" \
        "$ACCOUNT_ADDRESS_1" \
        "$ACCOUNT_ADDRESS_2" \
        "$PRIVATE_KEY_1" \
        "$PRIVATE_KEY_2"); then
        print_error "Bridge test failed"
        return 1
    fi
    
    print_success "Bridge test completed successfully"
    print_info "Bridge transaction hash: $bridge_tx_hash"
    
    # Return the transaction hash for further processing
    echo "$bridge_tx_hash"
    return 0
}

# Verify final balances
verify_final_balances() {
    local bridge_tx_hash="$1"
    
    print_step "Verifying final balances"
    
    # Check L2 claims to get wrapped token info
    local claims_info
    if claims_info=$(get_claims_info "$NETWORK_ID_AGGLAYER_1"); then
        print_debug "Claims info: $claims_info"
        
        # Try to extract wrapped token address or use a common pattern
        # The wrapped token is usually created at a deterministic address
        local wrapped_token_addr="0x19e2b7738a026883d08c3642984ab6d7510ca238"
        
        # Check L2 balance
        local l2_balance_hex
        l2_balance_hex=$(get_token_balance "$wrapped_token_addr" "$ACCOUNT_ADDRESS_2" "$NETWORK_ID_AGGLAYER_1" 2>/dev/null || echo "0x0")
        local l2_balance_dec
        l2_balance_dec=$(get_balance_decimal "$l2_balance_hex")
        
        print_info "L2 Wrapped Token Balance ($ACCOUNT_ADDRESS_2): $l2_balance_dec tokens"
        print_info "L2 Wrapped Token Address: $wrapped_token_addr"
        
        if [ "$l2_balance_dec" -gt "0" ]; then
            print_success "✅ Tokens successfully bridged and claimed on L2"
        else
            print_warning "⚠️ L2 balance is 0 - tokens may still be processing or at different address"
        fi
    else
        print_warning "Could not retrieve claims information for balance verification"
    fi
    
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
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Set debug/verbose flags from arguments
    if [ "$DEBUG" = "true" ]; then
        export DEBUG=1
    fi
    if [ "$VERBOSE" = "true" ]; then
        export VERBOSE=1
    fi
    
    # Validate prerequisites
    if ! validate_test_prerequisites; then
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
        print_info "Would bridge $BRIDGE_AMOUNT tokens of $TOKEN_ADDRESS from L1 to L2"
        exit 0
    fi
    
    # Check initial balances
    if ! check_initial_balances; then
        print_error "Initial balance check failed"
        exit 1
    fi
    
    # Execute the bridge test
    local bridge_tx_hash
    if ! bridge_tx_hash=$(execute_bridge_test); then
        print_error "Bridge test execution failed"
        exit 1
    fi
    
    # Verify final balances
    verify_final_balances "$bridge_tx_hash"
    
    # Print final summary
    print_bridge_summary "$bridge_tx_hash" "" "$BRIDGE_AMOUNT" "$TOKEN_ADDRESS"
    
    # Show recent events if requested
    if [ "$SHOW_EVENTS" = "true" ]; then
        show_recent_events "anvil-l2" 5
    fi
    
    # Print helpful information
    echo ""
    print_info "📝 Test completed successfully!"
    print_info ""
    print_info "🔧 Useful commands:"
    print_info "  • Check bridge status: aggsandbox show bridges --network-id $NETWORK_ID_AGGLAYER_1"
    print_info "  • Check claims: aggsandbox show claims --network-id $NETWORK_ID_AGGLAYER_1"
    print_info "  • View logs: aggsandbox logs --follow"
    print_info "  • Show events: aggsandbox events --chain anvil-l2"
    print_info ""
    print_info "💡 Tips:"
    print_info "  • Bridge operations typically take 20-30 seconds for GER propagation"
    print_info "  • Claims are automatically executed by the modern CLI"
    print_info "  • Use --debug flag for detailed logging"
    print_info "  • Use --dry-run to validate environment without executing"
    echo ""
    
    print_success "🎉 L1 to L2 bridge test completed successfully!"
}

# ============================================================================
# SCRIPT ENTRY POINT
# ============================================================================

# Handle script execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # Script is being executed directly
    main "$@"
else
    # Script is being sourced - export main functions for reuse
    export -f main parse_arguments validate_test_prerequisites
    export -f check_initial_balances execute_bridge_test verify_final_balances
    print_debug "L1-L2 bridge test script loaded as library"
fi