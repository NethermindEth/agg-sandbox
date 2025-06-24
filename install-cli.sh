#!/bin/bash

# AggLayer Sandbox CLI Installation Script

set -e

echo "🦀 Building AggLayer Sandbox CLI..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first: https://rustup.rs/"
    exit 1
fi

# Build the CLI
cd cli
echo "📦 Building release version..."
cargo build --release

# Copy to a location in PATH (optional)
INSTALL_DIR="$HOME/.local/bin"
CLI_BINARY="./target/release/agg-sandbox"

if [ -d "$INSTALL_DIR" ]; then
    echo "📋 Installing to $INSTALL_DIR..."
    cp "$CLI_BINARY" "$INSTALL_DIR/"
    echo "✅ CLI installed successfully!"
    echo "🔧 Make sure $INSTALL_DIR is in your PATH"
    echo ""
    echo "Usage: agg-sandbox --help"
else
    echo "✅ CLI built successfully!"
    echo "📍 Binary location: $(pwd)/target/release/agg-sandbox"
    echo "🔧 You can run it directly or add it to your PATH"
    echo ""
    echo "Usage: ./cli/target/release/agg-sandbox --help"
fi

echo ""
echo "🚀 Ready to manage your AggLayer sandbox!" 