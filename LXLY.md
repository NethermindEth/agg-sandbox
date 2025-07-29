# LXLY Bridge Integration

This document describes the LXLY bridge functionality integrated into the aggsandbox CLI, enabling user-friendly cross-chain operations using direct Rust implementation with smart contract interactions.

## Overview

The LXLY bridge integration provides three main operations:

- **Bridge Assets**: Transfer ERC20 tokens or ETH between networks
- **Claim Assets**: Claim previously bridged assets on the destination network
- **Bridge Messages**: Bridge with contract calls (bridgeAndCall functionality)

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
- `--gas-limit`: Gas limit override (optional)
- `--gas-price`: Gas price override in wei (optional)

### Bridge Messages

Bridge with contract calls (bridgeAndCall functionality):

```bash
# Bridge ETH with contract call
aggsandbox bridge message \
  --network 0 \
  --destination-network 1 \
  --target 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 \
  --data 0xa9059cbb000000000000000000000000742d35cc6965c592342c6c16fb8eaeb90a23b5c00000000000000000000000000000000000000000000000000de0b6b3a7640000 \
  --amount 0.01 \
  --fallback-address 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

**Parameters:**

- `--network, -n`: Source network ID
- `--destination-network, -d`: Destination network ID
- `--target, -t`: Target contract address on destination network
- `--data, -D`: Contract call data (hex encoded)
- `--amount, -a`: Amount of ETH to send (optional)
- `--fallback-address`: Fallback address if call fails (optional, defaults to sender)
- `--gas-limit`: Gas limit override (optional)
- `--gas-price`: Gas price override in wei (optional)

## Network Configuration

### Network ID Mapping

The bridge service maps sandbox network IDs to actual chain IDs:

| Network ID | Chain ID | Description |
|------------|----------|-------------|
| 0 | 1 | Ethereum L1 (Mainnet simulation) |
| 1 | 1101 | Polygon zkEVM L2 |
| 2 | 137 | Polygon PoS L2 (Multi-L2 mode) |

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
cli/src/commands/
├── bridge.rs             # Bridge command implementation
│   ├── bridge_asset()    # Asset bridging with ERC20 approval
│   ├── claim_asset()     # Asset claiming with proof verification
│   └── bridge_message()  # Message bridging with contract calls
```

### Key Features

1. **Automatic Token Approval**: ERC20 tokens are automatically approved for bridging
2. **Network Availability Checking**: Verifies networks are accessible before operations
3. **Chain ID Mapping**: Maps sandbox network IDs to actual blockchain chain IDs
4. **Error Handling**: Comprehensive error reporting with transaction details
5. **Gas Optimization**: Configurable gas limits and prices
6. **Direct Smart Contract Interaction**: Uses ethers.rs for direct contract calls

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
