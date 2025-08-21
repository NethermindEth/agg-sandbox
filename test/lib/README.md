# Bridge Test Library - Modular Architecture

This directory contains the modular bridge test library for the Agglayer Nether Sandbox. The library is structured as separate modules for each type of bridge operation, making it easier to maintain and understand.

## Architecture Overview

```
test/lib/
├── bridge_test_lib.sh          # Main index file that loads all modules
├── bridge_asset.sh             # Asset bridging operations
├── bridge_message.sh           # Message bridging operations
├── claim_asset.sh              # Asset claiming operations
├── claim_message.sh            # Message claiming operations
├── bridge_and_call.sh          # Bridge and call operations
├── claim_bridge_and_call.sh    # Bridge and call claiming operations
└── README.md                   # This file
```

## Main Library (bridge_test_lib.sh)

The main library acts as an index that:
- Loads all individual modules
- Provides core utility functions (logging, environment validation, etc.)
- Maintains backward compatibility with legacy functions
- Exports all functions for use in test scripts

### Usage

```bash
# Source the main library in your test script
source "test/lib/bridge_test_lib.sh"

# All module functions are now available
bridge_asset_modern 0 1 100 $TOKEN_ADDRESS $DEST_ADDRESS $PRIVATE_KEY
```

## Module Breakdown

### 1. Bridge Asset Module (`bridge_asset.sh`)

Handles asset bridging operations using the modern aggsandbox CLI.

**Key Functions:**
- `bridge_asset_modern()` - Bridge assets between networks
- `bridge_asset_with_approval()` - Bridge with automatic token approval
- `execute_l1_to_l2_asset_bridge()` - Complete L1→L2 asset flow
- `execute_l2_to_l1_asset_bridge()` - Complete L2→L1 asset flow

**Example:**
```bash
# Bridge 100 tokens from L1 to L2
bridge_tx_hash=$(bridge_asset_modern 0 1 100 $TOKEN_ADDRESS $DEST_ADDRESS $PRIVATE_KEY)
```

### 2. Bridge Message Module (`bridge_message.sh`)

Handles message bridging operations.

**Key Functions:**
- `bridge_message_modern()` - Bridge arbitrary message data
- `bridge_text_message()` - Bridge simple text messages
- `bridge_function_call_message()` - Bridge encoded function calls
- `execute_l1_to_l2_message_bridge()` - Complete L1→L2 message flow
- `execute_l2_to_l1_message_bridge()` - Complete L2→L1 message flow

**Example:**
```bash
# Bridge a text message
bridge_tx_hash=$(bridge_text_message 0 1 $TARGET_CONTRACT "Hello World!" $PRIVATE_KEY)
```

### 3. Claim Asset Module (`claim_asset.sh`)

Handles claiming of bridged assets.

**Key Functions:**
- `claim_asset_modern()` - Claim bridged assets
- `claim_asset_with_retry()` - Claim with retry logic
- `get_claim_proof()` - Get proof data for manual claiming
- `is_asset_claimed()` - Check if asset is already claimed
- `wait_for_asset_claimable()` - Wait for GER propagation
- `verify_claim_transaction()` - Verify claim was successful

**Example:**
```bash
# Claim bridged assets with retry
claim_tx_hash=$(claim_asset_with_retry 1 $BRIDGE_TX_HASH 0 "" $PRIVATE_KEY)
```

### 4. Claim Message Module (`claim_message.sh`)

Handles claiming of bridged messages.

**Key Functions:**
- `claim_message_modern()` - Claim bridged messages
- `claim_message_with_retry()` - Claim with retry logic
- `is_message_claimed()` - Check if message is already claimed
- `get_message_proof()` - Get proof data for manual claiming
- `wait_for_message_claimable()` - Wait for GER propagation
- `verify_message_claim_transaction()` - Verify claim and execution
- `decode_message_data()` - Decode message data for debugging

**Example:**
```bash
# Claim bridged message
claim_tx_hash=$(claim_message_modern 1 $BRIDGE_TX_HASH 0 "" $PRIVATE_KEY)
```

### 5. Bridge and Call Module (`bridge_and_call.sh`)

Handles bridge and call operations (atomic token bridge + function execution).

