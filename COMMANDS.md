# End to end bridging flow

This document provides the commands and endpoints needed to perform a complete asset bridging flow from L1 to L2 using the Polygon ZkEVM bridge.

## Prerequisites

Before starting the bridging process, make sure to start the agg-sandbox environment and source your environment variables:

```bash
cp .env.example .env
```

```bash
aggsandbox start --detach
```

```bash
source .env
```

This ensures all the required environment variables (like `$AGG_ERC20_L1`, `$POLYGON_ZKEVM_BRIDGE_L1`, `$PRIVATE_KEY_1`, etc.) are available in your shell session.

## Step 1: Approve Bridge Contract to Spend Tokens

Before bridging assets, you need to approve the bridge contract to spend your tokens on your behalf.

```bash
cast call $AGG_ERC20_L1 "balanceOf(address)" $ACCOUNT_ADDRESS_1 --rpc-url $RPC_1
```

```bash
cast send $AGG_ERC20_L1 "approve(address,uint256)" $POLYGON_ZKEVM_BRIDGE_L1 100 --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1
```

**Explanation**: This command approves the Polygon ZkEVM bridge contract on L1 to spend 100 tokens from your account. The `approve` function is a standard ERC20 function that allows another contract (the bridge) to transfer tokens on your behalf.

## Step 2: Bridge Assets from L1 to L2

Initiate the bridging process by calling the bridge contract to transfer assets to the destination chain.

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L1 "bridgeAsset(uint32,address,uint256,address,bool,bytes)" $CHAIN_ID_AGGLAYER_1 $ACCOUNT_ADDRESS_2 10 $AGG_ERC20_L1 true 0x --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1
```

```bash
cast call $AGG_ERC20_L1 "balanceOf(address)" $ACCOUNT_ADDRESS_1 --rpc-url $RPC_1
```

**Explanation**: This command initiates the bridging of 10 tokens from L1 to the destination L2 chain. The parameters are:

- `$CHAIN_ID_AGGLAYER_1`: The destination chain ID
- `$ACCOUNT_ADDRESS_2`: The recipient address on L2
- `10`: Amount of tokens to bridge
- `$AGG_ERC20_L1`: The token contract address
- `true`: Whether to force the bridge (bypass some checks)
- `0x`: Additional data (empty in this case)

### Check Bridge Details

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
      "bridge_hash": "0xbdc21b1ceb90347c5fe8abdbbf8996fe062241770ea5bcf8b4e654e79848144b",
  ],
}
```

**Key information from the response:**

- `deposit_count`: Use this value in the claimAsset command
- `bridge_hash`: The unique identifier for this bridge transaction
- `tx_hash`: The transaction hash on L1
- `sandbox_metadata`: Indicates this is running in sandbox mode with instant claims enabled

### Get L1 Info Tree Index

After getting the deposit count, you need to retrieve the L1 info tree index to use in the claim-proof endpoint:

```bash
aggsandbox show l1-info-tree-index --network-id 1 --deposit-count 0
```

This will return the L1 info tree index data. Example response:

```json
{
  "l1_info_tree_index": 0
}
```

**Key information from the response:**

- `l1_info_tree_index`: Use this value as the `leaf_index` parameter in the claim-proof endpoint

## Step 3: Prepare Token Metadata

Create the metadata needed for claiming the bridged tokens on L2.

```bash
METADATA=$(cast abi-encode "f(string,string,uint8)" "AggERC20" "AGGERC20" 18)
```

**Explanation**: This command encodes the token metadata (name, symbol, and decimals) into the format expected by the bridge. The metadata is used when claiming tokens on the destination chain to ensure the correct token properties are set.

## Step 4: Get Claim Proof

Before claiming assets, you need to get the proof data using the CLI command. Use the `l1_info_tree_index` value from the previous step as the `leaf_index` parameter:

```bash
aggsandbox show claim-proof --network-id 1 --leaf-index 0 --deposit-count 1
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
    "hash": "0x105fbdbf7fb2642958334a88a4dab4af1d4981023a6dc62c4dd5f533ef81e068",
  }
}
```

