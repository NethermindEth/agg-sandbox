# End to end bridging flow

This document provides the commands and endpoints needed to perform a complete asset bridging flow from L1 to L2 using the Polygon ZkEVM bridge.

## Table of Contents

- [Prerequisites](#prerequisites)
  - [System Requirements](#system-requirements)
  - [Environment Setup](#environment-setup)
  - [Environment Validation](#environment-validation)
- [Quick Start Guide](#quick-start-guide)
- [Basic L1 to L2 Bridge Flow](#basic-l1-to-l2-bridge-flow)
  - [Step 1: Approve Bridge Contract](#step-1-approve-bridge-contract-to-spend-tokens)
  - [Step 2: Bridge Assets](#step-2-bridge-assets-from-l1-to-l2)
  - [Step 3: Prepare Token Metadata](#step-3-prepare-token-metadata)
  - [Step 4: Get Claim Proof](#step-4-get-claim-proof)
  - [Step 5: Claim Bridged Assets](#step-5-claim-bridged-assets-on-l2)
  - [Step 6: Verify Token Balance](#step-6-verify-token-balance)
- [Bidirectional Bridge (L2 to L1)](#step-7-bridge-back-to-l1)
- [Advanced Features](#bridge-and-call)
  - [Bridge and Call Overview](#bridge-and-call-overview)
  - [Step-by-Step Process](#step-by-step-bridge-and-call-process)
  - [Use Cases](#use-cases-for-bridge-and-call)
- [Manual Claiming (Advanced)](#manual-claiming-advanced)
- [API Reference](#aggkit-api-endpoints)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **Rust** >= 1.70.0
- **Foundry** (cast CLI)
- **Docker** and Docker Compose
- **jq** (for JSON processing)

### Environment Setup

Before starting the bridging process, make sure to start the agg-sandbox environment and source your environment variables:

```bash
# Copy environment template
cp .env.example .env

# Start the sandbox
aggsandbox start --detach

# Source environment variables
source .env
```

### Environment Validation

Verify that all required environment variables are properly set:

```bash
# Verify required variables are set
echo "Verifying environment variables..."
required_vars=("AGG_ERC20_L1" "POLYGON_ZKEVM_BRIDGE_L1" "PRIVATE_KEY_1" "ACCOUNT_ADDRESS_1" "RPC_1" "RPC_2")
for var in "${required_vars[@]}"; do
  if [ -z "${!var}" ]; then
    echo "❌ Missing required variable: $var"
    exit 1
  else
    echo "✅ $var is set"
  fi
done

# Test RPC connectivity
echo "Testing RPC connections..."
cast block-number --rpc-url $RPC_1 && echo "✅ L1 RPC connected"
cast block-number --rpc-url $RPC_2 && echo "✅ L2 RPC connected"
```

This ensures all the required environment variables (like `$AGG_ERC20_L1`, `$POLYGON_ZKEVM_BRIDGE_L1`, `$PRIVATE_KEY_1`, etc.) are available in your shell session.

## Quick Start Guide

For experienced users, here's the complete L1→L2 bridge flow:

```bash
# 1. Setup and approve
source .env
cast send $AGG_ERC20_L1 \
  "approve(address,uint256)" \
  $POLYGON_ZKEVM_BRIDGE_L1 100 \
  --private-key $PRIVATE_KEY_1 \
  --rpc-url $RPC_1

# 2. Bridge assets
cast send $POLYGON_ZKEVM_BRIDGE_L1 \
  "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
  $CHAIN_ID_AGGLAYER_1 $ACCOUNT_ADDRESS_2 10 $AGG_ERC20_L1 true 0x \
  --private-key $PRIVATE_KEY_1 \
  --rpc-url $RPC_1

# 3. Get proof data
DEPOSIT_COUNT=$(aggsandbox show bridges --network-id 1 | jq -r '.bridges[0].deposit_count')
LEAF_INDEX=$(aggsandbox show l1-info-tree-index --network-id 1 --deposit-count $DEPOSIT_COUNT | jq -r '.l1_info_tree_index')
PROOF_DATA=$(aggsandbox show claim-proof --network-id 1 --leaf-index $LEAF_INDEX --deposit-count $DEPOSIT_COUNT)

# 4. Extract proof values
MAINNET_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.mainnet_exit_root')
ROLLUP_EXIT_ROOT=$(echo $PROOF_DATA | jq -r '.l1_info_tree_leaf.rollup_exit_root')

# 5. Claim on L2
METADATA=$(cast abi-encode "f(string,string,uint8)" "AggERC20" "AGGERC20" 18)
cast send $POLYGON_ZKEVM_BRIDGE_L2 \
  "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
  $DEPOSIT_COUNT $MAINNET_EXIT_ROOT $ROLLUP_EXIT_ROOT \
  1 $AGG_ERC20_L1 1101 $ACCOUNT_ADDRESS_2 10 $METADATA \
  --private-key $PRIVATE_KEY_2 \
  --rpc-url $RPC_2 \
  --gas-limit 3000000
```

## Basic L1 to L2 Bridge Flow

### Step 1: Approve Bridge Contract to Spend Tokens

Before bridging assets, you need to approve the bridge contract to spend your tokens on your behalf.

#### Check Current Balance

```bash
cast call $AGG_ERC20_L1 \
  "balanceOf(address)" \
  $ACCOUNT_ADDRESS_1 \
  --rpc-url $RPC_1
```

#### Approve Bridge Contract

```bash
cast send $AGG_ERC20_L1 \
  "approve(address,uint256)" \
  $POLYGON_ZKEVM_BRIDGE_L1 100 \
  --private-key $PRIVATE_KEY_1 \
  --rpc-url $RPC_1
```

#### Verify Approval

```bash
cast call $AGG_ERC20_L1 \
  "allowance(address,address)" \
  $ACCOUNT_ADDRESS_1 $POLYGON_ZKEVM_BRIDGE_L1 \
  --rpc-url $RPC_1
```

**Explanation**: This command approves the Polygon ZkEVM bridge contract on L1 to spend 100 tokens from your account. The `approve` function is a standard ERC20 function that allows another contract (the bridge) to transfer tokens on your behalf.

### Step 2: Bridge Assets from L1 to L2

Initiate the bridging process by calling the bridge contract to transfer assets to the destination chain.

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L1 \
  "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
  $CHAIN_ID_AGGLAYER_1 $ACCOUNT_ADDRESS_2 10 $AGG_ERC20_L1 true 0x \
  --private-key $PRIVATE_KEY_1 \
  --rpc-url $RPC_1
```

**Parameters:**

1. `$CHAIN_ID_AGGLAYER_1` - The destination chain ID
2. `$ACCOUNT_ADDRESS_2` - The recipient address on L2
3. `10` - Amount of tokens to bridge
4. `$AGG_ERC20_L1` - The token contract address
5. `true` - Whether to force the bridge (bypass some checks)
6. `0x` - Additional data (empty in this case)

#### Verify Balance After Bridge

```bash
cast call $AGG_ERC20_L1 \
  "balanceOf(address)" \
  $ACCOUNT_ADDRESS_1 \
  --rpc-url $RPC_1
```

#### Check Bridge Details

After initiating the bridge, you can check the bridge details using the CLI command:

```bash
aggsandbox show bridges --network-id 1
```

This will return bridge information including transaction details, deposit count, and metadata. Example response:

```json
{
  "bridges": [
    {
      "block_num": 7,
      "block_pos": 1,
      "from_address": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
      "tx_hash": "0x4a0e66947eceb49c887cf56f1a92872b2b7e16177a02c3cf79ea4846fab30fe0",
      "calldata": "0xcd586579000000000000000000000000000000000000000000000000000000000000044d00000000000000000000000070997970c51812dc3a010c7d01b50e0d17dc79c8000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000005fbdb2315678afecb367f032d93f642f64180aa3000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000000",
      "block_timestamp": 1751237153,
      "leaf_type": 0,
      "origin_network": 1,
      "origin_address": "0x5FbDB2315678afecb367f032d93F642f64180aa3",
      "destination_network": 1101,
      "destination_address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      "amount": "10",
      "metadata": "0x000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000008416767455243323000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000084147474552433230000000000000000000000000000000000000000000000000",
      "deposit_count": 0,
      "is_native_token": false,
      "bridge_hash": "0xbdc21b1ceb90347c5fe8abdbbf8996fe062241770ea5bcf8b4e654e79848144b"
    }
  ]
}
```

**Key information from the response:**

- `deposit_count` - Use this value in the claimAsset command
- `bridge_hash` - The unique identifier for this bridge transaction
- `tx_hash` - The transaction hash on L1
- `sandbox_metadata` - Indicates this is running in sandbox mode with instant claims enabled

#### Get L1 Info Tree Index

After getting the deposit count, you need to retrieve the L1 info tree index to use in the claim-proof endpoint:

```bash
# Replace <DEPOSIT_COUNT> with deposit_count from bridge response
aggsandbox show l1-info-tree-index --network-id 1 --deposit-count <DEPOSIT_COUNT>
```

Example response:

```json
{
  "l1_info_tree_index": 0
}
```

**Key information from the response:**

- `l1_info_tree_index` - Use this value as the `leaf_index` parameter in the claim-proof endpoint

### Step 3: Prepare Token Metadata

Create the metadata needed for claiming the bridged tokens on L2.

```bash
METADATA=$(cast abi-encode "f(string,string,uint8)" "AggERC20" "AGGERC20" 18)
```

**Explanation**: This command encodes the token metadata (name, symbol, and decimals) into the format expected by the bridge. The metadata is used when claiming tokens on the destination chain to ensure the correct token properties are set.

### Step 4: Get Claim Proof

Before claiming assets, you need to get the proof data using the CLI command. Use the `l1_info_tree_index` value from the previous step as the `leaf_index` parameter:

```bash
# Replace <LEAF_INDEX> with l1_info_tree_index from previous response
# Replace <DEPOSIT_COUNT> with deposit_count from bridge response
aggsandbox show claim-proof --network-id 1 --leaf-index <LEAF_INDEX> --deposit-count <DEPOSIT_COUNT>
```

This will return the proof data including the `mainnet_exit_root` and `rollup_exit_root` needed for the claimAsset call. Example response:

```json
{
  "l1_info_tree_leaf": {
    "block_num": 7,
    "block_pos": 2,
    "l1_info_tree_index": 0,
    "previous_block_hash": "0x72b28944a2fb8e1122add9716376caad46750cc443d1b515570a5346316de27a",
    "timestamp": 1751237153,
    "mainnet_exit_root": "0x50b0cc5cad7791d8f04f43e13c74b4849b42497b1b17185e6641265c98daa686",
    "rollup_exit_root": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "global_exit_root": "0x5246c6ddb93c3075c3521042950639ba6fb01c7f6e92377bc0590fffef75025c",
    "hash": "0x105fbdbf7fb2642958334a88a4dab4af1d4981023a6dc62c4dd5f533ef81e068"
  }
}
```

**Key values to extract:**

- `mainnet_exit_root` - Use this as the merkle root in claimAsset
- `rollup_exit_root` - Use this as the nullifier in claimAsset

### Step 5: Claim Bridged Assets on L2

Claim the bridged tokens on the destination L2 chain using the proof generated by the bridge.

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L2 \
  "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
  <DEPOSIT_COUNT> \
  <MAINNET_EXIT_ROOT> \
  <ROLLUP_EXIT_ROOT> \
  1 \
  $AGG_ERC20_L1 \
  1101 \
  $ACCOUNT_ADDRESS_2 \
  10 \
  $METADATA \
  --private-key $PRIVATE_KEY_2 \
  --rpc-url $RPC_2 \
  --gas-limit 3000000
```

**Parameters:**

1. `<DEPOSIT_COUNT>` - Deposit count from bridge response
2. `<MAINNET_EXIT_ROOT>` - Mainnet exit root from claim proof
3. `<ROLLUP_EXIT_ROOT>` - Rollup exit root from claim proof
4. `1` - Origin network ID
5. `$AGG_ERC20_L1` - Token contract address
6. `1101` - Destination network ID
7. `$ACCOUNT_ADDRESS_2` - Recipient address
8. `10` - Amount to claim
9. `$METADATA` - Token metadata

#### Verify Claim Processing

After claiming assets, you can verify the claim was processed using the CLI command:

```bash
aggsandbox show claims --network-id 1101
```

This will return information about processed claims. Example response:

```json
{
  "claims": [
    {
      "block_num": 5,
      "block_timestamp": 1751237496,
      "tx_hash": "0xfdfba7deeea4945eaaea1a91d423f62c99714f30bde6f7c29fc64ff56695ddfe",
      "global_index": "1",
      "origin_address": "0x5FbDB2315678afecb367f032d93F642f64180aa3",
      "origin_network": 1,
      "destination_address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      "destination_network": 0,
      "amount": "10",
      "from_address": "0x5FbDB2315678afecb367f032d93F642f64180aa3",
      "mainnet_exit_root": "0x0000000000000000000000000000000000000000000000000000000000000000"
    }
  ],
  "count": 1
}
```

**Key information from the response:**

- `tx_hash` - The L2 transaction hash for the claim
- `amount` - The amount that was claimed
- `destination_address` - The address that received the tokens
- `block_timestamp` - When the claim was processed

### Step 6: Verify Token Balance

Fetch the events on L2 to retrieve the new TokenWrapped address:

```bash
aggsandbox events --network-id 1101
```

You'll see the following event:

```bash
⏰ Time: 2025-07-21 19:49:54 UTC
🧱 Block: 7
📄 Transaction: 0x2fc393c97d42fe4eff3e52b29fa62663eabc30cc913c6e98af4ef28530c8137b
📍 Contract: 0x5fbdb2315678afecb367f032d93f642f64180aa3
🎯 Event: NewWrappedToken(uint32,address,address,bytes)
  🪙 New Wrapped Token:
  🌐 Origin Network: 1
  📍 Origin Token: 0xdc64a140aa3e981100a9beca4e685f962f0cf6c9
  🎁 Wrapped Token: 0x19e2b7738a026883d08c3642984ab6d7510ca238 # <-- New token address (may be different for you)
```

Check that the tokens were successfully received on the destination chain:

```bash
# Set the wrapped token address from the event
export TOKENWRAPPED=0x19e2b7738a026883d08c3642984ab6d7510ca238

# Check balance
cast call $TOKENWRAPPED \
  "balanceOf(address)" \
  $ACCOUNT_ADDRESS_2 \
  --rpc-url $RPC_2
```

## Step 7: Bridge Back to L1

To complete the round-trip bridging process, you can bridge the tokens back from L2 to L1. This demonstrates the bidirectional nature of the bridge.

### Step 7a: Approve Bridge Contract on L2

First, approve the L2 bridge contract to spend your tokens:

```bash
cast send $TOKENWRAPPED \
  "approve(address,uint256)" \
  $POLYGON_ZKEVM_BRIDGE_L2 10 \
  --private-key $PRIVATE_KEY_2 \
  --rpc-url $RPC_2
```

**Explanation**: This approves the L2 bridge contract to spend 10 tokens from your L2 account. The token address `$TOKENWRAPPED` is the L2 representation of your original token.

### Step 7b: Bridge Assets from L2 to L1

Initiate the bridge back to L1:

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L2 \
  "bridgeAsset(uint32,address,uint256,address,bool,bytes)" \
  $CHAIN_ID_MAINNET $ACCOUNT_ADDRESS_1 10 $TOKENWRAPPED true 0x \
  --private-key $PRIVATE_KEY_2 \
  --rpc-url $RPC_2
```

**Parameters:**

1. `$CHAIN_ID_MAINNET` - Destination chain ID (1 for Ethereum mainnet)
2. `$ACCOUNT_ADDRESS_1` - Recipient address on L1 (your original account)
3. `10` - Amount of tokens to bridge
4. `$TOKENWRAPPED` - L2 token contract address
5. `true` - Force bridge flag
6. `0x` - Additional data (empty)

### Step 7c: Get Bridge Details and Proof for L2→L1

Follow the same process as before to get the claim proof:

```bash
# Get bridge details
aggsandbox show bridges --network-id 1101

# Get L1 info tree index
aggsandbox show l1-info-tree-index --network-id 1101 --deposit-count <DEPOSIT_COUNT>

# Get claim proof
aggsandbox show claim-proof --network-id 1101 --leaf-index <LEAF_INDEX> --deposit-count <DEPOSIT_COUNT>
```

### Step 7d: Claim Assets on L1

Claim the bridged tokens back on L1:

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L1 \
  "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
  <DEPOSIT_COUNT> \
  <MAINNET_EXIT_ROOT> \
  <ROLLUP_EXIT_ROOT> \
  1101 \
  $TOKENWRAPPED \
  1 \
  $ACCOUNT_ADDRESS_1 \
  10 \
  0x \
  --private-key $PRIVATE_KEY_1 \
  --rpc-url $RPC_1 \
  --gas-limit 3000000
```

**Parameter notes:**

- `1101` - Origin network ID (L2 chain ID)
- `1` - Destination network ID (L1 chain ID)
- The merkle root and nullifier values come from the claim-proof endpoint

### Step 7e: Verify Final Balance

Check that the tokens have been successfully bridged back:

```bash
cast call $AGG_ERC20_L1 \
  "balanceOf(address)" \
  $ACCOUNT_ADDRESS_1 \
  --rpc-url $RPC_1
```

**Explanation**: This verifies that your L1 account has received the tokens back, completing the round-trip bridge.

## Bridge and Call

### Bridge and Call Overview

**Warning**: The following commands assume that you restarted the aggsandbox with:

```bash
aggsandbox stop --volumes
aggsandbox start --detach
```

The "Bridge and Call" feature allows you to bridge assets and simultaneously execute a function call on the destination chain in a single transaction. This is useful for complex DeFi operations that require both asset transfer and contract interaction.

### Step-by-Step Bridge and Call Process

#### Step 1: Approve Bridge Extension Contract

First, approve the bridge extension contract to spend your tokens:

```bash
cast send $AGG_ERC20_L1 \
  "approve(address,uint256)" \
  $BRIDGE_EXTENSION_L1 100 \
  --private-key $PRIVATE_KEY_1 \
  --rpc-url $RPC_1
```

**Explanation**: This approves the bridge extension contract (different from the main bridge contract) to spend 100 tokens from your L1 account.

#### Step 2: Execute Bridge and Call

Bridge tokens and execute a function call on the destination chain:

```bash
# Encode the transfer call data (transfer 1 token to ACCOUNT_ADDRESS_1)
TRANSFER_DATA=$(cast calldata "transfer(address,uint256)" $ACCOUNT_ADDRESS_1 1)

# Get the precalculated L2 token address 
L2_TOKEN_ADDRESS=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
  "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
  1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
  --rpc-url $RPC_2 | sed 's/0x000000000000000000000000/0x/')

# Execute bridge and call
cast send $BRIDGE_EXTENSION_L1 \
  "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" \
  $AGG_ERC20_L1 10 $CHAIN_ID_AGGLAYER_1 $L2_TOKEN_ADDRESS $ACCOUNT_ADDRESS_2 $TRANSFER_DATA true \
  --private-key $PRIVATE_KEY_1 \
  --rpc-url $RPC_1
```

**Parameters:**

1. `$AGG_ERC20_L1` - Token contract address to bridge
2. `10` - Amount to bridge
3. `$CHAIN_ID_AGGLAYER_1` - Destination chain ID
4. `$L2_TOKEN_ADDRESS` - Target contract address on L2 (precalculated wrapped token address)
5. `$ACCOUNT_ADDRESS_2` - Fallback address if call fails
6. `$TRANSFER_DATA` - Encoded transfer function call
7. `true` - Force bridge flag

#### Step 3: Get Bridge Information and Claim Asset

**Important**: `bridgeAndCall` creates two bridge transactions:

1. **Asset Bridge**: Bridges tokens to a precalculated JumpPoint address (deposit_count = 0)
2. **Message Bridge**: Contains the call instructions for execution (deposit_count = 1)

**⚠️ Critical**: The asset bridge MUST be claimed first before the message can be processed.

##### Get Bridge Information

Check the bridges to get the deposit counts and proof data:

```bash
aggsandbox show bridges --network-id 1
```

Look for both bridge entries in the response. Note the `deposit_count` values:

- **First bridge entry** (asset): `deposit_count = 0`
- **Second bridge entry** (message): `deposit_count = 1`

##### Step 3a: Claim the Asset Bridge

First, claim the asset bridge to the JumpPoint address:

```bash
# Get L1 info tree index for asset bridge
aggsandbox show l1-info-tree-index --network-id 1 --deposit-count 0

# Get claim proof for asset bridge
aggsandbox show claim-proof --network-id 1 --leaf-index <L1_INFO_TREE_INDEX> --deposit-count 0

# Prepare token metadata for asset claim
METADATA=$(cast abi-encode "f(string,string,uint8)" "AggERC20" "AGGERC20" 18)

# Claim the asset bridge (replace placeholders with actual values from claim-proof response)
cast send $POLYGON_ZKEVM_BRIDGE_L2 \
  "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
  0 \
  <MAINNET_EXIT_ROOT_FROM_PROOF> \
  <ROLLUP_EXIT_ROOT_FROM_PROOF> \
  1 \
  $AGG_ERC20_L1 \
  $CHAIN_ID_AGGLAYER_1 \
  $ACCOUNT_ADDRESS_2 \
  10 \
  $METADATA \
  --private-key $PRIVATE_KEY_2 \
  --rpc-url $RPC_2 \
  --gas-limit 3000000
```

##### Step 3b: Get Message Bridge Claim Proof

Get the proof data for the message bridge:

```bash
# Get L1 info tree index for message bridge
aggsandbox show l1-info-tree-index --network-id 1 --deposit-count 1

# Get claim proof for message bridge
aggsandbox show claim-proof --network-id 1 --leaf-index <L1_INFO_TREE_INDEX_MESSAGE> --deposit-count 1
```

##### Step 3c: Claim the Message Bridge

Execute the message claim to trigger the automatic execution:

```bash
# Create the metadata for the bridge extension call
METADATA=$(cast abi-encode "f(uint256,address,address,uint32,address,bytes)" \
  0 $L2_TOKEN_ADDRESS $ACCOUNT_ADDRESS_2 1 $AGG_ERC20_L1 $TRANSFER_DATA)

# Claim the message bridge (replace placeholders with actual values from your environment)
cast send $POLYGON_ZKEVM_BRIDGE_L2 \
  "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
  1 \
  <MAINNET_EXIT_ROOT_FROM_MESSAGE_PROOF> \
  <ROLLUP_EXIT_ROOT_FROM_MESSAGE_PROOF> \
  1 \
  $BRIDGE_EXTENSION_L1 \
  $CHAIN_ID_AGGLAYER_1 \
  $BRIDGE_EXTENSION_L2 \
  0 \
  $METADATA \
  --private-key $PRIVATE_KEY_2 \
  --rpc-url $RPC_2 \
  --gas-limit 3000000
```

**Parameters:**

1. `1` - Global index for message bridge (deposit_count = 1)
2. `<MAINNET_EXIT_ROOT_FROM_MESSAGE_PROOF>` - Use the mainnet exit root from the message claim proof
3. `<ROLLUP_EXIT_ROOT_FROM_MESSAGE_PROOF>` - Use the rollup exit root from the message claim proof
4. `1` - Origin network (L1)
5. `$BRIDGE_EXTENSION_L1` - Origin address (Bridge Extension on L1)
6. `$CHAIN_ID_AGGLAYER_1` - Destination network
7. `$BRIDGE_EXTENSION_L2` - Destination address (Bridge Extension on L2)
8. `0` - Amount (no ether with message)
9. `$METADATA` - Encoded parameters containing:
   - `dependsOnIndex`: The asset bridge deposit count (0)
   - `callAddress`: Target contract address (L2 token)
   - `fallbackAddress`: Fallback address if call fails
   - `assetOriginalNetwork`: Original asset network (1)
   - `assetOriginalAddress`: Original asset address (L1 token)
   - `callData`: Encoded function call to execute

#### What Happens During Execution

When you claim the message bridge, the BridgeExtension on L2:

1. **Validates the claim**: Ensures the corresponding asset was claimed first
2. **Deploys JumpPoint**: Creates a temporary contract using CREATE2
3. **Executes the call**: JumpPoint transfers assets and executes your function call
4. **Handles fallback**: If the call fails, assets go to the fallback address

### Use Cases for Bridge and Call

This feature is particularly useful for:

- **DeFi Operations**: Bridge tokens and immediately deposit them into a lending protocol
- **DEX Interactions**: Bridge tokens and execute a swap on the destination chain
- **Yield Farming**: Bridge tokens and stake them in a yield farm in a single transaction
- **Cross-Chain Governance**: Bridge governance tokens and immediately vote on proposals

#### Monitoring Execution

You can monitor the execution by checking bridge events:

```bash
# Get bridge details to see both asset and message bridges
aggsandbox show bridges --network-id 1

# Check claims to see execution status
aggsandbox show claims --network-id 1101
```

The system creates two bridge events:

- **Asset Bridge**: Bridges tokens to JumpPoint address (should remain unclaimed)
- **Message Bridge**: Contains the call instructions (requires manual claiming)

**Important**: In production, the asset bridge remains unclaimed until JumpPoint claims it. The message bridge requires manual claiming to trigger the BridgeExtension flow. In sandbox mode, auto-claiming may interfere with this process.

## Manual Claiming (Advanced)

**Warning**: Manual claiming of bridge messages should only be used for regular bridge operations, NOT for BridgeExtension messages created by `bridgeAndCall`.

### Understanding Message Types

There are two types of bridge messages:

1. **Regular Bridge Messages**: Created by `bridgeMessage()` calls - these CAN be manually claimed
2. **BridgeExtension Messages**: Created by `bridgeAndCall()` - these should execute automatically

### When Manual Claiming Fails

If you try to manually claim a BridgeExtension message using `claimMessage()`, you'll get a `MessageFailed()` error (0x37e391c3) because:

1. **Wrong Context**: The message was designed for automatic execution through the BridgeExtension system
2. **onMessageReceived**: The call triggers `onMessageReceived()` in BridgeExtension, which expects specific conditions
3. **CREATE2 Deployment**: The function tries to deploy a JumpPoint contract, which may fail for various reasons

### Correct Manual Claiming (Regular Messages Only)

For regular bridge messages (not from `bridgeAndCall`), you can claim manually:

```bash
# Example: Claiming a regular bridge message
cast send $POLYGON_ZKEVM_BRIDGE_L2 \
  "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
  1 \
  0x6974b4e71fdf57bb87aca8d85ce07a6eb1269064076c25476226fc1b7182076c \
  0x0000000000000000000000000000000000000000000000000000000000000000 \
  1 \
  $BRIDGE_EXTENSION_L1 \
  $CHAIN_ID_AGGLAYER_1 \
  $BRIDGE_EXTENSION_L2 \
  0 \
  $METADATA \
  --private-key $PRIVATE_KEY_2 \
  --rpc-url $RPC_2 \
  --gas-limit 3000000
```

### Key Differences

| Message Type | Creation Method | Claiming Method | Destination |
|--------------|----------------|-----------------|-------------|
| Regular Bridge | `bridgeMessage()` | Manual `claimMessage()` | Any address |
| BridgeExtension | `bridgeAndCall()` | Automatic execution | BridgeExtension address |

### Troubleshooting Manual Claims

If manual claiming fails with `MessageFailed()`:

1. **Check Message Origin**: Is it from `bridgeAndCall()`? If yes, don't claim manually
2. **Verify Parameters**: Ensure all parameters match the original bridge event
3. **Check Dependencies**: For BridgeExtension messages, ensure dependent assets are claimed first
4. **Monitor Automatic Execution**: BridgeExtension messages should execute automatically when ready

## AggKit API Endpoints

The AggKit provides REST API endpoints for interacting with the bridge system and retrieving bridge-related information. These can be accessed directly via HTTP or through the CLI wrapper commands.

### CLI Commands

```bash
# Get available bridges
aggsandbox show bridges --network-id 1

# Get L1 info tree index
aggsandbox show l1-info-tree-index --network-id 1 --deposit-count 0

# Get claim proof
aggsandbox show claim-proof --network-id 1 --leaf-index 0 --deposit-count 1

# Get claims
aggsandbox show claims --network-id 1101
```

### Direct API Endpoints

For direct HTTP access to the underlying endpoints:

#### Get Available Bridges

```url
http://localhost:5577/bridge/v1/bridges?network_id=1
```

**Explanation**: This endpoint returns information about available bridges for the specified network ID. It provides details about bridge contracts, supported tokens, and bridge configurations.

#### L1 Info Tree Index

```url
http://localhost:5577/bridge/v1/l1-info-tree-index?network_id=1&deposit_count=0
```

**Explanation**: This endpoint retrieves the L1 info tree index for a given deposit count. The returned index is used as the `leaf_index` parameter in the claim-proof endpoint to generate the correct merkle proof for claiming bridged assets.

#### Get Claim Proof

```url
http://localhost:5577/bridge/v1/claim-proof?network_id=1&leaf_index=0&deposit_count=1
```

**Explanation**: This endpoint generates the merkle proof needed to claim bridged assets. The proof is required to verify that the bridge transaction was included in the bridge's merkle tree and is used in the `claimAsset` function.

#### Get Claims

```url
http://localhost:5577/bridge/v1/claims?network_id=1101
```

**Explanation**: This endpoint returns a list of all claims (successful bridge transactions) for the specified network ID. It can be used to track bridge activity and verify the status of bridge transactions.

## Troubleshooting

### Common Issues

#### Transaction Failures

**Gas limit too low**

```bash
# Solution: Increase gas limit for complex operations
--gas-limit 3000000
```

**Insufficient balance**

```bash
# Check token balance before bridging
cast call $AGG_ERC20_L1 "balanceOf(address)" $ACCOUNT_ADDRESS_1 --rpc-url $RPC_1
```

**Approval not set**

```bash
# Verify bridge contract is approved to spend tokens
cast call $AGG_ERC20_L1 "allowance(address,address)" $ACCOUNT_ADDRESS_1 $POLYGON_ZKEVM_BRIDGE_L1 --rpc-url $RPC_1
```

#### Bridge State Issues

**Deposit count mismatch**

```bash
# Use aggsandbox to get correct deposit count
DEPOSIT_COUNT=$(aggsandbox show bridges --network-id 1 | jq -r '.bridges[0].deposit_count')
```

**Invalid proof data**

```bash
# Regenerate proof using correct leaf index and deposit count
aggsandbox show claim-proof --network-id 1 --leaf-index <LEAF_INDEX> --deposit-count <DEPOSIT_COUNT>
```

**Timing issues**

```bash
# Wait for block confirmations before claiming
cast block-number --rpc-url $RPC_1
cast block-number --rpc-url $RPC_2
```

#### Environment Problems

```bash
# Debug environment variables
echo "L1 RPC: $RPC_1"
echo "L2 RPC: $RPC_2"
echo "Bridge L1: $POLYGON_ZKEVM_BRIDGE_L1"
echo "Bridge L2: $POLYGON_ZKEVM_BRIDGE_L2"

# Test RPC connectivity
cast block-number --rpc-url $RPC_1 && echo "✅ L1 RPC connected"
cast block-number --rpc-url $RPC_2 && echo "✅ L2 RPC connected"
```

#### Bridge and Call Issues

**Asset bridge not claimed first**

```bash
# Ensure asset bridge (deposit_count = 0) is claimed before message bridge (deposit_count = 1)
aggsandbox show claims --network-id 1101
```

**JumpPoint deployment fails**

```bash
# Check that the precalculated address calculation is correct
L2_TOKEN_ADDRESS=$(cast call $POLYGON_ZKEVM_BRIDGE_L2 \
  "precalculatedWrapperAddress(uint32,address,string,string,uint8)" \
  1 $AGG_ERC20_L1 "AggERC20" "AGGERC20" 18 \
  --rpc-url $RPC_2)
```

### Error Codes

| Error Code | Description | Solution |
|------------|-------------|----------|
| `0x37e391c3` | MessageFailed | Don't manually claim BridgeExtension messages |
| Gas estimation failed | Insufficient gas | Increase `--gas-limit` parameter |
| Insufficient balance | Not enough tokens | Check balance and approve more tokens |
| Invalid proof | Wrong merkle proof | Regenerate proof with correct parameters |

### Getting Help

If you encounter issues not covered in this guide:

1. Check the Agglayer documentation
2. Verify all environment variables are correctly set
3. Ensure you're using the latest version of the CLI tools
4. Check that the sandbox is running and accessible
