use crate::error::Result;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

/// Bridge operation subcommands
#[derive(Debug, clap::Subcommand)]
pub enum BridgeCommands {
    /// 🔄 Bridge assets between networks
    #[command(
        long_about = "Transfer assets between L1 and L2 networks.\n\nBridge ETH or ERC20 tokens from source network to destination network.\nThe command handles token approvals automatically when needed.\n\nNetwork IDs:\n  • 1 = Ethereum L1\n  • 1101 = Polygon zkEVM L2\n  • 1102 = Additional L2 (if multi-L2 enabled)\n\nExamples:\n  aggsandbox bridge asset --network 1 --destination-network 1101 --amount 0.1 --token-address 0x0000000000000000000000000000000000000000\n  aggsandbox bridge asset -n 1 -d 1101 -a 1.5 -t 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC --to-address 0x123..."
    )]
    Asset {
        /// Source network ID (1=L1, 1101=L2, etc.)
        #[arg(short, long, help = "Source network ID")]
        network: u64,
        /// Destination network ID
        #[arg(short = 'd', long, help = "Destination network ID")]
        destination_network: u64,
        /// Amount to bridge (in token units)
        #[arg(short, long, help = "Amount to bridge")]
        amount: String,
        /// Token contract address (use 0x0000000000000000000000000000000000000000 for ETH)
        #[arg(short, long, help = "Token contract address")]
        token_address: String,
        /// Recipient address (defaults to sender if not specified)
        #[arg(long, help = "Recipient address on destination network")]
        to_address: Option<String>,
        /// Gas limit override
        #[arg(long, help = "Gas limit for the transaction")]
        gas_limit: Option<u64>,
        /// Gas price override (in wei)
        #[arg(long, help = "Gas price in wei")]
        gas_price: Option<String>,
    },
    /// 📥 Claim bridged assets on destination network
    #[command(
        long_about = "Claim assets that were bridged from another network.\n\nUse the transaction hash from the original bridge operation to claim\nthe corresponding assets on the destination network.\n\nClaiming typically requires waiting for the bridge to process the deposit\nand generate the necessary proofs.\n\nExamples:\n  aggsandbox bridge claim --network 1101 --tx-hash 0xabc123... --source-network 1\n  aggsandbox bridge claim -n 1 -t 0xdef456... -s 1101"
    )]
    Claim {
        /// Network to claim assets on
        #[arg(short, long, help = "Network ID to claim assets on")]
        network: u64,
        /// Original bridge transaction hash
        #[arg(short, long, help = "Transaction hash of the original bridge operation")]
        tx_hash: String,
        /// Source network of the original bridge
        #[arg(short = 's', long, help = "Source network ID of original bridge")]
        source_network: u64,
        /// Gas limit override
        #[arg(long, help = "Gas limit for the transaction")]
        gas_limit: Option<u64>,
        /// Gas price override (in wei)
        #[arg(long, help = "Gas price in wei")]
        gas_price: Option<String>,
    },
    /// 📬 Bridge with contract call (bridgeAndCall)
    #[command(
        long_about = "Bridge assets or ETH with a contract call on the destination network.\n\nThis combines bridging with executing a contract call, allowing for\ncomplex cross-chain interactions in a single transaction.\n\nThe call data should be hex-encoded function call data for the target contract.\nIf the contract call fails, assets will be sent to the fallback address.\n\nExamples:\n  aggsandbox bridge message --network 1 --destination-network 1101 --target 0x123... --data 0xabc...\n  aggsandbox bridge message -n 1 -d 1101 -t 0x456... -D 0xdef... --amount 0.1 --fallback-address 0x789..."
    )]
    Message {
        /// Source network ID
        #[arg(short, long, help = "Source network ID")]
        network: u64,
        /// Destination network ID
        #[arg(short = 'd', long, help = "Destination network ID")]
        destination_network: u64,
        /// Target contract address on destination network
        #[arg(short, long, help = "Target contract address")]
        target: String,
        /// Call data for the contract (hex encoded)
        #[arg(short = 'D', long, help = "Contract call data (hex encoded)")]
        data: String,
        /// Amount of ETH to send with the call
        #[arg(short, long, help = "Amount of ETH to send")]
        amount: Option<String>,
        /// Fallback address if contract call fails
        #[arg(long, help = "Fallback address if call fails")]
        fallback_address: Option<String>,
        /// Gas limit override
        #[arg(long, help = "Gas limit for the transaction")]
        gas_limit: Option<u64>,
        /// Gas price override (in wei)
        #[arg(long, help = "Gas price in wei")]
        gas_price: Option<String>,
    },
}

