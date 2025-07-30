use crate::api_client::OptimizedApiClient;
use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

/// Gas options for transactions
#[derive(Debug, Clone)]
pub struct GasOptions {
    pub gas_limit: Option<u64>,
    pub gas_price: Option<String>,
}

impl GasOptions {
    pub fn new(gas_limit: Option<u64>, gas_price: Option<&str>) -> Self {
        Self {
            gas_limit,
            gas_price: gas_price.map(|s| s.to_string()),
        }
    }

    pub fn apply_to_call_with_return<M: Middleware + 'static, D: ethers::core::abi::Detokenize>(
        &self,
        mut call: ContractCall<M, D>,
    ) -> ContractCall<M, D> {
        if let Some(gas) = self.gas_limit {
            call = call.gas(gas);
        }
        if let Some(price) = &self.gas_price {
            if let Ok(price_wei) = U256::from_dec_str(price) {
                call = call.gas_price(price_wei);
            }
        }
        call
    }
}

/// Parameters for bridge message operations
#[derive(Debug, Clone)]
pub struct BridgeMessageParams {
    pub target: String,
    pub data: String,
    pub amount: Option<String>,
    pub fallback_address: Option<String>,
}

impl BridgeMessageParams {
    pub fn new(
        target: String,
        data: String,
        amount: Option<String>,
        fallback_address: Option<String>,
    ) -> Self {
        Self {
            target,
            data,
            amount,
            fallback_address,
        }
    }
}

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
    },
    /// 📬 Bridge with contract call (bridgeAndCall)
    #[command(
        long_about = "Bridge assets or ETH with a contract call on the destination network.\n\nThis combines bridging with executing a contract call, allowing for\ncomplex cross-chain interactions in a single transaction.\n\nThe call data should be hex-encoded function call data for the target contract.\nIf the contract call fails, assets will be sent to the fallback address.\n\nExamples:\n  aggsandbox bridge message --network 1 --destination-network 1101 --target 0x123... --data 0xabc...\n  aggsandbox bridge message -n 1 -d 1101 -t 0x456... --data 0xdef... --amount 0.1 --fallback-address 0x789..."
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
        /// Amount to bridge (in token units)
        #[arg(short, long, help = "Amount to bridge")]
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
    },
}

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
    ]"#,
);

// Bridge extension contract ABI functions we need
abigen!(
    BridgeExtensionContract,
    r#"[
        function bridgeAndCall(address token, uint256 amount, uint32 destinationNetwork, address callAddress, address fallbackAddress, bytes callData, bool forceUpdateGlobalExitRoot) external payable
    ]"#,
);

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
            bridge_asset(BridgeAssetArgs {
                config: &config,
                source_network: network,
                destination_network,
                amount: &amount,
                token_address: &token_address,
                to_address: to_address.as_deref(),
                gas_options,
                private_key: private_key.as_deref(),
            })
            .await
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
        } => {
            info!(
                network = network,
                tx_hash = %tx_hash,
                source_network = source_network,
                "Executing bridge claim command"
            );

            let gas_options = GasOptions::new(gas_limit, gas_price.as_deref());
            claim_asset(
                &config,
                network,
                &tx_hash,
                source_network,
                deposit_count,
                token_address.as_deref(),
                gas_options,
                private_key.as_deref(),
                data.as_deref(),
            )
            .await
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
            let message_params = BridgeMessageParams::new(target, data, amount, fallback_address);
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
            bridge_and_call_with_approval(
                &config,
                network,
                destination_network,
                &token,
                &amount,
                &target,
                &data,
                &fallback,
                gas_options,
                private_key.as_deref(),
            )
            .await
        }
    }
}

