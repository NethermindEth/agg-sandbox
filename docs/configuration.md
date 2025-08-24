# Configuration Guide

Complete configuration reference for customizing your Agglayer Sandbox environment.

## Configuration Sources

The sandbox supports multiple configuration methods with priority order:

1. **Command-line arguments** (highest priority)
2. **Environment variables** (.env file)
3. **TOML configuration files** (aggsandbox.toml)
4. **YAML configuration files** (aggsandbox.yaml)
5. **Default values** (lowest priority)

## Environment Variables

### Core Configuration

Create and configure your `.env` file:

```bash
# Copy the example file
cp .env.example .env
```

**Essential Variables:**

```bash
# RPC endpoints for blockchain networks
RPC_URL_1=http://127.0.0.1:8545  # L1 Ethereum
RPC_URL_2=http://127.0.0.1:8546  # L2 zkEVM
RPC_URL_3=http://127.0.0.1:8547  # L3 Multi-L2 (optional)

# Network ID mappings
NETWORK_ID_MAINNET=0        # L1 network identifier
NETWORK_ID_AGGLAYER_1=1     # First L2 network identifier
NETWORK_ID_AGGLAYER_2=2     # Second L2 network identifier
```

### Bridge Service Configuration

```bash
# Bridge service endpoints
BRIDGE_SERVICE_HOST=127.0.0.1
BRIDGE_SERVICE_PORT=5577
BRIDGE_SERVICE_URL=http://127.0.0.1:5577

# Additional bridge service ports (multi-L2 mode)
BRIDGE_SERVICE_L3_PORT=5578
BRIDGE_SERVICE_L3_URL=http://127.0.0.1:5578
```

### Account Configuration

Pre-configured test accounts with known private keys:

```bash
# Primary test account (Anvil account #0)
ACCOUNT_ADDRESS_1=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
PRIVATE_KEY_1=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Secondary test account (Anvil account #1)
ACCOUNT_ADDRESS_2=0x70997970C51812dc3A010C7d01b50e0d17dc79C8
PRIVATE_KEY_2=0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d

# Additional accounts
ACCOUNT_ADDRESS_3=0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
PRIVATE_KEY_3=0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a
```

⚠️ **Security Warning**: These are well-known test keys from Anvil. **Never use them with real funds or in production environments.**

### Contract Addresses

Bridge contracts are automatically deployed and configured:

```bash
# L1 Bridge contracts
POLYGON_ZKEVM_BRIDGE_L1=0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82
BRIDGE_EXTENSION_L1=0x0B306BF915C4d645ff596e518fAf3F9669b97016

# L2 Bridge contracts
POLYGON_ZKEVM_BRIDGE_L2=0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6
BRIDGE_EXTENSION_L2=0x8A791620dd6260079BF849Dc5567aDC3F2FdC318

# L3 Bridge contracts (Multi-L2 mode)
POLYGON_ZKEVM_BRIDGE_L3=0x5FbDB2315678afecb367f032d93F642f64180aa3
BRIDGE_EXTENSION_L3=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512

# Test ERC20 tokens (automatically deployed)
AGG_ERC20_L1=0x5FbDB2315678afecb367f032d93F642f64180aa3
AGG_ERC20_L2=0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0
```

## Fork Mode Configuration

### Basic Fork Setup

Configure fork URLs to test against real network state:

```bash
# Ethereum mainnet fork
FORK_URL_MAINNET=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY

# Polygon PoS fork (recommended over zkEVM due to Anvil compatibility)
FORK_URL_AGGLAYER_1=https://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY

# Additional L2 fork URL (for multi-L2 mode)
FORK_URL_AGGLAYER_2=https://your-second-l2.com/v1/YOUR_API_KEY
```

### Fork Block Configuration

Pin to specific block heights for consistent testing:

```bash
# Fork from specific blocks
FORK_BLOCK_MAINNET=18500000
FORK_BLOCK_AGGLAYER_1=50000000
FORK_BLOCK_AGGLAYER_2=45000000
```

### Supported Fork Networks

| Network              | Status          | Notes                      |
| -------------------- | --------------- | -------------------------- |
| **Ethereum Mainnet** | ✅ Supported    | Full compatibility         |
| **Polygon PoS**      | ✅ Supported    | Recommended for L2         |
| **Polygon zkEVM**    | ⚠️ Limited      | Anvil compatibility issues |
| **Arbitrum**         | ⚠️ Experimental | Limited testing            |
| **Optimism**         | ⚠️ Experimental | Limited testing            |

