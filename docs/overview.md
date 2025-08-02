# Agglayer Sandbox Overview

The Agglayer Sandbox provides a comprehensive development environment for testing cross-chain bridging operations, smart contract interactions, and multi-layer blockchain scenarios.

## What is Agglayer Sandbox?

A Docker-based development environment that simulates the Polygon Agglayer ecosystem locally, supporting:

- **Cross-chain bridging** between L1 and L2 networks
- **Multi-L2 scenarios** with up to 3 chains
- **Fork mode** for testing against real network state
- **LXLY bridge operations** with complete asset and message bridging

## System Architecture

### Standard Mode

```text
┌─────────────────┐         ┌─────────────────────┐         ┌─────────────────┐
│   L1 (Anvil)    │◄────────┤      AggKit         ├────────►│   L2 (Anvil)    │
│   Port: 8545    │         │  REST API: 5577     │         │   Port: 8546    │
│   Network ID: 0 │         │  RPC: 8555          │         │   Network ID: 1 │
│                 │         │  Telemetry: 8080    │         │                 │
└─────────────────┘         └─────────────────────┘         └─────────────────┘
         ▲                                                           ▲
         │                                                           │
         └─────────────────────────────┼─────────────────────────────┘
                                       │
                          ┌─────────────────┐
                          │ Contract Deploy │
                          │    Service      │
                          │ (runs once)     │
                          └─────────────────┘
```

### Multi-L2 Mode

```text
                     ┌─────────────────────┐
                     │     AggKit-L2       │
              ┌──────┤  REST API: 5577     ├──────┐
              │      │  RPC: 8555          │      │
              │      │  Telemetry: 8080    │      │
              │      └─────────────────────┘      │
              ▼                                   ▼
   ┌─────────────┐                     ┌─────────────┐
   │ L1 (Anvil)  │                     │L2-1 (Anvil) │
   │ Port: 8545  │                     │ Port: 8546  │
   │Network ID: 0│                     │Network ID: 1│
   └─────────────┘                     └─────────────┘
              │
              │      ┌─────────────────────┐
              └──────┤     AggKit-L3       ├──────┐
                     │  REST API: 5578     │      │
                     │  RPC: 8556          │      │
                     │  Telemetry: 8081    │      │
                     └─────────────────────┘      ▼
                                        ┌─────────────┐
                                        │L2-2 (Anvil) │
                                        │ Port: 8547  │
                                        │Network ID: 2│
                                        └─────────────┘
```

## Core Components

### CLI Tool (`aggsandbox`)

- **Built in Rust** for performance and reliability
- **Rich UI** with progress bars and detailed error messages
- **Comprehensive commands** for all sandbox operations
- **JSON output** support for automation

### Smart Contracts

- **Bridge Contracts**: LXLY-compatible bridge implementation
- **Bridge Extensions**: Advanced bridging with contract calls
- **ERC20 Tokens**: Test token contracts for bridging
- **Foundry-based**: Modern Solidity development stack

### Docker Environment

- **Anvil Nodes**: Local blockchain simulation
- **AggKit Services**: Bridge oracles and REST APIs
- **Database**: PostgreSQL for bridge state
- **Automatic Deployment**: Contracts deployed on startup

### Bridge Services

- **REST API**: Query bridge state and proofs
- **RPC Interface**: Additional blockchain access
- **Telemetry**: Monitoring and observability
- **Oracle Functions**: Cross-chain state synchronization

## Operating Modes

### Local Mode (Default)

- **Fully local** blockchain simulation
- **No external dependencies**
- **Fast startup** and deterministic behavior
- **Ideal for development** and CI/CD

### Fork Mode

- **Fork real networks** for testing
- **Uses live blockchain data**
- **Requires API keys** (Alchemy, Infura, etc.)
- **Test against production state**

### Multi-L2 Mode

- **Three-chain setup**: L1 + L2-1 + L2-2
- **Cross-L2 bridging** scenarios
- **Can combine with fork mode**
- **Advanced testing capabilities**

## Network Configuration

| Mode         | L1 Chain     | L2-1 Chain    | L2-2 Chain  |
| ------------ | ------------ | ------------- | ----------- |
| **Local**    | Ethereum Sim | zkEVM Sim     | PoS Sim     |
| **Fork**     | ETH Mainnet  | Polygon zkEVM | Polygon PoS |
| **Multi-L2** | ✅           | ✅            | ✅          |

## Key Technologies

### LXLY Bridge

- **Asset bridging** with ERC20 and ETH support
- **Message bridging** with contract execution
- **Merkle proof verification** for security
- **Bridge-and-call** functionality

### Development Stack

- **Rust CLI** with ethers.rs integration
- **Foundry** for smart contract development
- **Docker Compose** for orchestration
- **PostgreSQL** for bridge state management

## Use Cases

### Development

- **Smart contract testing** across chains
- **Bridge integration** development
- **DeFi protocol** cross-chain features
- **Multi-chain dApp** development

### Testing

- **Integration testing** with real bridge mechanics
- **Performance testing** under load
- **Security testing** with various scenarios
- **Regression testing** for bridge operations

### Education

- **Learning cross-chain concepts**
- **Understanding bridge mechanics**
- **Experimenting with L2 technologies**
- **Polygon ecosystem exploration**

## Next Steps

- **[Quick Start Guide](quickstart.md)** - Get up and running
- **[CLI Reference](cli-reference.md)** - Complete command documentation
- **[Bridge Operations](bridge-operations.md)** - Learn bridging workflows
- **[Configuration](configuration.md)** - Environment setup details
