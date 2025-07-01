# CLI Development Guide

This guide covers all aspects of developing the AggLayer Sandbox CLI, from project structure to CI consistency.

## Project Structure

```bash
agg-sandbox/
├── cli/                    # Rust CLI application (this directory)
│   ├── src/               # Source code
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
```

## CI Consistency

The tools in this directory ensure your local development environment matches CI exactly:

- **`check-ci.sh`**: Runs the exact same sequence of checks as GitHub Actions CI
- **`Makefile`**: Provides convenient targets for different types of checks
- **`clippy.toml`**: Basic clippy configuration for consistent behavior

## Installation

**Note**: Installation commands should be run from the **project root**, not from the `cli/` directory:

```bash
# From project root
make install      # Build and install CLI to ~/.local/bin
make uninstall    # Remove CLI from ~/.local/bin

# Alternative: use the install script
./install-cli.sh
```

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
