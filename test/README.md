# Bridge Test Suite v2.0

Modern, modular bridge testing suite for the aggsandbox project, updated to use the latest aggsandbox CLI commands.

## 🚀 Quick Start

```bash
# Ensure aggsandbox is running
aggsandbox start --detach

# Run all bridge tests
./test/run_bridge_tests.sh

# Run specific tests
./test/run_bridge_tests.sh --l1-l2-only    # Only L1→L2 test
./test/run_bridge_tests.sh --l2-l1-only    # Only L2→L1 test

# Run with custom amount
./test/run_bridge_tests.sh --amount 100

# Dry run to check environment
./test/run_bridge_tests.sh --dry-run
```

## 📁 Project Structure

```
test/
├── lib/
│   └── bridge_test_lib.sh          # Shared library functions
├── L1-L2/
│   └── bridge-asset-l1-to-l2-success.sh   # L1→L2 bridge test
├── L2-L1/
│   └── bridge-asset-l2-to-l1-success.sh   # L2→L1 bridge test
├── run_bridge_tests.sh             # Test runner
└── README.md                       # This file
```

## 🔧 Individual Test Scripts

### L1 to L2 Bridge Test

Test bridging ERC20 tokens from L1 to L2:

```bash
# Basic usage
./test/L1-L2/bridge-asset-l1-to-l2-success.sh

# With options
./test/L1-L2/bridge-asset-l1-to-l2-success.sh --verbose --amount 50

# Help
./test/L1-L2/bridge-asset-l1-to-l2-success.sh --help
```

**Options:**
- `--token-address ADDR` - Token contract address (default: AGG_ERC20_L1)
- `--show-events` - Show blockchain events after test
- `--verbose` - Enable verbose output
- `--debug` - Enable debug logging
- `--dry-run` - Validate environment without executing
- `--help` - Show help message

### L2 to L1 Bridge Test

Test bridging assets back from L2 to L1:

```bash
# Basic usage (run L1→L2 test first to have tokens on L2)
./test/L2-L1/bridge-asset-l2-to-l1-success.sh

# With options
./test/L2-L1/bridge-asset-l2-to-l1-success.sh --verbose --amount 25
```

**Note:** This script bridges tokens FROM L2 TO L1. You should run the L1→L2 test first to ensure you have wrapped tokens on L2.

## 📚 Shared Library

The `bridge_test_lib.sh` provides reusable functions that all test scripts can use:

### Core Functions

- `init_bridge_test_environment()` - Initialize and validate test environment
- `bridge_asset_modern()` - Bridge assets using modern aggsandbox CLI
- `claim_asset_modern()` - Claim assets using modern aggsandbox CLI
- `execute_l1_to_l2_bridge()` - Complete L1→L2 bridge flow
- `wait_for_bridge_indexing()` - Wait for bridge indexing with retry logic

### Utility Functions

- `get_token_balance()` - Get token balance for any network
- `get_balance_decimal()` - Convert hex balance to decimal
- `print_bridge_summary()` - Print formatted test results
- `show_recent_events()` - Display recent blockchain events

### Environment Functions

- `load_environment()` - Load .env file
- `validate_bridge_environment()` - Check required variables
- `validate_sandbox_status()` - Ensure aggsandbox is running

## 🎯 Key Improvements from v1.0

### Modern CLI Integration
- ✅ Uses `aggsandbox bridge asset` instead of direct contract calls
- ✅ Uses `aggsandbox bridge claim` for automatic claiming
- ✅ Uses `aggsandbox show` commands for bridge information
- ✅ Automatic approval handling by the CLI

### Modular Design
- ✅ Shared library for common functions
- ✅ Reusable components across test scripts
- ✅ Easy to extend for new bridge scenarios
- ✅ Consistent error handling and logging

### Environment-Driven Configuration
- ✅ Maximum use of environment variables
- ✅ Configurable defaults that can be overridden
- ✅ Automatic environment validation
- ✅ Better error messages for missing configuration

### Enhanced User Experience
- ✅ Rich command-line interface with help
- ✅ Verbose and debug modes
- ✅ Dry-run capability for validation
- ✅ Progress indicators and clear status messages
- ✅ Comprehensive test summaries

