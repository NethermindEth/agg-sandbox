# CLI Reference

Complete command reference for the `aggsandbox` CLI tool.

## Global Options

Available for all commands:

```bash
--verbose, -v      # Enable verbose output for debugging
--quiet, -q        # Quiet mode (only errors and warnings)
--help, -h         # Show comprehensive help
--version, -V      # Show version information
```

## Core Commands

### `aggsandbox start`

Start the sandbox environment.

```bash
aggsandbox start [OPTIONS]
```

**Options:**

- `--detach, -d` - Run in detached mode
- `--build, -b` - Build images before starting
- `--fork, -f` - Enable fork mode (uses real blockchain data)
- `--multi-l2, -m` - Enable multi-L2 mode (runs with second L2 chain)
- `--verbose, -v` - Enable verbose output

**Examples:**

```bash
# Start in local mode
aggsandbox start --detach

# Start with fork mode
aggsandbox start --fork --detach

# Start multi-L2 mode
aggsandbox start --multi-l2 --detach

# Start with image rebuilding
aggsandbox start --build --detach
```

### `aggsandbox stop`

Stop the sandbox environment.

```bash
aggsandbox stop [OPTIONS]
```

**Options:**

- `--volumes` - Remove volumes (destructive, clean slate)

**Examples:**

```bash
# Stop gracefully
aggsandbox stop

# Stop and remove all data
aggsandbox stop --volumes
```

### `aggsandbox status`

Check sandbox status.

```bash
aggsandbox status
```

Shows the status of all running services.

### `aggsandbox info`

Display comprehensive configuration information.

```bash
aggsandbox info [OPTIONS]
```

**Options:**

- `--verbose, -v` - Show detailed configuration

### `aggsandbox logs`

View service logs.

```bash
aggsandbox logs [OPTIONS] [SERVICE]
```

**Options:**

- `--follow, -f` - Follow log output in real-time
- `--tail <lines>` - Number of lines to show from the end
- `--since <time>` - Show logs since timestamp
- `--verbose, -v` - Verbose log output

**Examples:**

```bash
# View all logs
aggsandbox logs

# Follow all logs
aggsandbox logs --follow

# View specific service logs
aggsandbox logs bridge-service
aggsandbox logs anvil-l1
aggsandbox logs anvil-l2

# Follow specific service
aggsandbox logs --follow anvil-l1
```

## Bridge Commands

### `aggsandbox bridge asset`

Bridge ERC20 tokens or ETH between networks.

```bash
aggsandbox bridge asset [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Source network ID (0=L1, 1=L2, 2=L3)
- `--destination-network-id, -d <ID>` - Destination network ID
- `--amount, -a <AMOUNT>` - Amount to bridge (in token units)
- `--token-address, -t <ADDRESS>` - Token contract address (use `0x0000000000000000000000000000000000000000` for ETH)

**Optional Options:**

- `--to-address <ADDRESS>` - Recipient address (defaults to sender)
- `--gas-limit <LIMIT>` - Gas limit override
- `--gas-price <PRICE>` - Gas price override in wei
- `--private-key <KEY>` - Private key to use

**Examples:**

```bash
# Bridge ETH from L1 to L2
aggsandbox bridge asset \
  --network-id 0 \
  --destination-network-id 1 \
  --amount 0.1 \
  --token-address 0x0000000000000000000000000000000000000000

# Bridge ERC20 tokens from L2 to L1
aggsandbox bridge asset \
  --network-id 1 \
  --destination-network-id 0 \
  --amount 100 \
  --token-address 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC \
  --to-address 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