/// Handle bridge commands by calling the Node.js bridge service
pub async fn handle_bridge(subcommand: BridgeCommands) -> Result<()> {
    let bridge_service_path = Path::new("cli/bridge-service");
    
    if !bridge_service_path.exists() {
        error!("Bridge service directory not found at cli/bridge-service");
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::missing_required("Bridge service directory")
        ));
    }

    let dist_path = bridge_service_path.join("dist");
    if !dist_path.exists() {
        error!("Bridge service not built. Please run 'cd cli/bridge-service && pnpm install && pnpm run build'");
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::missing_required("Built bridge service")
        ));
    }

    let mut cmd = Command::new("node");
    cmd.arg("dist/index.js");
    cmd.current_dir(bridge_service_path);

    match subcommand {
        BridgeCommands::Asset {
            network,
            destination_network,
            amount,
            token_address,
            to_address,
            gas_limit,
            gas_price,
        } => {
            info!(
                network = network,
                destination_network = destination_network,
                amount = %amount,
                token_address = %token_address,
                "Executing bridge asset command"
            );

            cmd.arg("bridge-asset")
                .arg("--network").arg(network.to_string())
                .arg("--destination-network").arg(destination_network.to_string())
                .arg("--amount").arg(amount)
                .arg("--token-address").arg(token_address);

            if let Some(to_addr) = to_address {
                cmd.arg("--to-address").arg(to_addr);
            }
            if let Some(gas_lim) = gas_limit {
                cmd.arg("--gas-limit").arg(gas_lim.to_string());
            }
            if let Some(gas_pr) = gas_price {
                cmd.arg("--gas-price").arg(gas_pr);
            }
        }
        BridgeCommands::Claim {
            network,
            tx_hash,
            source_network,
            gas_limit,
            gas_price,
        } => {
            info!(
                network = network,
                tx_hash = %tx_hash,
                source_network = source_network,
                "Executing bridge claim command"
            );

            cmd.arg("claim-asset")
                .arg("--network").arg(network.to_string())
                .arg("--tx-hash").arg(tx_hash)
                .arg("--source-network").arg(source_network.to_string());

            if let Some(gas_lim) = gas_limit {
                cmd.arg("--gas-limit").arg(gas_lim.to_string());
            }
            if let Some(gas_pr) = gas_price {
                cmd.arg("--gas-price").arg(gas_pr);
            }
        }
        BridgeCommands::Message {
            network,
            destination_network,
            target,
            data,
            amount,
            fallback_address,
            gas_limit,
            gas_price,
        } => {
            info!(
                network = network,
                destination_network = destination_network,
                target = %target,
                "Executing bridge message command"
            );

            cmd.arg("bridge-message")
                .arg("--network").arg(network.to_string())
                .arg("--destination-network").arg(destination_network.to_string())
                .arg("--target").arg(target)
                .arg("--data").arg(data);

            if let Some(amt) = amount {
                cmd.arg("--amount").arg(amt);
            }
            if let Some(fallback) = fallback_address {
                cmd.arg("--fallback-address").arg(fallback);
            }
            if let Some(gas_lim) = gas_limit {
                cmd.arg("--gas-limit").arg(gas_lim.to_string());
            }
            if let Some(gas_pr) = gas_price {
                cmd.arg("--gas-price").arg(gas_pr);
            }
        }
    }

    // Execute the command and handle the result
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                // Print stdout from the bridge service
                if !output.stdout.is_empty() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                }
            } else {
                error!("Bridge command failed with exit code: {:?}", output.status.code());
                // Print stderr from the bridge service
                if !output.stderr.is_empty() {
                    eprint!("{}", String::from_utf8_lossy(&output.stderr));
                }
                return Err(crate::error::AggSandboxError::Config(
                    crate::error::ConfigError::validation_failed("Bridge operation failed")
                ));
            }
        }
        Err(e) => {
            error!("Failed to execute bridge service: {}", e);
            return Err(crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!(
                    "Failed to execute bridge service: {}. Make sure Node.js is installed and the bridge service is built.",
                    e
                ))
            ));
        }
    }

    Ok(())
}