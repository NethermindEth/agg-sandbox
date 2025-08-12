# Bridge Operations Guide

Complete guide to cross-chain bridge operations using the LXLY bridge integration.

## Overview

The Agglayer Sandbox includes comprehensive LXLY bridge functionality for:

- **Asset Bridging**: Transfer ERC20 tokens or ETH between networks
- **Message Bridging**: Bridge with contract calls
- **Claim Operations**: Claim bridged assets on destination networks
- **Bridge-and-Call**: Advanced bridging with automatic execution

## Prerequisites

### Environment Setup

```bash
# Start sandbox and source environment
aggsandbox start --detach
source .env
```

### Required Variables

Ensure these environment variables are set in your `.env`:

```bash
# Core variables
RPC_URL_1=http://127.0.0.1:8545
RPC_URL_2=http://127.0.0.1:8546
NETWORK_ID_MAINNET=0
NETWORK_ID_AGGLAYER_1=1
ACCOUNT_ADDRESS_1=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
PRIVATE_KEY_1=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Bridge contracts (automatically configured)
POLYGON_ZKEVM_BRIDGE_L1=0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82
POLYGON_ZKEVM_BRIDGE_L2=0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6
```

## Basic Asset Bridging

### L1 to L2 Bridge Flow

#### 1. Bridge ETH from L1 to L2

```bash
# Bridge 0.1 ETH from L1 to L2
aggsandbox bridge asset \
  --network 0 \
  --destination-network 1 \
  --amount 0.1 \
  --token-address 0x0000000000000000000000000000000000000000
```

#### 2. Monitor Bridge Transaction

```bash
# Check bridge status
aggsandbox show bridges --network-id 0
```

Example response:

```json
{
  "bridges": [
    {
      "tx_hash": "0x4a0e66947eceb49c887cf56f1a92872b2b7e16177a02c3cf79ea4846fab30fe0",
      "deposit_count": 0,
      "amount": "100000000000000000",
      "destination_network": 1,
      "destination_address": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    }
  ]
}
```

#### 3. Claim Assets on L2

```bash
# Claim bridged ETH on L2
aggsandbox bridge claim \
  --network 1 \
  --tx-hash 0x4a0e66947eceb49c887cf56f1a92872b2b7e16177a02c3cf79ea4846fab30fe0 \
  --source-network 0
```

#### 4. Verify Claim

```bash
# Check claim status
aggsandbox show claims --network-id 1
```

#### 5. Using claimsponsor

When using flag `claim-all` to start the sandbox there's no need to do anything else. The claim will be done automatically so the claims can be checked directly. It might take a few seconds to update the GER before the claim transaction is successful.

If `claim-all` flag was not set, the claim can still be added manually to the claimsponsor by:

```bash
aggsandbox sponsor-claim --deposit <deposit_count>
```

**Parameters:**
- `origin_network` is `0` by default
- `destination_network` is `1` by default

To check the status of the claim in the claimsponsor process:

```bash
aggsandbox claim-status --global-index <global_index> --network-id <network_id>
```


### ERC20 Token Bridging

#### 1. Bridge ERC20 Tokens

```bash
# Bridge 100 tokens from L1 to L2
aggsandbox bridge asset \
  --network 0 \
  --destination-network 1 \
  --amount 100 \
  --token-address $AGG_ERC20_L1 \
  --to-address $ACCOUNT_ADDRESS_2
```

The CLI automatically:

- Approves the bridge contract to spend tokens
- Executes the bridge transaction
- Provides transaction hash for claiming

#### 2. Find Wrapped Token Address

```bash
# Monitor events to find the wrapped token address
aggsandbox events --network-id 1
```

Look for the `NewWrappedToken` event:

```
🎯 Event: NewWrappedToken(uint32,address,address,bytes)
  🌐 Origin Network: 0
  📍 Origin Token: 0x5FbDB2315678afecb367f032d93F642f64180aa3
  🎁 Wrapped Token: 0x19e2b7738a026883d08c3642984ab6d7510ca238
```

#### 3. Claim ERC20 Tokens

```bash
# Claim the ERC20 tokens on L2
aggsandbox bridge claim \
  --network 1 \
  --tx-hash <bridge_tx_hash> \
  --source-network 0
```

The CLI automatically:

- Detects it's an ERC20 bridge operation
- Generates proper token metadata
- Calls `claimAsset` with correct parameters

### Bidirectional Bridging (L2 to L1)

#### 1. Bridge Back to L1

