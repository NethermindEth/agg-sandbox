# LXLY Bridge Integration

This document describes the LXLY bridge functionality integrated into the aggsandbox CLI, enabling user-friendly cross-chain operations using direct Rust implementation with smart contract interactions.

## Overview

The LXLY bridge integration provides comprehensive cross-chain operations:

- **Bridge Assets**: Transfer ERC20 tokens or ETH between networks
- **Claim Assets**: Claim previously bridged assets on the destination network
- **Bridge Messages**: Bridge with contract calls (bridgeAndCall functionality)
- **Bridge and Call**: Advanced bridgeAndCall with automatic token approval and two-phase claiming using existing claim command
- **Bridge Utilities**: Advanced utilities for bridge operations including payload building, index calculations, and token mapping

## Architecture

The bridge functionality is implemented directly in Rust using the ethers library for smart contract interactions and the existing API client for bridge data:

```text
┌─────────────────┐    ethers.rs     ┌─────────────────────┐    API calls    ┌─────────────────┐
│   Rust CLI      │ ────────────────►│  Smart Contracts    │ ───────────────►│  Bridge API     │
│  (aggsandbox)   │                  │  (Bridge, ERC20)    │                 │  Endpoints      │
└─────────────────┘                  └─────────────────────┘                 └─────────────────┘
```

## Installation

### Prerequisites

- **Rust** >= 1.70.0 (with aggsandbox CLI built)
- **Running aggsandbox environment**

The bridge functionality is built into the CLI - no additional setup required.

## Usage

### Bridge Assets

Transfer ERC20 tokens or ETH between networks:

```bash
# Bridge ETH from L1 to L2
aggsandbox bridge asset \
  --network 0 \
  --destination-network 1 \
  --amount 0.1 \
  --token-address 0x0000000000000000000000000000000000000000

# Bridge ERC20 tokens from L2 to L1
aggsandbox bridge asset \
  --network 0 \
  --destination-network 1 \
  --amount 100 \
  --token-address $AGG_ERC20_L1 \
  --to-address $ACCOUNT_ADDRESS_2
```

**Parameters:**

- `--network, -n`: Source network ID (0=L1, 1=L2, 2=L3)
- `--destination-network, -d`: Destination network ID
- `--amount, -a`: Amount to bridge (in token units)
- `--token-address, -t`: Token contract address (use `0x0000000000000000000000000000000000000000` for ETH)
- `--to-address`: Recipient address (optional, defaults to sender)
- `--gas-limit`: Gas limit override (optional)
- `--gas-price`: Gas price override in wei (optional)

### Claim Assets

Claim assets that were previously bridged:

```bash
# Claim assets on L2 using L1 bridge transaction hash
aggsandbox bridge claim \
  --network 1 \
  --tx-hash 0xb7118cfb20825861028ede1e9586814fc7ccf81745a325db5df355d382d96b4e \
  --source-network 0
```

**Parameters:**

- `--network, -n`: Network to claim assets on
- `--tx-hash, -t`: Original bridge transaction hash
- `--source-network, -s`: Source network of the original bridge
- `--deposit-count, -c`: Deposit count for specific bridge (0=asset, 1=message, auto-detected if not provided)
- `--data`: Custom metadata for message bridge claims (hex encoded, for BridgeExtension messages)
- `--gas-limit`: Gas limit override (optional)
- `--gas-price`: Gas price override in wei (optional)

### Bridge Messages

Bridge with contract calls (bridgeAndCall functionality):

```bash
# Encode the message call data (transfer 1 token to ACCOUNT_ADDRESS_1)
MESSAGE=$(cast calldata "transfer(address,uint256)" $ACCOUNT_ADDRESS_1 1)
```