## 🔨 Creating New Test Scripts

To create a new bridge test script using the modular library:

```bash
#!/bin/bash
# New bridge test script

# Load the shared library
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/bridge_test_lib.sh"

# Initialize environment
init_bridge_test_environment

# Use library functions
bridge_tx_hash=$(bridge_asset_modern \
    "$NETWORK_ID_MAINNET" \
    "$NETWORK_ID_AGGLAYER_1" \
    "100" \
    "$AGG_ERC20_L1" \
    "$ACCOUNT_ADDRESS_2" \
    "$PRIVATE_KEY_1")

# Wait for indexing and claim
wait_for_bridge_indexing "$NETWORK_ID_AGGLAYER_1" "$bridge_tx_hash"
claim_asset_modern "$NETWORK_ID_AGGLAYER_1" "$bridge_tx_hash" "$NETWORK_ID_MAINNET" "" "$PRIVATE_KEY_2"

# Print summary
print_bridge_summary "$bridge_tx_hash" "" "100" "$AGG_ERC20_L1"
```

## 🌐 Environment Variables

The test scripts use these environment variables from `.env`:

### Required Variables
- `PRIVATE_KEY_1`, `PRIVATE_KEY_2` - Account private keys
- `ACCOUNT_ADDRESS_1`, `ACCOUNT_ADDRESS_2` - Account addresses
- `RPC_1`, `RPC_2` - RPC endpoints for L1 and L2
- `NETWORK_ID_MAINNET`, `NETWORK_ID_AGGLAYER_1` - Network IDs
- `CHAIN_ID_AGGLAYER_1` - L2 chain ID

### Contract Addresses (Auto-deployed)
- `AGG_ERC20_L1`, `AGG_ERC20_L2` - Test token contracts
- `POLYGON_ZKEVM_BRIDGE_L1`, `POLYGON_ZKEVM_BRIDGE_L2` - Bridge contracts

### Optional Configuration
- `DEFAULT_BRIDGE_AMOUNT` - Default bridge amount (default: 100)
- `DEFAULT_WAIT_TIME` - Wait time for GER propagation (default: 20s)
- `DEFAULT_MAX_RETRIES` - Max retries for bridge indexing (default: 10)
- `DEBUG` - Enable debug mode (set to 1)
- `VERBOSE` - Enable verbose mode (set to 1)

## 🐛 Troubleshooting

### Common Issues

**"aggsandbox CLI not found"**
```bash
# Install the CLI
make install

# Ensure PATH includes ~/.local/bin
export PATH="$HOME/.local/bin:$PATH"
```

**"Sandbox is not running"**
```bash
# Start the sandbox
aggsandbox start --detach

# Check status
aggsandbox status
```

**"Insufficient balance"**
```bash
# Check token balances
aggsandbox show bridges --network-id 0
aggsandbox show claims --network-id 1

# Verify environment variables
source .env && echo "L1 Token: $AGG_ERC20_L1"
```

**"Bridge indexing timeout"**
- Wait longer for global exit root propagation (normal delay: 20-30s)
- Check bridge service logs: `aggsandbox logs aggkit`
- Retry the test after a few minutes

### Debug Mode

Enable debug mode for detailed logging:
```bash
DEBUG=1 ./test/run_bridge_tests.sh --debug
```

This provides:
- Detailed command execution logs
- API response data
- Step-by-step progress information
- Error context and suggestions

## 📊 Test Output

The scripts provide comprehensive output including:

- ✅ **Step-by-step progress** with clear status indicators
- 📊 **Bridge transaction details** (hash, amount, token addresses)
- 🔍 **Balance verification** before and after operations
- 📈 **Test summaries** with pass/fail status
- 🔗 **Useful commands** for further investigation
- ⚠️ **Error details** with troubleshooting suggestions

## 🚀 Next Steps

This modular design makes it easy to:

1. **Add new test scenarios** (L2↔L2, multi-hop bridges, etc.)
2. **Create specialized tests** (stress testing, error scenarios)
3. **Integrate with CI/CD** using the standardized exit codes
4. **Build complex workflows** by combining individual functions
5. **Customize for specific use cases** through environment configuration

The library functions are designed to be stable and reusable, so you can focus on test logic rather than boilerplate code. 