# AggLayer Sandbox CLI

A Rust CLI tool for managing your AggLayer sandbox environment.

## Installation

### Build from source

```bash
cd cli
cargo build --release
```

The binary will be available at `target/release/agg-sandbox`.

### Install globally

```bash
cd cli
cargo install --path .
```

## Usage

Make sure you're in the project root directory (where `docker-compose.yml` exists) before running commands.

### Commands

#### Start the sandbox
```bash
# Start normally (interactive)
agg-sandbox start

# Start in detached mode
agg-sandbox start --detach

# Start with building images
agg-sandbox start --build

# Start detached with build
agg-sandbox start --detach --build
```

#### Stop the sandbox
```bash
# Stop normally
agg-sandbox stop

# Stop and remove volumes
agg-sandbox stop --volumes
```

#### Check status
```bash
agg-sandbox status
```

#### View logs
```bash
# Show all logs
agg-sandbox logs

# Follow logs (like tail -f)
agg-sandbox logs --follow

# Show logs for specific service
agg-sandbox logs anvil-mainnet
agg-sandbox logs anvil-polygon
agg-sandbox logs contract-deployer

# Follow logs for specific service
agg-sandbox logs --follow anvil-mainnet
```

#### Restart
```bash
agg-sandbox restart
```

#### Show configuration and accounts
```bash
agg-sandbox info
```

### Help
```bash
agg-sandbox --help
agg-sandbox start --help
agg-sandbox logs --help
```

## Services Managed

Based on your `docker-compose.yml`, this CLI manages:

- **anvil-mainnet**: Ethereum testnet on port 8545
- **anvil-polygon**: Polygon testnet on port 8546  
- **contract-deployer**: Deploys contracts to both networks

## Development

### Run directly with Cargo
```bash
cd cli
cargo run -- start --detach
cargo run -- status
cargo run -- logs --follow
```

### Add new commands
Edit `src/main.rs` and add new variants to the `Commands` enum, then implement the corresponding function. 