# Deployed Contracts Reference

This document provides a comprehensive overview of all contracts deployed in the Agglayer Sandbox environment, their purposes, and how they interact to enable cross-chain bridging operations.

## Overview

The Agglayer Sandbox deploys a comprehensive suite of smart contracts across three networks:
- **L1 (Ethereum)**: The main chain where the core bridge infrastructure and rollup management resides
- **L2 (Agglayer Chain 1)**: First Polygon zkEVM rollup chain (Chain ID: 1101, Network ID: 1)
- **L3 (Agglayer Chain 2)**: Second Polygon zkEVM rollup chain (Chain ID: 137, Network ID: 2)

## L1 (Ethereum) Contracts

### Core Bridge Infrastructure

#### PolygonZkEVMBridgeV2
- **Address**: `POLYGON_ZKEVM_BRIDGE_L1`
- **Purpose**: Main bridge contract responsible for cross-chain asset and message transfers
- **Key Functions**:
  - `bridgeAsset()`: Bridge tokens from L1 to L2/L3
  - `bridgeMessage()`: Send messages across chains
  - `claimAsset()`: Claim bridged assets on destination chain
  - `claimMessage()`: Claim and execute bridged messages
- **Features**:
  - Supports both native ETH and ERC20 token bridging
  - Merkle proof-based claim verification
  - Emergency pause functionality
  - Gas token management

#### PolygonZkEVMGlobalExitRootV2
- **Address**: `POLYGON_ZKEVM_GLOBAL_EXIT_ROOT_L1`
- **Purpose**: Manages the global exit root for all connected rollup chains
- **Key Functions**:
  - Aggregates exit roots from all rollup chains
  - Provides unified state root for cross-chain verification
  - Enables trustless bridge operations through cryptographic proofs

#### PolygonRollupManager
- **Address**: `POLYGON_ROLLUP_MANAGER_L1`
- **Purpose**: Central coordinator for all Polygon zkEVM rollup chains
- **Key Functions**:
  - Registers and manages rollup chains
  - Coordinates state transitions and finalization
  - Manages rollup verification and consensus
  - Handles rollup-specific configurations

### Rollup Chain Contracts

#### PolygonZkEVM (L1 Main)
- **Address**: `POLYGON_ZKEVM_L1`
- **Purpose**: Main rollup contract for the primary L2 chain
- **Chain ID**: 1101 (Network ID: 1)

#### PolygonZkEVM (L2 Rollup)
- **Purpose**: Rollup contract for the second L2 chain
- **Chain ID**: 137 (Network ID: 2)
- **Note**: Registered in the RollupManager for multi-L2 operations

### Verification & Security

#### FflonkVerifier
- **Address**: `FFLONK_VERIFIER_L1`
- **Purpose**: Zero-knowledge proof verifier for rollup state transitions (deployed for backward compatibility only)
- **Technology**: Uses FFlonk (Fast-FRI Low-degree-Test with Kate) proving system
- **Status**: Not actively used in the sandbox environment - deployed only for contract interface compatibility
- **Note**: In production environments, this would validate computational integrity of rollup batches

#### PolygonZkEVMTimelock
- **Address**: `POLYGON_ZKEVM_TIMELOCK_L1`
- **Purpose**: Governance timelock controller for delayed execution of critical operations
- **Configuration**:
  - Minimum delay: 1 hour (3600 seconds)
  - Proposers: Deployer account
  - Executors: Deployer account
- **Use Cases**: Protocol upgrades, parameter changes, emergency actions

### Token & Extension Contracts

#### AggERC20
- **Address**: `AGG_ERC20_L1`
- **Purpose**: Mock ERC20 token for testing bridge operations
- **Features**: Standard ERC20 with minting capabilities for testing scenarios

#### BridgeExtension
- **Address**: `BRIDGE_EXTENSION_L1`
- **Purpose**: Extended bridge functionality for complex cross-chain operations
- **Key Features**:
  - `bridgeAndCall()`: Bridge assets and execute calls atomically
  - Supports conditional execution based on bridge success
  - Enables cross-chain DeFi operations
  - Jump point deployment for deterministic addresses

## L2 (Agglayer Chain 1) Contracts

### Bridge Infrastructure

