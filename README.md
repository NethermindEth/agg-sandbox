# AggLayer Sandbox

A development sandbox environment for the AggLayer with support for local blockchain simulation and fork mode.

## Features

- **Local Mode**: Run completely local blockchain nodes for development
- **Fork Mode**: Fork existing blockchains to test against real network state
- **Multi-L2 Mode**: Run with a second L2 chain for multi-chain testing (supports both local and fork modes)
- Easy CLI management of the sandbox environment
- Pre-configured accounts and private keys
- Docker-based deployment for consistent environments

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust (for CLI compilation)

### Installation

1. Clone the repository
2. Install the CLI tool:

   ```bash
   ./install-cli.sh
   ```

### Usage

The CLI provides a single `start` command with different flags for various modes:

#### Local Mode (Default)

Start with completely local blockchain simulation:

```bash
aggsandbox start --detach
```

#### Available Flags

- `--detach` (`-d`): Run in detached mode
- `--build` (`-b`): Build images before starting  
- `--fork` (`-f`): Enable fork mode (uses real blockchain data)
- `--multi-l2` (`-m`): Enable multi-L2 mode (runs with a second L2 chain)

#### Fork Mode

Fork existing blockchains for testing against real network state:

1. First, configure your fork URLs in `.env`:

   ```bash
   cp env.example .env
   # Edit .env and set your fork URLs:
   # FORK_URL_MAINNET=
   # FORK_URL_AGGLAYER_1=
   ```

2. Start in fork mode:

   ```bash
   aggsandbox start --fork --detach
   ```

#### Multi-L2 Mode

Run with a second L2 chain for multi-chain testing:

1. **Local Multi-L2**: Run with local blockchain simulation on three chains:

   ```bash
   aggsandbox start --multi-l2 --detach
   ```

2. **Fork Multi-L2**: Fork existing blockchains with a second L2 chain:

   ```bash
   # Configure all fork URLs in .env including FORK_URL_AGGLAYER_2
   aggsandbox start --multi-l2 --fork --detach
   ```

#### Quick Reference

```bash
# Local mode (2 chains: L1 + L2)
aggsandbox start --detach

# Fork mode (2 chains: forked from real networks)  
aggsandbox start --fork --detach

# Multi-L2 local mode (3 chains: L1 + L2 + L2)
aggsandbox start --multi-l2 --detach

# Multi-L2 fork mode (3 chains: all forked from real networks)
aggsandbox start --multi-l2 --fork --detach
```

#### Other Commands

```bash
# Check status (works for all modes)
aggsandbox status

# View logs (works for all modes)
aggsandbox logs --follow

# Stop the sandbox (automatically detects and stops all configurations)
aggsandbox stop

# Show configuration info
aggsandbox info
```

#### Bridge Information Commands

Query bridge endpoints for debugging and monitoring:

```bash
# Show bridges for a network (default: network_id=1)
aggsandbox show bridges
aggsandbox show bridges --network-id 2

# Show claims for a network (default: network_id=1101)
aggsandbox show claims
aggsandbox show claims --network-id 1

# Show claim proof (default: network_id=1, leaf_index=0, deposit_count=1)
aggsandbox show claim-proof
aggsandbox show claim-proof --network-id 1 --leaf-index 5 --deposit-count 10
```

These commands query the bridge service at `http://localhost:5577` and display:
- **bridges**: Available bridges for the specified network
- **claims**: Claims information for the specified network  
- **claim-proof**: Claim proof data with configurable parameters

## Configuration

The sandbox uses environment variables defined in the `.env` file:

### Local Mode Variables

- `RPC_URL_1`, `RPC_URL_2`: Internal RPC URLs for services
- `CHAIN_ID_MAINNET`, `CHAIN_ID_AGGLAYER_1`: Chain IDs for the networks

### Fork Mode Variables

- `FORK_URL_MAINNET`: Ethereum mainnet fork URL (e.g., Alchemy, Infura)
- `FORK_URL_AGGLAYER_1`: Polygon zkEVM fork URL
- `FORK_URL_AGGLAYER_2`: Additional chain fork URL (optional)

### Account Configuration

Pre-configured test accounts with known private keys:

- `ACCOUNT_ADDRESS_1`, `PRIVATE_KEY_1`: Primary test account
- `ACCOUNT_ADDRESS_2`, `PRIVATE_KEY_2`: Secondary test account

## Network Configuration

### Local Mode

- **L1 (Ethereum Simulation)**: `http://127.0.0.1:8545` (Chain ID: 1)
- **L2 (Polygon zkEVM Simulation)**: `http://127.0.0.1:8546` (Chain ID: 1101)

### Fork Mode

- **L1 (Ethereum Fork)**: `http://127.0.0.1:8545` (Uses real Ethereum state)
- **L2 (Polygon zkEVM Fork)**: `http://127.0.0.1:8546` (Uses real Polygon state)

### Multi-L2 Mode

#### Local Multi-L2

- **L1 (Ethereum Simulation)**: `http://127.0.0.1:8545` (Chain ID: 1)
- **L2-1 (Polygon zkEVM Simulation)**: `http://127.0.0.1:8546` (Chain ID: 1101)
- **L2-2 (Second AggLayer Chain Simulation)**: `http://127.0.0.1:8547` (Chain ID: 1102)

#### Fork Multi-L2

- **L1 (Ethereum Fork)**: `http://127.0.0.1:8545` (Uses real Ethereum state)
- **L2-1 (Polygon zkEVM Fork)**: `http://127.0.0.1:8546` (Uses real Polygon state)
- **L2-2 (Second AggLayer Chain Fork)**: `http://127.0.0.1:8547` (Uses real second chain state)

## Development

### CLI Development

The CLI is written in Rust and located in the `cli/` directory:

```bash
cd cli
cargo build --release
```

### Contract Development

Smart contracts are in the `agglayer-contracts/` directory using Foundry.

## Architecture

### Standard Mode

The sandbox consists of:

- Two Anvil nodes (L1 and L2) running in Docker containers
- A contract deployer service that automatically deploys required contracts
- A CLI tool for managing the environment

### Multi-L2 Mode

The multi-L2 sandbox consists of:

- Three Anvil nodes (L1 and two L2 chains) running in Docker containers
- A contract deployer service that automatically deploys required contracts to all chains
- A CLI tool for managing the environment
- Uses Docker Compose override files for flexible configuration

## Troubleshooting

### Fork Mode Issues

- Ensure your fork URLs are accessible and support the required RPC methods
- Check that your API keys (if required) are properly configured
- Some RPC providers have rate limits that may affect performance

### Docker Issues

- Ensure Docker daemon is running
- Try rebuilding images: `aggsandbox start --build`
- Check logs: `aggsandbox logs`

## License

[Add your license information here]
