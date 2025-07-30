# Makefile for CLI project - matches CI exactly
.PHONY: check format clippy test build clean all

# Run all checks (same as CI)
all: format clippy check test

# Check code formatting (same as CI)
format:
	cargo fmt --all -- --check

# Run clippy (matches cargo clippy behavior)
clippy:
	cargo clippy

# Run cargo check
check:
	cargo check --all-targets --all-features

# Run tests
test:
	cargo test --all-features

# Build release binary
build:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean

# Format code (non-checking)
fmt:
	cargo fmt --all

# Quick development check (format + clippy)
dev-check: fmt clippy 