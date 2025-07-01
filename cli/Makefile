# Makefile for CLI project - matches CI exactly
.PHONY: check format clippy test build clean all

# Run all checks (same as CI)
all: format clippy check test

# Check code formatting (same as CI)
format:
	cargo fmt --all -- --check

# Run clippy with same flags as CI + explicit strict lints
clippy:
	cargo clippy --all-targets --all-features -- -D warnings -D clippy::uninlined_format_args

# Run clippy with CI-exact flags only
clippy-ci:
	cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with extra strict lints for development
clippy-strict:
	cargo clippy --all-targets --all-features -- -D warnings \
		-D clippy::uninlined_format_args \
		-D clippy::redundant_field_names \
		-D clippy::redundant_static_lifetimes \
		-D clippy::single_match_else \
		-D clippy::unnecessary_wraps \
		-D clippy::used_underscore_binding

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