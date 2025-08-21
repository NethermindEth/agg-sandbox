# Bridge Test Library - Python Implementation

This directory contains the Python implementation of the modular bridge test library for the Agglayer Nether Sandbox. The Python version provides clean, maintainable code with proper error handling and JSON parsing.

## Architecture Overview

```
test/lib/
├── __init__.py                 # Package initialization and exports
├── bridge_lib.py               # Core utilities and configuration
├── bridge_asset.py             # Asset bridging operations
├── bridge_message.py           # Message bridging operations
├── claim_asset.py              # Asset claiming operations
├── claim_message.py            # Message claiming operations
├── bridge_and_call.py          # Bridge and call operations
├── claim_bridge_and_call.py    # Bridge and call claiming operations
└── README_python.md            # This file
```

## Key Advantages Over Bash

✅ **Clean JSON Parsing** - No more grep/jq complexity  
✅ **Proper Error Handling** - Try/catch blocks instead of bash error codes  
✅ **Type Safety** - Type hints and dataclasses  
✅ **Better Debugging** - Clear error messages and logging  
✅ **Maintainable Code** - Object-oriented structure  
✅ **No Regex Hell** - Simple string methods  

## Quick Start

```python
#!/usr/bin/env python3
import sys
sys.path.append('test/lib')

from bridge_lib import init_bridge_environment, BridgeLogger, BRIDGE_CONFIG
from bridge_asset import BridgeAsset
from claim_asset import ClaimAsset

# Initialize environment
if not init_bridge_environment():
    exit(1)

# Bridge 100 tokens from L1 to L2
bridge_tx, claim_tx = BridgeAsset.execute_l1_to_l2_bridge(
    amount=100,
    token_address=BRIDGE_CONFIG.agg_erc20_l1,
    source_account=BRIDGE_CONFIG.account_address_1,
    dest_account=BRIDGE_CONFIG.account_address_2,
    source_private_key=BRIDGE_CONFIG.private_key_1,
    dest_private_key=BRIDGE_CONFIG.private_key_2
)

BridgeLogger.success(f"Bridge TX: {bridge_tx}")
BridgeLogger.success(f"Claim TX: {claim_tx}")
```

## Module Breakdown

### 1. Core Library (`bridge_lib.py`)

**Key Classes:**
- `BridgeConfig` - Environment configuration dataclass
- `BridgeLogger` - Colored logging with step/info/success/error methods
- `BridgeEnvironment` - Environment loading and validation
- `AggsandboxAPI` - Clean interface to aggsandbox CLI commands
- `BridgeUtils` - Utility functions for common operations

**Example:**
```python
from bridge_lib import BridgeLogger, AggsandboxAPI

# Clean logging
BridgeLogger.step("Starting bridge operation")
BridgeLogger.success("Operation completed!")

# Clean JSON parsing
bridge_data = AggsandboxAPI.get_bridges(network_id=0)
bridges = bridge_data['bridges']
```

### 2. Bridge Asset (`bridge_asset.py`)

**Key Methods:**
- `bridge_asset()` - Bridge assets between networks
- `wait_for_bridge_indexing()` - Wait for bridge to be indexed
- `execute_l1_to_l2_bridge()` - Complete L1→L2 flow
- `execute_l2_to_l1_bridge()` - Complete L2→L1 flow

**Example:**
```python
from bridge_asset import BridgeAsset

# Simple asset bridge
tx_hash = BridgeAsset.bridge_asset(0, 1, 100, token_addr, dest_addr, private_key)

# Complete flow with claiming
bridge_tx, claim_tx = BridgeAsset.execute_l1_to_l2_bridge(
    100, token_addr, src_addr, dest_addr, src_key, dest_key
)
```

### 3. Bridge Message (`bridge_message.py`)

**Key Methods:**
- `bridge_message()` - Bridge arbitrary message data
- `bridge_text_message()` - Bridge simple text messages  
- `bridge_function_call_message()` - Bridge encoded function calls

**Example:**
```python
from bridge_message import BridgeMessage

# Bridge text message
tx_hash = BridgeMessage.bridge_text_message(0, 1, target_addr, "Hello World!", private_key)

# Bridge function call
tx_hash = BridgeMessage.bridge_function_call_message(
    0, 1, target_addr, "transfer(address,uint256)", private_key, dest_addr, 100
)
```

### 4. Claim Asset (`claim_asset.py`)

**Key Methods:**
- `claim_asset()` - Claim bridged assets
- `claim_asset_with_retry()` - Claim with retry logic
- `verify_claim_transaction()` - Verify claim success

**Example:**
```python
from claim_asset import ClaimAsset

# Simple claim
claim_tx = ClaimAsset.claim_asset(1, bridge_tx_hash, 0, private_key)

# Claim with retry
claim_tx = ClaimAsset.claim_asset_with_retry(1, bridge_tx_hash, 0, private_key)
```

### 5. Bridge and Call (`bridge_and_call.py`)

**Key Methods:**
- `bridge_and_call()` - Execute bridge and call
- `bridge_and_call_function()` - Bridge and call with function encoding
- `deploy_bridge_call_receiver()` - Deploy test receiver contracts
- `verify_bridge_and_call_execution()` - Verify call execution