#### BridgeL2SovereignChain
- **Address**: `POLYGON_ZKEVM_BRIDGE_L2`
- **Purpose**: L2-specific bridge contract optimized for sovereign chain operations
- **Network ID**: 1
- **Features**:
  - Inherits from PolygonZkEVMBridgeV2 with L2-specific optimizations
  - Manages L2-to-L1 and L2-to-L3 bridging
  - Handles gas token management for the sovereign chain

#### GlobalExitRootManagerL2SovereignChain
- **Address**: `GLOBAL_EXIT_ROOT_MANAGER_L2`
- **Purpose**: L2-specific global exit root management
- **Functions**:
  - Synchronizes with L1 global exit root
  - Manages local exit root calculations
  - Enables trustless cross-chain proof verification

### Governance & Security

#### PolygonZkEVMTimelock (L2)
- **Address**: `POLYGON_ZKEVM_TIMELOCK_L2`
- **Purpose**: L2 governance timelock for local protocol operations
- **Configuration**: Mirrors L1 timelock settings

### Testing & Utility Contracts

#### AggERC20 (L2)
- **Address**: `AGG_ERC20_L2`
- **Purpose**: L2 instance of the test ERC20 token

#### BridgeExtension (L2)
- **Address**: `BRIDGE_EXTENSION_L2`
- **Purpose**: L2 instance of the bridge extension contract

#### AssetAndCallReceiver
- **Address**: `ASSET_AND_CALL_RECEIVER_L2`
- **Purpose**: Test contract for receiving bridge-and-call operations
- **Key Functions**:
  - `onMessageReceived()`: Handles incoming bridge messages
  - `processTransferAndCall()`: Processes asset transfers with function calls
  - Tracks total transferred amounts and call counters
- **Use Cases**: Testing complex cross-chain interactions and atomic operations

#### PolygonZkEVM L2 Rollup
- **Address**: `POLYGON_ZKEVM_L2_ROLLUP`
- **Purpose**: Local rollup contract instance for L2 operations

## L3 (Agglayer Chain 2) Contracts

The L3 deployment follows the same pattern as L2 but is currently in development. The following addresses are configured:

- **PolygonZkEVM L3 Rollup**: `POLYGON_ZKEVM_L3_ROLLUP`
- Other L3 contracts (bridge, timelock, etc.) will be deployed when multi-L3 functionality is enabled

## Cross-Chain Architecture

### Bridge Flow
1. **Asset Bridging**: 
   - User calls `bridgeAsset()` on source chain
   - Bridge emits BridgeEvent with merkle tree leaf
   - Global exit root is updated
   - User can claim on destination using merkle proof

2. **Message Bridging**:
   - User calls `bridgeMessage()` on source chain
   - Message data is included in bridge leaf
   - On destination, `claimMessage()` executes the message
   - BridgeExtension enables atomic asset+call operations

### Verification System
- **FFlonk Verifier**: Deployed for compatibility but not actively used in sandbox environment
- **Global Exit Root**: Aggregates all chain states for cross-chain verification
- **Merkle Proofs**: Enable trustless claim verification without relying on external oracles

### Security Features
- **Emergency Manager**: Can pause bridge operations in case of detected issues
- **Timelock Controllers**: Ensure delayed execution of critical parameter changes
- **Reentrancy Guards**: Protect against reentrancy attacks
- **Role-based Access Control**: Manages permissions for different contract functions

## Development and Testing

### Configuration
All contract addresses are automatically populated in `.env` during deployment. The sandbox uses deterministic deployment to ensure consistent addresses across restarts.

### Funding
- L1 and L2 bridges are pre-funded with 50 ETH each for testing operations
- AggERC20 tokens are minted to test accounts for bridge testing
- All test accounts have sufficient ETH for transaction fees

### Network IDs
- **L1 (Ethereum)**: Network ID 0, Chain ID 1
- **L2 (Agglayer 1)**: Network ID 1, Chain ID 1101  
- **L3 (Agglayer 2)**: Network ID 2, Chain ID 137

This architecture enables comprehensive testing of cross-chain bridge operations, including complex scenarios like asset bridging, message passing, and atomic bridge-and-call operations across multiple rollup chains.