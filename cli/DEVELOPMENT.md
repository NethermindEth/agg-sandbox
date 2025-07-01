# Development Guide

This guide helps you run the same checks locally that CI runs, ensuring your code will pass CI before you push.

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
```

## CI Consistency

The tools in this directory ensure your local development environment matches CI exactly:

- **`check-ci.sh`**: Runs the exact same sequence of checks as GitHub Actions CI
- **`Makefile`**: Provides convenient targets for different types of checks
- **`clippy.toml`**: Basic clippy configuration for consistent behavior

## Before You Push

Always run one of these before pushing:
```bash
./check-ci.sh    # Full CI simulation
make all         # Same as above via make
make dev-check   # Quick format + clippy check
```

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