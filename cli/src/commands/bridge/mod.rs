// Bridge command module
pub mod bridge_asset;
pub mod bridge_call;
pub mod claim_asset;
pub mod claim_message;
pub mod common;
pub mod utilities;

// Re-export main types and functions
pub use bridge_asset::{bridge_asset, BridgeAssetArgs, GasOptions};
pub use bridge_call::{
    bridge_and_call_with_approval, bridge_message, BridgeAndCallArgs, BridgeMessageParams,
};
pub use claim_asset::{claim_asset, ClaimAssetArgs};
pub use utilities::{handle_utility_command, UtilityCommands};

use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

// ERC20 contract ABI functions we need
abigen!(
    ERC20Contract,
    r#"[
        function approve(address spender, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
        function decimals() external view returns (uint8)
        function name() external view returns (string)
        function symbol() external view returns (string)
    ]"#,
);

// Bridge contract ABI functions we need
abigen!(
    BridgeContract,
    r#"[
        function bridgeAsset(uint32 destinationNetwork, address destinationAddress, uint256 amount, address token, bool forceUpdateGlobalExitRoot, bytes permitData) external payable
        function claimAsset(uint256 globalIndex, bytes32 mainnetExitRoot, bytes32 rollupExitRoot, uint32 originNetwork, address originTokenAddress, uint32 destinationNetwork, address destinationAddress, uint256 amount, bytes metadata) external
        function claimMessage(uint256 globalIndex, bytes32 mainnetExitRoot, bytes32 rollupExitRoot, uint32 originNetwork, address originAddress, uint32 destinationNetwork, address destinationAddress, uint256 amount, bytes metadata) external
        function precalculatedWrapperAddress(uint32 originNetwork, address originTokenAddress, string name, string symbol, uint8 decimals) external view returns (address)
        function getTokenWrappedAddress(uint32 originNetwork, address originTokenAddress) external view returns (address)
        function wrappedTokenToTokenInfo(address wrappedToken) external view returns (uint32, address)
        function isClaimed(uint32 leafIndex, uint32 sourceBridgeNetwork) external view returns (bool)
        function networkID() external view returns (uint32)
    ]"#,
);

// Bridge extension contract ABI functions we need
abigen!(
    BridgeExtensionContract,
    r#"[
        function bridgeAndCall(address token, uint256 amount, uint32 destinationNetwork, address callAddress, address fallbackAddress, bytes callData, bool forceUpdateGlobalExitRoot) external payable
    ]"#,
);