**Key Functions:**
- `bridge_and_call_modern()` - Execute bridge and call
- `bridge_and_call_function()` - Bridge and call with function encoding
- `deploy_bridge_call_receiver()` - Deploy test receiver contracts
- `execute_l1_to_l2_bridge_and_call()` - Complete L1→L2 bridge and call flow
- `execute_l2_to_l1_bridge_and_call()` - Complete L2→L1 bridge and call flow
- `verify_bridge_and_call_execution()` - Verify call was executed

**Example:**
```bash
# Deploy receiver and execute bridge and call
receiver=$(deploy_bridge_call_receiver 1 $PRIVATE_KEY)
call_data=$(cast abi-encode "receiveTokens(address,uint256,string)" $TOKEN $AMOUNT "Hello")
bridge_tx_hash=$(bridge_and_call_modern 0 1 100 $TOKEN $receiver $call_data $PRIVATE_KEY)
```

### 6. Claim Bridge and Call Module (`claim_bridge_and_call.sh`)

Handles claiming of bridge and call transactions.

**Key Functions:**
- `claim_bridge_and_call_modern()` - Claim bridge and call
- `claim_bridge_and_call_with_retry()` - Claim with retry logic
- `get_bridge_and_call_info()` - Get bridge and call information
- `wait_for_bridge_and_call_claimable()` - Wait for both asset and message indexing
- `verify_bridge_and_call_claim()` - Verify claim and call execution
- `extract_bridge_and_call_details()` - Extract event details

**Example:**
```bash
# Claim bridge and call transaction
claim_tx_hash=$(claim_bridge_and_call_modern 1 $BRIDGE_TX_HASH 0 "" $PRIVATE_KEY)
```

## Common Patterns

### Complete Bridge Flow

```bash
# L1 to L2 Asset Bridge
source "test/lib/bridge_test_lib.sh"

# Initialize environment
init_bridge_test_environment

# Execute complete flow
bridge_tx_hash=$(execute_l1_to_l2_asset_bridge 100 $TOKEN_ADDRESS $SRC_ADDR $DEST_ADDR $SRC_KEY $DEST_KEY)
```

### Bridge and Call Flow

```bash
# Deploy receiver contract
receiver=$(deploy_bridge_call_receiver 1 $PRIVATE_KEY)

# Prepare call data
call_data=$(cast abi-encode "receiveTokensWithMessage(address,uint256,string)" $TOKEN 10 "Hello")

# Execute bridge and call
bridge_tx_hash=$(execute_l1_to_l2_bridge_and_call 100 $TOKEN $receiver $call_data)

# Verify execution
verify_bridge_and_call_execution $receiver 1 "Hello" 10
```

### Error Handling

All functions return appropriate exit codes:
- `0` - Success
- `1` - General failure
- `2` - Already claimed/processed
- `3` - Global Exit Root invalid (retry recommended)
- `4` - Dependency not met (e.g., unclaimed asset)

### Debugging

Enable debug mode for detailed logging:
```bash
export DEBUG=1
source "test/lib/bridge_test_lib.sh"
```

## Migration from Legacy Library

The modular library maintains backward compatibility:

```bash
# Old way (still works)
execute_l1_to_l2_bridge 100 $TOKEN $SRC $DEST $SRC_KEY $DEST_KEY

# New way (recommended)
execute_l1_to_l2_asset_bridge 100 $TOKEN $SRC $DEST $SRC_KEY $DEST_KEY
```

## Best Practices

1. **Always source the main library**: Use `bridge_test_lib.sh` as your entry point
2. **Use specific functions**: Choose the most appropriate function for your use case
3. **Handle errors**: Check return codes and handle common error patterns
4. **Enable debugging**: Use `DEBUG=1` for troubleshooting
5. **Wait for indexing**: Use the provided waiting functions for proper timing
6. **Verify results**: Use verification functions to ensure operations completed

## Environment Requirements

- `aggsandbox` CLI installed and running
- `cast` (Foundry) for blockchain interactions
- `jq` for JSON parsing
- Required environment variables set (see main library for details)

## Contributing

When adding new functionality:
1. Choose the appropriate module or create a new one
2. Follow the existing function naming conventions
3. Add proper error handling and debugging
4. Export new functions in the module
5. Update this README with examples