```

### `aggsandbox bridge claim`

Claim previously bridged assets.

```bash
aggsandbox bridge claim [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Network to claim assets on
- `--tx-hash, -t <HASH>` - Original bridge transaction hash
- `--source-network-id, -s <ID>` - Source network of the original bridge

**Optional Options:**

- `--deposit-count, -c <COUNT>` - Deposit count for specific bridge (auto-detected if not provided)
- `--data <HEX>` - Custom metadata for message bridge claims (hex encoded)
- `--gas-limit <LIMIT>` - Gas limit override
- `--gas-price <PRICE>` - Gas price override in wei
- `--private-key <KEY>` - Private key to use

**Examples:**

```bash
# Claim assets on L2
aggsandbox bridge claim \
  --network-id 1 \
  --tx-hash 0xb7118cfb20825861028ede1e9586814fc7ccf81745a325db5df355d382d96b4e \
  --source-network-id 0

# Claim specific bridge with deposit count
aggsandbox bridge claim \
  --network-id 1 \
  --tx-hash 0xb7118cfb20825861028ede1e9586814fc7ccf81745a325db5df355d382d96b4e \
  --source-network-id 0 \
  --deposit-count 0
```

### `aggsandbox bridge message`

Bridge with contract calls.

```bash
aggsandbox bridge message [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Source network ID
- `--destination-network-id, -d <ID>` - Destination network ID
- `--target, -t <ADDRESS>` - Target contract address on destination network
- `--data <HEX>` - Contract call data (hex encoded)

**Optional Options:**

- `--amount, -a <AMOUNT>` - Amount of ETH to send
- `--fallback-address <ADDRESS>` - Fallback address if call fails (defaults to sender)
- `--gas-limit <LIMIT>` - Gas limit override
- `--gas-price <PRICE>` - Gas price override in wei
- `--private-key <KEY>` - Private key to use

**Examples:**

```bash
# Bridge ETH with contract call
aggsandbox bridge message \
  --network-id 0 \
  --destination-network-id 1 \
  --target 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 \
  --data 0xa9059cbb000000000000000000000000742d35cc6965c592342c6c16fb8eaeb90a23b5c00000000000000000000000000000000000000000000000000de0b6b3a7640000 \
  --amount 0.01
```

### `aggsandbox bridge bridge-and-call`

Execute bridgeAndCall operations with automatic token approval.

```bash
aggsandbox bridge bridge-and-call [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Source network ID
- `--destination-network-id, -d <ID>` - Destination network ID
- `--token, -t <ADDRESS>` - Token contract address to bridge
- `--amount, -a <AMOUNT>` - Amount to bridge (in token units)
- `--target <ADDRESS>` - Target contract address on destination network
- `--data <HEX>` - Contract call data (hex encoded)
- `--fallback <ADDRESS>` - Fallback address if contract call fails

**Optional Options:**

- `--gas-limit <LIMIT>` - Gas limit override
- `--gas-price <PRICE>` - Gas price override in wei
- `--private-key <KEY>` - Private key to use

**Examples:**

```bash
# Bridge ERC20 tokens with contract call
aggsandbox bridge bridge-and-call \
  --network-id 0 \
  --destination-network-id 1 \
  --token 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC \
  --amount 10 \
  --target 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 \
  --data 0xa9059cbb000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb922660000000000000000000000000000000000000000000000000de0b6b3a7640000 \
  --fallback 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

## Information Commands

### `aggsandbox show bridges`

Show bridge information for a specific network.

```bash
aggsandbox show bridges --network-id <ID> [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Network ID (0=L1, 1=L2, 2=L3)

**Optional Options:**

- `--json` - Output raw JSON without formatting
- `--verbose, -v` - Verbose output

**Examples:**

```bash
# Show bridges for L1
aggsandbox show bridges --network-id 0

# Show bridges with JSON output
aggsandbox show bridges --network-id 1 --json
```

### `aggsandbox show claims`

Show claims information for a specific network.

```bash
aggsandbox show claims --network-id <ID> [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Network ID

**Optional Options:**

- `--json` - Output raw JSON without formatting

**Examples:**

```bash
# Show claims for L2
aggsandbox show claims --network-id 1

# Show claims with JSON output
aggsandbox show claims --network-id 1 --json
```

### `aggsandbox show claim-proof`

Generate claim proof for bridged assets.

```bash
aggsandbox show claim-proof [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Network ID
- `--leaf-index, -l <INDEX>` - Leaf index for proof
- `--deposit-count, -d <COUNT>` - Deposit count for proof

**Optional Options:**

- `--json` - Output raw JSON without formatting

**Examples:**

```bash
# Generate claim proof
aggsandbox show claim-proof \
  --network-id 0 \
  --leaf-index 0 \
  --deposit-count 1

# Generate proof with JSON output
aggsandbox show claim-proof \
  --network-id 0 \
  --leaf-index 0 \
  --deposit-count 1 \
  --json
```

### `aggsandbox show l1-info-tree-index`

Get L1 info tree index for deposit verification.

```bash
aggsandbox show l1-info-tree-index [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Network ID
- `--deposit-count, -d <COUNT>` - Deposit count

**Optional Options:**

- `--json` - Output raw JSON without formatting

**Examples:**

```bash
# Get L1 info tree index
aggsandbox show l1-info-tree-index \
  --network-id 0 \
  --deposit-count 0

# Get index with JSON output
aggsandbox show l1-info-tree-index \
  --network-id 0 \
  --deposit-count 0 \
  --json
```

## Event Monitoring

### `aggsandbox events`

Monitor and decode blockchain events.

```bash
aggsandbox events [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Network ID to query

**Optional Options:**

- `--blocks, -b <COUNT>` - Number of recent blocks to scan (default: 10)
- `--address, -a <ADDRESS>` - Filter events by contract address

**Examples:**

```bash
# Show events from L1 (last 10 blocks)
aggsandbox events --network-id 0

# Show events with custom block range
aggsandbox events --network-id 1 --blocks 20

# Filter events by contract address
aggsandbox events \
  --network-id 0 \
  --blocks 5 \
  --address 0x5fbdb2315678afecb367f032d93f642f64180aa3
```

## Bridge Utilities

### `aggsandbox bridge utils build-payload`

Build complete claim payload from bridge transaction.

```bash
aggsandbox bridge utils build-payload [OPTIONS]
```

**Required Options:**

- `--tx-hash, -t <HASH>` - Bridge transaction hash
- `--source-network-id, -s <ID>` - Source network ID

**Optional Options:**

- `--bridge-index <INDEX>` - Bridge index for multi-bridge transactions
- `--json` - Output as JSON format

### `aggsandbox bridge utils compute-index`

Calculate global bridge index from local index.

```bash
aggsandbox bridge utils compute-index [OPTIONS]
```

**Required Options:**

- `--local-index <INDEX>` - Local bridge index
- `--source-network-id <ID>` - Source network ID

**Optional Options:**

- `--json` - Output as JSON format

### `aggsandbox bridge utils get-mapped`

Get wrapped token address for an origin token.

```bash
aggsandbox bridge utils get-mapped [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Target network ID
- `--origin-network <ID>` - Origin network ID
- `--origin-token <ADDRESS>` - Origin token contract address

**Optional Options:**

- `--private-key <KEY>` - Private key
- `--json` - Output as JSON format

### `aggsandbox bridge utils precalculate`

Pre-calculate wrapped token address before deployment.

```bash
aggsandbox bridge utils precalculate [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Target network ID
- `--origin-network <ID>` - Origin network ID
- `--origin-token <ADDRESS>` - Origin token contract address

**Optional Options:**

- `--json` - Output as JSON format

### `aggsandbox bridge utils get-origin`

Get origin token information from wrapped token.

```bash
aggsandbox bridge utils get-origin [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Network ID
- `--wrapped-token <ADDRESS>` - Wrapped token contract address

**Optional Options:**

- `--json` - Output as JSON format

### `aggsandbox bridge utils is-claimed`

Check if a bridge has been claimed.

```bash
aggsandbox bridge utils is-claimed [OPTIONS]
```

**Required Options:**

- `--network-id, -n <ID>` - Network ID to check
- `--index <INDEX>` - Bridge index to check
- `--source-network-id <ID>` - Source bridge network ID

**Optional Options:**

- `--private-key <KEY>` - Private key
- `--json` - Output as JSON format

## JSON Output

All `show` commands and utility commands support the `--json` flag for machine-readable output:

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

## Environment Variables

The CLI respects these environment variables:

```bash
# RPC endpoints
RPC_URL_1=http://127.0.0.1:8545
RPC_URL_2=http://127.0.0.1:8546
RPC_URL_3=http://127.0.0.1:8547

# Network IDs
NETWORK_ID_MAINNET=0
NETWORK_ID_AGGLAYER_1=1
NETWORK_ID_AGGLAYER_2=2

# Bridge service
BRIDGE_SERVICE_HOST=127.0.0.1
BRIDGE_SERVICE_PORT=5577

# Default account
ACCOUNT_ADDRESS_1=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
PRIVATE_KEY_1=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

## Exit Codes

The CLI uses standard exit codes:

- `0` - Success
- `1` - General error
- `2` - Misuse of shell command
- `130` - Process terminated by user (Ctrl+C)

## See Also

- **[Bridge Operations](bridge-operations.md)** - Complete bridging workflows
- **[Advanced Workflows](advanced-workflows.md)** - Complex scenarios
- **[Configuration](configuration.md)** - Environment setup
- **[Troubleshooting](troubleshooting.md)** - Common issues