**Key values to extract:**

- `mainnet_exit_root`: Use this as the merkle root in claimAsset
- `rollup_exit_root`: Use this as the nullifier in claimAsset

## Step 5: Claim Bridged Assets on L2

Claim the bridged tokens on the destination L2 chain using the proof generated by the bridge.

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L2 "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" 1 0x50b0cc5cad7791d8f04f43e13c74b4849b42497b1b17185e6641265c98daa686 0x0000000000000000000000000000000000000000000000000000000000000000 1 $AGG_ERC20_L1 1101 $ACCOUNT_ADDRESS_2 10 $METADATA --private-key $PRIVATE_KEY_2 --rpc-url $RPC_2   --gas-limit 3000000
```

**Explanation**: This command claims the bridged tokens on L2. The parameters include:

- `1`: Deposit count (unique identifier for the bridge transaction)
- `0x50b0cc5cad7791d8f04f43e13c74b4849b42497b1b17185e6641265c98daa686`: Merkle root for the proof
- `0x0000000000000000000000000000000000000000000000000000000000000000`: Nullifier (prevents double-spending)
- `1`: Origin network ID
- `$AGG_ERC20_L1`: Token address
- `1101`: Destination network ID
- `$ACCOUNT_ADDRESS_2`: Recipient address
- `10`: Amount to claim
- `$METADATA`: Token metadata
- `--gas-limit 3000000`: Higher gas limit for complex claim operation

### Verify Claim Processing

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

- `tx_hash`: The L2 transaction hash for the claim
- `amount`: The amount that was claimed
- `destination_address`: The address that received the tokens
- `block_timestamp`: When the claim was processed

## Step 6: Verify Token Balance

Check that the tokens were successfully received on the destination chain.

```bash
cast call 0xe806a11ebf128faa3d1a3aa94c2db46c5f1b60b4 "balanceOf(address)" $ACCOUNT_ADDRESS_2 --rpc-url $RPC_2
```

## Step 7: Bridge Back to L1

To complete the round-trip bridging process, you can bridge the tokens back from L2 to L1. This demonstrates the bidirectional nature of the bridge.

### Step 7a: Approve Bridge Contract on L2

First, approve the L2 bridge contract to spend your tokens:

```bash
cast send 0xe806a11ebf128faa3d1a3aa94c2db46c5f1b60b4 "approve(address,uint256)" $POLYGON_ZKEVM_BRIDGE_L2 10 --private-key $PRIVATE_KEY_2 --rpc-url $RPC_2
```

**Explanation**: This approves the L2 bridge contract to spend 10 tokens from your L2 account. The token address `0xe806a11ebf128faa3d1a3aa94c2db46c5f1b60b4` is the L2 representation of your original token.

### Step 7b: Bridge Assets from L2 to L1

Initiate the bridge back to L1:

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L2 "bridgeAsset(uint32,address,uint256,address,bool,bytes)" $CHAIN_ID_MAINNET $ACCOUNT_ADDRESS_1 10 0xe806a11ebf128faa3d1a3aa94c2db46c5f1b60b4 true 0x --private-key $PRIVATE_KEY_2 --rpc-url $RPC_2
```

**Explanation**: This bridges 10 tokens from L2 back to L1. The parameters are:

- `$CHAIN_ID_MAINNET`: Destination chain ID (1 for Ethereum mainnet)
- `$ACCOUNT_ADDRESS_1`: Recipient address on L1 (your original account)
- `10`: Amount of tokens to bridge
- `0xe806a11ebf128faa3d1a3aa94c2db46c5f1b60b4`: L2 token contract address
- `true`: Force bridge flag
- `0x`: Additional data (empty)

### Step 7c: Get Bridge Details and Proof for L2→L1

Follow the same process as before to get the claim proof:

```bash
# Get bridge details
aggsandbox show bridges --network-id 1101

# Get L1 info tree index (use deposit_count from bridge details)
aggsandbox show l1-info-tree-index --network-id 1101 --deposit-count 1

# Get claim proof (use l1_info_tree_index as leaf_index)
aggsandbox show claim-proof --network-id 1101 --leaf-index 0 --deposit-count 1
```