/// Bridge operation subcommands
#[derive(Debug, clap::Subcommand)]
pub enum BridgeCommands {
    /// 🔄 Bridge assets between networks
    #[command(
        long_about = "Transfer assets between L1 and L2 networks.\n\nBridge ETH or ERC20 tokens from source network to destination network.\nThe command handles token approvals automatically when needed.\n\nNetwork IDs:\n  • 0 = Ethereum L1 (Chain ID 1)\n  • 1 = L2 AggLayer 1 (Chain ID 1101)\n  • 2 = L2 AggLayer 2 (Chain ID 137, if multi-L2 enabled)\n\nExamples:\n  aggsandbox bridge asset --network 0 --destination-network 1 --amount 0.1 --token-address 0x0000000000000000000000000000000000000000\n  aggsandbox bridge asset -n 0 -d 1 -a 1.5 -t 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC --to-address 0x123..."
    )]
    Asset {
        /// Source network ID (0=L1, 1=L2, etc.)
        #[arg(short, long, help = "Source network ID")]
        network: u64,
        /// Destination network ID
        #[arg(short = 'd', long, help = "Destination network ID")]
        destination_network: u64,
        /// Amount to bridge (in wei)
        #[arg(short, long, help = "Amount to bridge (in wei)")]
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
        /// Private key to use for the transaction (hex string with 0x prefix)
        #[arg(long, help = "Private key to use for the transaction")]
        private_key: Option<String>,
    },
    /// 📥 Claim bridged assets on destination network
    #[command(
        long_about = "Claim assets that were bridged from another network.\n\nUse the transaction hash from the original bridge operation to claim\nthe corresponding assets on the destination network.\n\nFor bridgeAndCall operations that create multiple bridges with the same tx_hash,\nuse the --deposit-count parameter to specify which bridge to claim:\n  • 0 = Asset bridge (must be claimed first)\n  • 1 = Message bridge (claimed after asset bridge)\n\nFor BridgeExtension message claims, use --data to provide custom metadata.\n\nClaiming typically requires waiting for the bridge to process the deposit\nand generate the necessary proofs.\n\nExamples:\n  aggsandbox bridge claim --network 1 --tx-hash 0xabc123... --source-network 0\n  aggsandbox bridge claim -n 1 -t 0xdef456... -s 0 --deposit-count 0  # Claim asset bridge\n  aggsandbox bridge claim -n 1 -t 0xdef456... -s 0 --deposit-count 1 --data 0x123...  # Claim message bridge with custom data"
    )]
    Claim {
        /// Network to claim assets on
        #[arg(short, long, help = "Network ID to claim assets on")]
        network: u64,
        /// Original bridge transaction hash
        #[arg(
            short,
            long,
            help = "Transaction hash of the original bridge operation"
        )]
        tx_hash: String,
        /// Source network of the original bridge
        #[arg(short = 's', long, help = "Source network ID of original bridge")]
        source_network: u64,
        /// Deposit count for the specific bridge (0=asset, 1=message, auto-detected if not provided)
        #[arg(
            short = 'c',
            long,
            help = "Deposit count for the specific bridge (0=asset, 1=message, auto-detected if not provided)"
        )]
        deposit_count: Option<u64>,
        /// Token contract address that was bridged (auto-detected if not provided)
        #[arg(
            long,
            help = "Token contract address that was bridged (auto-detected if not provided)"
        )]
        token_address: Option<String>,
        /// Gas limit override
        #[arg(long, help = "Gas limit for the transaction")]
        gas_limit: Option<u64>,
        /// Gas price override (in wei)
        #[arg(long, help = "Gas price in wei")]
        gas_price: Option<String>,
        /// Private key to use for the transaction (hex string with 0x prefix)
        #[arg(long, help = "Private key to use for the transaction")]
        private_key: Option<String>,
        /// Custom metadata for message bridge claims (hex encoded)
        #[arg(
            long,
            help = "Custom metadata for message bridge claims (hex encoded, for BridgeExtension messages)"
        )]
        data: Option<String>,
        /// ETH value to send with message bridge claim (in wei)
        #[arg(
            long,
            help = "ETH value to send with contract call for message bridge claims (in wei)"
        )]
        msg_value: Option<String>,
    },
    /// 📬 Bridge with contract call (bridgeAndCall)
    #[command(
        long_about = "Bridge assets or ETH with a contract call on the destination network.\n\nThis combines bridging with executing a contract call, allowing for\ncomplex cross-chain interactions in a single transaction.\n\nThe call data should be hex-encoded function call data for the target contract.\nIf the contract call fails, assets will be sent to the fallback address.\n\nExamples:\n  aggsandbox bridge message --network 1 --destination-network 1101 --target 0x123... --data 0xabc...\n  aggsandbox bridge message -n 1 -d 0 -t 0x456... --data 0xdef... --amount 0.1 --fallback-address 0x789..."
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
        #[arg(long, help = "Contract call data (hex encoded)")]
        data: String,
        /// Amount of ETH to send with the call (in wei)
        #[arg(short, long, help = "Amount of ETH to send (in wei)")]
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
        /// Private key to use for the transaction (hex string with 0x prefix)
        #[arg(long, help = "Private key to use for the transaction")]
        private_key: Option<String>,
    },
    /// 🔗 Bridge tokens and execute contract call (bridgeAndCall with token approval)
    #[command(
        long_about = "Bridge ERC20 tokens and execute a contract call on the destination network.\\n\\nThis command handles the complete bridgeAndCall workflow:\\n1. Approves the bridge extension contract to spend tokens\\n2. Executes bridgeAndCall to bridge tokens and create a call message\\n3. Provides instructions for claiming the asset and message bridges\\n\\nNote: This creates TWO bridge transactions:\\n- Asset bridge (deposit_count = 0) - must be claimed first\\n- Message bridge (deposit_count = 1) - contains call instructions\\n\\nExamples:\\n  aggsandbox bridge bridge-and-call --network 0 --destination-network 1 --token 0x123... --amount 10 --target 0x456... --data 0xabc... --fallback 0x789...\\n  aggsandbox bridge bridge-and-call -n 0 -d 1 -t 0x123... -a 10 --target 0x456... --data 0xdef... --fallback 0x789..."
    )]
    BridgeAndCall {
        /// Source network ID (0=L1, 1=L2, etc.)
        #[arg(short, long, help = "Source network ID")]
        network: u64,
        /// Destination network ID
        #[arg(short = 'd', long, help = "Destination network ID")]
        destination_network: u64,
        /// Token contract address to bridge
        #[arg(short = 't', long, help = "Token contract address")]
        token: String,
        /// Amount to bridge (in wei)
        #[arg(short, long, help = "Amount to bridge (in wei)")]
        amount: String,
        /// Target contract address on destination network
        #[arg(long, help = "Target contract address for call")]
        target: String,
        /// Call data for the contract (hex encoded)
        #[arg(long, help = "Contract call data (hex encoded)")]
        data: String,
        /// Fallback address if contract call fails
        #[arg(long, help = "Fallback address if call fails")]
        fallback: String,
        /// Gas limit override
        #[arg(long, help = "Gas limit for the transaction")]
        gas_limit: Option<u64>,
        /// Gas price override (in wei)
        #[arg(long, help = "Gas price in wei")]
        gas_price: Option<String>,
        /// Private key to use for the transaction (hex string with 0x prefix)
        #[arg(long, help = "Private key to use for the transaction")]
        private_key: Option<String>,
        /// ETH value to send with the contract call on destination network (in wei)
        #[arg(long, help = "ETH value to send with contract call (in wei)")]
        msg_value: Option<String>,
    },
    /// 🔧 Bridge utility functions
    #[command(subcommand)]
    Utils(UtilityCommands),
}

