# End to end bridging flow

This document provides the commands and endpoints needed to perform a complete asset bridging flow from L1 to L2 using the Polygon ZkEVM bridge.

## Prerequisites

Before starting the bridging process, make sure to start the agg-sandbox environment and source your environment variables:

```bash
cp .env.example .env
```

```bash
agg-sandbox start --detach
```

```bash
source .env
```

This ensures all the required environment variables (like `$ERC20`, `$POLYGON_ZKEVM_BRIDGE_L1`, `$PRIVATE_KEY_1`, etc.) are available in your shell session.

## Step 1: Approve Bridge Contract to Spend Tokens

Before bridging assets, you need to approve the bridge contract to spend your tokens on your behalf.

```bash
cast send $ERC20 "approve(address,uint256)" $POLYGON_ZKEVM_BRIDGE_L1 100 --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1
```

**Explanation**: This command approves the Polygon ZkEVM bridge contract on L1 to spend 100 tokens from your account. The `approve` function is a standard ERC20 function that allows another contract (the bridge) to transfer tokens on your behalf.

## Step 2: Bridge Assets from L1 to L2

Initiate the bridging process by calling the bridge contract to transfer assets to the destination chain.

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L1 "bridgeAsset(uint32,address,uint256,address,bool,bytes)" $CHAIN_ID_AGGLAYER_1 $ACCOUNT_ADDRESS_2 10 $ERC20 true 0x --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1
```

**Explanation**: This command initiates the bridging of 10 tokens from L1 to the destination L2 chain. The parameters are:

- `$CHAIN_ID_AGGLAYER_1`: The destination chain ID
- `$ACCOUNT_ADDRESS_2`: The recipient address on L2
- `10`: Amount of tokens to bridge
- `$ERC20`: The token contract address
- `true`: Whether to force the bridge (bypass some checks)
- `0x`: Additional data (empty in this case)

### Check Bridge Details

After initiating the bridge, you can check the bridge details using the AggKit endpoint:

```bash
curl "http://localhost:5577/bridge/v1/bridges?network_id=1"
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

## Step 3: Prepare Token Metadata

Create the metadata needed for claiming the bridged tokens on L2.

```bash
METADATA=$(cast abi-encode "f(string,string,uint8)" "AggERC20" "AGGERC20" 18)
```

**Explanation**: This command encodes the token metadata (name, symbol, and decimals) into the format expected by the bridge. The metadata is used when claiming tokens on the destination chain to ensure the correct token properties are set.

## Step 4: Get Claim Proof

Before claiming assets, you need to get the proof data from the AggKit endpoint:

```bash
curl "http://localhost:5577/bridge/v1/claim-proof?network_id=1&leaf_index=0&deposit_count=1"
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
cast send $POLYGON_ZKEVM_BRIDGE_L2 "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" 1 0x50b0cc5cad7791d8f04f43e13c74b4849b42497b1b17185e6641265c98daa686   0x0000000000000000000000000000000000000000000000000000000000000000 1 $ERC20 1101 $ACCOUNT_ADDRESS_2 10 $METADATA --private-key $PRIVATE_KEY_2 --rpc-url $RPC_2   --gas-limit 3000000
```

**Explanation**: This command claims the bridged tokens on L2. The parameters include:

- `1`: Deposit count (unique identifier for the bridge transaction)
- `0x50b0cc5cad7791d8f04f43e13c74b4849b42497b1b17185e6641265c98daa686`: Merkle root for the proof
- `0x0000000000000000000000000000000000000000000000000000000000000000`: Nullifier (prevents double-spending)
- `1`: Origin network ID
- `$ERC20`: Token address
- `1101`: Destination network ID
- `$ACCOUNT_ADDRESS_2`: Recipient address
- `10`: Amount to claim
- `$METADATA`: Token metadata
- `--gas-limit 3000000`: Higher gas limit for complex claim operation

### Verify Claim Processing

After claiming assets, you can verify the claim was processed by querying the claims endpoint:

```bash
curl "http://localhost:5577/bridge/v1/claims?network_id=1101"
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

**Explanation**: This command checks the token balance of the recipient address on L2 to confirm the bridging was successful. The `balanceOf` function is a standard ERC20 function that returns the token balance for a given address.

## Brigde and Call

```bash
cast send $ERC20 "approve(address,uint256)" $BRIDGE_EXTENSION 100 --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1
```

```bash
cast send $BRIDGE_EXTENSION "bridgeAndCall(address,uint256,uint32,address,address,bytes,bool)" $ERC20 10 $CHAIN_ID_AGGLAYER_1 $ACCOUNT_ADDRESS_2 $ACCOUNT_ADDRESS_2 0x true --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1 --value 0
```

## AggKit API Endpoints

The AggKit provides REST API endpoints for interacting with the bridge system and retrieving bridge-related information.

### Get Available Bridges

```url
http://localhost:5577/bridge/v1/bridges?network_id=1
```

**Explanation**: This endpoint returns information about available bridges for the specified network ID. It provides details about bridge contracts, supported tokens, and bridge configurations.

### Get L1 Info Leaf

```url
http://localhost:5577/bridge/v1/injected-l1-info-leaf?network_id=1&leaf_index=0
```

**Explanation**: This endpoint retrieves L1 information leaf data from the bridge's merkle tree. The leaf index specifies which leaf to retrieve, and this data is used in the bridging process for verification and proof generation.

### Get Claim Proof

```url
http://localhost:5577/bridge/v1/claim-proof?network_id=1&leaf_index=0&deposit_count=1
```

**Explanation**: This endpoint generates the merkle proof needed to claim bridged assets. The proof is required to verify that the bridge transaction was included in the bridge's merkle tree and is used in the `claimAsset` function.

### Get Claims

```url
http://localhost:5577/bridge/v1/claims?network_id=1101
```

**Explanation**: This endpoint returns a list of all claims (successful bridge transactions) for the specified network ID. It can be used to track bridge activity and verify the status of bridge transactions.
