# Development Guide

Comprehensive guide for contributing to and developing the Agglayer Sandbox.

## Development Environment Setup

### Prerequisites

```bash
# Required tools
rustc --version    # >= 1.70.0
cargo --version
docker --version   # >= 20.0
make --version
git --version

# Optional but recommended
foundry --version  # For smart contract development
jq --version      # For JSON processing
```

### Repository Setup

```bash
# Clone repository
git clone https://github.com/NethermindEth/agg-sandbox.git
cd agg-sandbox

# Install development dependencies
make install-dev

# Set up pre-commit hooks
cargo install pre-commit
pre-commit install
```

### Development Workflow

```bash
# Build CLI in development mode
make cli-build
cd cli && cargo build

# Run tests
make test
cd cli && cargo test

# Run all checks (format, clippy, test)
make cli-check
cd cli && make all

# Install local development version
make install-dev
```

## Project Architecture

### Repository Structure

```
agg-sandbox/
├── cli/                    # Rust CLI implementation
│   ├── src/
│   │   ├── commands/       # CLI command implementations
│   │   │   ├── bridge/     # Bridge operations
│   │   │   │   ├── bridge_asset.rs
│   │   │   │   ├── bridge_call.rs
│   │   │   │   ├── claim_asset.rs
│   │   │   │   ├── claim_message.rs
│   │   │   │   └── utilities.rs
│   │   │   ├── show/       # Information commands
│   │   │   └── start.rs    # Environment management
│   │   ├── api_client.rs   # HTTP client for AggKit API
│   │   ├── config.rs       # Configuration management
│   │   ├── docker.rs       # Docker Compose operations
│   │   ├── error.rs        # Error handling
│   │   ├── types.rs        # Data structures
│   │   └── main.rs         # Entry point
│   ├── Cargo.toml         # Rust dependencies
│   └── tests/             # Integration tests
├── agglayer-contracts/     # Smart contracts (Foundry)
│   ├── src/               # Contract source code
│   ├── script/            # Deployment scripts
│   ├── test/              # Contract tests
│   └── foundry.toml       # Foundry configuration
├── images/                # Custom Docker images
├── scripts/               # Utility scripts
├── config/                # Configuration files
├── docker-compose.yml     # Standard mode
├── docker-compose.multi-l2.yml  # Multi-L2 mode
├── Makefile              # Build targets
└── docs/                 # Documentation
```

### CLI Architecture

The CLI is built in Rust using the following key components:

#### Command Structure

```rust
// cli/src/commands/mod.rs
pub enum Commands {
    Start(StartArgs),
    Stop(StopArgs),
    Status,
    Info(InfoArgs),
    Logs(LogsArgs),
    Bridge(BridgeCommands),
    Show(ShowCommands),
    Events(EventsArgs),
}
```

#### Bridge Commands

```rust
// cli/src/commands/bridge/mod.rs
pub enum BridgeCommands {
    Asset(AssetArgs),
    Claim(ClaimArgs),
    Message(MessageArgs),
    BridgeAndCall(BridgeAndCallArgs),
    Utils(UtilityCommands),
}
```

#### Configuration Management

```rust
// cli/src/config.rs
pub struct Config {
    pub api: ApiConfig,
    pub networks: HashMap<String, NetworkConfig>,
    pub accounts: AccountsConfig,
    pub docker: DockerConfig,
    pub logging: LoggingConfig,
}
```

## Development Tasks

### Adding New CLI Commands

#### 1. Define Command Structure

```rust
// cli/src/commands/your_command.rs
use clap::Args;
use crate::error::CliResult;

#[derive(Args, Debug)]
pub struct YourCommandArgs {
    /// Description of your argument
    #[arg(long, short)]
    pub your_arg: String,

    /// Optional argument with default
    #[arg(long, default_value = "default")]
    pub optional_arg: String,
}

pub async fn execute(args: YourCommandArgs) -> CliResult<()> {
    // Implementation here
    println!("Executing your command with arg: {}", args.your_arg);
    Ok(())
}
```

#### 2. Add to Command Enum

```rust
// cli/src/commands/mod.rs
pub enum Commands {
    // ... existing commands
    YourCommand(YourCommandArgs),
}
```

#### 3. Update Command Router

```rust
// cli/src/main.rs or appropriate router
match cli.command {
    Commands::YourCommand(args) => your_command::execute(args).await,
    // ... other commands
}
```

#### 4. Add Tests

```rust
// cli/src/commands/your_command.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_your_command() {
        let args = YourCommandArgs {
            your_arg: "test".to_string(),
            optional_arg: "test".to_string(),
        };

        let result = execute(args).await;
        assert!(result.is_ok());
    }
}
```