/// Handle bridge commands using direct Rust implementation
#[allow(clippy::disallowed_methods)] // Allow tracing macros
pub async fn handle_bridge(subcommand: BridgeCommands) -> Result<()> {
    let config = Config::load()?;

    match subcommand {
        BridgeCommands::Asset {
            network,
            destination_network,
            amount,
            token_address,
            to_address,
            gas_limit,
            gas_price,
            private_key,
        } => {
            info!(
                network = network,
                destination_network = destination_network,
                amount = %amount,
                token_address = %token_address,
                "Executing bridge asset command"
            );

            let gas_options = GasOptions::new(gas_limit, gas_price.as_deref());
            let mut builder = BridgeAssetArgs::builder()
                .config(&config)
                .source_network(network)
                .destination_network(destination_network)
                .amount(&amount)
                .token_address(&token_address)
                .gas_options(gas_options);

            if let Some(addr) = to_address.as_deref() {
                builder = builder.recipient_address(addr);
            }
            if let Some(key) = private_key.as_deref() {
                builder = builder.private_key(key);
            }

            let args = builder.build_with_crate_error()?;
            bridge_asset(args).await
        }
        BridgeCommands::Claim {
            network,
            tx_hash,
            source_network,
            deposit_count,
            token_address,
            gas_limit,
            gas_price,
            private_key,
            data,
            msg_value,
        } => {
            info!(
                network = network,
                tx_hash = %tx_hash,
                source_network = source_network,
                "Executing bridge claim command"
            );

            let gas_options = GasOptions::new(gas_limit, gas_price.as_deref());
            let mut builder = ClaimAssetArgs::builder()
                .config(&config)
                .network(network)
                .tx_hash(&tx_hash)
                .source_network(source_network)
                .gas_options(gas_options);

            if let Some(count) = deposit_count {
                builder = builder.deposit_count(Some(count));
            }
            if let Some(addr) = token_address.as_deref() {
                builder = builder.token_address(Some(addr));
            }
            if let Some(key) = private_key.as_deref() {
                builder = builder.private_key(key);
            }
            if let Some(custom_data) = data.as_deref() {
                builder = builder.custom_data(Some(custom_data));
            }
            if let Some(value) = msg_value.as_deref() {
                builder = builder.msg_value(Some(value));
            }

            let args = builder.build_with_crate_error()?;
            claim_asset(args).await
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
            private_key,
        } => {
            info!(
                network = network,
                destination_network = destination_network,
                target = %target,
                "Executing bridge message command"
            );

            let gas_options = GasOptions::new(gas_limit, gas_price.as_deref());
            let mut builder = BridgeMessageParams::builder().target(&target).data(&data);

            if let Some(amt) = &amount {
                builder = builder.amount(amt);
            }
            if let Some(addr) = &fallback_address {
                builder = builder.fallback_address(addr);
            }

            let message_params = builder.build_with_crate_error()?;
            bridge_message(
                &config,
                network,
                destination_network,
                message_params,
                gas_options,
                private_key.as_deref(),
            )
            .await
        }
        BridgeCommands::BridgeAndCall {
            network,
            destination_network,
            token,
            amount,
            target,
            data,
            fallback,
            gas_limit,
            gas_price,
            private_key,
            msg_value,
        } => {
            info!(
                network = network,
                destination_network = destination_network,
                token = %token,
                amount = %amount,
                target = %target,
                "Executing bridge and call command"
            );

            let gas_options = GasOptions::new(gas_limit, gas_price.as_deref());
            let mut builder = BridgeAndCallArgs::builder()
                .config(&config)
                .source_network(network)
                .destination_network(destination_network)
                .token_address(&token)
                .amount(&amount)
                .target(&target)
                .data(&data)
                .fallback(&fallback)
                .gas_options(gas_options);

            if let Some(key) = private_key.as_deref() {
                builder = builder.private_key(key);
            }
            if let Some(value) = msg_value.as_deref() {
                builder = builder.msg_value(value);
            }

            let args = builder.build_with_crate_error()?;
            bridge_and_call_with_approval(args).await
        }
        BridgeCommands::Utils(utility_command) => {
            info!("Executing bridge utility command");
            handle_utility_command(&config, utility_command).await
        }
    }
}

