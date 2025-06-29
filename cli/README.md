# AggLayer Sandbox CLI

A Rust CLI tool for managing your AggLayer sandbox environment.

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

## Usage

Make sure you're in the project root directory (where `docker-compose.yml` exists) before running commands.

### Commands

#### Start the sandbox

```bash
# Start normally (interactive)
aggsandbox start

# Start in detached mode
aggsandbox start --detach

# Start with building images
aggsandbox start --build

# Start detached with build
aggsandbox start --detach --build
```

#### Stop the sandbox

```bash
# Stop normally
aggsandbox stop

# Stop and remove volumes
aggsandbox stop --volumes
```

#### Check status

```bash
aggsandbox status
```

#### View logs

```bash
# Show all logs
aggsandbox logs

# Follow logs (like tail -f)
aggsandbox logs --follow

# Show logs for specific service
aggsandbox logs anvil-l1
aggsandbox logs anvil-l2
aggsandbox logs contract-deployer

# Follow logs for specific service
aggsandbox logs --follow anvil-l1
```

#### Restart

```bash
aggsandbox restart
```

#### Show configuration and accounts

```bash
aggsandbox info
```

### Help

```bash
aggsandbox --help
aggsandbox start --help
aggsandbox logs --help
```

## Services Managed

Based on your `docker-compose.yml`, this CLI manages:

- **anvil-l1**: Ethereum testnet on port 8545
- **anvil-l2**: PolygonZkVM testnet on port 8546  
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
