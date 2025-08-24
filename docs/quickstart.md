# Quick Start Guide

Get the Agglayer Sandbox running in minutes.

## Prerequisites

### System Requirements

- **Docker** >= 20.0 and Docker Compose >= 1.27
- **Rust** >= 1.70.0 - [Install Rust](https://rustup.rs/)
- **Make** (usually pre-installed on Unix systems)
- **Git** for cloning the repository

### PATH Configuration

Ensure `~/.local/bin` is in your PATH:

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
```

## Installation

### 1. Clone Repository

```bash
git clone https://github.com/agglayer/aggsandbox
cd aggsandbox
```

### 2. Install CLI Tool

```bash
make install
```

This will:

- Build and install the CLI to `~/.local/bin`
- Set up bridge service dependencies
- Configure bridge functionality

### 3. Verify Installation

```bash
aggsandbox --help
```

You should see comprehensive help with examples and rich formatting.

### 4. Uninstall (if needed)

```bash
make uninstall
```

## Basic Usage

### 1. Environment Setup

```bash
# Create environment file
cp .env.example .env && source .env
```

### 2. Start Sandbox

```bash
# Start in local mode (default)
aggsandbox start --detach
```

The CLI displays a progress bar with step-by-step feedback during startup.

### 3. Check Status

```bash
aggsandbox status
```

### 4. First Bridge Operation

```bash
# Bridge ETH from L1 to L2
aggsandbox bridge asset \
  --network 0 \
  --destination-network 1 \
  --amount 0.1 \
  --token-address 0x0000000000000000000000000000000000000000
```

### 5. View Bridge Information

```bash
# Check bridges on L1
aggsandbox show bridges --network-id 0

# Check claims on L2
aggsandbox show claims --network-id 1
```

### 6. Stop Sandbox

```bash
aggsandbox stop
```

## Verification Tests

### Test Network Connectivity

```bash
# Check that both chains are running
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'

curl -X POST http://127.0.0.1:8546 \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'
```

### Test Bridge API

```bash
# Test bridge service health
curl http://localhost:5577/health

# Test bridge API endpoint
curl http://localhost:5577/bridge/v1/bridges?network_id=0
```

## Usage Modes

### Local Mode (Default)

```bash
aggsandbox start --detach
```

- Fully local simulation
- Fast startup
- No external dependencies

### Fork Mode

1. **Configure fork URLs in `.env`:**

   ```bash
   # Ethereum mainnet fork
   FORK_URL_MAINNET=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY

   # Polygon PoS fork
   FORK_URL_AGGLAYER_1=https://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY
   ```

2. **Start in fork mode:**
   ```bash
   aggsandbox start --fork --detach
   ```

### Multi-L2 Mode

```bash
# Local multi-L2 (3 chains)
aggsandbox start --multi-l2 --detach

# Fork multi-L2 (requires all fork URLs configured)
aggsandbox start --multi-l2 --fork --detach
```

## Common Commands

### Environment Management

```bash
# Start with progress tracking
aggsandbox start --detach

# Start with verbose output
aggsandbox start --detach --verbose

# Start with image rebuilding
aggsandbox start --build --detach

# Start with automatic sponsor of all claims
aggsandbox start --detach --claim-all

# Stop gracefully
aggsandbox stop

# Stop and remove volumes (clean slate)
aggsandbox stop --volumes
```

### Information Commands

```bash
# Check status
aggsandbox status

# Show configuration
aggsandbox info

# View logs
aggsandbox logs --follow

# View specific service logs
aggsandbox logs bridge-service
aggsandbox logs anvil-l1
```

### Bridge Commands

```bash
# Bridge assets
aggsandbox bridge asset --network 0 --destination-network 1 --amount 0.1 --token-address 0x0000000000000000000000000000000000000000

# Claim assets
aggsandbox bridge claim --network 1 --tx-hash <hash> --source-network 0

# Show bridge information
aggsandbox show bridges --network-id 0
aggsandbox show claims --network-id 1

# Monitor events
aggsandbox events --network-id 0
```

## Environment Variables

Key variables in your `.env` file:

```bash
# RPC endpoints
RPC_URL_1=http://127.0.0.1:8545  # L1
RPC_URL_2=http://127.0.0.1:8546  # L2

# Network IDs
NETWORK_ID_MAINNET=0
NETWORK_ID_AGGLAYER_1=1
NETWORK_ID_AGGLAYER_2=2

# Pre-configured test accounts
ACCOUNT_ADDRESS_1=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
PRIVATE_KEY_1=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

⚠️ **Security Note**: These are well-known test keys. Never use them with real funds.

## Next Steps

Now that you have the sandbox running:

1. **[Bridge Operations](bridge-operations.md)** - Learn complete bridging workflows
2. **[CLI Reference](cli-reference.md)** - Explore all available commands
3. **[Advanced Workflows](advanced-workflows.md)** - Complex bridging scenarios
4. **[Configuration](configuration.md)** - Customize your environment

## Getting Help

```bash
# General help
aggsandbox --help

# Command-specific help
aggsandbox start --help
aggsandbox bridge --help

# Enable verbose logging
aggsandbox start --detach --verbose
```

For issues not covered here, see [Troubleshooting](troubleshooting.md).