```bash
# Bridge ETH with contract call
aggsandbox bridge message \
  --network 0 \
  --destination-network 1 \
  --target 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 \
  --data $MESSAGE \
  --amount 0.01 \
  --fallback-address 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

**Parameters:**

- `--network, -n`: Source network ID
- `--destination-network, -d`: Destination network ID
- `--target, -t`: Target contract address on destination network
- `--data`: Contract call data (hex encoded)
- `--amount, -a`: Amount of ETH to send (optional)
- `--fallback-address`: Fallback address if call fails (optional, defaults to sender)
- `--gas-limit`: Gas limit override (optional)
- `--gas-price`: Gas price override in wei (optional)

### Bridge and Call

Execute bridgeAndCall operations with automatic token approval and two-phase claiming:

#### Step 1: Bridge and Call

```bash
source .env
# Encode the transfer call data (transfer 1 token to ACCOUNT_ADDRESS_1)
TRANSFER_DATA=$(cast calldata "transfer(address,uint256)" $ACCOUNT_ADDRESS_1 1)

# Get the precalculated L2 token address
L2_TOKEN_ADDRESS=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
  "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
  1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
  --rpc-url $RPC_2 | sed 's/0x000000000000000000000000/0x/')
```

```bash
# Bridge ERC20 tokens with contract call
aggsandbox bridge bridge-and-call \
  --network 0 \
  --destination-network 1 \
  --token $AGG_ERC20_L1 \
  --amount 10 \
  # TODO: change to destination-address
  --target $L2_TOKEN_ADDRESS \
  --data $TRANSFER_DATA \
  --fallback 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

**Parameters:**

- `--network, -n`: Source network ID (0=L1, 1=L2, etc.)
- `--destination-network, -d`: Destination network ID
- `--token, -t`: Token contract address to bridge
- `--amount, -a`: Amount to bridge (in token units)
- `--target`: Target contract address on destination network
- `--data`: Contract call data (hex encoded)
- `--fallback`: Fallback address if contract call fails
- `--gas-limit`: Gas limit override (optional)
- `--gas-price`: Gas price override in wei (optional)
- `--private-key`: Private key to use (optional)

#### Step 2: Find and Claim Asset Bridge

**⚠️ Important**: The asset bridge MUST be claimed first before the message can be processed.

```bash
# First, find the bridge transactions from bridgeAndCall
aggsandbox show bridges --network-id 0
```

```bash
# Prepare token metadata for asset claim
METADATA=$(cast abi-encode "f(string,string,uint8)" "AggERC20" "AGGERC20" 18)
```

```bash
# Claim the asset bridge first (deposit_count = 0) - REQUIRED FIRST
aggsandbox bridge claim \
  --network 1 \
  --tx-hash <bridge_tx_hash> \
  --source-network 0 \
  --deposit-count 0
```

#### Step 3: Claim Message Bridge

```bash
# Create the metadata for the bridge extension call
METADATA=$(cast abi-encode "f(uint256,address,address,uint32,address,bytes)" \
  0 $L2_TOKEN_ADDRESS $ACCOUNT_ADDRESS_2 1 $AGG_ERC20_L1 $TRANSFER_DATA)
```

```bash
# Claim the message bridge (deposit_count = 1) - MUST be done after asset bridge
aggsandbox bridge claim \
  --network 1 \
  --tx-hash <bridge_tx_hash> \
  --source-network 0 \
  --deposit-count 1 \
  --data $METADATA
```

**Important**: Both bridges share the same `tx_hash` but have different `deposit_count` values:
- Asset bridge: `deposit_count = 0` (must be claimed first)
- Message bridge: `deposit_count = 1` (claimed after asset bridge)

The `--deposit-count` parameter allows you to specify which specific bridge to claim when multiple bridges share the same transaction hash.

**Note**: For BridgeExtension message claims, you must ensure:
1. The origin address in claimMessage should be the BRIDGE_EXTENSION address
2. Use custom `--data` metadata that matches the BridgeExtension encoding format
3. The asset bridge must be claimed first with the correct global index

### How Bridge and Call Works

