# Polygon ZkEVM Contract Deployment

This repository contains scripts to deploy Polygon ZkEVM contracts to local Anvil instances running in Docker.

## Prerequisites

- Docker and Docker Compose
- Foundry (Forge, Anvil, Cast)

## Setup

1. Clone this repository:
```bash
git clone <repository-url>
cd <repository-directory>
```

2. Make sure the deployment scripts are executable:
```bash
chmod +x scripts/deploy.sh scripts/deploy-contracts.sh
```

## Deployment

To deploy all contracts to local Anvil instances:

```bash
./scripts/deploy.sh
```

This script will:
1. Start Docker containers with Anvil instances (L1 and L2)
2. Deploy L1 contracts to the first Anvil instance
3. Deploy L2 contracts to the second Anvil instance
4. Save all contract addresses to the `.env` file

## Manual Deployment

If you want to run the deployment script manually:

```bash
./scripts/deploy-contracts.sh .env
```

## Contract Addresses

After deployment, the following contract addresses will be saved to the `.env` file:

### L1 Contracts
- `FFLONK_VERIFIER_L1`: The FflonkVerifier contract
- `POLYGON_ZKEVM_L1`: The PolygonZkEVM contract
- `POLYGON_ZKEVM_BRIDGE_L1`: The PolygonZkEVMBridgeV2 contract
- `POLYGON_ZKEVM_TIMELOCK_L1`: The PolygonZkEVMTimelock contract
- `POLYGON_ZKEVM_GLOBAL_EXIT_ROOT_L1`: The PolygonZkEVMGlobalExitRootV2 contract
- `POLYGON_ROLLUP_MANAGER_L1`: The PolygonRollupManager contract

### L2 Contracts
- `POLYGON_ZKEVM_BRIDGE_L2`: The PolygonZkEVMBridgeV2 contract
- `POLYGON_ZKEVM_TIMELOCK_L2`: The PolygonZkEVMTimelock contract

## Docker Services

The `docker-compose.yml` file defines two services:

1. `anvil-mainnet`: Simulates L1 (Ethereum mainnet)
   - Port: 8545
   - URL: http://localhost:8545

2. `anvil-polygon`: Simulates L2 (Polygon ZkEVM)
   - Port: 8546
   - URL: http://localhost:8546

## Environment Variables

You can customize the deployment by setting the following environment variables in the `.env` file:

- `RPC_URL_1`: RPC URL for L1 (default: http://anvil-mainnet:8545 in Docker, http://localhost:8545 outside Docker)
- `RPC_URL_2`: RPC URL for L2 (default: http://anvil-polygon:8545 in Docker, http://localhost:8546 outside Docker)
- `PRIVATE_KEY_1`: Private key for L1 deployment (default: Anvil's first account private key)
- `PRIVATE_KEY_2`: Private key for L2 deployment (default: same as PRIVATE_KEY_1)