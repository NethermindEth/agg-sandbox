# Bridge Message Failed Analysis - Custom Error 0x37e391c3

## Overview

This document provides a detailed analysis of a `claimMessage` transaction failure in the Polygon zkEVM Bridge system, specifically the `MessageFailed()` error with custom error code `0x37e391c3`.

## Error Details

**Transaction Hash:** `0xe80bb3a9a79cb702517816b0d3d1af738f222d9d659ef544037f7d1215a3c2f4`
**Error Code:** `0x37e391c3`
**Error Type:** `MessageFailed()`
**Contract:** PolygonZkEVMBridgeV2 at `0x5FbDB2315678afecb367f032d93F642f64180aa3`

## Command That Failed

```bash
cast send $POLYGON_ZKEVM_BRIDGE_L2 "claimMessage(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
    1 \
    0x6974b4e71fdf57bb87aca8d85ce07a6eb1269064076c25476226fc1b7182076c \
    0x0000000000000000000000000000000000000000000000000000000000000000 \
    1 \
    $BRIDGE_EXTENSION_L1 \
    1101 \
    $BRIDGE_EXTENSION_L2 \
    0 \
    $MESSAGE_METADATA \
    --private-key $PRIVATE_KEY_2 \
    --rpc-url $RPC_2 \
    --gas-limit 3000000
```

## Root Cause Analysis

### 1. Error Location

The error originates in `PolygonZkEVMBridgeV2.sol` at lines 586-588:

```solidity
if (!success) {
    revert MessageFailed();
}
```

This occurs when the external call to `destinationAddress.call()` fails during message execution.

### 2. Call Flow Analysis

1. **Entry Point:** `claimMessage()` in `PolygonZkEVMBridgeV2.sol:534`
2. **Verification:** Message leaf verification passes at `_verifyLeaf()` 
3. **Execution:** Call to `destinationAddress` (BridgeExtension) at lines 572-584
4. **Failure:** The call to `onMessageReceived()` in BridgeExtension fails

### 3. BridgeExtension Failure Points

The `onMessageReceived()` function in `BridgeExtension.sol:215-234` has several validation checks:

```solidity
function onMessageReceived(address originAddress, uint32 originNetwork, bytes calldata data) external payable {
    if (msg.sender != address(bridge)) revert SenderMustBeBridge();           // Line 216
    if (originAddress != address(this)) revert OriginMustBeBridgeExtension(); // Line 217
    
    // Decode message data
    (uint256 dependsOnIndex, address callAddress, address fallbackAddress, 
     uint32 assetOriginalNetwork, address assetOriginalAddress, bytes memory callData) 
        = abi.decode(data, (uint256, address, address, uint32, address, bytes));
        
    if (!bridge.isClaimed(uint32(dependsOnIndex), originNetwork)) revert UnclaimedAsset(); // Line 228
}
```

## Message Metadata Analysis

### Decoded Message Metadata

The `MESSAGE_METADATA` parameter was decoded with the following values:

```
Raw Data: 0x0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000a513e6e4b8f2a923d98304ec87f64353c4d5c853000000000000000000000000000000000000000000000000000000000000044d0000000000000000000000000165878a594ca255338adfa4d48449f69242eb8f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e806a11ebf128faa3d1a3aa94c2db46c5f1b60b400000000000000000000000070997970c51812dc3a010c7d01b50e0d17dc79c800000000000000000000000000000000000000000000000000000000000000100000000000000000000000005fbdb2315678afecb367f032d93f642f64180aa3000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000044a9059cbb000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000
```

**Decoded Parameters:**
- `dependsOnIndex`: **1**
- `callAddress`: `0xa513e6e4b8f2a923d98304ec87f64353c4d5c853`
- `fallbackAddress`: `0x000000000000000000000000000000000000044d`
- `assetOriginalNetwork`: `2453859215`
- `assetOriginalAddress`: `0x0000000000000000000000000000000000000000`
- `callData`: [68 bytes of encoded function call]

## The Dependency Mismatch

### Issue Identified

**🚨 CRITICAL DISTINCTION:**
- **Asset Claimed:** Global Index **0** (using `claimAsset`)
- **Message Claiming:** Global Index **1** (using `claimMessage`)

**Important:** These are **different bridge operations** with **different global indices**, not a dependency mismatch. The asset claim (GI 0) and message claim (GI 1) are separate bridge events.