When you execute `bridge-and-call`, it creates **TWO** bridge transactions:

1. **Asset Bridge** (deposit_count = 0): Bridges tokens to a precalculated JumpPoint address
2. **Message Bridge** (deposit_count = 1): Contains the call instructions for execution

The claiming process must be done in order:
1. First claim the asset bridge to transfer tokens to the JumpPoint
2. Then claim the message bridge to trigger the contract call execution

When the message bridge is claimed, the BridgeExtension contract:
1. Validates that the corresponding asset was claimed first
2. Deploys a temporary JumpPoint contract using CREATE2
3. Executes the contract call with the bridged tokens
4. Handles fallback if the call fails

## Bridge Utilities

The bridge utilities provide advanced functions for bridge operations, helpful for developers and advanced users.

### Available Utilities

#### Build Claim Payload

Build complete claim payload data from a bridge transaction hash:

```bash
# Build claim payload from bridge transaction
aggsandbox bridge utils build-payload \
  --tx-hash 0xb7118cfb20825861028ede1e9586814fc7ccf81745a325db5df355d382d96b4e \
  --source-network 0

# Output as JSON
aggsandbox bridge utils build-payload \
  --tx-hash 0xb7118cfb20825861028ede1e9586814fc7ccf81745a325db5df355d382d96b4e \
  --source-network 0 \
  --bridge-index 1 \
  --json
```

**Parameters:**
- `--tx-hash, -t`: Bridge transaction hash
- `--source-network, -s`: Source network ID (0=Mainnet, 1=AggLayer-1, 2=AggLayer-2)
- `--bridge-index`: Bridge index for multi-bridge transactions (optional)
- `--json`: Output as JSON format

#### Compute Global Index

Calculate global bridge index from local index and network:

```bash
# Calculate global index for L1 bridge
aggsandbox bridge utils compute-index \
  --local-index 42 \
  --source-network 0

# Output: Global Index = 2147483690 (42 + 2^31)

# Calculate global index for L2 bridge with JSON output
aggsandbox bridge utils compute-index \
  --local-index 100 \
  --source-network 1 \
  --json
```

**Global Index Formula:**
- **Mainnet (network 0)**: `globalIndex = localIndex + 2^31`
- **AggLayer networks (network 1+)**: `globalIndex = localIndex + (networkId - 1) * 2^32`

#### Get Mapped Token Address

Get the wrapped token address for an origin token:

```bash
# Get AggLayer-1 wrapped token address for Mainnet token
aggsandbox bridge utils get-mapped \
  --network 1 \
  --origin-network 0 \
  --origin-token 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC
```

**Parameters:**
- `--network, -n`: Target network ID where you want the wrapped token address
- `--origin-network`: Origin network ID where the original token exists
- `--origin-token`: Origin token contract address
- `--private-key`: Private key (optional, uses default if not provided)
- `--json`: Output as JSON format

#### Pre-calculate Token Address

Pre-calculate what the wrapped token address will be before deployment:

```bash
# Pre-calculate L2 token address for L1 token
aggsandbox bridge utils precalculate \
  --network 1 \
  --origin-network 0 \
  --origin-token 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC

# With JSON output
aggsandbox bridge utils precalculate \
  --network 1 \
  --origin-network 0 \
  --origin-token 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC \
  --json
```

This is useful for knowing the token address before any tokens are bridged and the wrapper is deployed.

#### Get Origin Token Information

Get original token information from a wrapped token address:

```bash
# Get origin info for wrapped token
aggsandbox bridge utils get-origin \
  --network 1 \
  --wrapped-token 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0

# With JSON output
aggsandbox bridge utils get-origin \
  --network 1 \
  --wrapped-token 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 \
  --json
```

**Returns:**
- Origin network ID
- Origin token contract address

#### Check Claim Status

Check if a bridge has been claimed:

```bash
# Check if bridge is claimed
aggsandbox bridge utils is-claimed \
  --network 1 \
  --index 42 \
  --source-network 0

# With JSON output
aggsandbox bridge utils is-claimed \
  --network 1 \
  --index 42 \
  --source-network 0 \
  --json
```

**Parameters:**
- `--network, -n`: Network ID to check
- `--index`: Bridge index to check
- `--source-network`: Source bridge network ID
- `--private-key`: Private key (optional)
- `--json`: Output as JSON format

### Utility Command Reference

All bridge utility commands support:
- **Validation**: Automatic validation of network IDs (0=Mainnet, 1=AggLayer-1, 2=AggLayer-2) and Ethereum addresses
- **JSON Output**: Use `--json` flag for machine-readable output
- **Error Handling**: Comprehensive error messages with suggestions
- **Help Text**: Use `--help` on any command for detailed information

```bash
# List all available utilities
aggsandbox bridge utils --help

# Get help for specific utility
aggsandbox bridge utils build-payload --help
```

### JSON Output Format

All utilities support JSON output for integration with scripts and tools:

```json
{
  "local_index": 42,
  "source_network": 0,
  "global_index": "2147483690"
}
```

JSON output is consistent across all utilities and includes all relevant information for programmatic use.

## Network Configuration

### Network ID Mapping

The bridge service maps sandbox network IDs to actual chain IDs:

| Network ID | Chain ID | Description |
|------------|----------|-------------|
| 0 | 1 | Mainnet (L1 simulation) |
| 1 | 1101 | AggLayer-1 (zkEVM L2) |
| 2 | 137 | AggLayer-2 (Multi-L2 mode) |

### Contract Addresses

Bridge contracts are automatically deployed and configured:

- **L1 Bridge**: `0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82`
- **L2 Bridge**: `0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6`
- **L3 Bridge**: `0x5FbDB2315678afecb367f032d93F642f64180aa3` (Multi-L2 mode)

### Bridge Extensions (for bridgeAndCall)

- **L1 Extension**: `0x0B306BF915C4d645ff596e518fAf3F9669b97016`
- **L2 Extension**: `0x8A791620dd6260079BF849Dc5567aDC3F2FdC318`
- **L3 Extension**: `0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512` (Multi-L2 mode)

## Implementation Details

### Service Architecture

The bridge functionality is implemented directly in the Rust CLI:

```bash
cli/src/commands/bridge/
├── mod.rs                             # Bridge command module and CLI integration
├── bridge_asset.rs                    # Asset bridging with ERC20 approval
├── bridge_call.rs                     # Message bridging and bridgeAndCall operations
├── claim_asset.rs                     # Asset claiming with proof verification
├── claim_message.rs                   # Message claiming operations
└── utilities.rs                       # Bridge utility functions (NEW)
    ├── build_payload_for_claim()      # Build complete claim payloads
    ├── compute_global_index()         # Calculate global bridge indices
    ├── get_mapped_token_info()        # Get wrapped token addresses
    ├── precalculated_mapped_token_info() # Pre-calculate token addresses
    ├── get_origin_token_info()        # Get origin token from wrapped token
    └── is_claimed()                   # Check bridge claim status
```

### Key Features

1. **Automatic Token Approval**: ERC20 tokens are automatically approved for bridging
2. **Automatic Bridge Type Detection**: The claim command automatically detects asset vs message bridges and calls the appropriate contract function (`claimAsset` or `claimMessage`)
3. **Two-Phase Bridge and Call**: Separate asset and message claiming for bridgeAndCall operations
4. **Precalculated Address Support**: Automatically calculates L2 token wrapper addresses
5. **Network Availability Checking**: Verifies networks are accessible before operations
6. **Chain ID Mapping**: Maps sandbox network IDs to actual blockchain chain IDs
7. **Error Handling**: Comprehensive error reporting with transaction details
8. **Gas Optimization**: Configurable gas limits and prices
9. **Direct Smart Contract Interaction**: Uses ethers.rs for direct contract calls
10. **JumpPoint Contract Integration**: Supports CREATE2-based temporary contract execution
11. **Bridge Utilities**: Advanced utility functions for developers and power users
12. **JSON Output Support**: Machine-readable output for all utility commands
13. **Input Validation**: Automatic validation of addresses, network IDs, and parameters
14. **Global Index Calculations**: Accurate lxly.js-compatible global index computations

