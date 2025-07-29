# Agglayer Sandbox

A development sandbox environment for the Agglayer with support for local blockchain simulation and fork mode.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
  - [Installation](#installation)
  - [Basic Usage](#basic-usage)
  - [Verification](#verification)
- [Architecture](#architecture)
- [Usage Modes](#usage-modes)
  - [Local Mode](#local-mode)
  - [Fork Mode](#fork-mode)
  - [Multi-L2 Mode](#multi-l2-mode)
- [CLI Commands Reference](#cli-commands-reference)
  - [Core Commands](#core-commands)
  - [Bridge Information Commands](#bridge-information-commands)
  - [Event Monitoring Commands](#event-monitoring-commands)
  - [Command-Line Options](#command-line-options)
- [Configuration](#configuration)
  - [Environment Variables](#environment-variables)
  - [Configuration Files](#configuration-files)
  - [Account Configuration](#account-configuration)
- [Network Configuration](#network-configuration)
- [Advanced Features](#advanced-features)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

## Overview

The Agglayer Sandbox provides a comprehensive development environment for testing cross-chain bridging operations, smart contract interactions, and multi-layer blockchain scenarios. It supports both completely local blockchain simulation and forking from real networks for testing against live data.

## Features

- **🏠 Local Mode**: Run completely local blockchain nodes for development
- **🍴 Fork Mode**: Fork existing blockchains to test against real network state  
- **🔗 Multi-L2 Mode**: Run with a second L2 chain for multi-chain testing (supports both local and fork modes)
- **🎨 Enhanced CLI** with rich help messages, progress tracking, and intelligent error handling
- **📊 JSON Scripting Support** with `--json` flag for clean machine-readable output and automation
- **⚙️ Advanced Configuration** with TOML/YAML file support and environment variable management
- **⚡ Performance Optimizations** with HTTP connection pooling and response caching
- **📊 Comprehensive Monitoring** with structured logging and detailed troubleshooting guides
- **🔑 Pre-configured Accounts** and private keys for immediate testing
- **🐳 Docker-based Deployment** for consistent environments across platforms

## Prerequisites

### System Requirements

- **Docker** >= 20.0 and Docker Compose >= 1.27
- **Rust** >= 1.70.0 (for CLI compilation) - [Install Rust](https://rustup.rs/)
- **Make** (for using Makefile targets) - usually pre-installed on Unix systems
- **Git** (for cloning the repository)
- **pnpm** (for bridge service dependencies) - [Install pnpm](https://pnpm.io/installation)

### PATH Configuration

Ensure `~/.local/bin` is in your PATH for CLI installation:

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export PATH="$HOME/.local/bin:$PATH"

# Or add it temporarily
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Verify Prerequisites

```bash
# Check all required tools
docker --version && echo "✅ Docker installed"
docker compose version && echo "✅ Docker Compose installed"
rustc --version && echo "✅ Rust installed"
make --version && echo "✅ Make installed"
git --version && echo "✅ Git installed"
pnpm --version && echo "✅ pnpm installed"
```

## Quick Start

### Installation

1. **Clone the repository:**

   ```bash
   git clone https://github.com/NethermindEth/agg-sandbox.git
   cd agg-sandbox
   ```

2. **Install the CLI tool:**

   ```bash
   make install
   ```

   This will automatically:
   - Build and install the CLI to `~/.local/bin`
   - Set up the bridge service dependencies (requires `pnpm`)
   - Configure bridge functionality for cross-chain operations

3. **Verify installation:**

   ```bash
   aggsandbox --help
   ```

   You should see comprehensive help with examples and rich formatting.

4. **Uninstall (if needed):**

   ```bash
   make uninstall
   ```

### Basic Usage

**Create .env:**

```bash
cp .env.example .env
```

**Start the sandbox in local mode:**

```bash
aggsandbox start --detach
```

The CLI will display a progress bar with step-by-step feedback during startup.

**Check status:**

```bash
aggsandbox status
```

**Stop the sandbox:**

```bash
aggsandbox stop
```

### Verification

**Test the environment:**

```bash
# Check that both chains are running
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'

curl -X POST http://127.0.0.1:8546 \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'
```

## Architecture

### Standard Mode Architecture

The sandbox consists of:

```text
┌─────────────────┐         ┌─────────────────────┐         ┌─────────────────┐
│   L1 (Anvil)    │◄────────┤      AggKit         ├────────►│   L2 (Anvil)    │
│   Port: 8545    │         │  REST API: 5577     │         │   Port: 8546    │
│   Chain ID: 1   │         │  RPC: 8555          │         │   Chain ID:1101 │
│                 │         │  Telemetry: 8080    │         │                 │
└─────────────────┘         └─────────────────────┘         └─────────────────┘
         ▲                                                           ▲
         │                                                           │
         └─────────────────────────────┼─────────────────────────────┘
                                       │
                          ┌─────────────────┐
                          │ Contract Deploy │
                          │    Service      │
                          │ (runs once)     │
                          └─────────────────┘
```

**Components:**

- **L1 Anvil Node**: Simulates Ethereum mainnet (port 8545)
- **L2 Anvil Node**: Simulates Polygon zkEVM (port 8546)
- **AggKit Service**: Bridges L1 ↔ L2, handles oracle functions, and provides API endpoints
  - REST API for bridge queries (port 5577)
  - RPC interface (port 8555)
  - Telemetry and monitoring (port 8080)
- **Contract Deployer**: Automatically deploys required contracts (runs once)
- **CLI Tool**: Manages the entire environment

### Multi-L2 Architecture

For multi-chain testing with dual AggKit instances:

```text
                     ┌─────────────────────┐
                     │     AggKit-L2       │
              ┌──────┤  REST API: 5577     ├──────┐
              │      │  RPC: 8555          │      │
              │      │  Telemetry: 8080    │      │
              │      └─────────────────────┘      │
              ▼                                   ▼
   ┌─────────────┐                     ┌─────────────┐
   │ L1 (Anvil)  │                     │L2-1 (Anvil) │
   │ Port: 8545  │                     │ Port: 8546  │
   │Chain ID: 1  │                     │Chain ID:1101│
   └─────────────┘                     └─────────────┘
              │                                   
              │      ┌─────────────────────┐      
              └──────┤     AggKit-L3       ├──────┐
                     │  REST API: 5578     │      │
                     │  RPC: 8556          │      │
                     │  Telemetry: 8081    │      │
                     └─────────────────────┘      ▼
                                        ┌─────────────┐
                                        │L2-2 (Anvil) │
                                        │ Port: 8547  │
                                        │Chain ID:137 │
                                        └─────────────┘
```

**Additional Components:**

- **L3 Anvil Node**: Second L2 chain (typically Polygon PoS, Chain ID 137)
- **AggKit-L2 Instance**: Bridges L1 ↔ L2 operations (ports 5577, 8555, 8080)
- **AggKit-L3 Instance**: Bridges L1 ↔ L3 operations (ports 5578, 8556, 8081)
- **Dual Database**: Separate database instances for each bridge service
- **Contract Deployer**: Deploys contracts to all three chains
- **Docker Compose Override**: Uses `docker-compose.multi-l2.yml` configuration

## Usage Modes

### Local Mode

**Default mode** - runs completely local blockchain simulation for development and testing.

#### Start Local Mode

```bash
aggsandbox start --detach
```

#### Features

- ✅ Fast startup and execution  
- ✅ Deterministic behavior
- ✅ No external dependencies
- ✅ Ideal for development and CI/CD

#### Use Cases

- Smart contract development
- Integration testing
- CI/CD pipelines
- Learning and experimentation

### Fork Mode

**Fork real networks** to test against actual blockchain state and data.

> ⚠️ **Note**: Currently only Polygon PoS can be used for forking. Polygon zkEVM will not work due to an Anvil compatibility issue.

#### Configure Fork Mode

1. **Set up your environment:**

   ```bash
   cp .env.example .env
   ```

2. **Edit `.env` and add your fork URLs:**

   ```bash
   # Ethereum mainnet fork URL (Alchemy, Infura, etc.)
   FORK_URL_MAINNET=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY
   
   # Polygon PoS fork URL  
   FORK_URL_AGGLAYER_1=https://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY
   ```

3. **Start in fork mode:**

   ```bash
   aggsandbox start --fork --detach
   ```

#### Features

- ✅ Test against real network state
- ✅ Use actual contract deployments
- ✅ Access to real transaction history
- ⚠️ Requires API keys and network access

#### Use Cases

- Testing against production data
- Debugging mainnet issues
- Integration testing with real contracts
- Performance testing with real load

### Multi-L2 Mode

**Run multiple L2 chains** for cross-chain testing scenarios.

#### Local Multi-L2

Run three chains with local simulation:

```bash
aggsandbox start --multi-l2 --detach
```

#### Fork Multi-L2

Fork real networks with additional L2 chain:

```bash
# Configure all fork URLs in .env including FORK_URL_AGGLAYER_2
FORK_URL_AGGLAYER_2=https://your-second-l2.com/v1/YOUR_API_KEY

aggsandbox start --multi-l2 --fork --detach
```

#### Features

- ✅ Test multi-chain scenarios
- ✅ Cross-L2 bridging operations  
- ✅ Complex DeFi interactions
- ✅ Full production-ready implementation
- ⚠️ Higher resource requirements

## CLI Commands Reference

The CLI provides comprehensive commands with enhanced user experience including progress tracking, detailed help, and intelligent error messages.

### Core Commands

#### Start/Stop Operations

```bash
# Start with progress tracking
aggsandbox start --detach

# Start with verbose output
aggsandbox start --detach --verbose

# Start with image rebuilding
aggsandbox start --build --detach

# Stop gracefully
aggsandbox stop

# Stop and remove volumes (destructive)
aggsandbox stop --volumes
```

#### Status and Information

```bash
# Check current status
aggsandbox status

# Show comprehensive configuration
aggsandbox info

# Get version information
aggsandbox --version
```

#### Log Management

```bash
# View all logs with real-time following
aggsandbox logs --follow

# View specific service logs
aggsandbox logs bridge-service
aggsandbox logs anvil-l1
aggsandbox logs anvil-l2

# Follow specific service logs
aggsandbox logs --follow anvil-l1

# View logs with verbose output
aggsandbox logs --verbose
```

### Bridge Commands

Perform cross-chain bridge operations using the integrated LXLY bridge:

#### Bridge Operations

```bash
# Bridge ETH from L1 to L2
aggsandbox bridge asset \
  --network 0 \
  --destination-network 1 \
  --amount 0.1 \
  --token-address 0x0000000000000000000000000000000000000000

# Bridge ERC20 tokens from L2 to L1
aggsandbox bridge asset \
  --network 1 \
  --destination-network 0 \
  --amount 100 \
  --token-address 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC

# Claim bridged assets on destination network
aggsandbox bridge claim \
  --network 1 \
  --tx-hash 0xb7118cfb20825861028ede1e9586814fc7ccf81745a325db5df355d382d96b4e \
  --source-network 0

# Bridge with contract call (bridgeAndCall)
aggsandbox bridge message \
  --network 0 \
  --destination-network 1 \
  --target 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 \
  --data 0xa9059cbb000000000000000000000000742d35cc6965c592342c6c16fb8eaeb90a23b5c00000000000000000000000000000000000000000000000000de0b6b3a7640000
```

**Bridge Command Parameters:**
- `--network, -n`: Source network ID (0=L1, 1=L2, 2=L3)
- `--destination-network, -d`: Destination network ID
- `--amount, -a`: Amount to bridge (in token units)
- `--token-address, -t`: Token contract address (use `0x0000000000000000000000000000000000000000` for ETH)
- `--tx-hash, -t`: Transaction hash for claim operations
- `--target`: Target contract address for bridge messages
- `--data, -D`: Contract call data (hex encoded)

For detailed bridge documentation, see [LXLY.md](LXLY.md).

### Bridge Information Commands

Query bridge endpoints with enhanced formatting and detailed explanations:

#### Bridge Operations

```bash
# Show bridges for L1 (Ethereum)
aggsandbox show bridges --network-id 0

# Show bridges for first L2 (Polygon zkEVM)  
aggsandbox show bridges --network-id 1

# Show bridges for second L2 (if running multi-L2 mode)
aggsandbox show bridges --network-id 2

# Show bridges with verbose output
aggsandbox show bridges --network-id 0 --verbose

# Raw JSON output for scripting (no decorative formatting)
aggsandbox show bridges --network-id 1 --json
```

#### Claims Management

```bash
# Show L1 claims (deposits to be claimed on L2)
aggsandbox show claims --network-id 0

# Show L2 claims (withdrawals to be claimed on L1)
aggsandbox show claims --network-id 1

# Show claims with raw JSON output for parsing
aggsandbox show claims --network-id 1 --json
```

#### Proof Generation

```bash
# Show claim proof with verification data
aggsandbox show claim-proof \
  --network-id 0 \
  --leaf-index 0 \
  --deposit-count 1

# Short form options with JSON output
aggsandbox show claim-proof -n 1 -l 5 -d 10 --json

# Show L1 info tree index for deposit verification
aggsandbox show l1-info-tree-index \
  --network-id 0 \
  --deposit-count 0

# Raw JSON for scripting integration
aggsandbox show l1-info-tree-index \
  --network-id 0 \
  --deposit-count 0 \
  --json
```

#### JSON Output for Scripting

All `show` commands support a `--json` flag for machine-readable output:

```bash
# Extract specific values with jq
DEPOSIT_COUNT=$(aggsandbox show bridges --network-id 1 --json | jq -r '.bridges[0].deposit_count')

# Parse bridge data in scripts
BRIDGE_DATA=$(aggsandbox show bridges --network-id 0 --json)
echo "$BRIDGE_DATA" | jq '.count'

# Chain multiple operations
LEAF_INDEX=$(aggsandbox show l1-info-tree-index --network-id 0 --deposit-count 0 --json | jq -r '.')
aggsandbox show claim-proof --network-id 0 --leaf-index "$LEAF_INDEX" --deposit-count 1 --json
```

**Features of JSON Output:**

- ✅ Clean JSON without decorative formatting
- ✅ No status messages or progress indicators
- ✅ Perfect for piping to `jq` or other JSON parsers
- ✅ Ideal for shell scripts and automation
- ✅ Maintains same data structure as formatted output

#### Help and Documentation

```bash
# Overview of all bridge commands
aggsandbox show --help

# Detailed bridge command help
aggsandbox show bridges --help

# Detailed claim proof help
aggsandbox show claim-proof --help
```

**Service Information:**
These commands query the bridge service at `http://localhost:5577` and display:

- **bridges**: Available bridges for the specified network
- **claims**: Claims information for the specified network  
- **claim-proof**: Claim proof data with configurable parameters
- **l1-info-tree-index**: L1 info tree index data with configurable network and deposit count

### Event Monitoring Commands

Monitor and decode blockchain events in human-readable format:

#### Basic Event Monitoring

```bash
# Show events from L1 chain (last 10 blocks by default)
aggsandbox events --network-id 0

# Show events from first L2 chain with custom block range
aggsandbox events --network-id 1 --blocks 20

# Show events from second L2 chain (if running multi-l2 mode)
aggsandbox events --network-id 2 --blocks 30
```

#### Advanced Filtering

```bash
# Filter events by contract address
aggsandbox events \
  --network-id 0 \
  --blocks 5 \
  --address 0x5fbdb2315678afecb367f032d93f642f64180aa3

# Show events with comprehensive monitoring
aggsandbox events --network-id 1 --blocks 50

# Legacy syntax (deprecated - shows warning)
aggsandbox events --chain anvil-l1 --blocks 10
```

#### Event Display Format

Each event displays:

- 🕐 **Timestamp and block number**
- 📄 **Transaction hash**
- 📍 **Contract address**
- 🎯 **Event signature and decoded parameters**
- 🔍 **Raw data for debugging**

### Command-Line Options

#### Global Options

```bash
# Available for all commands
--verbose, -v      # Enable verbose output for debugging
--quiet, -q        # Quiet mode (only errors and warnings)
--help, -h         # Show comprehensive help
--version, -V      # Show version information
```

#### Start Command Options

```bash
--detach, -d       # Run in detached mode
--build, -b        # Build images before starting  
--fork, -f         # Enable fork mode (uses real blockchain data)
--multi-l2, -m     # Enable multi-L2 mode (runs with second L2 chain)
```

#### Log Command Options

```bash
--follow, -f       # Follow log output in real-time
--tail <lines>     # Number of lines to show from the end
--since <time>     # Show logs since timestamp
```

#### Show Command Options

```bash
--network-id, -n   # Specify network ID (0=L1, 1=first L2, 2=second L2)
--leaf-index, -l   # Leaf index for proof generation
--deposit-count, -d # Deposit count for proof verification
--json             # Output raw JSON without decorative formatting (for scripting)
```

#### Events Command Options

```bash
--network-id, -n   # Network ID to query (0=L1, 1=first L2, 2=second L2)
--chain, -c        # [DEPRECATED] Chain name (use --network-id instead)
--blocks, -b       # Number of recent blocks to scan (default: 10)
--address, -a      # Filter events by contract address
```

## Configuration

The sandbox supports multiple configuration methods with enhanced validation and error reporting.

### Environment Variables

#### Basic Configuration

Create and edit your `.env` file:

```bash
cp .env.example .env
```

**Core Variables:**

```bash
# Internal RPC URLs for services
RPC_URL_1=http://127.0.0.1:8545
RPC_URL_2=http://127.0.0.1:8546

# Chain IDs for the networks
CHAIN_ID_MAINNET=1
CHAIN_ID_AGGLAYER_1=1101
CHAIN_ID_AGGLAYER_2=137  # For multi-L2 mode
```

#### Fork Mode Variables

```bash
# Ethereum mainnet fork URL (Alchemy, Infura, etc.)
FORK_URL_MAINNET=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY

# Polygon PoS fork URL
FORK_URL_AGGLAYER_1=https://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY

# Additional chain fork URL (optional, for multi-L2)
FORK_URL_AGGLAYER_2=https://your-second-l2.com/v1/YOUR_API_KEY
```

#### Service Configuration

```bash
# Bridge service configuration
BRIDGE_SERVICE_PORT=5577
BRIDGE_SERVICE_HOST=127.0.0.1

# Docker configuration
COMPOSE_PROJECT_NAME=agg-sandbox
DOCKER_BUILDKIT=1
```

### Account Configuration

Pre-configured test accounts with known private keys for immediate testing:

```bash
# Primary test account (Anvil account #0)
ACCOUNT_ADDRESS_1=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
PRIVATE_KEY_1=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Secondary test account (Anvil account #1)  
ACCOUNT_ADDRESS_2=0x70997970C51812dc3A010C7d01b50e0d17dc79C8
PRIVATE_KEY_2=0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
```

**⚠️ Security Note**: These are well-known test keys. Never use them with real funds or in production environments.

### Configuration Files

The CLI supports TOML and YAML configuration files for more structured configuration:

#### TOML Configuration (`aggsandbox.toml`)

```toml
[api]
base_url = "http://localhost:5577"
timeout = "30s"
retry_attempts = 3

[networks.l1]
name = "Ethereum-L1"
chain_id = "1"
rpc_url = "http://localhost:8545"

[networks.l2]
name = "Polygon-zkEVM-L2"
chain_id = "1101"
rpc_url = "http://localhost:8546"

[networks.l3]
name = "Second-L2-Chain"
chain_id = "1102"
rpc_url = "http://localhost:8547"

[accounts]
accounts = [
  "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
  "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
]
private_keys = [
  "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
  "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
]

[logging]
level = "info"
format = "pretty"
```

#### YAML Configuration (`aggsandbox.yaml`)

```yaml
api:
  base_url: "http://localhost:5577"
  timeout: "30s"
  retry_attempts: 3

networks:
  l1:
    name: "Ethereum-L1"
    chain_id: "1"
    rpc_url: "http://localhost:8545"
  l2:
    name: "Polygon-zkEVM-L2"
    chain_id: "1101"
    rpc_url: "http://localhost:8546"
  l3:
    name: "Second-L2-Chain"
    chain_id: "1102"
    rpc_url: "http://localhost:8547"

accounts:
  accounts:
    - "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    - "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
  private_keys:
    - "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    - "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"

logging:
  level: "info"
  format: "pretty"
```

#### Configuration Priority

Configuration sources are prioritized as follows (highest to lowest):

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **TOML configuration file** (`aggsandbox.toml`)
4. **YAML configuration file** (`aggsandbox.yaml`)
5. **Default values** (lowest priority)

This allows for flexible overrides while maintaining reasonable defaults.

## Network Configuration

### Local Mode Networks

| Network | URL | Chain ID | Description |
|---------|-----|----------|-------------|
| L1 (Ethereum Simulation) | `http://127.0.0.1:8545` | 1 | Local Ethereum simulation |
| L2 (Polygon zkEVM Simulation) | `http://127.0.0.1:8546` | 1101 | Local Polygon zkEVM simulation |

### Fork Mode Networks

| Network | URL | Chain ID | Description |
|---------|-----|----------|-------------|
| L1 (Ethereum Fork) | `http://127.0.0.1:8545` | 1 | Uses real Ethereum state |
| L2 (Polygon Fork) | `http://127.0.0.1:8546` | 1101 | Uses real Polygon state |

### Multi-L2 Mode Networks

#### Local Multi-L2

| Network | URL | Chain ID | Description |
|---------|-----|----------|-------------|
| L1 (Ethereum Simulation) | `http://127.0.0.1:8545` | 1 | Local Ethereum simulation |
| L2-1 (Polygon zkEVM Simulation) | `http://127.0.0.1:8546` | 1101 | First L2 simulation |
| L2-2 (Polygon PoS Simulation) | `http://127.0.0.1:8547` | 137 | Second L2 simulation |

#### Fork Multi-L2

| Network | URL | Chain ID | Description |
|---------|-----|----------|-------------|
| L1 (Ethereum Fork) | `http://127.0.0.1:8545` | 1 | Uses real Ethereum state |
| L2-1 (Polygon zkEVM Fork) | `http://127.0.0.1:8546` | 1101 | Uses real Polygon zkEVM state |
| L2-2 (Polygon PoS Fork) | `http://127.0.0.1:8547` | 137 | Uses real Polygon PoS state |

### Port Configuration

**Default Ports:**

*Standard Mode:*

- **8545**: L1 Ethereum RPC endpoint
- **8546**: L2 Polygon zkEVM RPC endpoint  
- **5577**: AggKit REST API endpoint
- **8555**: AggKit RPC endpoint
- **8080**: AggKit Telemetry endpoint

*Multi-L2 Mode (additional):*

- **8547**: L3 Second L2 RPC endpoint (Polygon PoS)
- **5578**: AggKit-L3 REST API endpoint
- **8556**: AggKit-L3 RPC endpoint
- **8081**: AggKit-L3 Telemetry endpoint

**Customizing Ports:**

```bash
# In docker-compose.yml or docker-compose.override.yml
ports:
  - "8545:8545"  # L1
  - "8546:8546"  # L2  
  - "8547:8547"  # L3 (multi-L2)
  - "5577:5577"  # Bridge service
```

## Advanced Features

### Enhanced CLI Experience

The CLI includes several user experience improvements:

#### Rich User Interface

- **🎨 Rich Help Messages**: Comprehensive help with examples, emojis, and detailed explanations
- **📊 Progress Tracking**: Visual progress bars with step-by-step feedback during long operations
- **🚨 Smart Error Handling**: Context-specific error messages with troubleshooting suggestions
- **🔍 Verbose Logging**: Configurable log levels for debugging (`-v` for debug, `-vv` for trace)
- **⚡ Performance Optimizations**: HTTP connection pooling and response caching for better performance

#### Logging and Verbosity Control

Control output verbosity and format:

```bash
# Enable verbose output for debugging
aggsandbox start --detach --verbose        # Debug level
aggsandbox start --detach -vv              # Trace level (very detailed)

# Quiet mode (only errors and warnings)
aggsandbox start --detach --quiet

# Different log formats
aggsandbox start --detach --log-format json     # Machine-readable JSON logs
aggsandbox start --detach --log-format compact  # Compact format  
aggsandbox start --detach --log-format pretty   # Default human-readable format
```

#### Error Handling and Troubleshooting

When errors occur, the CLI provides:

- **🔧 Specific Issue Categories**: Docker, Configuration, API, or Blockchain Event issues
- **💡 Quick Fixes**: Step-by-step commands to resolve common problems
- **📚 Additional Context**: Links to documentation and troubleshooting guides
- **🎯 Helpful Suggestions**: Context-aware recommendations based on the error type

**Example error output:**

```bash
❌ Error: Docker daemon not running

🐳 Docker Issue
💡 Troubleshooting Steps:
   1. Check Docker is running:
      docker --version
   2. Start Docker Desktop or Docker daemon:
      sudo systemctl start docker  # Linux
      # or open Docker Desktop      # macOS/Windows
   3. Try again: aggsandbox start --detach

🔗 Need more help?
   • Run aggsandbox --help for detailed information
   • Check logs with aggsandbox logs
   • Visit our troubleshooting guide
```

### Performance Optimizations

#### Connection Pooling

- HTTP connection reuse for API calls
- Reduced latency for repeated operations
- Better resource utilization

#### Response Caching

- Intelligent caching of bridge data
- Faster response times for repeated queries
- Configurable cache TTL

#### Resource Management

- Optimized Docker resource allocation
- Smart container lifecycle management
- Efficient volume handling

## Development

### Developer Workflow

```bash
# Development mode with auto-rebuild
aggsandbox start --detach --verbose

# Watch logs during development
aggsandbox logs --follow

# Clean restart (recommended when making changes)
aggsandbox stop --volumes
aggsandbox start --build --detach
```

> **⚠️ Developer Note**: When modifying services or contracts during development, always clear volumes before starting a new environment to ensure a clean state.

### Project Structure

```
agg-sandbox/
├── cli/                    # Rust CLI implementation
├── agglayer-contracts/     # Smart contracts (Foundry)
├── config/                 # Configuration files
├── docker-compose.yml      # Standard mode configuration
├── docker-compose.multi-l2.yml  # Multi-L2 mode configuration
├── scripts/                # Deployment and utility scripts
└── Makefile               # Build targets and commands
```

### Build Targets

```bash
# Show all available make targets
make help

# Install CLI tool
make install

# Uninstall CLI tool  
make uninstall

# Build Docker images
make build

# Clean up build artifacts
make clean
```

## Troubleshooting

The CLI provides comprehensive error handling with context-specific guidance. Most issues will be automatically diagnosed with helpful suggestions.

### Enhanced Error Handling

When errors occur, you'll see:

1. **Clear Error Description**: What went wrong
2. **Issue Category**: Docker, Configuration, API, or Event-related  
3. **Quick Fix Steps**: Specific commands to resolve the issue
4. **Additional Help**: Links to detailed troubleshooting

### Common Issues and Solutions

#### Fork Mode Issues

**Fork URL validation failed:**

```bash
❌ Error: Fork URL validation failed

🔧 Configuration Issue
💡 Quick Fix:
   1. Check your .env file:
      cat .env
   2. Verify fork URLs are accessible:
      curl -X POST "$FORK_URL_MAINNET" \
        -H "Content-Type: application/json" \
        --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'
   3. Check API key validity (if required)
   4. Verify rate limits aren't exceeded
```

**Manual troubleshooting:**

- Ensure your fork URLs are accessible and support the required RPC methods
- Check that your API keys (if required) are properly configured  
- Some RPC providers have rate limits that may affect performance
- Test fork URLs independently before using them with the sandbox

#### Docker Issues

**Docker daemon not running:**

```bash
❌ Error: Docker daemon not running

🐳 Docker Issue
💡 Troubleshooting Steps:
   1. Check Docker is running:
      docker --version
   2. Start Docker Desktop or Docker daemon:
      sudo systemctl start docker  # Linux
      # or open Docker Desktop      # macOS/Windows
   3. Try again: aggsandbox start --detach
```

**Port conflicts:**

```bash
❌ Error: Port 8545 already in use

🐳 Docker Issue  
💡 Quick Fix:
   1. Stop existing containers:
      aggsandbox stop
   2. Check what's using the port:
      lsof -i :8545            # macOS/Linux
      netstat -ano | findstr 8545  # Windows
   3. Either stop the conflicting service or change ports in docker-compose.yml
```

**Manual troubleshooting:**

```bash
# Try rebuilding images
aggsandbox start --build

# Check detailed logs
aggsandbox logs --verbose

# Use verbose mode for more information
aggsandbox start --detach -vv

# Check Docker system resources
docker system df
docker system prune  # Clean up if needed
```

#### Configuration Issues

**Missing environment variables:**

```bash
❌ Error: Required environment variable FORK_URL_MAINNET not found

🔧 Configuration Issue
💡 Quick Fix:
   1. Create or edit your .env file:
      echo 'FORK_URL_MAINNET=your_url' >> .env
   2. Or set it temporarily:
      export FORK_URL_MAINNET=your_url
   3. Verify the variable is set:
      echo $FORK_URL_MAINNET
```

**Configuration validation:**

```bash
# Check all environment variables
env | grep -E "(FORK_URL|RPC_URL|CHAIN_ID)"

# Validate configuration files
aggsandbox info --validate

# Test configuration
aggsandbox start --dry-run
```

#### API Connection Issues

**Bridge service not responding:**

```bash
❌ Error: Bridge service not responding

🌐 API Connection Issue
💡 Troubleshooting Steps:
   1. Check sandbox status:
      aggsandbox status
   2. Start if not running:
      aggsandbox start --detach
   3. Wait for services to be ready (30-60s)
   4. Check service logs:
      aggsandbox logs bridge-service
   5. Verify service health:
      curl http://localhost:5577/health
```

**Service health checks:**

```bash
# Check if bridge service is responding
curl -f http://localhost:5577/health || echo "Service not healthy"

# Check all services
aggsandbox status --detailed

# Restart specific service
docker compose restart bridge-service
```

#### Performance Issues

**Slow startup times:**

```bash
# Use cached images
aggsandbox start --detach  # Don't use --build unless necessary

# Clean up Docker system
docker system prune --volumes

# Check system resources
docker stats
```

**High resource usage:**

```bash
# Monitor resource usage
docker stats --no-stream

# Reduce resource allocation in docker-compose.yml
services:
  anvil-l1:
    deploy:
      resources:
        limits:
          memory: 512M
        reservations:
          memory: 256M
```

### Getting Additional Help

```bash
# Get comprehensive help
aggsandbox --help

# Command-specific help with examples
aggsandbox start --help
aggsandbox show --help

# Enable verbose logging for debugging
aggsandbox start --detach -vv

# Check service status and logs
aggsandbox status
aggsandbox logs                    # All services
aggsandbox logs bridge-service     # Specific service
aggsandbox logs --follow anvil-l1  # Follow specific service

# Validate configuration
aggsandbox info --validate
```

### Diagnostic Commands

```bash
# System health check
aggsandbox status --health

# Configuration dump
aggsandbox info --verbose

# Network connectivity test
aggsandbox test-connectivity

# Service logs with timestamps
aggsandbox logs --timestamps --verbose
```

## Contributing

We welcome contributions to the Agglayer Sandbox! Here's how you can help:

### Development Setup

```bash
# Clone the repository
git clone https://github.com/NethermindEth/agg-sandbox.git
cd agg-sandbox

# Install development dependencies
make install-dev

# Run tests
make test
```

### Areas for Contribution

- **CLI Development**: See [`cli/DEVELOPMENT.md`](cli/DEVELOPMENT.md) for detailed development workflows
- **Smart Contracts**: Located in `agglayer-contracts/` using Foundry
- **Documentation**: Help improve this README and other documentation
- **Testing**: Add test cases and improve test coverage
- **Bug Fixes**: Fix issues and improve stability

### Development Guidelines

1. **Code Style**: Follow Rust formatting guidelines (`cargo fmt`)
2. **Testing**: Add tests for new features (`cargo test`)
3. **Documentation**: Update documentation for new features
4. **Linting**: Run `cargo clippy` to check for common issues

### Project Management

```bash
# Show all available make targets
make help

# Build the project
make build

# Run tests
make test

# Install development version
make install-dev

# Clean build artifacts
make clean
```

### Submitting Changes

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests and documentation
5. Submit a pull request

## License

[Add your license information here]
