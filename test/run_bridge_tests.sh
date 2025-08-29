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

# Test directory paths
L1_L2_DIR="$SCRIPT_DIR/L1-L2"
L2_L1_DIR="$SCRIPT_DIR/L2-L1"
L2_L2_DIR="$SCRIPT_DIR/L2-L2"

# Default configuration
RUN_L1_L2=true
RUN_L2_L1=true
RUN_L2_L2=true
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
  --l1-l2-only          Run only L1 to L2 bridge tests
  --l2-l1-only          Run only L2 to L1 bridge tests
  --l2-l2-only          Run only L2 to L2 bridge tests
  --amount AMOUNT       Amount to bridge in each test (default: $BRIDGE_AMOUNT)
  --verbose             Enable verbose output
  --debug               Enable debug mode with detailed logging
  --dry-run             Validate environment without executing transactions
  --help, -h            Show this help message

Examples:
  $0                    # Run all L1->L2, L2->L1, and L2->L2 tests with default amount
  $0 --amount 100       # Run all tests with 100 tokens each
  $0 --l1-l2-only       # Run only L1 to L2 tests
  $0 --l2-l2-only       # Run only L2 to L2 tests
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
                RUN_L2_L2=false
                shift
                ;;
            --l2-l1-only)
                RUN_L1_L2=false
                RUN_L2_L1=true
                RUN_L2_L2=false
                shift
                ;;
            --l2-l2-only)
                RUN_L1_L2=false
                RUN_L2_L1=false
                RUN_L2_L2=true
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

# Get all Python test files in a directory
get_test_files() {
    local test_dir="$1"
    find "$test_dir" -name "test_*.py" -type f | sort
}

# Validate test environment
validate_test_environment() {
    print_step "Validating test environment"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Initialize environment
    if ! init_bridge_test_environment; then
        return 1
    fi
    
    # Check Python is available
    if ! command -v python3 &> /dev/null; then
        print_error "python3 is required but not available"
        return 1
    fi
    
    # Check test directories exist
    if [ "$RUN_L1_L2" = "true" ] && [ ! -d "$L1_L2_DIR" ]; then
        print_error "L1-L2 test directory not found: $L1_L2_DIR"
        return 1
    fi
    
    if [ "$RUN_L2_L1" = "true" ] && [ ! -d "$L2_L1_DIR" ]; then
        print_error "L2-L1 test directory not found: $L2_L1_DIR"
        return 1
    fi
    
    if [ "$RUN_L2_L2" = "true" ] && [ ! -d "$L2_L2_DIR" ]; then
        print_error "L2-L2 test directory not found: $L2_L2_DIR"
        return 1
    fi
    
    # Check test files exist
    if [ "$RUN_L1_L2" = "true" ]; then
        local l1_l2_tests=$(get_test_files "$L1_L2_DIR")
        if [ -z "$l1_l2_tests" ]; then
            print_error "No test_*.py files found in $L1_L2_DIR"
            return 1
        fi
    fi
    
    if [ "$RUN_L2_L1" = "true" ]; then
        local l2_l1_tests=$(get_test_files "$L2_L1_DIR")
        if [ -z "$l2_l1_tests" ]; then
            print_error "No test_*.py files found in $L2_L1_DIR"
            return 1
        fi
    fi
    
    if [ "$RUN_L2_L2" = "true" ]; then
        local l2_l2_tests=$(get_test_files "$L2_L2_DIR")
        if [ -z "$l2_l2_tests" ]; then
            print_error "No test_*.py files found in $L2_L2_DIR"
            return 1
        fi
    fi
    
    print_success "Test environment validated"
    return 0
}

# Run Python test file
run_python_test() {
    local test_file="$1"
    local amount="$2"
    local test_name=$(basename "$test_file" .py)
    
    print_info "Running: python3 $test_file $amount"
    
    if python3 "$test_file" "$amount"; then
        print_success "✅ $test_name completed successfully"
        return 0
    else
        print_error "❌ $test_name failed"
        return 1
    fi
}

# Run all tests in a directory
run_tests_in_directory() {
    local test_dir="$1"
    local test_type="$2"
    local amount="$3"
    local failed_tests=0
    local total_tests=0
    
    print_step "Running $test_type bridge tests"
    
    local test_files=$(get_test_files "$test_dir")
    
    if [ -z "$test_files" ]; then
        print_warning "No test files found in $test_dir"
        return 0
    fi
    
    # Count tests
    total_tests=$(echo "$test_files" | wc -l)
    print_info "Found $total_tests test(s) in $test_dir"
    echo
    
    # Run each test
    while IFS= read -r test_file; do
        if [ -n "$test_file" ]; then
            print_info "📋 Test: $(basename "$test_file")"
            
            if [ "$DRY_RUN" = "true" ]; then
                print_info "DRY RUN: Would execute python3 $test_file $amount"
                continue
            fi
            
            if run_python_test "$test_file" "$amount"; then
                echo
            else
                failed_tests=$((failed_tests + 1))
                echo
            fi
            
            # Add delay between tests
            if [ $total_tests -gt 1 ]; then
                print_info "⏳ Waiting 3 seconds before next test..."
                sleep 3
                echo
            fi
        fi
    done <<< "$test_files"
    
    # Summary for this test type
    local passed_tests=$((total_tests - failed_tests))
    if [ $failed_tests -eq 0 ]; then
        print_success "✅ All $total_tests $test_type tests passed!"
    else
        print_error "❌ $failed_tests out of $total_tests $test_type tests failed"
        print_success "✅ $passed_tests out of $total_tests $test_type tests passed"
    fi
    
    return $failed_tests
}