/// Get provider for a network
async fn get_provider(config: &Config, network_id: u64) -> Result<Arc<Provider<Http>>> {
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
async fn get_wallet_with_provider(
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

    let chain_id = network_id_to_chain_id(config, network_id)? as u64;

    let wallet_with_chain = wallet.with_chain_id(chain_id);
    let client = SignerMiddleware::new(provider, wallet_with_chain);

    Ok(client)
}

/// Get bridge contract address for a network
fn get_bridge_contract_address(config: &Config, network_id: u64) -> Result<Address> {
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
fn get_bridge_extension_address(config: &Config, network_id: u64) -> Result<Address> {
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
fn is_eth_address(address: &str) -> bool {
    address == "0x0000000000000000000000000000000000000000"
        || Address::from_str(address)
            .map(|addr| addr.is_zero())
            .unwrap_or(false)
}

/// Convert network ID to chain ID based on standard mapping
fn network_id_to_chain_id(config: &Config, network_id: u64) -> Result<u32> {
    let chain_id = match network_id {
        0 => config.networks.l1.chain_id.as_u64()?, // L1 Mainnet (Chain ID 1)
        1 => config.networks.l2.chain_id.as_u64()?, // AggLayer 1 (Chain ID 1101)
        2 => config
            .networks
            .l3
            .as_ref()
            .map(|l3| l3.chain_id.as_u64())
            .transpose()?
            .unwrap_or(137), // AggLayer 2 (Chain ID 137)
        _ => {
            return Err(crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!(
                    "Unsupported network ID: {network_id}"
                )),
            ))
        }
    };

    Ok(chain_id as u32)
}

struct BridgeAssetArgs<'a> {
    config: &'a Config,
    source_network: u64,
    destination_network: u64,
    amount: &'a str,
    token_address: &'a str,
    to_address: Option<&'a str>,
    gas_options: GasOptions,
    private_key: Option<&'a str>,
}

