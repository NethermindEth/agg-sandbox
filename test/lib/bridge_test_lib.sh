#!/bin/bash
# Bridge Test Library - Bash Implementation
# Minimal library to support run_bridge_tests.sh
# Version: 1.0 - Basic logging and environment functions

# Colors for output formatting
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    MAGENTA='\033[0;35m'
    CYAN='\033[0;36m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    MAGENTA=''
    CYAN=''
    NC=''
fi

# Logging functions
print_success() {
    echo -e "${GREEN}$1${NC}"
}

print_error() {
    echo -e "${RED}$1${NC}" >&2
}

print_warning() {
    echo -e "${YELLOW}$1${NC}"
}

print_info() {
    echo -e "${BLUE}$1${NC}"
}

print_step() {
    echo -e "${MAGENTA}▶ $1${NC}"
}

print_debug() {
    if [ "${DEBUG}" = "1" ] || [ "${DEBUG}" = "true" ]; then
        echo -e "${CYAN}DEBUG: $1${NC}" >&2
    fi
}

# Initialize bridge test environment
init_bridge_test_environment() {
    print_debug "Initializing bridge test environment"
    
    # Check if .env file exists
    if [ -f ".env" ]; then
        print_debug "Loading .env file"
        set -a  # automatically export all variables
        source .env
        set +a  # stop automatically exporting
    else
        print_warning "No .env file found, using environment variables"
    fi
    
    # Check critical environment variables
    local missing_vars=""
    
    if [ -z "$RPC_URL_1" ]; then
        missing_vars="$missing_vars RPC_URL_1"
    fi
    
    if [ -z "$RPC_URL_2" ]; then
        missing_vars="$missing_vars RPC_URL_2"
    fi
    
    if [ -z "$PRIVATE_KEY_1" ]; then
        missing_vars="$missing_vars PRIVATE_KEY_1"
    fi
    
    if [ -z "$ACCOUNT_ADDRESS_1" ]; then
        missing_vars="$missing_vars ACCOUNT_ADDRESS_1"
    fi
    
    if [ -n "$missing_vars" ]; then
        print_error "Missing required environment variables:$missing_vars"
        print_info "Please ensure aggsandbox is running and .env is properly configured"
        return 1
    fi
    
    # Check if aggsandbox CLI is available
    if ! command -v aggsandbox &> /dev/null; then
        print_error "aggsandbox CLI is not available in PATH"
        print_info "Please ensure aggsandbox is installed and accessible"
        return 1
    fi
    
    print_debug "Bridge test environment initialized successfully"
    return 0
}

# Export functions for use by other scripts
export -f print_success print_error print_warning print_info print_step print_debug
export -f init_bridge_test_environment