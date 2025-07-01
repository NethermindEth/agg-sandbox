#!/bin/bash

# AggLayer Sandbox CLI Installation Script
# This script uses the Makefile for consistent build and install process

set -e

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first: https://rustup.rs/"
    exit 1
fi

# Check if make is available
if ! command -v make &> /dev/null; then
    echo "❌ Make not found. Please install make or run manually:"
    echo "   cd cli && cargo build --release"
    exit 1
fi

echo "🔨 Using root Makefile for installation..."

# Use the root Makefile to build and install
make install 