/// Bridge assets between networks
#[allow(clippy::disallowed_methods)] // Allow tracing macros
async fn bridge_asset(args: BridgeAssetArgs<'_>) -> Result<()> {
    let client =
        get_wallet_with_provider(args.config, args.source_network, args.private_key).await?;
    let bridge_address = get_bridge_contract_address(args.config, args.source_network)?;
    let bridge = BridgeContract::new(bridge_address, Arc::new(client.clone()));

    let destination_chain_id = network_id_to_chain_id(args.config, args.destination_network)?;

    let recipient = if let Some(addr) = args.to_address {
        Address::from_str(addr).map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Invalid recipient address: {e}"),
            ))
        })?
    } else {
        client.address()
    };

    let amount_wei = U256::from_dec_str(args.amount).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid amount: {e}"),
        ))
    })?;

    let token_addr = Address::from_str(args.token_address).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid token address: {e}"),
        ))
    })?;

    // Handle ETH vs ERC20 token bridging
    if is_eth_address(args.token_address) {
        info!(
            "Bridging ETH from network {} to network {}",
            args.source_network, args.destination_network
        );
        println!("🔧 Bridging ETH - amount: {amount_wei} wei, destination_chain_id: {destination_chain_id}, recipient: {recipient:?}");

        let call = bridge
            .bridge_asset(
                destination_chain_id,
                recipient,
                amount_wei,
                token_addr,
                true,         // forceUpdateGlobalExitRoot
                Bytes::new(), // empty permit data
            )
            .value(amount_wei);

        let call = args.gas_options.apply_to_call_with_return(call);

        let tx = call.send().await.map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Failed to send bridge transaction: {e}"),
            ))
        })?;

        println!("✅ Bridge transaction submitted: {:#x}", tx.tx_hash());
    } else {
        info!(
            "Bridging ERC20 token {} from network {} to network {}",
            args.token_address, args.source_network, args.destination_network
        );
        println!("🔧 ERC20 Bridge Debug:");
        println!("  - Token address: {}", args.token_address);
        println!("  - Token address (parsed): {token_addr:?}");
        println!("  - From address: {:?}", client.address());
        println!("  - Bridge address: {bridge_address:?}");
        println!("  - Amount: {} (Wei: {amount_wei})", args.amount);
        println!("  - Destination chain ID: {destination_chain_id}");
        println!("  - Recipient: {recipient:?}");

        // First check and approve if needed
        let token = ERC20Contract::new(token_addr, Arc::new(client.clone()));

        println!(
            "🔧 Checking allowance: token.allowance({:?}, {bridge_address:?})",
            client.address()
        );
        let allowance = token
            .allowance(client.address(), bridge_address)
            .call()
            .await
            .map_err(|e| {
                crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                    &format!("Failed to check allowance: {e}"),
                ))
            })?;

        println!("🔧 Current allowance: {allowance}, Required: {amount_wei}");

        if allowance < amount_wei {
            info!("Approving bridge contract to spend {} tokens", args.amount);
            println!("🔧 Calling approve: token.approve({bridge_address:?}, {amount_wei})");
            let approve_call = token.approve(bridge_address, amount_wei);
            let approve_tx = approve_call.send().await.map_err(|e| {
                crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                    &format!("Failed to approve tokens: {e}"),
                ))
            })?;
            println!("✅ Token approval transaction: {:#x}", approve_tx.tx_hash());

            // Wait for approval to be mined
            approve_tx.await.map_err(|e| {
                crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                    &format!("Approval transaction failed: {e}"),
                ))
            })?;
        }

        // Now bridge the tokens
        println!("🔧 Calling bridgeAsset:");
        println!("  - destination_chain_id: {destination_chain_id}");
        println!("  - recipient: {recipient:?}");
        println!("  - amount_wei: {amount_wei}");
        println!("  - token_addr: {token_addr:?}");
        println!("  - forceUpdateGlobalExitRoot: true");

        let call = bridge.bridge_asset(
            destination_chain_id,
            recipient,
            amount_wei,
            token_addr,
            true,         // forceUpdateGlobalExitRoot
            Bytes::new(), // empty permit data
        );

        let call = args.gas_options.apply_to_call_with_return(call);

        let tx = call.send().await.map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Failed to send bridge transaction: {e}"),
            ))
        })?;

        println!("✅ Bridge transaction submitted: {:#x}", tx.tx_hash());
    }

    // Determine the correct source network for claiming
    // For bridge-back scenarios (wrapped tokens), we need to use the original token's network
    let claim_source_network = if !is_eth_address(args.token_address) {
        // For ERC20 tokens, check if this might be a wrapped token bridge-back
        // In bridge-back scenarios, the claim should use the original token's network (usually 0 for L1)
        // This is a heuristic: if bridging from L2 to L1, it's likely a bridge-back
        if args.source_network == 1 && args.destination_network == 0 {
            0 // Bridge-back from L2 to L1, use L1 as source for claim
        } else {
            args.source_network // Normal bridging, use actual source network
        }
    } else {
        args.source_network // ETH bridging, use actual source network
    };

    println!("💡 Use 'aggsandbox bridge claim --network {} --tx-hash <tx_hash> --source-network {claim_source_network}' to claim assets", args.destination_network);

    Ok(())
}

