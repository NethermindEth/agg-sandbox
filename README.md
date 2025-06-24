# Agg-Sandox

This repository contains a Docker-based setup to deploy Polygon ZkEVM contracts to local Anvil instances.

## Prerequisites

- Docker and Docker Compose

## Setup

1. Clone this repository:

```bash
git clone https://github.com/NethermindEth/agg-sandbox.git
cd agg-sandbox
```

2. Initialize git submodules:
   
```bash
git submodule update --init --recursive
```

## Quick Start

### Option 1: Using the CLI (Recommended)

Build and install the CLI tool:

```bash
./install-cli.sh
```

Then use the CLI to manage your sandbox:

```bash
# Start the sandbox (interactive mode)
agg-sandbox start

# Start in detached mode with build
agg-sandbox start --detach --build

# Check status
agg-sandbox status

# View logs
agg-sandbox logs --follow

# Stop the sandbox
agg-sandbox stop
```

### Option 2: Direct Docker Compose

To deploy all contracts to local Anvil instances directly:

```bash
docker-compose up --build
```

This single command will:

1. **Build all Docker images** with the Foundry toolchain and dependencies
2. **Start two Anvil instances**:
   - `anvil-mainnet` on port 8545 (simulates L1/Ethereum mainnet)
   - `anvil-polygon` on port 8546 (simulates L2/Polygon ZkEVM)
3. **Wait for both Anvil instances** to be healthy and ready
4. **Deploy L1 contracts** to the first Anvil instance
5. **Deploy L2 contracts** to the second Anvil instance
6. **Update your `.env` file** with all deployed contract addresses

## Contract Addresses

After deployment, the following contract addresses will be automatically saved to your `.env` file:

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

The `docker-compose.yml` file defines three services:

1. **`anvil-mainnet`**: Simulates L1 (Ethereum mainnet)
   - Port: 8545
   - URL: http://localhost:8545

2. **`anvil-polygon`**: Simulates L2 (Polygon ZkEVM)
   - Port: 8546
   - URL: http://localhost:8546

3. **`contract-deployer`**: Deploys contracts to both Anvil instances
   - Automatically waits for Anvil instances to be ready
   - Updates your `.env` file with deployed contract addresses
   - Exits after successful deployment

## Environment Configuration

The deployment uses your `env.example` file as the base configuration and automatically updates the RPC URLs for the Docker environment:

- `RPC_URL_1`: Set to `http://anvil-mainnet:8545` (L1)
- `RPC_URL_2`: Set to `http://anvil-polygon:8545` (L2)
- `PRIVATE_KEY_1` & `PRIVATE_KEY_2`: Uses the default Anvil account private keys from `env.example`

## Manual Deployment (Alternative)

If you want to run the deployment script manually outside of Docker:

```bash
./scripts/deploy-contracts.sh .env
```

Note: You'll need to have Foundry installed locally and Anvil instances running.

## CLI Usage

The `agg-sandbox` CLI provides a convenient interface for managing your sandbox environment:

### Available Commands

```bash
# Start services
agg-sandbox start [--detach] [--build]

# Stop services  
agg-sandbox stop [--volumes]

# Check service status
agg-sandbox status

# View and follow logs
agg-sandbox logs [--follow] [service-name]

# Restart all services
agg-sandbox restart

# Get help
agg-sandbox --help
```

### Examples

```bash
# Start in background and build images
agg-sandbox start --detach --build

# Follow logs for a specific service
agg-sandbox logs --follow anvil-mainnet

# Stop and remove all volumes
agg-sandbox stop --volumes
```

See `cli/README.md` for detailed CLI documentation.

## Troubleshooting

- **Solidity version errors**: The Docker setup automatically downloads the required Solidity compiler versions
- **Port conflicts**: Make sure ports 8545 and 8546 are not in use by other applications
- **Build failures**: Try running `docker-compose down` and then `docker-compose up --build` to rebuild from scratch
- **CLI not found**: Make sure you've run `./install-cli.sh` and that `~/.local/bin` is in your PATH
