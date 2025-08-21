#!/bin/bash
# Bridge Test Runner
# Demonstrates how to use the modular bridge test scripts
# Version: 2.0 - Updated for modern aggsandbox CLI

set -e  # Exit on error

# ============================================================================
# CONFIGURATION
# ============================================================================

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Load the shared bridge test library
LIB_PATH="$SCRIPT_DIR/lib/bridge_test_lib.sh"
if [ -f "$LIB_PATH" ]; then
    source "$LIB_PATH"
else
    echo "Error: Bridge test library not found at $LIB_PATH"
    echo "Please ensure the test library exists and is accessible."
    exit 1
fi

# Test script paths
L1_L2_SCRIPT="$SCRIPT_DIR/L1-L2/bridge-asset-l1-to-l2-success.sh"
L2_L1_SCRIPT="$SCRIPT_DIR/L2-L1/bridge-asset-l2-to-l1-success.sh"

# Default configuration
RUN_L1_L2=true
RUN_L2_L1=true
BRIDGE_AMOUNT=50
VERBOSE=false
DEBUG=false
DRY_RUN=false

# ============================================================================
# COMMAND LINE PARSING
# ============================================================================

print_usage() {
    cat << EOF
Bridge Test Runner v2.0
Run comprehensive bridge tests using the modern aggsandbox CLI

Usage: $0 [OPTIONS]

Options:
  --l1-l2-only          Run only L1 to L2 bridge test
  --l2-l1-only          Run only L2 to L1 bridge test
  --amount AMOUNT       Amount to bridge in each test (default: $BRIDGE_AMOUNT)
  --verbose             Enable verbose output
  --debug               Enable debug mode with detailed logging
  --dry-run             Validate environment without executing transactions
  --help, -h            Show this help message

Examples:
  $0                    # Run both L1->L2 and L2->L1 tests with default amount
  $0 --amount 100       # Run both tests with 100 tokens each
  $0 --l1-l2-only       # Run only L1 to L2 test
  $0 --verbose --debug  # Run with maximum logging
  $0 --dry-run          # Check environment without executing

Environment:
  Ensure aggsandbox is running: aggsandbox start --detach
  Contracts should be deployed and .env file should be present.

EOF
}

parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --l1-l2-only)
                RUN_L1_L2=true
                RUN_L2_L1=false
                shift
                ;;
            --l2-l1-only)
                RUN_L1_L2=false
                RUN_L2_L1=true
                shift
                ;;
            --amount)
                BRIDGE_AMOUNT="$2"
                shift 2
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
                print_error "Invalid argument: $1"
                print_usage
                exit 1
                ;;
        esac
    done
}

# ============================================================================
# TEST EXECUTION FUNCTIONS
# ============================================================================

# Validate test environment
validate_test_environment() {
    print_step "Validating test environment"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Initialize environment
    if ! init_bridge_test_environment; then
        return 1
    fi
    
    # Check test scripts exist
    if [ "$RUN_L1_L2" = "true" ] && [ ! -f "$L1_L2_SCRIPT" ]; then
        print_error "L1-L2 test script not found: $L1_L2_SCRIPT"
        return 1
    fi
    
    if [ "$RUN_L2_L1" = "true" ] && [ ! -f "$L2_L1_SCRIPT" ]; then
        print_error "L2-L1 test script not found: $L2_L1_SCRIPT"
        return 1
    fi
    
    # Check scripts are executable
    if [ "$RUN_L1_L2" = "true" ] && [ ! -x "$L1_L2_SCRIPT" ]; then
        print_warning "Making L1-L2 script executable"
        chmod +x "$L1_L2_SCRIPT"
    fi
    
    if [ "$RUN_L2_L1" = "true" ] && [ ! -x "$L2_L1_SCRIPT" ]; then
        print_warning "Making L2-L1 script executable"
        chmod +x "$L2_L1_SCRIPT"
    fi
    
    print_success "Test environment validated"
    return 0
}

# Run L1 to L2 bridge test
run_l1_l2_test() {
    print_step "Running L1 to L2 bridge test"
    
    local cmd_args=""
    if [ "$VERBOSE" = "true" ]; then
        cmd_args="$cmd_args --verbose"
    fi
    if [ "$DEBUG" = "true" ]; then
        cmd_args="$cmd_args --debug"
    fi
    if [ "$DRY_RUN" = "true" ]; then
        cmd_args="$cmd_args --dry-run"
    fi
    
    print_info "Executing: $L1_L2_SCRIPT $cmd_args $BRIDGE_AMOUNT"
    
    if "$L1_L2_SCRIPT" $cmd_args "$BRIDGE_AMOUNT"; then
        print_success "✅ L1 to L2 bridge test completed successfully"
        return 0
    else
        print_error "❌ L1 to L2 bridge test failed"
        return 1
    fi
}

# Run L2 to L1 bridge test
run_l2_l1_test() {
    print_step "Running L2 to L1 bridge test"
    
    local cmd_args=""
    if [ "$VERBOSE" = "true" ]; then
        cmd_args="$cmd_args --verbose"
    fi
    if [ "$DEBUG" = "true" ]; then
        cmd_args="$cmd_args --debug"
    fi
    if [ "$DRY_RUN" = "true" ]; then
        cmd_args="$cmd_args --dry-run"
    fi
    
    # For L2->L1, use a smaller amount since we're bridging back
    local l2_amount=$((BRIDGE_AMOUNT / 2))
    if [ $l2_amount -lt 1 ]; then
        l2_amount=1
    fi
    
    print_info "Executing: $L2_L1_SCRIPT $cmd_args $l2_amount"
    print_info "Note: Using smaller amount ($l2_amount) for L2->L1 to ensure sufficient L2 balance"
    
    if "$L2_L1_SCRIPT" $cmd_args "$l2_amount"; then
        print_success "✅ L2 to L1 bridge test completed successfully"
        return 0
    else
        print_error "❌ L2 to L1 bridge test failed"
        return 1
    fi
}