/// Claim bridged assets on destination network
async fn claim_asset(
    config: &Config,
    network: u64,
    tx_hash: &str,
    source_network: u64,
    deposit_count: Option<u64>,
    _token_address: Option<&str>,
    gas_options: GasOptions,
    private_key: Option<&str>,
    custom_data: Option<&str>,
) -> Result<()> {
    let client = get_wallet_with_provider(config, network, private_key).await?;
    let bridge_address = get_bridge_contract_address(config, network)?;
    let bridge = BridgeContract::new(bridge_address, Arc::new(client.clone()));
    let api_client = OptimizedApiClient::global();

    println!("🔍 Looking for bridge transaction with hash: {tx_hash}");

    // For bridge-back scenarios (L2→L1), we need special logic:
    // - The bridge transaction is on the intermediate network (L2)
    // - But the proof data comes from the original token's network (L1)
    // Try to detect this by checking if we're claiming on L1 with source_network=0
    let (bridge_tx_network, proof_source_network) = if network == 0 && source_network == 0 {
        // Potential bridge-back scenario: L2→L1 claim
        // First try to find the transaction on L2 (network 1)
        let l2_bridges = api_client.get_bridges(config, 1).await.ok();
        if let Some(l2_response) = l2_bridges {
            if let Some(bridges) = l2_response["bridges"].as_array() {
                if bridges
                    .iter()
                    .any(|bridge| bridge["tx_hash"].as_str() == Some(tx_hash))
                {
                    println!("🔍 Detected bridge-back scenario: transaction found on L2, using L2 for proof data");
                    (1u64, 1u64) // Bridge tx is on L2, proof data from L2
                } else {
                    (source_network, source_network) // Normal scenario
                }
            } else {
                (source_network, source_network) // Normal scenario
            }
        } else {
            (source_network, source_network) // Normal scenario
        }
    } else {
        (source_network, source_network) // Normal scenario
    };

    // Get bridges from the network where the transaction actually occurred
    let bridges_response = api_client
        .get_bridges(config, bridge_tx_network)
        .await
        .map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Failed to get bridges: {e}"),
            ))
        })?;

    let bridges = bridges_response["bridges"].as_array().ok_or_else(|| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            "Invalid bridges response",
        ))
    })?;

    // Find our bridge transaction
    let bridge_info = if let Some(specific_deposit_count) = deposit_count {
        println!("🔍 Looking for bridge with tx_hash: {tx_hash} and deposit_count: {specific_deposit_count}");
        bridges
            .iter()
            .find(|bridge| {
                bridge["tx_hash"].as_str() == Some(tx_hash)
                    && bridge["deposit_count"].as_u64() == Some(specific_deposit_count)
            })
            .ok_or_else(|| {
                crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                    &format!("Bridge transaction {tx_hash} with deposit_count {specific_deposit_count} not found"),
                ))
            })?
    } else {
        println!("🔍 Looking for bridge with tx_hash: {tx_hash} (deposit_count auto-detected)");
        bridges
            .iter()
            .find(|bridge| bridge["tx_hash"].as_str() == Some(tx_hash))
            .ok_or_else(|| {
                crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                    &format!("Bridge transaction {tx_hash} not found"),
                ))
            })?
    };

    let deposit_count = bridge_info["deposit_count"].as_u64().ok_or_else(|| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            "Missing deposit_count in bridge info",
        ))
    })?;

    println!("📋 Found bridge with deposit count: {deposit_count}");

    // Determine bridge type from bridge info
    let leaf_type = bridge_info["leaf_type"].as_u64().unwrap_or(0) as u8;
    println!("🔍 Bridge leaf type: {} (0=Asset, 1=Message)", leaf_type);

    // Get L1 info tree index from the proof source network
    // For bridge-back scenarios, this uses L2 (where the bridge tx occurred)
    let tree_index_response = api_client
        .get_l1_info_tree_index(config, proof_source_network, deposit_count)
        .await
        .map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Failed to get L1 info tree index: {e}"),
            ))
        })?;

    let leaf_index = tree_index_response["l1_info_tree_index"]
        .as_u64()
        .unwrap_or(tree_index_response.as_u64().unwrap_or(0));

    println!("🌳 L1 info tree index: {leaf_index}");

    // Get claim proof from the proof source network
    // For bridge-back scenarios, this uses L2 (where the bridge tx occurred)
    let proof_response = api_client
        .get_claim_proof(config, proof_source_network, leaf_index, deposit_count)
        .await
        .map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Failed to get claim proof: {e}"),
            ))
        })?;

    let l1_info_tree_leaf = &proof_response["l1_info_tree_leaf"];
    let mainnet_exit_root = l1_info_tree_leaf["mainnet_exit_root"]
        .as_str()
        .ok_or_else(|| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                "Missing mainnet_exit_root in proof",
            ))
        })?;

    let rollup_exit_root = l1_info_tree_leaf["rollup_exit_root"]
        .as_str()
        .ok_or_else(|| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                "Missing rollup_exit_root in proof",
            ))
        })?;

    println!("🔐 Got claim proof data");

    // Extract bridge parameters
    let origin_network = bridge_info["origin_network"]
        .as_u64()
        .map(|n| n as u32)
        .unwrap_or_else(|| network_id_to_chain_id(config, source_network).unwrap_or(1));
    let destination_network_id = bridge_info["destination_network"]
        .as_u64()
        .map(|n| n as u32)
        .unwrap_or_else(|| network_id_to_chain_id(config, network).unwrap_or(1101));

    // For message bridges (leaf_type = 1), use BridgeExtension addresses
    // For asset bridges (leaf_type = 0), use the original bridge addresses
    let (origin_address, destination_address) = if leaf_type == 1 {
        // Message bridge - use BridgeExtension addresses
        let origin_bridge_ext = get_bridge_extension_address(config, source_network)?;
        let dest_bridge_ext = get_bridge_extension_address(config, network)?;
        println!("🔗 Using BridgeExtension addresses for message bridge:");
        println!(
            "   Origin: {:#x} (network {})",
            origin_bridge_ext, source_network
        );
        println!(
            "   Destination: {:#x} (network {})",
            dest_bridge_ext, network
        );
        (
            format!("{:#x}", origin_bridge_ext),
            format!("{:#x}", dest_bridge_ext),
        )
    } else {
        // Asset bridge - use original addresses from bridge_info
        let origin_addr = bridge_info["origin_address"].as_str().ok_or_else(|| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                "Missing origin_address in bridge info",
            ))
        })?;
        let dest_addr = bridge_info["destination_address"].as_str().ok_or_else(|| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                "Missing destination_address in bridge info",
            ))
        })?;
        println!("🏦 Using original bridge addresses for asset bridge:");
        println!("   Origin: {} (network {})", origin_addr, source_network);
        println!("   Destination: {} (network {})", dest_addr, network);
        (origin_addr.to_string(), dest_addr.to_string())
    };
    let amount = bridge_info["amount"].as_str().ok_or_else(|| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            "Missing amount in bridge info",
        ))
    })?;
    let metadata = if let Some(custom) = custom_data {
        println!("🔧 Using custom metadata: {custom}");
        custom
    } else {
        bridge_info["metadata"].as_str().unwrap_or("0x")
    };

    // Convert addresses and amount
    let origin_addr = Address::from_str(&origin_address).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid origin address: {e}"),
        ))
    })?;
    let dest_addr = Address::from_str(&destination_address).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid destination address: {e}"),
        ))
    })?;
    let amount_wei = U256::from_dec_str(amount).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid amount: {e}"),
        ))
    })?;
    let mainnet_root = H256::from_str(mainnet_exit_root).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid mainnet exit root: {e}"),
        ))
    })?;
    let rollup_root = H256::from_str(rollup_exit_root).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid rollup exit root: {e}"),
        ))
    })?;

    // Encode ERC20 token metadata properly for claimAsset
    let metadata_bytes = if !metadata.is_empty() && metadata != "0x" {
        // Use metadata from API if available
        hex::decode(metadata.trim_start_matches("0x")).map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Invalid metadata hex: {e}"),
            ))
        })?
    } else {
        // For first-time bridges, fetch and encode ERC20 token details
        let source_client = get_wallet_with_provider(config, proof_source_network, None).await?;
        let token_contract = ERC20Contract::new(origin_addr, Arc::new(source_client));

        // Fetch token details
        let token_name = token_contract
            .name()
            .call()
            .await
            .unwrap_or_else(|_| "AggERC20".to_string());
        let token_symbol = token_contract
            .symbol()
            .call()
            .await
            .unwrap_or_else(|_| "AGGERC20".to_string());
        let token_decimals = token_contract.decimals().call().await.unwrap_or(18u8);

        info!(
            "Encoding ERC20 metadata: name={token_name}, symbol={token_symbol}, decimals={token_decimals}"
        );

        // ABI encode as (string,string,uint8)
        use ethers::abi::{encode, Token};
        let tokens = vec![
            Token::String(token_name),
            Token::String(token_symbol),
            Token::Uint(U256::from(token_decimals)),
        ];
        encode(&tokens)
    };

    // Call the appropriate claim function based on leaf type
    let tx_hash = if leaf_type == 0 {
        // Asset bridge - call claimAsset
        println!("💰 Claiming asset: {amount} tokens to {destination_address}");

        execute_claim_asset(
            &bridge,
            deposit_count,
            mainnet_root,
            rollup_root,
            origin_network,
            origin_addr,
            destination_network_id,
            dest_addr,
            amount_wei,
            metadata_bytes.clone(),
            &gas_options,
        )
        .await?
    } else {
        // Message bridge - call claimMessage
        println!("📨 Claiming message bridge to trigger contract execution");

        execute_claim_message(
            &bridge,
            deposit_count,
            mainnet_root,
            rollup_root,
            origin_network,
            origin_addr,
            destination_network_id,
            dest_addr,
            amount_wei,
            metadata_bytes,
            &gas_options,
        )
        .await?
    };

    println!("✅ Claim transaction submitted: {:#x}", tx_hash);
    if leaf_type == 0 {
        println!("🎉 Assets should be available once the transaction is mined!");
    } else {
        println!("🎉 Message bridge claimed! Contract call should execute automatically.");
    }

    Ok(())
}