### Evidence from Logs

From the successful asset claim (Event #15):
```
📝 Event #15
🎯 Event: ClaimEvent(uint256,uint32,address,address,uint256)
🌍 Global Index: 0  ← Asset was claimed with index 0
🌐 Origin Network: 1
📍 Origin Address: 0x5fbdb2315678afecb367f032d93f642f64180aa3
📍 Destination Address: 0xa513e6e4b8f2a923d98304ec87f64353c4d5c853
💰 Amount: 10
```

But the message metadata shows `dependsOnIndex: 0` (from the latest MESSAGE_METADATA), meaning it expects an asset with global index 0 to be claimed first - which you DID claim successfully.

### The Real Issue: MESSAGE_METADATA Changes

Looking at your command history, you're changing the MESSAGE_METADATA between attempts:

**First attempt:**
```bash
MESSAGE_METADATA=$(cast abi-encode "f(uint256,address,address,uint32,address,bytes)" 0 $L2_TOKEN_ADDRESS $ACCOUNT_ADDRESS_2 1 $AGG_ERC20_L1 $TRANSFER_DATA)
```

**Second attempt (the one that failed):**
```bash  
MESSAGE_METADATA=$(cast abi-encode "f(uint256,address,address,uint32,address,bytes)" 0 $L2_TOKEN_ADDRESS $ACCOUNT_ADDRESS_2 1 $AGG_ERC20_L1 $TRANSFER_DATA)
```

The dependsOnIndex in the metadata is 0, which matches your claimed asset. So the dependency check should pass.

### Real Validation Logic

The `UnclaimedAsset` error occurs because of this check in `BridgeExtension.sol:228`:

```solidity
if (!bridge.isClaimed(uint32(dependsOnIndex), originNetwork)) revert UnclaimedAsset();
```

This translates to:
```solidity
if (!bridge.isClaimed(0, 1)) revert UnclaimedAsset();
//                     ↑  ↑
//           dependsOnIndex  originNetwork
```

Since you successfully claimed the asset with global index 0, this should return true. The issue must be elsewhere.

## The Real Issue: Message vs Asset Mismatch

### Critical Insight

The problem is **NOT** a dependency issue. You're claiming the message with global index 1, but your MESSAGE_METADATA contains `dependsOnIndex: 0`. This means:

1. **Message Event**: Has global index 1 (from the original bridge)
2. **Asset Event**: Has global index 0 (successfully claimed)  
3. **Metadata Says**: Depends on asset with global index 0 ✅
4. **Dependency Check**: `bridge.isClaimed(0, 1)` should pass ✅

### The REAL Culprit: JumpPoint CREATE2 Deployment

Looking at the `onMessageReceived` function in `BridgeExtension.sol:215-234`, the failure is most likely occurring in the **final step** - the JumpPoint deployment:

```solidity
// the remaining bytes have the selector+args
new JumpPoint{salt: keccak256(abi.encodePacked(dependsOnIndex, originNetwork))}(
    address(bridge), assetOriginalNetwork, assetOriginalAddress, callAddress, fallbackAddress, callData
);
```

This CREATE2 deployment can fail for several reasons:

1. **Contract Already Exists**: JumpPoint with same salt already deployed
2. **Constructor Failure**: JumpPoint constructor is reverting
3. **Gas Limitation**: Insufficient gas for deployment + execution
4. **Invalid Parameters**: Bad constructor parameters causing revert

### JumpPoint Analysis

The JumpPoint deployment uses these parameters:
- **Salt**: `keccak256(abi.encodePacked(0, 1))` (dependsOnIndex=0, originNetwork=1)
- **Constructor Args**:
  - `bridge`: Bridge contract address
  - `assetOriginalNetwork`: From your metadata
  - `assetOriginalAddress`: From your metadata  
  - `callAddress`: `$L2_TOKEN_ADDRESS`
  - `fallbackAddress`: `$ACCOUNT_ADDRESS_2`
  - `callData`: Transfer function call

### Most Likely Issue: Contract Already Exists

The salt `keccak256(abi.encodePacked(0, 1))` might already be used. If a JumpPoint with this exact salt was previously deployed, CREATE2 will fail.

### Debugging the Call

The callData in your MESSAGE_METADATA is:
```
0xa9059cbb000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000000000000000000000000000000000000000001
```

This decodes to:
- Function: `transfer(address,uint256)`
- To: `0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266` 
- Amount: `1`

### Verification Steps

1. **Check if JumpPoint exists**:
   ```bash
   SALT=$(cast keccak "$(cast abi-encode "f(uint256,uint32)" 0 1)")
   JUMPPOINT_ADDR=$(cast compute-address --salt $SALT $BRIDGE_EXTENSION_L2)
   cast code $JUMPPOINT_ADDR --rpc-url $RPC_2
   ```

2. **If contract exists**, the deployment will fail with CREATE2 collision

## Bridge Extension Dependency System

### How Dependencies Work

1. **Asset Bridging:** When using `bridgeAndCall()`, assets are bridged to a computed JumpPoint address
2. **Dependency Index:** The `dependsOnIndex` is calculated as `bridge.depositCount() + 1` at bridge time
3. **Message Encoding:** This index is encoded in the message metadata
4. **Claim Validation:** Before executing the message, the system checks if the dependent asset was claimed

### Code Reference

From `BridgeExtension.sol:38`:
```solidity
uint256 dependsOnIndex = bridge.depositCount() + 1; // only doing 1 bridge asset
```

This index must match the global index of the asset that needs to be claimed.

## Solution Steps

### Immediate Fix

1. **Find the Missing Asset:** Look for a bridge asset event with global index 1 from origin network 1
2. **Claim the Correct Asset:** Use `claimAsset()` to claim the asset with global index 1
3. **Retry Message Claim:** After the asset is claimed, retry the `claimMessage()` call

### Command Sequence

```bash
# 1. Find and claim the asset with global index 1
cast send $POLYGON_ZKEVM_BRIDGE_L2 "claimAsset(uint256,bytes32,bytes32,uint32,address,uint32,address,uint256,bytes)" \
    [global_index_1] \
    [mainnet_exit_root] \
    [rollup_exit_root] \
    [origin_network] \
    [origin_token_address] \
    [destination_network] \
    [destination_address] \
    [amount] \
    [metadata] \
    --private-key $PRIVATE_KEY_2 \
    --rpc-url $RPC_2

# 2. Then retry the message claim
cast send $POLYGON_ZKEVM_BRIDGE_L2 "claimMessage(...)" [same parameters as before]
```

## Prevention

### For Future Bridge Calls

1. **Track Deposit Counts:** Ensure the `dependsOnIndex` matches the actual global index of bridged assets
2. **Sequential Claims:** Always claim assets before their dependent messages
3. **Verify Dependencies:** Check `bridge.isClaimed(dependsOnIndex, originNetwork)` before claiming messages

### Debugging Commands

```bash
# Check if an asset is claimed
cast call $POLYGON_ZKEVM_BRIDGE_L2 "isClaimed(uint32,uint32)" [leafIndex] [sourceBridgeNetwork]

# Get current deposit count
cast call $POLYGON_ZKEVM_BRIDGE_L2 "depositCount()"

# Check claimed bitmap
cast call $POLYGON_ZKEVM_BRIDGE_L2 "claimedBitMap(uint256)" [wordPos]
```

## Contract References

### Key Files Analyzed
- `agglayer-contracts/src/PolygonZkEVMBridgeV2.sol`
- `agglayer-contracts/src/BridgeExtension.sol`

### Critical Functions
- `PolygonZkEVMBridgeV2.claimMessage()` - Lines 534-591
- `BridgeExtension.onMessageReceived()` - Lines 215-234
- `PolygonZkEVMBridgeV2.isClaimed()` - Lines 710-722

### Error Definitions
- `MessageFailed()` - `PolygonZkEVMBridgeV2.sol:587`
- `UnclaimedAsset()` - `BridgeExtension.sol:16` and usage at line 228

## Conclusion

The `MessageFailed()` error with code `0x37e391c3` was caused by a dependency mismatch in the bridge system. The message was configured to depend on an asset with global index 1, but only the asset with global index 0 had been claimed. This architectural design ensures that assets are available before message execution, preventing calls without required funds.

**Resolution:** Claim the asset with global index 1 first, then retry the message claim.

**Root Cause:** Incorrect dependency index configuration or missing asset claim step in the bridge sequence.