### Configuration

The bridge functionality uses the same environment variables as other CLI operations:

```bash
# RPC URLs (from .env)
RPC_1=http://localhost:8545    # L1
RPC_2=http://localhost:8546    # L2
RPC_3=http://localhost:8547    # L3 (multi-L2 mode)

# Contract addresses (automatically sourced from deployment)
POLYGON_ZKEVM_BRIDGE_L1=0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82
POLYGON_ZKEVM_BRIDGE_L2=0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6
POLYGON_ZKEVM_BRIDGE_L3=0x5FbDB2315678afecb367f032d93F642f64180aa3

# Default account (Anvil test account #1)
ACCOUNT_ADDRESS_1=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
PRIVATE_KEY_1=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

## Examples

### Complete Bridge Workflow

1. **Start the sandbox:**

   ```bash
   aggsandbox start --detach
   ```

2. **Check network status:**

   ```bash
   aggsandbox status
   ```

3. **Bridge ETH from L1 to L2:**

   ```bash
   aggsandbox bridge asset \
     --network 0 \
     --destination-network 1 \
     --amount 0.5 \
     --token-address 0x0000000000000000000000000000000000000000
   ```

4. **Wait for bridge confirmation and claim on L2:**

   ```bash
   aggsandbox bridge claim \
     --network 1 \
     --tx-hash <bridge_transaction_hash> \
     --source-network 0
   ```

### Multi-L2 Bridge Example

1. **Start multi-L2 mode:**

   ```bash
   aggsandbox start --multi-l2 --detach
   ```

2. **Bridge between L2 networks:**

   ```bash
   # L2 (zkEVM) to L3 (PoS)
   aggsandbox bridge asset \
     --network 1 \
     --destination-network 2 \
     --amount 1.0 \
     --token-address 0x0000000000000000000000000000000000000000
   ```

### Complete Bridge and Call Workflow

1. **Start the sandbox:**

   ```bash
   aggsandbox start --detach
   ```

2. **Deploy a test ERC20 token (if needed):**

   ```bash
   # Deploy an AggERC20 token on L1
   cast send --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
     --rpc-url http://localhost:8545 \
     --create $(cat agglayer-contracts/out/AggERC20.sol/AggERC20.json | jq -r '.bytecode.object')
   ```

3. **Encode transfer call data:**

   ```bash
   # Transfer 1 token to a recipient address
   TRANSFER_DATA=$(cast calldata "transfer(address,uint256)" 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 1000000000000000000)
   echo "Transfer call data: $TRANSFER_DATA"
   ```

4. **Execute bridge and call:**

   ```bash
   aggsandbox bridge bridge-and-call \
     --network 0 \
     --destination-network 1 \
     --token 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC \
     --amount 10000000000000000000 \
     --target 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 \
     --data $TRANSFER_DATA \
     --fallback 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
   ```

5. **Wait for bridge processing and check bridges:**

   ```bash
   # Check for both bridge entries (asset and message)
   aggsandbox show bridges --network-id 0
   ```

6. **Claim asset bridge first (deposit_count = 0):**

   ```bash
   # Find the bridge transactions from bridgeAndCall
   aggsandbox show bridges --network-id 0

   # Claim the asset bridge using tx_hash and deposit_count=0
   aggsandbox bridge claim \
     --network 1 \
     --tx-hash <bridge_tx_hash> \
     --source-network 0 \
     --deposit-count 0
   ```

7. **Claim message bridge to trigger execution (deposit_count = 1):**

   ```bash
   # Claim the message bridge using the same tx_hash but deposit_count=1
   aggsandbox bridge claim \
     --network 1 \
     --tx-hash <bridge_tx_hash> \
     --source-network 0 \
     --deposit-count 1
   ```

8. **Verify the contract call was executed:**

   ```bash
   # Check the L2 token balance to confirm the transfer worked
   cast call 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 \
     "balanceOf(address)" 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
     --rpc-url http://localhost:8546
   ```

## Error Handling

### Common Errors and Solutions

**ERC20InsufficientBalance (0xe450d38c)**

- **Cause**: Insufficient token balance or allowance
- **Solution**: Ensure the account has sufficient tokens and approval is granted

**Invalid Network ID (0x0595ea2e)**

- **Cause**: Wrong network ID to chain ID mapping
- **Solution**: Use correct network IDs (0, 1, 2) for sandbox environment

**Transaction Timeout**

- **Cause**: Network congestion or RPC issues
- **Solution**: Check network connectivity and retry with higher gas price

**Bridge Contract Not Deployed**

- **Cause**: Contract addresses not found in configuration
- **Solution**: Ensure sandbox is started and contracts are deployed

**Asset Bridge Must Be Claimed First**

- **Cause**: Attempting to claim message bridge before asset bridge
- **Solution**: Always claim asset bridge (deposit_count = 0) before message bridge (deposit_count = 1) using the existing claim command

**Bridge Extension Not Found**

- **Cause**: BridgeExtension contract not deployed on the network
- **Solution**: Verify sandbox deployment includes bridge extension contracts

**Contract Call Execution Failed**

- **Cause**: Target contract call failed during message bridge claiming
- **Solution**: Check fallback address receives tokens; verify call data and target contract

**Bridge Transaction Not Found**

- **Cause**: Cannot find bridge transaction with specific tx_hash and/or deposit_count
- **Solution**: Use `aggsandbox show bridges --network-id <source_network>` to list all bridges and find the correct tx_hash. For bridgeAndCall operations, use `--deposit-count 0` for asset bridge and `--deposit-count 1` for message bridge

### Debugging

Enable verbose logging for debugging:

```bash
# Check bridge transaction details
aggsandbox show bridges --network-id 0