```bash
# Bridge wrapped tokens back to L1
aggsandbox bridge asset \
  --network 1 \
  --destination-network 0 \
  --amount 50 \
  --token-address $WRAPPED_TOKEN_ADDRESS \
  --to-address $ACCOUNT_ADDRESS_1
```

#### 2. Claim on L1

```bash
# Claim original tokens on L1
aggsandbox bridge claim \
  --network 0 \
  --tx-hash <bridge_tx_hash> \
  --source-network 1
```

## Message Bridging

### Simple Message Bridge

```bash
# Encode message data
MESSAGE_DATA=$(cast calldata "transfer(address,uint256)" $ACCOUNT_ADDRESS_1 1000000000000000000)

# Bridge message from L1 to L2
aggsandbox bridge message \
  --network 0 \
  --destination-network 1 \
  --target 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 \
  --data $MESSAGE_DATA \
  --amount 0.01 \
  --fallback-address $ACCOUNT_ADDRESS_1
```

### Claim Message Bridge

```bash
# Claim the message bridge
aggsandbox bridge claim \
  --network 1 \
  --tx-hash <message_bridge_tx_hash> \
  --source-network 0
```

The CLI automatically detects it's a message bridge and calls `claimMessage`.

## Advanced Bridge-and-Call

Bridge-and-Call combines asset bridging with contract execution in a single atomic operation.

### Setup Bridge-and-Call

#### 1. Prepare Call Data

```bash
# Encode transfer function call
TRANSFER_DATA=$(cast calldata "transfer(address,uint256)" $ACCOUNT_ADDRESS_1 1000000000000000000)

# Get precalculated L2 token address
L2_TOKEN_ADDRESS=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
  "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
  1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
  --rpc-url $RPC_2 | sed 's/0x000000000000000000000000/0x/')
```

#### 2. Execute Bridge-and-Call

```bash
# Execute bridge-and-call operation
aggsandbox bridge bridge-and-call \
  --network 0 \
  --destination-network 1 \
  --token $AGG_ERC20_L1 \
  --amount 10 \
  --target $L2_TOKEN_ADDRESS \
  --data $TRANSFER_DATA \
  --fallback $ACCOUNT_ADDRESS_1
```

This creates **two** bridge transactions:

- **Asset Bridge** (deposit_count = 0): Bridges tokens to JumpPoint
- **Message Bridge** (deposit_count = 1): Contains execution instructions

### Two-Phase Claiming Process

Bridge-and-call requires claiming in specific order:

#### Phase 1: Claim Asset Bridge

```bash
# Find bridge transactions
aggsandbox show bridges --network-id 0

# Claim asset bridge FIRST (deposit_count = 0)
aggsandbox bridge claim \
  --network 1 \
  --tx-hash <bridge_tx_hash> \
  --source-network 0 \
  --deposit-count 0
```

#### Phase 2: Claim Message Bridge

```bash
# Claim message bridge SECOND (deposit_count = 1)
aggsandbox bridge claim \
  --network 1 \
  --tx-hash <bridge_tx_hash> \
  --source-network 0 \
  --deposit-count 1
```

**Important**: The asset bridge must be claimed first. The message bridge automatically executes the contract call when claimed.

## Bridge Utilities

Advanced utilities for bridge operations:

### Build Claim Payload

```bash
# Build complete claim payload
aggsandbox bridge utils build-payload \
  --tx-hash 0xb7118cfb20825861028ede1e9586814fc7ccf81745a325db5df355d382d96b4e \
  --source-network 0
```

### Calculate Global Index

```bash
# Calculate global bridge index
aggsandbox bridge utils compute-index \
  --local-index 42 \
  --source-network 0
```

**Global Index Formula:**

- **Mainnet (network 0)**: `globalIndex = localIndex + 2^31`
- **AggLayer networks (network 1+)**: `globalIndex = localIndex + (networkId - 1) * 2^32`

### Token Address Utilities

```bash
# Get mapped token address
aggsandbox bridge utils get-mapped \
  --network 1 \
  --origin-network 0 \
  --origin-token $AGG_ERC20_L1

# Pre-calculate token address
aggsandbox bridge utils precalculate \
  --network 1 \
  --origin-network 0 \
  --origin-token $AGG_ERC20_L1

# Get origin token info
aggsandbox bridge utils get-origin \
  --network 1 \
  --wrapped-token $WRAPPED_TOKEN_ADDRESS
```

### Check Claim Status

```bash
# Check if bridge is claimed
aggsandbox bridge utils is-claimed \
  --network 1 \
  --index 42 \
  --source-network 0
```

## Multi-L2 Operations

### Start Multi-L2 Mode