### Adding Bridge Functionality

#### 1. Smart Contract Integration

```rust
// cli/src/commands/bridge/your_bridge_op.rs
use ethers::prelude::*;
use crate::error::CliResult;

pub async fn your_bridge_operation(
    provider: &Provider<Http>,
    contract_address: Address,
    args: &YourBridgeArgs,
) -> CliResult<TxHash> {
    // Create contract instance
    let contract = YourContract::new(contract_address, Arc::new(provider.clone()));

    // Prepare transaction
    let tx = contract
        .your_function(args.param1, args.param2)
        .gas(args.gas_limit.unwrap_or(3_000_000));

    // Send transaction
    let pending_tx = tx.send().await?;
    let receipt = pending_tx.await?.ok_or("Transaction failed")?;

    Ok(receipt.transaction_hash)
}
```

#### 2. API Client Extension

```rust
// cli/src/api_client.rs
impl ApiClient {
    pub async fn your_new_endpoint(
        &self,
        network_id: u32,
        param: &str,
    ) -> Result<YourResponse, Error> {
        let url = format!(
            "{}/bridge/v1/your-endpoint?network_id={}&param={}",
            self.base_url, network_id, param
        );

        let response = self.client.get(&url).send().await?;
        let data: YourResponse = response.json().await?;
        Ok(data)
    }
}
```

#### 3. Data Types

```rust
// cli/src/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YourResponse {
    pub field1: String,
    pub field2: u64,
    pub optional_field: Option<String>,
}
```

### Smart Contract Development

#### Project Setup

```bash
# Navigate to contracts directory
cd agglayer-contracts

# Install dependencies
forge install

# Build contracts
forge build

# Run tests
forge test

# Deploy locally
forge script script/deployL1.s.sol --rpc-url $RPC_1 --broadcast
```

#### Adding New Contracts

```solidity
// agglayer-contracts/src/YourContract.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract YourContract {
    event YourEvent(address indexed user, uint256 amount);

    mapping(address => uint256) public balances;

    function yourFunction(uint256 amount) external {
        balances[msg.sender] += amount;
        emit YourEvent(msg.sender, amount);
    }
}
```

#### Contract Tests

```solidity
// agglayer-contracts/test/YourContract.t.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/YourContract.sol";

contract YourContractTest is Test {
    YourContract public yourContract;
    address public user = makeAddr("user");

    function setUp() public {
        yourContract = new YourContract();
    }

    function testYourFunction() public {
        vm.startPrank(user);

        yourContract.yourFunction(100);

        assertEq(yourContract.balances(user), 100);
        vm.stopPrank();
    }
}
```

#### Deployment Scripts

```solidity
// agglayer-contracts/script/deployYourContract.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/YourContract.sol";

contract DeployYourContract is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY_1");

        vm.startBroadcast(deployerPrivateKey);

        YourContract yourContract = new YourContract();

        console.log("YourContract deployed at:", address(yourContract));

        vm.stopBroadcast();
    }
}
```

### Docker Development

#### Custom Docker Images

```dockerfile
# images/your-service/Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/your-service /usr/local/bin/
CMD ["your-service"]
```

#### Service Integration

```yaml
# docker-compose.yml
services:
  your-service:
    build:
      context: .
      dockerfile: images/your-service/Dockerfile
    ports:
      - "9000:9000"
    environment:
      - LOG_LEVEL=${LOG_LEVEL:-info}
    depends_on:
      - anvil-l1
      - anvil-l2
```

## Testing

### Unit Tests

```rust
// cli/src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let config = Config::from_file("test-config.toml").unwrap();
        assert_eq!(config.api.base_url, "http://localhost:5577");
    }

    #[tokio::test]
    async fn test_api_client() {
        let client = ApiClient::new("http://localhost:5577");
        // Mock server testing
    }
}
```

### Integration Tests

```rust
// cli/tests/integration_tests.rs
use aggsandbox_cli::*;
use tokio_test;

#[tokio::test]
async fn test_bridge_flow() {
    // Start test environment
    let env = TestEnvironment::new().await;

    // Test asset bridging
    let result = bridge_asset(&env.config, &bridge_args).await;
    assert!(result.is_ok());

    // Test claiming
    let claim_result = claim_asset(&env.config, &claim_args).await;
    assert!(claim_result.is_ok());

    env.cleanup().await;
}
```

### Contract Tests

```bash
# Run all contract tests
cd agglayer-contracts
forge test

# Run specific test
forge test --match-test testBridgeAsset

# Run with verbose output
forge test -vvv

# Run with coverage
forge coverage
```

