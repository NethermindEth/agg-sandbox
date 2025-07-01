#!/bin/bash
# Script to run the exact same checks as CI
# This ensures local development matches CI exactly

set -e  # Exit on any error

echo "🔍 Running CI checks locally..."

echo "📝 Checking code formatting..."
cargo fmt --all -- --check

echo "🔧 Running clippy with CI settings..."
cargo clippy --all-targets --all-features -- -D warnings

echo "✅ Running cargo check..."
cargo check --all-targets --all-features

echo "🧪 Running tests..."
cargo test --all-features

echo "🎉 All CI checks passed locally!" 