# Check claim status
aggsandbox show claims --network-id 1

# Check network connectivity
aggsandbox status
```

## Security Considerations

- **Test Environment Only**: The provided private keys are for testing only
- **Never Use in Production**: These keys are publicly known and should never hold real funds
- **Contract Verification**: Bridge contracts are deployed automatically but should be verified for production use
- **Gas Limits**: Set appropriate gas limits to prevent failed transactions

## Development

### Building from Source

```bash
# Build the CLI with bridge functionality
cargo build --release

# Run tests
cargo test
```

### Extending the Functionality

The bridge implementation can be extended by:

1. **Adding new contract ABIs** in `src/commands/bridge.rs`
2. **Implementing additional bridge operations** following the same pattern
3. **Adding validation logic** for specific token types
4. **Updating configuration** for new networks or contracts

## Troubleshooting

### Bridge Operations Failing

```bash
# Check sandbox is running
aggsandbox status

# Verify network connectivity
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'

# Check contract addresses
aggsandbox info
```

### Performance Issues

```bash
# Check Docker resources
docker stats

# Monitor transaction status
aggsandbox events --network-id 0

# Test with lower amounts first
aggsandbox bridge asset --network 0 --destination-network 1 --amount 0.001 --token-address 0x0000000000000000000000000000000000000000
```

## Further Reading

- [Polygon LxLy Bridge Documentation](https://wiki.polygon.technology/docs/zkEVM/protocol/lxly-bridge/)
- [Aggsandbox CLI Documentation](README.md)
- [Bridge API Endpoints](COMMANDS.md#aggkit-api-endpoints)
