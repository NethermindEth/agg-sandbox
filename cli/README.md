# AggLayer Sandbox CLI

🚀 A comprehensive Rust CLI tool for managing your AggLayer sandbox environment with advanced features and enhanced user experience.

## Features

✨ **Enhanced User Experience**
- Rich help messages with examples and detailed explanations
- Progress tracking with animated spinners for long operations
- Comprehensive error handling with step-by-step troubleshooting guides
- Visual feedback with colors and emojis throughout the interface

🏗️ **Core Functionality**
- Full Docker Compose orchestration for L1/L2 chains and bridge services
- Real-time log monitoring with filtering and following capabilities
- Bridge data querying and blockchain event monitoring
- Configuration management with TOML/YAML support
- Fork mode for testing with real blockchain data

⚡ **Performance & Reliability**
- HTTP connection pooling and response caching for improved performance
- Structured logging with configurable verbosity levels
- Comprehensive error handling with context-specific suggestions
- Async operations with progress feedback for better responsiveness

## Installation

### Build from source

```bash
cd cli
cargo build --release
```

The binary will be available at `target/release/aggsandbox`.

### Install globally

```bash
cd cli
cargo install --path .
```

## Quick Start

Make sure you're in the project root directory (where `docker-compose.yml` exists) before running commands.

```bash
# Get comprehensive help with examples
aggsandbox --help

# Start the sandbox with progress tracking
aggsandbox start --detach

# Check status of all services
aggsandbox status

# Follow logs in real-time
aggsandbox logs -f

# Query bridge information
aggsandbox show bridges --network 1

# Monitor blockchain events
aggsandbox events --chain anvil-l1 --blocks 5
```

## Commands Reference

### 🚀 Start Command

Start the AggLayer sandbox environment with Docker Compose.

```bash
# Basic usage
aggsandbox start                     # Start with default settings
aggsandbox start --detach            # Start in background with progress tracking
aggsandbox start --build             # Rebuild images before starting
aggsandbox start --fork              # Use real blockchain data
aggsandbox start --fork --multi-l2   # Fork mode with multiple L2 chains
```

**Options:**
- `-d, --detach`: Start services in background (detached mode)
- `-b, --build`: Rebuild Docker images before starting
- `-f, --fork`: Use real blockchain data from FORK_URL environment variables
- `-m, --multi-l2`: Start with a second L2 chain for multi-chain testing

**Fork Mode Setup:**
When using `--fork`, set these environment variables in your `.env` file:
```bash
FORK_URL_MAINNET=https://mainnet.infura.io/v3/YOUR_KEY
FORK_URL_AGGLAYER_1=https://polygon-zkevm.drpc.org
FORK_URL_AGGLAYER_2=https://polygon-zkevm.drpc.org  # For multi-L2 mode
```

### 🛑 Stop Command

Stop all sandbox services gracefully.

```bash
aggsandbox stop          # Stop services, keep data
aggsandbox stop -v       # Stop services and remove volumes (⚠️ destructive)
```

**Options:**
- `-v, --volumes`: Remove Docker volumes and all persistent data (destructive operation)

### 📊 Status Command

Display the current status of all sandbox services with health checks and port information.

```bash
aggsandbox status
```

### 📋 Logs Command

Display and monitor logs from sandbox services with advanced filtering.

```bash
# Basic log viewing
aggsandbox logs                    # Show all logs
aggsandbox logs bridge-service     # Show specific service logs
aggsandbox logs anvil-l1           # Show L1 node logs
aggsandbox logs anvil-l2           # Show L2 node logs

# Real-time log monitoring
aggsandbox logs -f                 # Follow all logs
aggsandbox logs -f bridge-service  # Follow specific service logs
```

**Options:**
- `-f, --follow`: Stream logs continuously (like 'tail -f')

**Available Services:**
- `anvil-l1`: Ethereum L1 chain
- `anvil-l2`: Polygon zkEVM L2 chain
- `bridge-service`: Cross-chain bridge service
- `agglayer`: AggLayer service
- `contract-deployer`: Contract deployment service

### 🔄 Restart Command

Restart all sandbox services (stop followed by start, preserving volumes).

```bash
aggsandbox restart
```

### ℹ️ Info Command

Display comprehensive sandbox configuration information including network details, account addresses, and contract deployments.

```bash
aggsandbox info
```

### 🌉 Show Commands

Access bridge data and blockchain information from the AggLayer bridge service API.

#### Bridge Information
```bash
aggsandbox show bridges --network 1     # Show L1 bridges (Ethereum)
aggsandbox show bridges --network 1101  # Show L2 bridges (Polygon zkEVM)
```

#### Claims Information
```bash
aggsandbox show claims --network 1      # Show L1 claims (deposits to be claimed on L2)
aggsandbox show claims --network 1101   # Show L2 claims (withdrawals to be claimed on L1)
```

