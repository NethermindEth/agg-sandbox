# Bridge Command Implementation Plan

## Overview

This document outlines the implementation plan for adding bridge commands to the aggsandbox CLI using the lxly.js library. The goal is to provide user-friendly bridge operations while maintaining consistency with the existing CLI structure.

## Current State Analysis

### Existing CLI Structure
- **Location**: `cli/src/main.rs`
- **Architecture**: Rust-based CLI using clap for command parsing
- **Commands**: start, stop, status, logs, restart, info, show, events
- **Show subcommands**: bridges, claims, claim-proof, l1-info-tree-index (read-only operations)
- **API Integration**: Existing bridge viewing commands use HTTP API calls to bridge service

### Current Bridge Operations
Users currently need to:
- Manually construct transactions using `cast` commands
- Know exact contract addresses and function signatures
- Handle encoding/decoding manually

## LXLY.js Library Analysis

### Key Capabilities
- **Package**: `@maticnetwork/lxlyjs` v2.3.2
- **Language**: TypeScript/JavaScript
- **Main Classes**:
  - `LxLyClient`: Main client for initialization
  - `ERC20`: Token operations (bridgeAsset, claimAsset)
  - `BridgeExtension`: bridgeAndCall functionality
  - `Bridge`: Core bridge operations

### Core Operations Available
1. **Asset Bridging**: `token.bridgeAsset(amount, userAddress, destinationNetworkId)`
2. **Asset Claiming**: `token.claimAsset(transactionHash, sourceNetworkId)`
3. **Bridge and Call**: `bridgeExtensions[network].bridgeAndCall(...)`
4. **Bridge Utilities**: Proof generation, status checks

## Implementation Architecture

### Option 1: Node.js Integration (Recommended)
**Approach**: Create a Node.js bridge service that wraps lxly.js functionality

**Pros**:
- Direct use of lxly.js without modifications
- Easier maintenance and updates
- Full feature compatibility
- Leverages existing TypeScript/JavaScript ecosystem

**Cons**:
- Requires Node.js runtime
- Additional dependency management
- Inter-process communication overhead

### Option 2: Rust Bindings
**Approach**: Create Rust bindings for lxly.js using node-bindgen or similar

**Pros**:
- Native Rust integration
- Single executable
- Better performance

**Cons**:
- Complex implementation
- Maintenance overhead
- Potential compatibility issues with lxly.js updates

### Option 3: HTTP Wrapper Service
**Approach**: Create a lightweight HTTP service that wraps lxly.js

**Pros**:
- Language agnostic
- Can be deployed separately
- Easy testing and debugging

**Cons**:
- Additional service to manage
- Network overhead
- More complex deployment

## Recommended Implementation: Node.js Integration

### Phase 1: Infrastructure Setup

#### 1.1 Create Node.js Bridge Module
```
cli/bridge-service/
├── package.json
├── src/
│   ├── index.ts
│   ├── bridge-client.ts
│   ├── operations/
│   │   ├── asset.ts
│   │   ├── claim.ts
│   │   └── message.ts
│   └── utils/
│       ├── config.ts
│       └── validation.ts
└── dist/
```

#### 1.2 Rust CLI Integration
- Add bridge command to `main.rs`
- Create `cli/src/commands/bridge.rs`
- Implement subcommand execution via Node.js subprocess calls

#### 1.3 Configuration Management
- Extend existing config system to support bridge operations
- Add network configuration for L1/L2 endpoints
- Add contract address mappings

### Phase 2: Core Bridge Operations

#### 2.1 Asset Bridging
```bash
aggsandbox bridge asset \
  --network 1 \
  --destination-network 1101 \
  --amount 0.1 \
  --token-address 0x0000... \
  --to-address 0x123...
```

**Implementation**:
- Validate network IDs and addresses
- Initialize lxly.js client with appropriate providers
- Execute `token.bridgeAsset()` operation
- Return transaction hash and status

#### 2.2 Asset Claiming
```bash
aggsandbox bridge claim \
  --network 1101 \
  --tx-hash 0xabc... \
  --source-network 1
```

**Implementation**:
- Validate transaction hash and network
- Use lxly.js to check claimability
- Execute `token.claimAsset()` operation
- Provide clear status feedback

#### 2.3 Bridge and Call
```bash
aggsandbox bridge message \
  --network 1 \
  --destination-network 1101 \
  --target 0x123... \
  --data 0xabc... \
  --amount 0.1
```

**Implementation**:
- Support contract calls with bridging
- Validate call data and target addresses
- Execute `bridgeExtensions[network].bridgeAndCall()`

### Phase 3: Enhanced Features

#### 3.1 Status and Validation
- Pre-flight checks for bridge operations
- Balance validation
- Allowance checking and approval
- Gas estimation

#### 3.2 Interactive Mode
- Guided bridge operations
- Address book for common contracts
- Transaction confirmation prompts

#### 3.3 Integration with Existing Commands
- Extend `show` commands with bridge operation history
- Add bridge status to `info` command
- Include bridge events in `events` command

## Technical Implementation Details

