# AggLayer Sandbox

A development sandbox environment for the AggLayer with support for local blockchain simulation and fork mode.

## Features

- **Local Mode**: Run completely local blockchain nodes for development
- **Fork Mode**: Fork existing blockchains to test against real network state
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

#### Local Mode (Default)
Start with completely local blockchain simulation:
```bash
agg-sandbox start --detach
```

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
   agg-sandbox start-fork --detach
   ```

#### Other Commands
```bash
# Check status
agg-sandbox status

# View logs
agg-sandbox logs --follow

# Stop the sandbox
agg-sandbox stop

# Show configuration info
agg-sandbox info
```

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

The sandbox consists of:
- Two Anvil nodes (L1 and L2) running in Docker containers
- A contract deployer service that automatically deploys required contracts
- A CLI tool for managing the environment

## Troubleshooting

### Fork Mode Issues
- Ensure your fork URLs are accessible and support the required RPC methods
- Check that your API keys (if required) are properly configured
- Some RPC providers have rate limits that may affect performance

### Docker Issues
- Ensure Docker daemon is running
- Try rebuilding images: `agg-sandbox start --build`
- Check logs: `agg-sandbox logs`

## License

[Add your license information here]