# Run L1 to L2 bridge tests
run_l1_l2_tests() {
    run_tests_in_directory "$L1_L2_DIR" "L1→L2" "$BRIDGE_AMOUNT"
}

# Run L2 to L1 bridge tests
run_l2_l1_tests() {
    # For L2->L1, use a smaller amount since we're bridging back
    local l2_amount=$((BRIDGE_AMOUNT / 2))
    if [ $l2_amount -lt 1 ]; then
        l2_amount=1
    fi
    
    print_info "Note: Using smaller amount ($l2_amount) for L2→L1 to ensure sufficient L2 balance"
    run_tests_in_directory "$L2_L1_DIR" "L2→L1" "$l2_amount"
}

# Run L2 to L2 bridge tests
run_l2_l2_tests() {
    run_tests_in_directory "$L2_L2_DIR" "L2→L2" "$BRIDGE_AMOUNT"
}

# Print test summary
print_test_summary() {
    local l1_l2_result="$1"
    local l2_l1_result="$2"
    local l2_l2_result="$3"
    
    echo ""
    print_info "==================== TEST SUMMARY ===================="
    
    if [ "$RUN_L1_L2" = "true" ]; then
        if [ "$l1_l2_result" = "0" ]; then
            print_success "✅ L1 to L2 Bridge Tests: PASSED"
        else
            print_error "❌ L1 to L2 Bridge Tests: FAILED ($l1_l2_result test(s) failed)"
        fi
    fi
    
    if [ "$RUN_L2_L1" = "true" ]; then
        if [ "$l2_l1_result" = "0" ]; then
            print_success "✅ L2 to L1 Bridge Tests: PASSED"
        else
            print_error "❌ L2 to L1 Bridge Tests: FAILED ($l2_l1_result test(s) failed)"
        fi
    fi
    
    if [ "$RUN_L2_L2" = "true" ]; then
        if [ "$l2_l2_result" = "0" ]; then
            print_success "✅ L2 to L2 Bridge Tests: PASSED"
        else
            print_error "❌ L2 to L2 Bridge Tests: FAILED ($l2_l2_result test(s) failed)"
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
    if [ "$RUN_L2_L2" = "true" ] && [ "$l2_l2_result" != "0" ]; then
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
    print_info "  • L1 to L2 Tests: $([ "$RUN_L1_L2" = "true" ] && echo "✅ Enabled" || echo "❌ Disabled")"
    print_info "  • L2 to L1 Tests: $([ "$RUN_L2_L1" = "true" ] && echo "✅ Enabled" || echo "❌ Disabled")"
    print_info "  • L2 to L2 Tests: $([ "$RUN_L2_L2" = "true" ] && echo "✅ Enabled" || echo "❌ Disabled")"
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
    local l2_l2_result=0
    
    # Run L1 to L2 tests
    if [ "$RUN_L1_L2" = "true" ]; then
        echo ""
        print_info "🚀 Starting L1 to L2 bridge tests..."
        echo ""
        
        run_l1_l2_tests
        l1_l2_result=$?
        
        echo ""
        print_info "⏳ Waiting 5 seconds before next test suite..."
        sleep 5
    fi
    
    # Run L2 to L1 tests (only if L1->L2 succeeded or if running standalone)
    if [ "$RUN_L2_L1" = "true" ]; then
        if [ "$RUN_L1_L2" = "false" ] || [ "$l1_l2_result" = "0" ]; then
            echo ""
            print_info "🚀 Starting L2 to L1 bridge tests..."
            echo ""
            
            run_l2_l1_tests
            l2_l1_result=$?
            
            echo ""
            print_info "⏳ Waiting 5 seconds before next test suite..."
            sleep 5
        else
            print_warning "⚠️ Skipping L2 to L1 tests because L1 to L2 tests failed"
            l2_l1_result=1
        fi
    fi
    
    # Run L2 to L2 tests
    if [ "$RUN_L2_L2" = "true" ]; then
        echo ""
        print_info "🚀 Starting L2 to L2 bridge tests..."
        echo ""
        
        run_l2_l2_tests
        l2_l2_result=$?
    fi
    
    # Print final summary
    print_test_summary "$l1_l2_result" "$l2_l1_result" "$l2_l2_result"
    
    # Provide useful information
    print_info "🔧 Useful commands after testing:"
    print_info "  • Check sandbox status: aggsandbox status"
    print_info "  • View all bridges: aggsandbox show bridges --network-id 0 && aggsandbox show bridges --network-id 1"
    print_info "  • View all claims: aggsandbox show claims --network-id 0 && aggsandbox show claims --network-id 1"
    print_info "  • View logs: aggsandbox logs --follow"
    print_info "  • Stop sandbox: aggsandbox stop"
    echo ""
    
    # Exit with appropriate code
    if [ "$l1_l2_result" = "0" ] && [ "$l2_l1_result" = "0" ] && [ "$l2_l2_result" = "0" ]; then
        exit 0
    else
        exit 1
    fi
}

# Handle script execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
else
    export -f main parse_arguments validate_test_environment get_test_files
    export -f run_python_test run_tests_in_directory 
    export -f run_l1_l2_tests run_l2_l1_tests run_l2_l2_tests print_test_summary
    print_debug "Bridge test runner loaded as library"
fi 