/// Execute claimAsset contract call
async fn execute_claim_asset(
    bridge: &BridgeContract<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>,
    deposit_count: u64,
    mainnet_root: H256,
    rollup_root: H256,
    origin_network: u32,
    origin_addr: Address,
    destination_network_id: u32,
    dest_addr: Address,
    amount_wei: U256,
    metadata_bytes: Vec<u8>,
    gas_options: &GasOptions,
) -> Result<H256> {
    let mut call = bridge.claim_asset(
        deposit_count.into(), // globalIndex
        mainnet_root.into(),  // mainnetExitRoot
        rollup_root.into(),   // rollupExitRoot
        origin_network,
        origin_addr,
        destination_network_id,
        dest_addr,
        amount_wei,
        ethers::types::Bytes::from(metadata_bytes), // metadata
    );

    if gas_options.gas_limit.is_none() {
        call = call.gas(3_000_000u64); // Default high gas limit for claims
    }

    let call = gas_options.apply_to_call_with_return(call);
    let tx = call.send().await.map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Failed to send claim asset transaction: {e}"),
        ))
    })?;
    Ok(tx.tx_hash())
}

/// Execute claimMessage contract call
async fn execute_claim_message(
    bridge: &BridgeContract<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>,
    deposit_count: u64,
    mainnet_root: H256,
    rollup_root: H256,
    origin_network: u32,
    origin_addr: Address,
    destination_network_id: u32,
    dest_addr: Address,
    amount_wei: U256,
    metadata_bytes: Vec<u8>,
    gas_options: &GasOptions,
) -> Result<H256> {
    let mut call = bridge.claim_message(
        deposit_count.into(), // globalIndex
        mainnet_root.into(),  // mainnetExitRoot
        rollup_root.into(),   // rollupExitRoot
        origin_network,
        origin_addr, // originAddress for message
        destination_network_id,
        dest_addr,
        amount_wei,
        ethers::types::Bytes::from(metadata_bytes), // message data
    );

    if gas_options.gas_limit.is_none() {
        call = call.gas(3_000_000u64); // Default high gas limit for claims
    }

    let call = gas_options.apply_to_call_with_return(call);
    let tx = call.send().await.map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Failed to send claim message transaction: {e}"),
        ))
    })?;
    Ok(tx.tx_hash())
}