## Docker Configuration

### Docker Compose Settings

```bash
# Docker project name
COMPOSE_PROJECT_NAME=aggsandbox

# Docker build settings
DOCKER_BUILDKIT=1
DOCKER_SCAN_SUGGEST=false

# Resource limits
DOCKER_MEMORY_LIMIT=4g
DOCKER_CPU_LIMIT=2
```

### Port Configuration

Customize service ports:

```bash
# Standard mode ports
L1_RPC_PORT=8545
L2_RPC_PORT=8546
BRIDGE_API_PORT=5577
BRIDGE_RPC_PORT=8555
TELEMETRY_PORT=8080

# Multi-L2 mode additional ports
L3_RPC_PORT=8547
BRIDGE_L3_API_PORT=5578
BRIDGE_L3_RPC_PORT=8556
TELEMETRY_L3_PORT=8081
```

### Volume Configuration

```bash
# Data persistence
PERSIST_BLOCKCHAIN_DATA=false
PERSIST_DATABASE_DATA=false

# Volume mount paths
DATA_DIR=./data
LOGS_DIR=./logs
CONFIG_DIR=./config
```

## Configuration Files

### TOML Configuration

Create `aggsandbox.toml` for structured configuration:

```toml
[api]
base_url = "http://localhost:5577"
timeout = "30s"
retry_attempts = 3
connection_pool_size = 10

[networks.l1]
name = "Ethereum-L1"
network_id = "0"
rpc_url = "http://localhost:8545"
chain_id = 1
gas_price = "20000000000"
gas_limit = "8000000"

[networks.l2]
name = "Polygon-zkEVM-L2"
network_id = "1"
rpc_url = "http://localhost:8546"
chain_id = 1101
gas_price = "30000000000"
gas_limit = "10000000"

[networks.l3]
name = "Second-L2-Chain"
network_id = "2"
rpc_url = "http://localhost:8547"
chain_id = 137
gas_price = "50000000000"
gas_limit = "12000000"

[accounts]
accounts = [
  "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
  "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
  "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
]
private_keys = [
  "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
  "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
  "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
]

[contracts.bridges]
l1_bridge = "0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82"
l2_bridge = "0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6"
l3_bridge = "0x5FbDB2315678afecb367f032d93F642f64180aa3"

[contracts.extensions]
l1_extension = "0x0B306BF915C4d645ff596e518fAf3F9669b97016"
l2_extension = "0x8A791620dd6260079BF849Dc5567aDC3F2FdC318"
l3_extension = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"

[logging]
level = "info"
format = "pretty"
enable_colors = true
log_to_file = false
file_path = "./logs/aggsandbox.log"

[docker]
compose_project = "aggsandbox"
build_context = "."
enable_buildkit = true

[fork]
mainnet_url = ""
agglayer_1_url = ""
agglayer_2_url = ""
block_number_mainnet = 0
block_number_agglayer_1 = 0
block_number_agglayer_2 = 0
```

### YAML Configuration

Create `aggsandbox.yaml` for YAML-style configuration:

```yaml
api:
  base_url: "http://localhost:5577"
  timeout: "30s"
  retry_attempts: 3
  connection_pool_size: 10

networks:
  l1:
    name: "Ethereum-L1"
    network_id: "0"
    rpc_url: "http://localhost:8545"
    chain_id: 1
    gas_price: "20000000000"
    gas_limit: "8000000"
  l2:
    name: "Polygon-zkEVM-L2"
    network_id: "1"
    rpc_url: "http://localhost:8546"
    chain_id: 1101
    gas_price: "30000000000"
    gas_limit: "10000000"
  l3:
    name: "Second-L2-Chain"
    network_id: "2"
    rpc_url: "http://localhost:8547"
    chain_id: 137
    gas_price: "50000000000"
    gas_limit: "12000000"

accounts:
  accounts:
    - "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    - "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
    - "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
  private_keys:
    - "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    - "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
    - "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"

contracts:
  bridges:
    l1_bridge: "0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82"
    l2_bridge: "0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6"
    l3_bridge: "0x5FbDB2315678afecb367f032d93F642f64180aa3"
  extensions:
    l1_extension: "0x0B306BF915C4d645ff596e518fAf3F9669b97016"
    l2_extension: "0x8A791620dd6260079BF849Dc5567aDC3F2FdC318"
    l3_extension: "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"

logging:
  level: "info"
  format: "pretty"
  enable_colors: true
  log_to_file: false
  file_path: "./logs/aggsandbox.log"

docker:
  compose_project: "aggsandbox"
  build_context: "."
  enable_buildkit: true

fork:
  mainnet_url: ""
  agglayer_1_url: ""
  agglayer_2_url: ""
  block_number_mainnet: 0
  block_number_agglayer_1: 0
  block_number_agglayer_2: 0
```

