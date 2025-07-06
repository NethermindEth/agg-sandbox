# AggLayer Sandbox

A development sandbox environment for the AggLayer with support for local blockchain simulation and fork mode.

## Features

- **Local Mode**: Run completely local blockchain nodes for development
- **Fork Mode**: Fork existing blockchains to test against real network state
- **Multi-L2 Mode**: Run with a second L2 chain for multi-chain testing (supports both local and fork modes)
- **Enhanced CLI** with rich help messages, progress tracking, and intelligent error handling
- **Advanced Configuration** with TOML/YAML file support and environment variable management
- **Performance Optimizations** with HTTP connection pooling and response caching
- **Comprehensive Monitoring** with structured logging and detailed troubleshooting guides
- Pre-configured accounts and private keys
- Docker-based deployment for consistent environments

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust (for CLI compilation) - [Install Rust](https://rustup.rs/)
- Make (for using Makefile targets) - usually pre-installed on Unix systems
- Ensure `~/.local/bin` is in your PATH for CLI installation

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/NethermindEth/agg-sandbox.git
   cd agg-sandbox
   ```

2. Install the CLI tool:

   ```bash
   make install
   ```

3. Verify installation:

   ```bash
   aggsandbox --help
   ```

   You should see comprehensive help with examples and rich formatting.

4. Uninstall (if needed):

   ```bash
   make uninstall
   ```

### Usage

The CLI provides comprehensive commands with enhanced user experience including progress tracking, detailed help, and intelligent error messages:

#### Local Mode (Default)

Start with completely local blockchain simulation (shows progress tracking):

```bash
aggsandbox start --detach
```

The CLI will display a progress bar with step-by-step feedback during startup.

#### Available Flags

- `--detach` (`-d`): Run in detached mode
- `--build` (`-b`): Build images before starting  
- `--fork` (`-f`): Enable fork mode (uses real blockchain data)
- `--multi-l2` (`-m`): Enable multi-L2 mode (runs with a second L2 chain)

#### Fork Mode

> ⚠️ **Note**: Currently only Polygon PoS can be used for forking. Polygon zkEVM will not work due to an Anvil compatibility issue.

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

> ⚠️ **Note**: Multi-L2 mode is currently not working and under development.

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

All commands now include enhanced help and error handling:

```bash
# Check status (works for all modes)
aggsandbox status

# View logs with improved filtering and real-time following
aggsandbox logs --follow
aggsandbox logs bridge-service  # Specific service logs
aggsandbox logs -f anvil-l1     # Follow L1 node logs

# Stop the sandbox (automatically detects and stops all configurations)
aggsandbox stop
aggsandbox stop --volumes  # ⚠️  Also remove volumes (destructive)

# Show comprehensive configuration info
aggsandbox info

# Get detailed help for any command
aggsandbox <command> --help
```

#### Bridge Information Commands

Query bridge endpoints with enhanced formatting and detailed explanations:

```bash
# Show bridges for a network (1=L1 Ethereum, 1101=L2 Polygon zkEVM)
aggsandbox show bridges --network 1      # L1 bridges  
aggsandbox show bridges --network 1101   # L2 bridges

# Show claims for a network
aggsandbox show claims --network 1       # L1 claims (deposits to be claimed on L2)
aggsandbox show claims --network 1101    # L2 claims (withdrawals to be claimed on L1)

# Show claim proof with cryptographic verification data
aggsandbox show claim-proof --network 1 --leaf-index 0 --deposit-count 1
aggsandbox show claim-proof -n 1101 -l 5 -d 10  # Short form options

# Show L1 info tree index for deposit verification
aggsandbox show l1-info-tree-index --network 1 --deposit-count 0
aggsandbox show l1-info-tree-index -n 1101 -d 5
```

All `show` commands now include comprehensive help with detailed explanations:
```bash
aggsandbox show --help           # Overview of all bridge commands
aggsandbox show bridges --help   # Detailed bridge command help
```

These commands query the bridge service at `http://localhost:5577` and display:

- **bridges**: Available bridges for the specified network
- **claims**: Claims information for the specified network  
- **claim-proof**: Claim proof data with configurable parameters
- **l1-info-tree-index**: L1 info tree index data with configurable network and deposit count

#### Event Monitoring Commands

Monitor and decode blockchain events in human-readable format:

```bash
# Show events from anvil-l1 chain (last 5 blocks by default)
aggsandbox events --chain anvil-l1

# Show events from anvil-l2 chain with custom block range
aggsandbox events --chain anvil-l2 --blocks 10

# Show events from anvil-l3 chain (if running multi-l2 mode)
aggsandbox events --chain anvil-l3 --blocks 20

# Filter events by contract address
aggsandbox events --chain anvil-l1 --blocks 5 --address 0x5fbdb2315678afecb367f032d93f642f64180aa3

# Show events with more blocks for comprehensive monitoring
aggsandbox events --chain anvil-l1 --blocks 50
```

Each event displays:

- 🕐 Timestamp and block number
- 📄 Transaction hash
- 📍 Contract address
- 🎯 Event signature and decoded parameters
- 🔍 Raw data for debugging

## Advanced Features

### Enhanced CLI Experience

The CLI now includes several user experience improvements:

- **🎨 Rich Help Messages**: Comprehensive help with examples, emojis, and detailed explanations
- **📊 Progress Tracking**: Visual progress bars with step-by-step feedback during long operations
- **🚨 Smart Error Handling**: Context-specific error messages with troubleshooting suggestions
- **🔍 Verbose Logging**: Configurable log levels for debugging (`-v` for debug, `-vv` for trace)
- **⚡ Performance Optimizations**: HTTP connection pooling and response caching for better performance

### Logging and Verbosity

Control output verbosity and format:

```bash
# Enable verbose output for debugging
aggsandbox start --detach -v        # Debug level
aggsandbox start --detach -vv       # Trace level (very detailed)

# Quiet mode (only errors and warnings)
aggsandbox start --detach --quiet

# Different log formats
aggsandbox start --detach --log-format json     # Machine-readable JSON logs
aggsandbox start --detach --log-format compact  # Compact format
aggsandbox start --detach --log-format pretty   # Default human-readable format
```

### Error Handling and Troubleshooting

When errors occur, the CLI provides:

- **🔧 Specific Issue Categories**: Docker, Configuration, API, or Blockchain Event issues
- **💡 Quick Fixes**: Step-by-step commands to resolve common problems
- **📚 Additional Context**: Links to documentation and troubleshooting guides
- **🎯 Helpful Suggestions**: Context-aware recommendations based on the error type

Example error output includes:
```bash
❌ Error: Docker daemon not running

🐳 Docker Issue
💡 Troubleshooting Steps:
   1. Check Docker is running:
      docker --version
   2. Start Docker Desktop
   3. Try again: aggsandbox start --detach

🔗 Need more help?
   • Run aggsandbox --help for detailed information
   • Check logs with aggsandbox logs
```

## Configuration

The sandbox supports multiple configuration methods with enhanced validation and error reporting:

### Environment Variables (`.env` file)

The traditional method using environment variables:

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

### Configuration Files (New!)

The CLI now supports TOML and YAML configuration files for more structured configuration:

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

[accounts]
accounts = ["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"]
private_keys = ["0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"]
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

accounts:
  accounts:
    - "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
  private_keys:
    - "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
```

**Configuration Priority**: Environment variables take precedence over configuration files, allowing for easy overrides.

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

## Contributing

For developers who want to contribute to the AggLayer Sandbox:

- **CLI Development**: See [`cli/DEVELOPMENT.md`](cli/DEVELOPMENT.md) for detailed development workflows
- **Smart Contracts**: Located in `agglayer-contracts/` using Foundry
- **Project Management**: Use `make help` from the project root to see available build targets

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

The CLI now provides comprehensive error handling with context-specific guidance. Most issues will be automatically diagnosed with helpful suggestions.

### Enhanced Error Handling

When errors occur, you'll see:

1. **Clear Error Description**: What went wrong
2. **Issue Category**: Docker, Configuration, API, or Event-related
3. **Quick Fix Steps**: Specific commands to resolve the issue
4. **Additional Help**: Links to detailed troubleshooting

### Common Issues and Solutions

#### Fork Mode Issues

```bash
# If fork URLs are not accessible
❌ Error: Fork URL validation failed

🔧 Configuration Issue
💡 Quick Fix:
   1. Check your .env file:
      cat .env
   2. Verify fork URLs are accessible:
      curl -X POST your_fork_url -H "Content-Type: application/json" --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'
   3. Check API key validity (if required)
```

- Ensure your fork URLs are accessible and support the required RPC methods
- Check that your API keys (if required) are properly configured  
- Some RPC providers have rate limits that may affect performance

#### Docker Issues

The CLI automatically detects and provides specific guidance for Docker issues:

```bash
# Docker not running
❌ Error: Docker daemon not running

🐳 Docker Issue
💡 Troubleshooting Steps:
   1. Check Docker is running:
      docker --version
   2. Start Docker Desktop
   3. Try again: aggsandbox start --detach

# Port conflicts
❌ Error: Port 8545 already in use

🐳 Docker Issue  
💡 Quick Fix:
   1. Stop existing containers:
      aggsandbox stop
   2. Check what's using the port:
      lsof -i :8545
   3. Either stop the conflicting service or change ports in docker-compose.yml
```

Manual troubleshooting:
- Try rebuilding images: `aggsandbox start --build`
- Check detailed logs: `aggsandbox logs -v`
- Use verbose mode for more information: `aggsandbox start --detach -vv`

#### Configuration Issues

```bash
# Missing environment variables
❌ Error: Required environment variable FORK_URL_MAINNET not found

🔧 Configuration Issue
💡 Quick Fix:
   1. Create or edit your .env file:
      echo 'FORK_URL_MAINNET=your_url' >> .env
   2. Or set it temporarily:
      export FORK_URL_MAINNET=your_url
```

#### API Connection Issues

```bash
# Services not ready
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
aggsandbox logs  # All services
aggsandbox logs bridge-service  # Specific service
```

## License

[Add your license information here]
