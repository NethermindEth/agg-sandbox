# Agglayer Sandbox CLI - Development Guide

This comprehensive guide covers all aspects of developing the Agglayer Sandbox CLI, from project structure to advanced development patterns.

## Project Structure

```bash
aggsandbox/
├── cli/                    # Rust CLI application (this directory)
│   ├── src/               # Source code
│   │   ├── main.rs                 # Entry point and CLI setup
│   │   ├── commands/               # Command handlers
│   │   │   ├── mod.rs             # Command exports and common functionality
│   │   │   ├── start.rs           # Start command with progress tracking
│   │   │   ├── stop.rs            # Stop command implementation
│   │   │   ├── status.rs          # Status command implementation
│   │   │   ├── show.rs            # Bridge data query commands
│   │   │   ├── events.rs          # Blockchain event monitoring
│   │   │   ├── info.rs            # Configuration display
│   │   │   ├── logs.rs            # Log viewing and filtering
│   │   │   └── restart.rs         # Restart command implementation
│   │   ├── config.rs              # Configuration management (TOML/YAML)
│   │   ├── docker.rs              # Docker Compose operations
│   │   ├── error.rs               # Custom error types and handling
│   │   ├── api.rs                 # HTTP API client for bridge endpoints
│   │   ├── api_client.rs          # Optimized HTTP client with caching
│   │   ├── batch_processor.rs     # Concurrent API operation processing
│   │   ├── performance.rs         # Performance monitoring and metrics
│   │   ├── progress.rs            # Progress tracking and user feedback
│   │   ├── events.rs              # Ethereum event fetching and decoding
│   │   ├── logging.rs             # Structured logging configuration
│   │   ├── validation.rs          # Input validation and sanitization
│   │   └── lib.rs                 # Library exports
│   ├── Cargo.toml         # Rust dependencies
│   ├── Makefile          # CLI-specific build targets
│   ├── clippy.toml       # Clippy configuration
│   ├── check-ci.sh       # CI simulation script
│   └── DEVELOPMENT.md    # This file
├── agglayer-contracts/     # Smart contracts (Foundry)
├── scripts/               # Shell scripts
├── images/                # Docker images
├── Makefile              # Project-level build targets
└── install-cli.sh        # Installation script
```

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Docker and Docker Compose
- Git

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/agglayer/aggsandbox
cd aggsandbox/cli

# Install dependencies and build
cargo build

# Run tests to verify setup
cargo test

# Install development tools
cargo install cargo-watch cargo-tarpaulin
```

## Development Workflow

### From Project Root

These commands work from the **project root** and are useful for general project management:

```bash
# Build CLI for development
make cli-build

# Run all CLI checks (format, clippy, tests)
make cli-check

# Clean build artifacts
make cli-clean

# Install/uninstall system-wide
make install
make uninstall

# See all available targets
make help
```

### From CLI Directory

For active CLI development, work from the `cli/` directory with these commands:

```bash
cd cli

# Run exactly what CI runs
make clippy-ci
./check-ci.sh

# Development with strict lints
make clippy

# Format code
make fmt

# Quick development check
make dev-check

# Build and run during development
cargo run -- --help
cargo run -- start --detach

# Watch for changes and rebuild
cargo watch -x 'build'
cargo watch -x 'test'

# Run with debug logging
RUST_LOG=debug cargo run -- start --detach -vv
```

## Quick Start

### Run All CI Checks Locally

```bash
# Using the shell script (recommended)
./check-ci.sh

# Or using make
make all
```

### Individual Commands

```bash
# Check code formatting
make format
# or: cargo fmt --all -- --check

# Run clippy (matches CI exactly)
make clippy-ci
# or: cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with extra strict lints (recommended for development)
make clippy
# or: make clippy-strict

# Run tests
make test
# or: cargo test --all-features

# Build release binary
make build
# or: cargo build --release

# Run strict linting (CI standard)
make clippy-strict

# Run all tests with coverage
cargo tarpaulin --out html --output-dir coverage/
```

## Code Quality Standards

### Code Organization

- **Single Responsibility**: Each module has a clear, focused purpose
- **Error Handling**: All functions return `Result<T, E>` with proper error context
- **Documentation**: All public APIs have comprehensive documentation
- **Testing**: Minimum 80% test coverage with unit and integration tests

### Coding Conventions

```rust
// Function naming: descriptive verbs
pub fn validate_network_id(id: u64) -> Result<u64, ConfigError> { }

// Error handling: custom error types with context
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Network request failed: {endpoint}")]
    NetworkError { endpoint: String, #[source] source: reqwest::Error },
}

// Logging: structured with context
#[instrument(fields(network_id = network_id), skip(config))]
pub async fn get_bridges(config: &Config, network_id: u64) -> Result<BridgeResponse> {
    debug!("Fetching bridges for network {}", network_id);
    // ...
}