#### Claim Proofs
```bash
aggsandbox show claim-proof --network 1 --leaf-index 0 --deposit-count 1
aggsandbox show claim-proof -n 1101 -l 5 -d 10  # Using short form options
```

#### L1 Info Tree Index
```bash
aggsandbox show l1-info-tree-index --network 1 --deposit-count 0
aggsandbox show l1-info-tree-index -n 1101 -d 5
```

**Network IDs:**
- `1` = Ethereum L1
- `1101` = Polygon zkEVM L2
- `1102` = Additional L2 (if multi-L2 enabled)

### 📡 Events Command

Monitor and fetch blockchain events from L1 and L2 chains.

```bash
# Basic event monitoring
aggsandbox events --chain anvil-l1              # Recent L1 events
aggsandbox events --chain anvil-l2 --blocks 20  # Last 20 L2 blocks
aggsandbox events --chain anvil-l1 --address 0x123  # Filter by contract address
```

**Options:**
- `-c, --chain`: Blockchain to query (anvil-l1, anvil-l2, or anvil-l3)
- `-b, --blocks`: Number of recent blocks to scan (default: 10)
- `-a, --address`: Contract address to filter events (0x...)

**Available Chains:**
- `anvil-l1`: Ethereum L1 chain
- `anvil-l2`: Polygon zkEVM L2 chain  
- `anvil-l3`: Additional L2 chain (if multi-L2 enabled)

## Global Options

All commands support these global options:

```bash
# Logging and verbosity
-v, --verbose       # Enable debug output (-v for debug, -vv for trace)
-q, --quiet         # Suppress all output except errors and warnings
--log-format FORMAT # Set log format: pretty, compact, or json

# Help
-h, --help          # Show command help with examples
-V, --version       # Show version information
```

## Configuration

### Environment Variables

The CLI supports configuration through environment variables and `.env` files:

```bash
# Fork mode URLs (for --fork option)
FORK_URL_MAINNET=https://mainnet.infura.io/v3/YOUR_KEY
FORK_URL_AGGLAYER_1=https://polygon-zkevm.drpc.org
FORK_URL_AGGLAYER_2=https://polygon-zkevm.drpc.org

# API Configuration (optional)
AGG_API_BASE_URL=http://localhost:8080
AGG_API_TIMEOUT=30
AGG_API_RETRY_ATTEMPTS=3
```

### Configuration Files

The CLI supports TOML and YAML configuration files (auto-detected):

```toml
# aggsandbox.toml
[api]
base_url = "http://localhost:8080"
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
```

## Error Handling & Troubleshooting

The CLI provides comprehensive error messages with context-specific troubleshooting guides:

### Common Issues

**Docker Issues:**
```bash
# If Docker is not running
aggsandbox status  # Check current state
docker --version   # Verify Docker installation

# If ports are in use
aggsandbox stop    # Stop existing containers
# Or change ports in docker-compose.yml
```

**Configuration Issues:**
```bash
# Missing environment variables
echo 'FORK_URL_MAINNET=your_url' >> .env

# Configuration validation
aggsandbox info    # Check current configuration
```

**API Connection Issues:**
```bash
# Check if services are running
aggsandbox status

# Start services if needed
aggsandbox start --detach

# Check service logs
aggsandbox logs bridge-service
```

### Getting Help

```bash
# General help with examples
aggsandbox --help

# Command-specific help
aggsandbox <command> --help

# Detailed troubleshooting
aggsandbox logs  # Check service logs for errors
```

## Services Managed

The CLI manages these Docker services:

- **anvil-l1**: Ethereum L1 testnet (port 8545)
- **anvil-l2**: Polygon zkEVM L2 testnet (port 8546)
- **anvil-l3**: Additional L2 chain (port 8547, multi-L2 mode only)
- **bridge-service**: Cross-chain bridge API service
- **agglayer**: AggLayer consensus service
- **contract-deployer**: Automated contract deployment

## Development

### Running from Source

```bash
cd cli
cargo run -- start --detach
cargo run -- status
cargo run -- logs --follow
```

### Building and Testing

```bash
# Build release version
cargo build --release

# Run all tests
cargo test

# Run with strict linting
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt
```

### Adding New Commands

1. Add new command variant to `Commands` enum in `src/main.rs`
2. Implement command handler in `src/commands/`
3. Add comprehensive help text and examples
4. Include unit and integration tests

## Performance Features

- **Connection Pooling**: HTTP client reuse for improved API performance
- **Response Caching**: Intelligent caching with configurable TTLs per endpoint
- **Async Operations**: Non-blocking operations with progress feedback
- **Batch Processing**: Concurrent API operations with configurable limits
- **Performance Monitoring**: Built-in metrics and timing information