## Advanced Configuration

### Performance Tuning

```bash
# HTTP client settings
HTTP_TIMEOUT=30
HTTP_RETRY_ATTEMPTS=3
HTTP_CONNECTION_POOL_SIZE=20

# Bridge operation settings
BRIDGE_CONFIRMATION_BLOCKS=1
BRIDGE_TIMEOUT_SECONDS=300
BRIDGE_GAS_MULTIPLIER=1.2

# Database settings (for bridge services)
DB_MAX_CONNECTIONS=10
DB_CONNECTION_TIMEOUT=30
DB_QUERY_TIMEOUT=60
```

### Resource Limits

```bash
# Anvil node settings
ANVIL_BLOCK_TIME=1
ANVIL_GAS_LIMIT=30000000
ANVIL_GAS_PRICE=1000000000

# Memory and CPU limits
MAX_MEMORY_USAGE=4096  # MB
MAX_CPU_USAGE=200      # Percentage
```

### Logging Configuration

```bash
# Log levels: trace, debug, info, warn, error
LOG_LEVEL=info
LOG_FORMAT=pretty      # Options: pretty, json, compact
LOG_TO_FILE=false
LOG_FILE_PATH=./logs/aggsandbox.log
LOG_MAX_SIZE=100       # MB
LOG_MAX_FILES=5
```

### Development Settings

```bash
# Development mode settings
DEV_MODE=false
ENABLE_DEBUG_ENDPOINTS=false
SKIP_CONTRACT_VERIFICATION=false
AUTO_CLAIM_BRIDGES=false

# Testing settings
FAST_FINALITY=true
SKIP_BRIDGE_DELAYS=true
MOCK_EXTERNAL_CALLS=false
```

## Network-Specific Configuration

### Local Mode

Optimized for development and testing:

```bash
# Fast block times for development
ANVIL_BLOCK_TIME=1
ANVIL_AUTO_MINE=true

# Disable delays
BRIDGE_PROCESSING_DELAY=0
CLAIM_PROCESSING_DELAY=0

# Enable all features
ENABLE_BRIDGE_AND_CALL=true
ENABLE_MESSAGE_BRIDGING=true
ENABLE_UTILITIES=true
```

### Fork Mode

Configured for real network interaction:

```bash
# Realistic block times
ANVIL_BLOCK_TIME=12    # Ethereum block time
FORK_BLOCK_CACHE_SIZE=1000

# Enable state preservation
PERSIST_FORK_STATE=true
FORK_STATE_PATH=./data/fork-state

# Rate limiting for RPC providers
RPC_RATE_LIMIT=10      # Requests per second
RPC_BURST_LIMIT=50
```

### Multi-L2 Mode

Extended configuration for three-chain setup:

```bash
# Enable all L2 services
ENABLE_L3_CHAIN=true
ENABLE_DUAL_AGGKIT=true

# Resource allocation
L3_MEMORY_LIMIT=1g
L3_CPU_LIMIT=0.5

# Additional telemetry
ENABLE_L3_TELEMETRY=true
L3_TELEMETRY_PORT=8081
```

## Configuration Validation

### Validate Configuration

```bash
# Check configuration validity
aggsandbox info --validate

# Show current configuration
aggsandbox info --verbose

# Test configuration without starting
aggsandbox start --dry-run
```

### Environment Validation Script