```bash
# Start with three chains: L1, L2-1, L2-2
aggsandbox start --multi-l2 --detach
```

### Cross-L2 Bridging

```bash
# Bridge from L2-1 to L2-2
aggsandbox bridge asset \
  --network 1 \
  --destination-network 2 \
  --amount 1.0 \
  --token-address 0x0000000000000000000000000000000000000000

# Claim on L2-2
aggsandbox bridge claim \
  --network 2 \
  --tx-hash <bridge_tx_hash> \
  --source-network 1
```

### Multi-L2 Network Mapping

| Network ID | Description   | Port |
| ---------- | ------------- | ---- |
| 0          | L1 (Ethereum) | 8545 |
| 1          | L2-1 (zkEVM)  | 8546 |
| 2          | L2-2 (PoS)    | 8547 |

## Monitoring and Debugging

### View Bridge Information

```bash
# Show all bridges for L1
aggsandbox show bridges --network-id 0

# Show claims for L2
aggsandbox show claims --network-id 1

# Get claim proof
aggsandbox show claim-proof \
  --network-id 0 \
  --leaf-index 0 \
  --deposit-count 1
```

### Monitor Events

```bash
# Monitor L1 events
aggsandbox events --network-id 0

# Monitor L2 events with address filter
aggsandbox events \
  --network-id 1 \
  --blocks 20 \
  --address 0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6
```

### JSON Output for Scripting

```bash
# Get bridge data as JSON
BRIDGE_DATA=$(aggsandbox show bridges --network-id 0 --json)
DEPOSIT_COUNT=$(echo $BRIDGE_DATA | jq -r '.bridges[0].deposit_count')

# Use in scripts
aggsandbox bridge claim \
  --network 1 \
  --tx-hash <hash> \
  --source-network 0 \
  --deposit-count $DEPOSIT_COUNT
```

## Best Practices

### Security Considerations

1. **Test Environment Only**: Use provided private keys only for testing
2. **Gas Limits**: Set appropriate gas limits to prevent failures
3. **Verification**: Always verify transactions before claiming
4. **Order Matters**: For bridge-and-call, claim asset bridge before message bridge

### Performance Optimization

1. **Batch Operations**: Group multiple bridges when possible
2. **Monitor Resources**: Check Docker resources with `docker stats`
3. **Connection Pooling**: CLI automatically reuses HTTP connections
4. **Caching**: Bridge data is cached for better performance

### Error Prevention

1. **Check Status**: Always verify sandbox is running with `aggsandbox status`
2. **Environment**: Ensure `.env` is sourced before operations
3. **Network IDs**: Use correct network IDs (0, 1, 2)
4. **Token Addresses**: Verify token contracts exist on source network

## Common Workflows

### Complete Asset Bridge Workflow

```bash
# 1. Start sandbox
aggsandbox start --detach && source .env

# 2. Bridge assets
aggsandbox bridge asset --network 0 --destination-network 1 --amount 0.5 --token-address 0x0000000000000000000000000000000000000000

# 3. Wait for confirmation
aggsandbox show bridges --network-id 0

# 4. Claim on destination
aggsandbox bridge claim --network 1 --tx-hash <hash> --source-network 0

# 5. Verify claim
aggsandbox show claims --network-id 1
```

### Bridge-and-Call Workflow

```bash
# 1. Prepare call data
TRANSFER_DATA=$(cast calldata "transfer(address,uint256)" $ACCOUNT_ADDRESS_1 1000000000000000000)
L2_TOKEN_ADDRESS=$(aggsandbox bridge utils precalculate --network 1 --origin-network 0 --origin-token $AGG_ERC20_L1 --json | jq -r '.precalculated_address')

# 2. Execute bridge-and-call
aggsandbox bridge bridge-and-call --network 0 --destination-network 1 --token $AGG_ERC20_L1 --amount 10 --target $L2_TOKEN_ADDRESS --data $TRANSFER_DATA --fallback $ACCOUNT_ADDRESS_1

# 3. Claim asset bridge (must be first)
aggsandbox bridge claim --network 1 --tx-hash <hash> --source-network 0 --deposit-count 0

# 4. Claim message bridge (triggers execution)
aggsandbox bridge claim --network 1 --tx-hash <hash> --source-network 0 --deposit-count 1
```

## Next Steps

- **[Advanced Workflows](advanced-workflows.md)** - Complex multi-chain scenarios
- **[CLI Reference](cli-reference.md)** - Complete command documentation
- **[Troubleshooting](troubleshooting.md)** - Common issues and solutions
- **[Configuration](configuration.md)** - Environment customization