/// Bridge with contract call (bridgeAndCall)
async fn bridge_message(
    config: &Config,
    source_network: u64,
    destination_network: u64,
    params: BridgeMessageParams,
    gas_options: GasOptions,
    private_key: Option<&str>,
) -> Result<()> {
    let client = get_wallet_with_provider(config, source_network, private_key).await?;
    let bridge_ext_address = get_bridge_extension_address(config, source_network)?;
    let bridge_ext = BridgeExtensionContract::new(bridge_ext_address, Arc::new(client.clone()));

    let destination_chain_id = network_id_to_chain_id(config, destination_network)?;

    let target_addr = Address::from_str(&params.target).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid target address: {e}"),
        ))
    })?;

    let fallback_addr = if let Some(addr) = &params.fallback_address {
        Address::from_str(addr).map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Invalid fallback address: {e}"),
            ))
        })?
    } else {
        client.address()
    };

    let call_data = hex::decode(params.data.trim_start_matches("0x")).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid call data hex: {e}"),
        ))
    })?;

    let eth_amount = if let Some(amt) = &params.amount {
        U256::from_dec_str(amt).map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Invalid amount: {e}"),
            ))
        })?
    } else {
        U256::zero()
    };

    info!(
        "Bridging message from network {} to network {} with call to {}",
        source_network, destination_network, params.target
    );

    // Use zero address as token for ETH bridging with call
    let token_addr = Address::zero();

    let mut call = bridge_ext.bridge_and_call(
        token_addr,
        eth_amount,
        destination_chain_id,
        target_addr,
        fallback_addr,
        call_data.into(),
        true, // forceUpdateGlobalExitRoot
    );

    if !eth_amount.is_zero() {
        call = call.value(eth_amount);
    }

    let call = gas_options.apply_to_call_with_return(call);

    let tx = call.send().await.map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Failed to send bridge and call transaction: {e}"),
        ))
    })?;

    println!(
        "✅ Bridge and call transaction submitted: {:#x}",
        tx.tx_hash()
    );
    println!("💡 This creates both asset and message bridges. The message should execute automatically when ready.");

    Ok(())
}