# Print test summary
print_test_summary() {
    local l1_l2_result="$1"
    local l2_l1_result="$2"
    
    echo ""
    print_info "==================== TEST SUMMARY ===================="
    
    if [ "$RUN_L1_L2" = "true" ]; then
        if [ "$l1_l2_result" = "0" ]; then
            print_success "✅ L1 to L2 Bridge Test: PASSED"
        else
            print_error "❌ L1 to L2 Bridge Test: FAILED"
        fi
    fi
    
    if [ "$RUN_L2_L1" = "true" ]; then
        if [ "$l2_l1_result" = "0" ]; then
            print_success "✅ L2 to L1 Bridge Test: PASSED"
        else
            print_error "❌ L2 to L1 Bridge Test: FAILED"
        fi
    fi
    
    # Overall result
    local overall_success=true
    if [ "$RUN_L1_L2" = "true" ] && [ "$l1_l2_result" != "0" ]; then
        overall_success=false
    fi
    if [ "$RUN_L2_L1" = "true" ] && [ "$l2_l1_result" != "0" ]; then
        overall_success=false
    fi
    
    echo ""
    if [ "$overall_success" = "true" ]; then
        print_success "🎉 ALL BRIDGE TESTS PASSED!"
    else
        print_error "❌ SOME BRIDGE TESTS FAILED"
    fi
    
    print_info "======================================================="
    echo ""
}

# ============================================================================
# MAIN EXECUTION FLOW
# ============================================================================

main() {
    # Print script header
    echo ""
    print_success "==================== BRIDGE TEST RUNNER v2.0 ===================="
    print_info "Comprehensive bridge testing using modern aggsandbox CLI"
    echo ""
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Set debug/verbose flags
    if [ "$DEBUG" = "true" ]; then
        export DEBUG=1
    fi
    if [ "$VERBOSE" = "true" ]; then
        export VERBOSE=1
    fi
    
    # Validate environment
    if ! validate_test_environment; then
        print_error "Environment validation failed"
        exit 1
    fi
    
    # Print test configuration
    print_info "Test Configuration:"
    print_info "  • L1 to L2 Test: $([ "$RUN_L1_L2" = "true" ] && echo "✅ Enabled" || echo "❌ Disabled")"
    print_info "  • L2 to L1 Test: $([ "$RUN_L2_L1" = "true" ] && echo "✅ Enabled" || echo "❌ Disabled")"
    print_info "  • Bridge Amount: $BRIDGE_AMOUNT tokens"
    print_info "  • Verbose Mode: $([ "$VERBOSE" = "true" ] && echo "✅ Enabled" || echo "❌ Disabled")"
    print_info "  • Debug Mode: $([ "$DEBUG" = "true" ] && echo "✅ Enabled" || echo "❌ Disabled")"
    print_info "  • Dry Run: $([ "$DRY_RUN" = "true" ] && echo "✅ Enabled" || echo "❌ Disabled")"
    echo ""
    
    # Check for dry run
    if [ "$DRY_RUN" = "true" ]; then
        print_success "✅ Dry run completed - environment is properly configured"
        print_info "Test scripts are ready to execute"
        exit 0
    fi
    
    # Execute tests
    local l1_l2_result=0
    local l2_l1_result=0
    
    # Run L1 to L2 test
    if [ "$RUN_L1_L2" = "true" ]; then
        echo ""
        print_info "🚀 Starting L1 to L2 bridge test..."
        echo ""
        
        if run_l1_l2_test; then
            l1_l2_result=0
        else
            l1_l2_result=1
            # Don't exit immediately - we might still want to run other tests
        fi
        
        echo ""
        print_info "⏳ Waiting 5 seconds before next test..."
        sleep 5
    fi
    
    # Run L2 to L1 test (only if L1->L2 succeeded or if running standalone)
    if [ "$RUN_L2_L1" = "true" ]; then
        if [ "$RUN_L1_L2" = "false" ] || [ "$l1_l2_result" = "0" ]; then
            echo ""
            print_info "🚀 Starting L2 to L1 bridge test..."
            echo ""
            
            if run_l2_l1_test; then
                l2_l1_result=0
            else
                l2_l1_result=1
            fi
        else
            print_warning "⚠️ Skipping L2 to L1 test because L1 to L2 test failed"
            l2_l1_result=1
        fi
    fi
    
    # Print final summary
    print_test_summary "$l1_l2_result" "$l2_l1_result"
    
    # Provide useful information
    print_info "🔧 Useful commands after testing:"
    print_info "  • Check sandbox status: aggsandbox status"
    print_info "  • View all bridges: aggsandbox show bridges --network-id 0 && aggsandbox show bridges --network-id 1"
    print_info "  • View all claims: aggsandbox show claims --network-id 0 && aggsandbox show claims --network-id 1"
    print_info "  • View logs: aggsandbox logs --follow"
    print_info "  • Stop sandbox: aggsandbox stop"
    echo ""
    
    # Exit with appropriate code
    if [ "$l1_l2_result" = "0" ] && [ "$l2_l1_result" = "0" ]; then
        exit 0
    else
        exit 1
    fi
}

# Handle script execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
else
    export -f main parse_arguments validate_test_environment
    export -f run_l1_l2_test run_l2_l1_test print_test_summary
    print_debug "Bridge test runner loaded as library"
fi 