// Configuration: use structured config with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    pub retry_attempts: u32,
}
```

### Performance Guidelines

- Use `Arc<T>` for shared immutable data
- Implement connection pooling for HTTP clients
- Cache responses with appropriate TTLs
- Use async operations for I/O bound tasks
- Profile performance-critical code paths

## Testing Strategy

### Test Types

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test command workflows end-to-end
3. **Performance Tests**: Benchmark critical operations
4. **Reliability Tests**: Error scenarios and edge cases

### Running Tests

```bash
# All tests
cargo test

# Specific test module
cargo test config::tests

# Integration tests only
cargo test --test integration_tests

# Performance benchmarks
cargo test --release -- --ignored perf

# Test with coverage
cargo tarpaulin --out html

# Test specific features
cargo test --features "yaml-config"
```

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_api_client_caching() {
        let mock_server = MockServer::start().await;
        // ... setup mock responses
        
        let client = OptimizedApiClient::new(CacheConfig::default());
        let result = client.get_bridges(&config, 1).await;
        
        assert!(result.is_ok());
        // Verify caching behavior
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            api: ApiConfig {
                base_url: "invalid-url".to_string(),
                timeout: Duration::from_secs(30),
                retry_attempts: 3,
            },
            // ...
        };
        
        assert!(config.validate().is_err());
    }
}
```

## Adding New Features

### 1. Adding a New Command

```bash
# 1. Create command handler
touch src/commands/new_command.rs

# 2. Add to commands/mod.rs
echo "pub mod new_command;" >> src/commands/mod.rs
echo "pub use new_command::handle_new_command;" >> src/commands/mod.rs

# 3. Add to main.rs Commands enum
# 4. Implement command logic with proper error handling
# 5. Add comprehensive tests
# 6. Update help documentation
```

Example command implementation:

```rust
// src/commands/new_command.rs
use crate::error::Result;
use crate::progress::StatusReporter;
use tracing::{info, instrument};

#[instrument]
pub async fn handle_new_command(param: String) -> Result<()> {
    let reporter = StatusReporter::new();
    
    info!("Executing new command with param: {}", param);
    reporter.info(&format!("Processing: {}", param)).await;
    
    // Implementation logic here
    
    reporter.success("Command completed successfully").await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_new_command() {
        let result = handle_new_command("test".to_string()).await;
        assert!(result.is_ok());
    }
}
```

### 2. Adding Configuration Options

```rust
// Add to config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFeatureConfig {
    pub enabled: bool,
    pub option1: String,
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
}

impl Default for NewFeatureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            option1: "default_value".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

// Add validation
impl NewFeatureConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.option1.is_empty() {
            return Err(ConfigError::InvalidValue {
                key: "option1".to_string(),
                value: self.option1.clone(),
                reason: "cannot be empty".to_string(),
            });
        }
        Ok(())
    }
}
```

### 3. Adding API Endpoints

```rust
// Add to api_client.rs
impl OptimizedApiClient {
    #[instrument(fields(param = param), skip(self, config))]
    pub async fn get_new_data(
        &self,
        config: &Config,
        param: u64,
    ) -> Result<serde_json::Value> {
        let cache_key = CacheKey::new("new-endpoint".to_string())
            .with_network_id(param);

        let url = format!(
            "{}/api/v1/new-endpoint?param={}",
            config.api.base_url, param
        );

        self.get_cached_or_fetch(cache_key, || async {
            self.get(&url).await
        }).await
    }
}
```

## Error Handling Best Practices

### Custom Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum MyModuleError {
    #[error("Invalid input: {input}")]
    InvalidInput { input: String },
    
    #[error("Operation failed: {operation}")]
    OperationFailed {
        operation: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
}
```

### Error Context and Propagation

```rust
use anyhow::Context;

pub async fn complex_operation(input: &str) -> Result<String> {
    let validated = validate_input(input)
        .with_context(|| format!("Failed to validate input: {}", input))?;
    
    let processed = process_data(&validated)
        .await
        .with_context(|| "Data processing failed")?;
    
    Ok(processed)
}
```

## Performance Monitoring

### Adding Metrics

```rust
use crate::performance::{global_performance_monitor, AsyncTimer};

#[instrument]
pub async fn monitored_operation() -> Result<String> {
    global_performance_monitor()
        .time_async("operation_name", async {
            // Your operation here
            Ok("result".to_string())
        })
        .await
}
```

### Benchmarking

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    #[ignore] // Mark as performance test
    async fn benchmark_api_operations() {
        let start = Instant::now();
        
        // Run operation multiple times
        for _ in 0..100 {
            let _ = api_operation().await;
        }
        
        let elapsed = start.elapsed();
        println!("100 operations took: {:?}", elapsed);
        assert!(elapsed < Duration::from_secs(5)); // Performance threshold
    }
}
```