/// Get precalculated L2 token address
#[allow(dead_code)]
async fn get_precalculated_l2_token_address(
    config: &Config,
    destination_network: u64,
    token_address: &str,
    private_key: Option<&str>,
) -> Result<Address> {
    let client = get_wallet_with_provider(config, destination_network, private_key).await?;
    let bridge_address = get_bridge_contract_address(config, destination_network)?;
    let bridge = BridgeContract::new(bridge_address, Arc::new(client.clone()));

    let token_addr = Address::from_str(token_address).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid token address: {e}"),
        ))
    })?;

    // Get token details from the source network (assuming L1 for now)
    let source_client = get_wallet_with_provider(config, 0, private_key).await?;
    let token_contract = ERC20Contract::new(token_addr, Arc::new(source_client));

    let token_name = token_contract
        .name()
        .call()
        .await
        .unwrap_or_else(|_| "AggERC20".to_string());
    let token_symbol = token_contract
        .symbol()
        .call()
        .await
        .unwrap_or_else(|_| "AGGERC20".to_string());
    let token_decimals = token_contract.decimals().call().await.unwrap_or(18u8);

    let l2_token_address = bridge
        .precalculated_wrapper_address(1, token_addr, token_name, token_symbol, token_decimals)
        .call()
        .await
        .map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Failed to get precalculated address: {e}"),
            ))
        })?;

    Ok(l2_token_address)
}