### Step 7d: Claim Assets on L1

Claim the bridged tokens back on L1:

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L1 "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" 1 0x50b0cc5cad7791d8f04f43e13c74b4849b42497b1b17185e6641265c98daa686 0x0000000000000000000000000000000000000000000000000000000000000000 1101 $AGG_ERC20_L1 1 $ACCOUNT_ADDRESS_1 10 0x --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1 --gas-limit 3000000
```

**Explanation**: This claims the tokens back on L1. Note the parameter changes:

- `1101`: Origin network ID (L2 chain ID)
- `1`: Destination network ID (L1 chain ID)
- The merkle root and nullifier values come from the claim-proof endpoint

### Step 7e: Verify Final Balance

Check that the tokens have been successfully bridged back:

```bash
cast call $AGG_ERC20_L1 "balanceOf(address)" $ACCOUNT_ADDRESS_1 --rpc-url $RPC_1
```

**Explanation**: This verifies that your L1 account has received the tokens back, completing the round-trip bridge.

**Explanation**: This command checks the token balance of the recipient address on L2 to confirm the bridging was successful. The `balanceOf` function is a standard ERC20 function that returns the token balance for a given address.

## Bridge and Call

The "Bridge and Call" feature allows you to bridge assets and simultaneously execute a function call on the destination chain. This is useful for complex DeFi operations that require both asset transfer and contract interaction.

### Step 1: Approve Bridge Extension Contract

First, approve the bridge extension contract to spend your tokens:

```bash
cast send $AGG_ERC20_L1 "approve(address,uint256)" $BRIDGE_EXTENSION_L1 100 --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1
```

**Explanation**: This approves the bridge extension contract (different from the main bridge contract) to spend 100 tokens from your L1 account. The bridge extension provides additional functionality like `bridgeAndCall`.

### Step 2: Execute Bridge and Call

Perform the bridge and call operation:

```bash
cast send $BRIDGE_EXTENSION_L1 "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" $AGG_ERC20_L1 10 $CHAIN_ID_AGGLAYER_1 $ACCOUNT_ADDRESS_2 $ACCOUNT_ADDRESS_2 0x true --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1 --value 0
```

**Explanation**: This command bridges 10 tokens and executes a call on the destination chain. The parameters are:

- `$AGG_ERC20_L1`: Token contract address to bridge
- `10`: Amount of tokens to bridge
- `$CHAIN_ID_AGGLAYER_1`: Destination chain ID
- `$ACCOUNT_ADDRESS_2`: Recipient address on L2
- `$ACCOUNT_ADDRESS_2`: Contract address to call on L2 (same as recipient in this example)
- `0x`: Calldata for the function call (empty in this example)
- `true`: Force bridge flag
- `--value 0`: No ETH value sent with the transaction

### Use Cases for Bridge and Call

This feature is particularly useful for:

- **DeFi Operations**: Bridge tokens and immediately deposit them into a lending protocol
- **DEX Interactions**: Bridge tokens and execute a swap on the destination chain
- **Yield Farming**: Bridge tokens and stake them in a yield farm in a single transaction
- **Cross-Chain Governance**: Bridge governance tokens and immediately vote on proposals

### Example: Bridge and Deposit to Lending Protocol

```bash
# Encode the deposit function call
DEPOSIT_CALLDATA=$(cast abi-encode "deposit(address,uint256)" $AGG_ERC20_L2 10)

# Execute bridge and call with deposit
cast send $BRIDGE_EXTENSION_L1 "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" $AGG_ERC20_L1 10 $CHAIN_ID_AGGLAYER_1 $ACCOUNT_ADDRESS_2 $LENDING_PROTOCOL_L2 $DEPOSIT_CALLDATA true --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1 --value 0
```

**Explanation**: This example shows how to bridge tokens and immediately deposit them into a lending protocol on the destination chain, all in a single transaction.

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