### End-to-End Tests

```bash
#!/bin/bash
# tests/e2e/full_bridge_test.sh

set -e

echo "Starting E2E bridge test..."

# Start sandbox
aggsandbox start --detach
sleep 30

# Test asset bridge
TX_HASH=$(aggsandbox bridge asset --network 0 --destination-network 1 --amount 0.1 --token-address 0x0000000000000000000000000000000000000000 --json | jq -r '.tx_hash')

# Wait for processing
sleep 10

# Test claim
aggsandbox bridge claim --network 1 --tx-hash $TX_HASH --source-network 0

# Verify success
CLAIMS=$(aggsandbox show claims --network-id 1 --json | jq '.claims | length')
if [ "$CLAIMS" -gt 0 ]; then
    echo "✅ E2E test passed"
else
    echo "❌ E2E test failed"
    exit 1
fi

# Cleanup
aggsandbox stop --volumes
```

## Code Quality

### Formatting

```bash
# Format Rust code
cargo fmt

# Check formatting
cargo fmt --check

# Format all files
make format
```

### Linting

```bash
# Run Clippy
cargo clippy

# Clippy with all features
cargo clippy --all-features

# Clippy as errors
cargo clippy -- -D warnings
```

### Documentation

````rust
/// Bridge assets between networks
///
/// # Arguments
/// * `args` - Bridge operation arguments
/// * `config` - CLI configuration
///
/// # Returns
/// * `CliResult<String>` - Transaction hash on success
///
/// # Examples
/// ```
/// let args = BridgeAssetArgs { /* ... */ };
/// let result = bridge_asset(args, &config).await?;
/// ```
pub async fn bridge_asset(
    args: BridgeAssetArgs,
    config: &Config,
) -> CliResult<String> {
    // Implementation
}
````

### Performance

```rust
// Use proper error handling
use anyhow::{Context, Result};

// Use efficient data structures
use std::collections::HashMap;
use indexmap::IndexMap;

// Async best practices
use tokio::task::JoinSet;
use futures::future::join_all;

// Connection pooling
use reqwest::Client;
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_max_idle_per_host(10)
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap()
});
```

## Release Process

### Version Management

```bash
# Update version in Cargo.toml
sed -i 's/version = "0.1.0"/version = "0.2.0"/' cli/Cargo.toml

# Create git tag
git tag v0.2.0
git push origin v0.2.0
```

### Release Checklist

- [ ] All tests passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Integration tests pass
- [ ] Performance benchmarks run
- [ ] Security review completed

### Build and Distribution

```bash
# Build release binaries
cargo build --release

# Cross-compilation for different targets
cargo install cross
cross build --target x86_64-pc-windows-gnu --release
cross build --target aarch64-apple-darwin --release

# Create distribution packages
make dist
```

## Contributing Guidelines

### Pull Request Process

1. **Fork the repository**
2. **Create feature branch** from `main`
3. **Make changes** following code style
4. **Add tests** for new functionality
5. **Update documentation** as needed
6. **Run all checks** (`make check`)
7. **Submit pull request** with clear description

### Code Style

- **Rust**: Follow `rustfmt` and `clippy` suggestions
- **Solidity**: Use consistent naming and documentation
- **Shell scripts**: Use `shellcheck` for validation
- **Documentation**: Clear, concise, with examples

### Commit Messages

```
type(scope): short description

Longer description explaining the change and why it was made.

Closes #123
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`

### Issue Templates

Use provided GitHub issue templates for:

- Bug reports
- Feature requests
- Documentation improvements
- Performance issues

## Development Tools

### Recommended VS Code Extensions

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "serayuzgur.crates",
    "tamasfe.even-better-toml",
    "JuanBlanco.solidity"
  ]
}
```

### Debug Configuration

```json
// .vscode/launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug CLI",
      "cargo": {
        "args": ["build", "--bin", "aggsandbox"],
        "filter": {
          "name": "aggsandbox",
          "kind": "bin"
        }
      },
      "args": ["bridge", "asset", "--help"],
      "cwd": "${workspaceFolder}/cli"
    }
  ]
}
```

### Git Hooks

```bash
#!/bin/sh
# .git/hooks/pre-commit
set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt --check

# Lint check
cargo clippy -- -D warnings

# Test check
cargo test

echo "All checks passed!"
```

## Next Steps

- **[CLI Reference](cli-reference.md)** - Complete command documentation
- **[Configuration](configuration.md)** - Advanced configuration options
- **[Troubleshooting](troubleshooting.md)** - Debug development issues
- **[Bridge Operations](bridge-operations.md)** - Understanding bridge mechanics