/// Bridge tokens and execute contract call with automatic approval
#[allow(clippy::disallowed_methods)]
async fn bridge_and_call_with_approval(
    config: &Config,
    source_network: u64,
    destination_network: u64,
    token_address: &str,
    amount: &str,
    target: &str,
    data: &str,
    fallback: &str,
    gas_options: GasOptions,
    private_key: Option<&str>,
) -> Result<()> {
    let client = get_wallet_with_provider(config, source_network, private_key).await?;
    let bridge_ext_address = get_bridge_extension_address(config, source_network)?;
    let bridge_ext = BridgeExtensionContract::new(bridge_ext_address, Arc::new(client.clone()));

    let destination_chain_id = network_id_to_chain_id(config, destination_network)?;

    // Parse addresses and amounts
    let token_addr = Address::from_str(token_address).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid token address: {e}"),
        ))
    })?;

    let target_addr = Address::from_str(target).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid target address: {e}"),
        ))
    })?;

    let fallback_addr = Address::from_str(fallback).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid fallback address: {e}"),
        ))
    })?;

    let amount_wei = U256::from_dec_str(amount).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid amount: {e}"),
        ))
    })?;

    let call_data_bytes = hex::decode(data.trim_start_matches("0x")).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid call data hex: {e}"),
        ))
    })?;

    println!("🔧 Bridge and Call Debug:");
    println!("  - Token address: {}", token_address);
    println!("  - Amount: {} (Wei: {amount_wei})", amount);
    println!("  - Target: {}", target);
    println!("  - Fallback: {}", fallback);
    println!("  - Destination chain ID: {destination_chain_id}");
    println!("  - Bridge Extension: {bridge_ext_address:?}");

    // Step 1: Check and approve bridge extension to spend tokens
    let token = ERC20Contract::new(token_addr, Arc::new(client.clone()));

    println!("🔧 Checking allowance for bridge extension...");
    let allowance = token
        .allowance(client.address(), bridge_ext_address)
        .call()
        .await
        .map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Failed to check allowance: {e}"),
            ))
        })?;

    println!("🔧 Current allowance: {allowance}, Required: {amount_wei}");

    if allowance < amount_wei {
        info!(
            "Approving bridge extension contract to spend {} tokens",
            amount
        );
        println!("🔧 Calling approve: token.approve({bridge_ext_address:?}, {amount_wei})");
        let approve_call = token.approve(bridge_ext_address, amount_wei);
        let approve_tx = approve_call.send().await.map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Failed to approve tokens: {e}"),
            ))
        })?;
        println!("✅ Token approval transaction: {:#x}", approve_tx.tx_hash());

        // Wait for approval to be mined
        approve_tx.await.map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Approval transaction failed: {e}"),
            ))
        })?;
    }

    // Step 2: Execute bridgeAndCall
    println!("🔧 Executing bridgeAndCall...");

    let mut call = bridge_ext.bridge_and_call(
        token_addr,
        amount_wei,
        destination_chain_id,
        target_addr,
        fallback_addr,
        call_data_bytes.into(),
        true, // forceUpdateGlobalExitRoot
    );

    if gas_options.gas_limit.is_none() {
        call = call.gas(3_000_000u64); // Default high gas limit
    }

    let call = gas_options.apply_to_call_with_return(call);

    let tx = call.send().await.map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Failed to send bridge and call transaction: {e}"),
        ))
    })?;

    println!(
        "✅ Bridge and call transaction submitted: {:#x}",
        tx.tx_hash()
    );
    println!("🔧 This creates TWO bridge transactions:");
    println!("   1. Asset bridge (deposit_count = 0) - bridges tokens to JumpPoint");
    println!("   2. Message bridge (deposit_count = 1) - contains call instructions");
    println!();
    println!("💡 To complete the process, you need to claim both bridges:");
    println!(
        "   1. First claim asset: aggsandbox show bridges --network-id {} (find asset bridge)",
        source_network
    );
    println!("   2. Then: aggsandbox bridge claim --network {} --tx-hash <asset_bridge_tx_hash> --source-network {}", destination_network, source_network);
    println!(
        "   3. Then claim message: aggsandbox show bridges --network-id {} (find message bridge)",
        source_network
    );
    println!("   4. Finally: aggsandbox bridge claim --network {} --tx-hash <message_bridge_tx_hash> --source-network {}", destination_network, source_network);

    Ok(())
}