```bash
#!/bin/bash
# validate_env.sh - Validate environment configuration

echo "Validating Agglayer Sandbox configuration..."

# Required variables
required_vars=(
  "RPC_URL_1" "RPC_URL_2"
  "NETWORK_ID_MAINNET" "NETWORK_ID_AGGLAYER_1"
  "ACCOUNT_ADDRESS_1" "PRIVATE_KEY_1"
)

for var in "${required_vars[@]}"; do
  if [ -z "${!var}" ]; then
    echo "❌ Missing required variable: $var"
    exit 1
  else
    echo "✅ $var is set"
  fi
done

# Test RPC connectivity
echo "Testing RPC connections..."
if cast block-number --rpc-url $RPC_URL_1 >/dev/null 2>&1; then
  echo "✅ L1 RPC connected"
else
  echo "❌ L1 RPC connection failed"
fi

if cast block-number --rpc-url $RPC_URL_2 >/dev/null 2>&1; then
  echo "✅ L2 RPC connected"
else
  echo "❌ L2 RPC connection failed"
fi

# Test bridge service
if curl -s http://localhost:5577/health >/dev/null 2>&1; then
  echo "✅ Bridge service accessible"
else
  echo "⚠️  Bridge service not running (start sandbox first)"
fi

echo "Configuration validation complete"
```

## Custom Docker Configuration

### Override Docker Compose

Create `docker-compose.override.yml` for custom settings:

```yaml
version: "3.8"

services:
  anvil-l1:
    environment:
      - ANVIL_BLOCK_TIME=2
      - ANVIL_GAS_LIMIT=50000000
    ports:
      - "8545:8545"
    deploy:
      resources:
        limits:
          memory: 1g
          cpus: "0.5"

  anvil-l2:
    environment:
      - ANVIL_BLOCK_TIME=1
    ports:
      - "8546:8546"
    deploy:
      resources:
        limits:
          memory: 1g
          cpus: "0.5"

  bridge-service:
    environment:
      - LOG_LEVEL=debug
      - DB_MAX_CONNECTIONS=20
    ports:
      - "5577:5577"
      - "8555:8555"
      - "8080:8080"
    deploy:
      resources:
        limits:
          memory: 2g
          cpus: "1.0"
```

### Environment-Specific Overrides

Create environment-specific compose files:

```bash
# Development environment
docker-compose.dev.yml

# Testing environment
docker-compose.test.yml

# Production-like environment
docker-compose.prod.yml
```

## Configuration Examples

### Development Configuration

Optimized for fast development cycles:

```bash
# .env.dev
LOG_LEVEL=debug
ANVIL_BLOCK_TIME=1
AUTO_CLAIM_BRIDGES=true
SKIP_BRIDGE_DELAYS=true
ENABLE_DEBUG_ENDPOINTS=true
FAST_FINALITY=true
```

### Testing Configuration

Configured for automated testing:

```bash
# .env.test
LOG_LEVEL=warn
LOG_FORMAT=json
LOG_TO_FILE=true
ANVIL_BLOCK_TIME=1
BRIDGE_CONFIRMATION_BLOCKS=1
SKIP_CONTRACT_VERIFICATION=true
```

### Production-Like Configuration

Realistic settings for production testing:

```bash
# .env.prod
LOG_LEVEL=info
ANVIL_BLOCK_TIME=12
BRIDGE_CONFIRMATION_BLOCKS=3
PERSIST_BLOCKCHAIN_DATA=true
ENABLE_DEBUG_ENDPOINTS=false
RPC_RATE_LIMIT=5
```

## Troubleshooting Configuration

### Common Configuration Issues

**Port Conflicts:**

```bash
# Check port usage
lsof -i :8545 -i :8546 -i :5577

# Change ports in .env
L1_RPC_PORT=18545
L2_RPC_PORT=18546
BRIDGE_API_PORT=15577
```

**Memory Issues:**

```bash
# Reduce resource limits
DOCKER_MEMORY_LIMIT=2g
ANVIL_GAS_LIMIT=10000000
DB_MAX_CONNECTIONS=5
```

**Fork URL Issues:**

```bash
# Test fork URLs manually
curl -X POST $FORK_URL_MAINNET \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'
```

### Configuration Reset

```bash
# Reset to defaults
rm aggsandbox.toml aggsandbox.yaml
cp .env.example .env

# Clean slate restart
aggsandbox stop --volumes
aggsandbox start --detach
```

## Next Steps

- **[Troubleshooting](troubleshooting.md)** - Debug configuration issues
- **[Development](development.md)** - Contribute to configuration options
- **[Advanced Workflows](advanced-workflows.md)** - Use custom configurations
- **[CLI Reference](cli-reference.md)** - Command-line configuration options
