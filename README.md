# Agglayer Sandbox

A development sandbox environment for testing cross-chain bridging operations using the Polygon zkEVM bridge.

## Quick Start

```bash
# Clone and install
git clone https://github.com/NethermindEth/agg-sandbox.git
cd agg-sandbox
make install

# Start sandbox
cp .env.example .env && source .env
aggsandbox start --detach

# Verify installation
aggsandbox status
```

## Features

- **Local Mode**: Fully local blockchain simulation
- **Fork Mode**: Fork real networks for testing
- **Multi-L2 Mode**: Test with multiple L2 chains
- **LXLY Bridge Integration**: Complete cross-chain operations
- **Docker-based**: Consistent environments

## Architecture

```text
L1 (Anvil:8545) ←→ AggKit (API:5577, RPC:8555) ←→ L2 (Anvil:8546)
```

## Prerequisites

- **Docker** >= 20.0 and Docker Compose >= 1.27
- **Rust** >= 1.70.0
- **Make** for build targets

## Documentation

- **[Overview](docs/overview.md)** - Architecture and components
- **[Quick Start](docs/quickstart.md)** - Installation and first steps
- **[CLI Reference](docs/cli-reference.md)** - Complete command guide
- **[Bridge Operations](docs/bridge-operations.md)** - LXLY bridge guide
- **[Deployed Contracts](docs/deployed-contracts.md)** - Smart contract reference
- **[Advanced Workflows](docs/advanced-workflows.md)** - Complex scenarios
- **[Configuration](docs/configuration.md)** - Environment setup
- **[Troubleshooting](docs/troubleshooting.md)** - Common issues
- **[Development](docs/development.md)** - Contributing guide

## Core Commands

```bash
aggsandbox start --detach        # Start sandbox
aggsandbox status               # Check status
aggsandbox stop                # Stop sandbox

# Bridge operations
aggsandbox bridge asset --network-id 0 --destination-network-id 1 --amount 0.1 --token-address 0x0000000000000000000000000000000000000000
aggsandbox bridge claim --network-id 1 --tx-hash <hash> --source-network-id 0

# Information
aggsandbox show bridges --network-id 0
aggsandbox show claims --network-id 1
aggsandbox events --network-id 0
```

## Support

- **CLI Help**: `aggsandbox --help`
- **Command Help**: `aggsandbox <command> --help`
- **Issues**: [GitHub Issues](https://github.com/NethermindEth/agg-sandbox/issues)

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
