# Makefile for CLI project - matches CI exactly
.PHONY: check format clippy test build clean all help

# Run all checks (same as CI)
all: format clippy check test

# Check code formatting (same as CI)
format:
	cargo fmt --all -- --check

# Run clippy (matches cargo clippy behavior)
clippy:
	cargo clippy --all-targets --all-features -- -D warnings -A clippy::disallowed_methods -A clippy::disallowed_types

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

# Show available targets
help:
	@echo "Available targets:"
	@echo "  all        - Run all checks (format, clippy, check, test)"
	@echo "  format     - Check code formatting"
	@echo "  fmt        - Format code"
	@echo "  clippy     - Run clippy linter"
	@echo "  check      - Run cargo check"
	@echo "  test       - Run all tests"
	@echo "  build      - Build release binary"
	@echo "  clean      - Clean build artifacts"
	@echo "  dev-check  - Quick format and clippy check"
	@echo "  help       - Show this help message"