### Node.js Bridge Service

#### Package Dependencies
```json
{
  "dependencies": {
    "@maticnetwork/lxlyjs": "^2.3.2",
    "ethers": "^5.7.0",
    "commander": "^9.0.0",
    "dotenv": "^16.0.0"
  }
}
```

#### Configuration Structure
```typescript
interface BridgeConfig {
  networks: {
    [networkId: string]: {
      rpcUrl: string;
      bridgeAddress: string;
      wrapperAddress?: string;
      bridgeExtensionAddress?: string;
    };
  };
  defaultAccount: string;
  gasSettings: {
    gasLimit: number;
    gasPrice?: string;
    maxFeePerGas?: string;
    maxPriorityFeePerGas?: string;
  };
}
```

#### Command Interface
```typescript
interface BridgeAssetCommand {
  network: number;
  destinationNetwork: number;
  amount: string;
  tokenAddress: string;
  toAddress?: string;
  gasSettings?: GasSettings;
}
```

### Rust CLI Integration

#### Command Structure
```rust
#[derive(Subcommand)]
pub enum BridgeCommands {
    Asset {
        #[arg(short, long)]
        network: u64,
        #[arg(short = 'd', long)]
        destination_network: u64,
        #[arg(short, long)]
        amount: String,
        #[arg(short, long)]
        token_address: String,
        #[arg(short, long)]
        to_address: Option<String>,
    },
    Claim {
        #[arg(short, long)]
        network: u64,
        #[arg(short, long)]
        tx_hash: String,
        #[arg(short = 's', long)]
        source_network: u64,
    },
    Message {
        #[arg(short, long)]
        network: u64,
        #[arg(short = 'd', long)]
        destination_network: u64,
        #[arg(short, long)]
        target: String,
        #[arg(short, long)]
        data: String,
        #[arg(short, long)]
        amount: Option<String>,
    },
}
```

#### Subprocess Execution
```rust
pub async fn handle_bridge(subcommand: BridgeCommands) -> Result<()> {
    let bridge_service_path = Path::new("cli/bridge-service");
    
    match subcommand {
        BridgeCommands::Asset { network, destination_network, amount, token_address, to_address } => {
            let output = Command::new("node")
                .arg("dist/index.js")
                .arg("bridge-asset")
                .arg("--network").arg(network.to_string())
                .arg("--destination-network").arg(destination_network.to_string())
                .arg("--amount").arg(amount)
                .arg("--token-address").arg(token_address)
                .current_dir(bridge_service_path)
                .output()
                .await?;
                
            handle_bridge_response(output)
        }
        // ... other commands
    }
}
```

## Error Handling and Validation

### Input Validation
- Network ID validation against supported networks
- Address format validation (checksummed)
- Amount validation (positive numbers, decimal precision)
- Gas settings validation

### Error Messages
- Clear, actionable error messages
- Suggestions for common issues
- Links to documentation or help resources

### Transaction Monitoring
- Transaction hash display
- Block confirmation tracking
- Failure reason analysis

## Testing Strategy

### Unit Tests
- Command parsing and validation
- Configuration loading
- Error handling scenarios

### Integration Tests
- End-to-end bridge operations on testnet
- Network connectivity tests
- Contract interaction validation

### Manual Testing
- User acceptance testing with common workflows
- Error scenario testing
- Performance benchmarking

## Deployment and Distribution

### Dependencies
- Node.js runtime (v16+)
- npm packages bundled with CLI distribution
- Network connectivity to L1/L2 RPCs

### Installation
- Bundle Node.js bridge service with CLI binary
- Automated dependency installation
- Configuration file templates

### Documentation
- Command usage examples
- Network configuration guide
- Troubleshooting guide
- Security best practices

## Future Enhancements

### Phase 4: Advanced Features
- Batch operations support
- Custom token support
- Advanced gas optimization
- Multi-signature wallet support

### Phase 5: UI/UX Improvements
- Progress bars for long operations
- Transaction history tracking
- Saved configuration profiles
- Web-based dashboard integration

## Security Considerations

### Private Key Management
- Never log or store private keys
- Support for hardware wallets
- Environment variable configuration
- Secure key derivation

### Transaction Safety
- Pre-flight validation
- Confirmation prompts for large amounts
- Slippage protection
- MEV protection considerations

### Network Security
- RPC endpoint validation
- SSL/TLS enforcement
- Rate limiting considerations
- Fallback RPC providers

## Success Metrics

### User Experience
- Reduced time to complete bridge operations
- Decreased error rates
- Improved user satisfaction scores

### Technical Metrics
- Command execution time
- Success rate of bridge operations
- Error handling effectiveness
- Performance benchmarks

## Conclusion

This implementation plan provides a comprehensive approach to adding bridge command functionality to the aggsandbox CLI. By leveraging the lxly.js library through a Node.js integration, we can provide users with a powerful, user-friendly interface for bridge operations while maintaining the existing CLI architecture and patterns.

The phased approach allows for incremental development and testing, ensuring a stable and robust implementation that meets user needs and integrates seamlessly with the existing sandbox environment.