/// Get provider for a network
pub async fn get_provider(config: &Config, network_id: u64) -> Result<Arc<Provider<Http>>> {
    let rpc_url = match network_id {
        0 => config.networks.l1.rpc_url.as_str(),
        1 => config.networks.l2.rpc_url.as_str(),
        2 => config
            .networks
            .l3
            .as_ref()
            .map(|l3| l3.rpc_url.as_str())
            .unwrap_or(""),
        _ => {
            return Err(crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!(
                    "Unsupported network ID: {network_id}"
                )),
            ))
        }
    };

    let provider = Provider::<Http>::try_from(rpc_url).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Failed to create provider: {e}"),
        ))
    })?;

    Ok(Arc::new(provider))
}

/// Get wallet with provider for a network
pub async fn get_wallet_with_provider(
    config: &Config,
    network_id: u64,
    private_key: Option<&str>,
) -> Result<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>> {
    let provider = get_provider(config, network_id).await?;

    // Use provided private key or default to first one from config
    let private_key_str = if let Some(pk) = private_key {
        pk
    } else {
        config.accounts.private_keys.first().ok_or_else(|| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                "No private keys configured",
            ))
        })?
    };

    let wallet = LocalWallet::from_str(private_key_str).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid private key: {e}"),
        ))
    })?;

    let chain_id = match network_id {
        0 => config.networks.l1.chain_id.as_u64()?,
        1 => config.networks.l2.chain_id.as_u64()?,
        2 => config
            .networks
            .l3
            .as_ref()
            .map(|l3| l3.chain_id.as_u64())
            .transpose()?
            .unwrap_or(137),
        _ => {
            return Err(crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!(
                    "Unsupported network ID: {network_id}"
                )),
            ))
        }
    };

    let wallet_with_chain = wallet.with_chain_id(chain_id);
    let client = SignerMiddleware::new(provider, wallet_with_chain);

    Ok(client)
}

/// Get bridge contract address for a network
pub fn get_bridge_contract_address(config: &Config, network_id: u64) -> Result<Address> {
    let layer = match network_id {
        0 => "l1",
        1 => "l2",
        2 => "l3",
        _ => {
            return Err(crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!(
                    "Unsupported network ID: {network_id}"
                )),
            ))
        }
    };

    let address_str = config.contracts.get_contract(layer, "PolygonZkEVMBridge");
    if address_str == "Not deployed" {
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!(
                "Bridge contract not deployed on network {network_id}"
            )),
        ));
    }

    Address::from_str(&address_str).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid bridge contract address: {e}"),
        ))
    })
}

/// Get bridge extension contract address for a network
pub fn get_bridge_extension_address(config: &Config, network_id: u64) -> Result<Address> {
    let layer = match network_id {
        0 => "l1",
        1 => "l2",
        2 => "l3",
        _ => {
            return Err(crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!(
                    "Unsupported network ID: {network_id}"
                )),
            ))
        }
    };

    let address_str = config.contracts.get_contract(layer, "BridgeExtension");
    if address_str == "Not deployed" {
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!(
                "Bridge extension contract not deployed on network {network_id}"
            )),
        ));
    }

    Address::from_str(&address_str).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid bridge extension address: {e}"),
        ))
    })
}

/// Check if address is the zero address (for ETH)
pub fn is_eth_address(address: &str) -> bool {
    address == "0x0000000000000000000000000000000000000000"
        || Address::from_str(address)
            .map(|addr| addr.is_zero())
            .unwrap_or(false)
}
