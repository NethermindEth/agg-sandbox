# Agglayer Sandbox Project Makefile
.PHONY: install uninstall cli-check cli-build cli-clean clean-docker integration-test help

# Default target
help:
	@echo "🦀 Agglayer Sandbox Project"
	@echo ""
	@echo "Available targets:"
	@echo "  install          - Build and install the CLI to ~/.local/bin"
	@echo "  uninstall        - Remove the CLI from ~/.local/bin"
	@echo "  cli-build        - Build the CLI (development)"
	@echo "  cli-check        - Run all CLI checks (format, clippy, test)"
	@echo "  cli-clean        - Clean CLI build artifacts"
	@echo "  clean-docker     - Clean Docker images and force fresh image pulls"
	@echo "  integration-test - Run comprehensive bridge integration tests"
	@echo "  help             - Show this help message"

# Install the CLI binary to user's local bin directory
install:
	@echo "🦀 Installing Agglayer Sandbox CLI..."
	@if [ ! -d "$(HOME)/.local/bin" ]; then \
		echo "📁 Creating $(HOME)/.local/bin directory..."; \
		mkdir -p "$(HOME)/.local/bin"; \
	fi
	@echo "📦 Building release version..."
	@cd cli && cargo build --release --quiet
	@echo "📋 Installing to $(HOME)/.local/bin..."
	@cp cli/target/release/aggsandbox "$(HOME)/.local/bin/"
	@echo "✅ CLI installed successfully!"
	@echo "🔧 Make sure $(HOME)/.local/bin is in your PATH"
	@echo ""
	@echo "Usage: aggsandbox --help"
	@echo "🚀 Ready to manage your Agglayer sandbox!"

# Uninstall the CLI binary
uninstall:
	@echo "🗑️  Uninstalling Agglayer Sandbox CLI..."
	@if [ -f "$(HOME)/.local/bin/aggsandbox" ]; then \
		rm "$(HOME)/.local/bin/aggsandbox"; \
		echo "✅ CLI uninstalled successfully!"; \
	else \
		echo "⚠️  CLI not found in $(HOME)/.local/bin"; \
	fi

# CLI development targets (delegate to cli/Makefile)
cli-build:
	@cd cli && make build

cli-check:
	@cd cli && make all

cli-clean:
	@cd cli && make clean

# Clean Docker images used by compose files
clean-docker:
	@echo "🧹 Cleaning Docker images..."
	@./scripts/clean-docker-images.sh

# Run comprehensive bridge integration tests
integration-test:
	@echo "🧪 Running Agglayer Bridge Integration Tests..."
	@echo "📋 This will test all bridge operations: L1↔L2, L2↔L1, L2↔L2"
	@echo ""
	@if ! aggsandbox status >/dev/null 2>&1; then \
		echo "❌ Sandbox is not running. Please start it first:"; \
		echo "   aggsandbox start --multi-l2 --detach"; \
		exit 1; \
	fi
	@echo "✅ Sandbox is running, starting tests..."
	@echo ""
	@./test/run_bridge_tests.sh
	@echo ""
	@echo "🎉 Integration tests completed!"