**Example:**
```python
from bridge_and_call import BridgeAndCall

# Deploy receiver
receiver_addr = BridgeAndCall.deploy_bridge_call_receiver(1, private_key)

# Bridge and call
tx_hash = BridgeAndCall.bridge_and_call_function(
    0, 1, 100, token_addr, receiver_addr, 
    "receiveTokens(address,uint256,string)", private_key, 
    None, token_addr, 10, "Hello"
)
```

## Usage Patterns

### Simple Asset Bridge

```python
#!/usr/bin/env python3
import sys
sys.path.append('test/lib')

from bridge_lib import init_bridge_environment, BRIDGE_CONFIG, BridgeLogger
from bridge_asset import BridgeAsset

def main():
    # Initialize
    if not init_bridge_environment():
        return 1
    
    # Bridge assets
    bridge_tx, claim_tx = BridgeAsset.execute_l1_to_l2_bridge(
        amount=50,
        token_address=BRIDGE_CONFIG.agg_erc20_l1,
        source_account=BRIDGE_CONFIG.account_address_1,
        dest_account=BRIDGE_CONFIG.account_address_2,
        source_private_key=BRIDGE_CONFIG.private_key_1,
        dest_private_key=BRIDGE_CONFIG.private_key_2
    )
    
    if bridge_tx and claim_tx:
        BridgeLogger.success("🎉 Bridge and claim completed!")
        BridgeLogger.info(f"Bridge TX: {bridge_tx}")
        BridgeLogger.info(f"Claim TX: {claim_tx}")
        return 0
    else:
        BridgeLogger.error("Bridge or claim failed")
        return 1

if __name__ == "__main__":
    sys.exit(main())
```

### Bridge and Call Flow

```python
#!/usr/bin/env python3
import sys
sys.path.append('test/lib')

from bridge_lib import init_bridge_environment, BRIDGE_CONFIG, BridgeLogger
from bridge_and_call import BridgeAndCall
from claim_bridge_and_call import ClaimBridgeAndCall

def main():
    # Initialize
    if not init_bridge_environment():
        return 1
    
    # Deploy receiver contract
    receiver = BridgeAndCall.deploy_bridge_call_receiver(1, BRIDGE_CONFIG.private_key_1)
    if not receiver:
        return 1
    
    # Execute bridge and call
    bridge_tx = BridgeAndCall.bridge_and_call_function(
        0, 1, 100, BRIDGE_CONFIG.agg_erc20_l1, receiver,
        "receiveTokensWithMessage(address,uint256,string)",
        BRIDGE_CONFIG.private_key_1, None,
        BRIDGE_CONFIG.agg_erc20_l1, 10, "Hello from L1!"
    )
    
    if not bridge_tx:
        return 1
    
    # Claim bridge and call
    claim_tx = ClaimBridgeAndCall.claim_bridge_and_call(
        1, bridge_tx, 0, BRIDGE_CONFIG.private_key_2
    )
    
    if claim_tx:
        # Verify execution
        BridgeAndCall.verify_bridge_and_call_execution(receiver, 1, "Hello from L1!", 10)
        BridgeLogger.success("🎉 Bridge and call completed!")
        return 0
    else:
        BridgeLogger.error("Bridge and call failed")
        return 1

if __name__ == "__main__":
    sys.exit(main())
```

## Error Handling

Python version provides clean error handling:

```python
try:
    bridge_tx = BridgeAsset.bridge_asset(0, 1, 100, token, dest, key)
    if not bridge_tx:
        BridgeLogger.error("Bridge failed")
        return None
    
    claim_tx = ClaimAsset.claim_asset(1, bridge_tx, 0, key)
    if claim_tx == "already_claimed":
        BridgeLogger.info("Asset was already claimed")
    elif not claim_tx:
        BridgeLogger.error("Claim failed")
        return None
    
    return bridge_tx, claim_tx
    
except Exception as e:
    BridgeLogger.error(f"Unexpected error: {e}")
    return None
```

## Benefits Over Bash

| Feature | Bash Version | Python Version |
|---------|-------------|----------------|
| **JSON Parsing** | grep/jq complexity | `json.loads()` |
| **Error Handling** | Exit codes | Try/catch blocks |
| **Data Types** | String manipulation | Proper types |
| **Code Structure** | Functions | Classes and methods |
| **Debugging** | Complex logging | Clean error messages |
| **Maintainability** | Fragile regex | Robust parsing |
| **Testing** | Hard to unit test | Easy to test |

## Environment Variables

Same as bash version - loaded from `.env` file:

```
PRIVATE_KEY_1=0x...
PRIVATE_KEY_2=0x...
ACCOUNT_ADDRESS_1=0x...
ACCOUNT_ADDRESS_2=0x...
RPC_1=http://localhost:8545
RPC_2=http://localhost:8546
# ... etc
```

## Migration from Bash

The Python version maintains the same functionality:

```bash
# Bash
bridge_tx_hash=$(bridge_asset_modern 0 1 100 $TOKEN $DEST $PRIVATE_KEY)
```

```python
# Python
bridge_tx_hash = BridgeAsset.bridge_asset(0, 1, 100, token, dest, private_key)
```

Much cleaner and more reliable! 🐍✨