## CI Consistency

The tools in this directory ensure your local development environment matches CI exactly:

- **`check-ci.sh`**: Runs the exact same sequence of checks as GitHub Actions CI
- **`Makefile`**: Provides convenient targets for different types of checks
- **`clippy.toml`**: Basic clippy configuration for consistent behavior

### CI/CD Pipeline

The project uses GitHub Actions for:

- **Continuous Integration**: Run tests on all PRs
- **Code Quality**: Enforce formatting and linting
- **Security**: Audit dependencies for vulnerabilities
- **Performance**: Track performance regressions

## Installation

**Note**: Installation commands should be run from the **project root**, not from the `cli/` directory:

```bash
# From project root
make install      # Build and install CLI to ~/.local/bin
make uninstall    # Remove CLI from ~/.local/bin

# Alternative: use the install script
./install-cli.sh
```

### Build Targets

```bash
# Development build
make build

# Release build
make release

# Cross-compilation
make build-linux
make build-windows
make build-macos

# Install locally
make install

# Uninstall
make uninstall
```

## Before You Push

Always run one of these before pushing:

```bash
./check-ci.sh    # Full CI simulation
make all         # Same as above via make
make dev-check   # Quick format + clippy check
```

## Debugging

### Logging Levels

```bash
# Error level only
RUST_LOG=error cargo run -- command

# Info level (default)
RUST_LOG=info cargo run -- command

# Debug level (detailed)
RUST_LOG=debug cargo run -- command -v

# Trace level (very detailed)
RUST_LOG=trace cargo run -- command -vv

# Module-specific logging
RUST_LOG=aggsandbox::api=debug cargo run -- command
```

### Common Debug Techniques

```rust
// Add temporary debug prints
dbg!(&variable);

// Use debug logging
debug!("Processing item: {:?}", item);

// Add timing information
let start = Instant::now();
// ... operation
debug!("Operation took: {:?}", start.elapsed());

// Conditional compilation for debug builds
#[cfg(debug_assertions)]
println!("Debug info: {:?}", debug_data);
```

## Release Process

### Version Management

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with new features and fixes
3. Run full test suite: `make test-all`
4. Run strict linting: `make clippy-strict`
5. Build release binary: `cargo build --release`
6. Tag release: `git tag v0.2.0`

## Contributing Guidelines

### Pull Request Process

1. **Fork and Branch**: Create feature branch from `main`
2. **Development**: Implement changes with tests
3. **Quality Checks**: Run `make lint test`
4. **Documentation**: Update relevant documentation
5. **Pull Request**: Create PR with detailed description
6. **Review**: Address feedback from maintainers
7. **Merge**: Squash and merge after approval

### Code Review Checklist

- [ ] Code follows style guidelines (`cargo fmt`)
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] Error handling implemented
- [ ] Performance considerations addressed
- [ ] Security implications reviewed

### Commit Message Format

```
type(scope): brief description

Detailed explanation of the change, including:
- Why the change was needed
- What was changed
- Any breaking changes or migration notes

Fixes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Troubleshooting

### Clippy Errors

If clippy fails locally but you think it should pass:

1. Make sure you're using `make clippy-ci` (matches CI exactly)
2. Check that your code follows the inline format args style: `format!("text {variable}")` instead of `format!("text {}", variable)`

### Format Errors

Run `make fmt` to auto-format your code, then run `make format` to verify.

### Version Differences

The CI uses the latest stable Rust. Update your toolchain:

```bash
rustup update stable
```

### Common Build Issues

```bash
# Clean build artifacts
cargo clean && cargo build

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated

# Audit for security issues
cargo audit
```

## IDE Setup

### VS Code Extensions

- `rust-analyzer`: Language server
- `CodeLLDB`: Debugging support
- `crates`: Dependency management
- `Even Better TOML`: TOML syntax support

### IntelliJ/CLion

- Install Rust plugin
- Enable Cargo integration
- Configure debugger for Rust

## External Dependencies

### Key Dependencies

- **clap**: Command-line argument parsing
- **tokio**: Async runtime
- **reqwest**: HTTP client
- **serde**: Serialization/deserialization
- **tracing**: Structured logging
- **anyhow**: Error handling
- **thiserror**: Custom error types

### Adding Dependencies

```bash
# Add runtime dependency
cargo add dependency-name

# Add development dependency
cargo add --dev test-dependency

# Add feature-gated dependency
cargo add optional-dep --optional

# Update Cargo.toml with features
```

Consider: license compatibility, maintenance status, security, performance impact.
