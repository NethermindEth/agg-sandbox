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
        mut call: ContractCall<M, D>
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
    },
    /// 📥 Claim bridged assets on destination network
    #[command(
        long_about = "Claim assets that were bridged from another network.\n\nUse the transaction hash from the original bridge operation to claim\nthe corresponding assets on the destination network.\n\nClaiming typically requires waiting for the bridge to process the deposit\nand generate the necessary proofs.\n\nExamples:\n  aggsandbox bridge claim --network 1 --tx-hash 0xabc123... --source-network 0\n  aggsandbox bridge claim -n 1 -t 0xdef456... -s 0 --token-address 0x456..."
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
        /// Token contract address that was bridged (auto-detected if not provided)
        #[arg(long, help = "Token contract address that was bridged (auto-detected if not provided)")]
        token_address: Option<String>,
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
        function claimAsset(bytes32[32] smtProofLocalExitRoot, bytes32[32] smtProofRollupExitRoot, uint256 globalIndex, bytes32 mainnetExitRoot, bytes32 rollupExitRoot, uint32 originNetwork, address originTokenAddress, uint32 destinationNetwork, address destinationAddress, uint256 amount, bytes metadata) external
        function claimMessage(bytes32[32] smtProofLocalExitRoot, bytes32[32] smtProofRollupExitRoot, uint256 globalIndex, bytes32 mainnetExitRoot, bytes32 rollupExitRoot, uint32 originNetwork, address originAddress, uint32 destinationNetwork, address destinationAddress, uint256 amount, bytes message) external
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
        } => {
            info!(
                network = network,
                destination_network = destination_network,
                amount = %amount,
                token_address = %token_address,
                "Executing bridge asset command"
            );

            let gas_options = GasOptions::new(gas_limit, gas_price.as_deref());
            bridge_asset(
                &config,
                network,
                destination_network,
                &amount,
                &token_address,
                to_address.as_deref(),
                gas_options,
            ).await
        }
        BridgeCommands::Claim {
            network,
            tx_hash,
            source_network,
            token_address,
            gas_limit,
            gas_price,
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
                token_address.as_deref(),
                gas_options,
            ).await
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

            let gas_options = GasOptions::new(gas_limit, gas_price.as_deref());
            let message_params = BridgeMessageParams::new(
                target,
                data,
                amount,
                fallback_address,
            );
            bridge_message(
                &config,
                network,
                destination_network,
                message_params,
                gas_options,
            ).await
        }
    }
}

/// Get provider for a network
async fn get_provider(config: &Config, network_id: u64) -> Result<Arc<Provider<Http>>> {
    let rpc_url = match network_id {
        0 => config.networks.l1.rpc_url.as_str(),
        1 => config.networks.l2.rpc_url.as_str(), 
        2 => config.networks.l3.as_ref().map(|l3| l3.rpc_url.as_str()).unwrap_or(""),
        _ => return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Unsupported network ID: {}", network_id))
        )),
    };

    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Failed to create provider: {}", e))
        ))?;
    
    Ok(Arc::new(provider))
}

/// Get wallet with provider for a network
async fn get_wallet_with_provider(config: &Config, network_id: u64) -> Result<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>> {
    let provider = get_provider(config, network_id).await?;
    
    // Use the first private key from config
    let private_key = config.accounts.private_keys.first()
        .ok_or_else(|| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed("No private keys configured")
        ))?;
    
    let wallet = LocalWallet::from_str(private_key)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid private key: {}", e))
        ))?;
    
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
        _ => return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Unsupported network ID: {}", network_id))
        )),
    };
    
    let address_str = config.contracts.get_contract(layer, "PolygonZkEVMBridge");
    if address_str == "Not deployed" {
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Bridge contract not deployed on network {}", network_id))
        ));
    }
    
    Address::from_str(&address_str)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid bridge contract address: {}", e))
        ))
}

/// Get bridge extension contract address for a network
fn get_bridge_extension_address(config: &Config, network_id: u64) -> Result<Address> {
    let layer = match network_id {
        0 => "l1",
        1 => "l2",
        2 => "l3", 
        _ => return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Unsupported network ID: {}", network_id))
        )),
    };
    
    let address_str = config.contracts.get_contract(layer, "BridgeExtension");
    if address_str == "Not deployed" {
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Bridge extension contract not deployed on network {}", network_id))
        ));
    }
    
    Address::from_str(&address_str)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid bridge extension address: {}", e))
        ))
}

/// Check if address is the zero address (for ETH)
fn is_eth_address(address: &str) -> bool {
    address == "0x0000000000000000000000000000000000000000" || 
    Address::from_str(address).map(|addr| addr.is_zero()).unwrap_or(false)
}

/// Convert network ID to chain ID based on standard mapping
fn network_id_to_chain_id(config: &Config, network_id: u64) -> Result<u32> {
    let chain_id = match network_id {
        0 => config.networks.l1.chain_id.as_u64()?, // L1 Mainnet (Chain ID 1)
        1 => config.networks.l2.chain_id.as_u64()?, // AggLayer 1 (Chain ID 1101)
        2 => config.networks.l3.as_ref().map(|l3| l3.chain_id.as_u64()).transpose()?.unwrap_or(137), // AggLayer 2 (Chain ID 137)
        _ => return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Unsupported network ID: {}", network_id))
        )),
    };
    
    Ok(chain_id as u32)
}

/// Bridge assets between networks
async fn bridge_asset(
    config: &Config,
    source_network: u64,
    destination_network: u64,
    amount: &str,
    token_address: &str,
    to_address: Option<&str>,
    gas_options: GasOptions,
) -> Result<()> {
    let client = get_wallet_with_provider(config, source_network).await?;
    let bridge_address = get_bridge_contract_address(config, source_network)?;
    let bridge = BridgeContract::new(bridge_address, Arc::new(client.clone()));
    
    let destination_chain_id = network_id_to_chain_id(config, destination_network)?;
    
    let recipient = if let Some(addr) = to_address {
        Address::from_str(addr)
            .map_err(|e| crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!("Invalid recipient address: {}", e))
            ))?
    } else {
        client.address()
    };
    
    let amount_wei = U256::from_dec_str(amount)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid amount: {}", e))
        ))?;
    
    let token_addr = Address::from_str(token_address)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid token address: {}", e))
        ))?;
    
    // Handle ETH vs ERC20 token bridging
    if is_eth_address(token_address) {
        info!("Bridging ETH from network {} to network {}", source_network, destination_network);
        
        let call = bridge.bridge_asset(
            destination_chain_id,
            recipient,
            amount_wei,
            token_addr,
            true, // forceUpdateGlobalExitRoot
            Bytes::new(), // empty permit data
        ).value(amount_wei);
        
        let call = gas_options.apply_to_call_with_return(call);
        
        let tx = call.send().await
            .map_err(|e| crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!("Failed to send bridge transaction: {}", e))
            ))?;
        
        println!("✅ Bridge transaction submitted: {:#x}", tx.tx_hash());
        
    } else {
        info!("Bridging ERC20 token {} from network {} to network {}", token_address, source_network, destination_network);
        
        // First check and approve if needed
        let token = ERC20Contract::new(token_addr, Arc::new(client.clone()));
        
        let allowance = token.allowance(client.address(), bridge_address).call().await
            .map_err(|e| crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!("Failed to check allowance: {}", e))
            ))?;
        
        if allowance < amount_wei {
            info!("Approving bridge contract to spend {} tokens", amount);
            let approve_call = token.approve(bridge_address, amount_wei);
            let approve_tx = approve_call.send().await
                .map_err(|e| crate::error::AggSandboxError::Config(
                    crate::error::ConfigError::validation_failed(&format!("Failed to approve tokens: {}", e))
                ))?;
            println!("✅ Token approval transaction: {:#x}", approve_tx.tx_hash());
            
            // Wait for approval to be mined
            approve_tx.await
                .map_err(|e| crate::error::AggSandboxError::Config(
                    crate::error::ConfigError::validation_failed(&format!("Approval transaction failed: {}", e))
                ))?;
        }
        
        // Now bridge the tokens
        let call = bridge.bridge_asset(
            destination_chain_id,
            recipient,
            amount_wei,
            token_addr,
            true, // forceUpdateGlobalExitRoot
            Bytes::new(), // empty permit data
        );
        
        let call = gas_options.apply_to_call_with_return(call);
        
        let tx = call.send().await
            .map_err(|e| crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!("Failed to send bridge transaction: {}", e))
            ))?;
        
        println!("✅ Bridge transaction submitted: {:#x}", tx.tx_hash());
    }
    
    println!("💡 Use 'aggsandbox bridge claim --network {} --tx-hash <tx_hash> --source-network {}' to claim assets", destination_network, source_network);
    
    Ok(())
}

/// Claim bridged assets on destination network
async fn claim_asset(
    config: &Config,
    network: u64,
    tx_hash: &str,
    source_network: u64,
    _token_address: Option<&str>,
    gas_options: GasOptions,
) -> Result<()> {
    let client = get_wallet_with_provider(config, network).await?;
    let bridge_address = get_bridge_contract_address(config, network)?;
    let bridge = BridgeContract::new(bridge_address, Arc::new(client.clone()));
    let api_client = OptimizedApiClient::global();
    
    println!("🔍 Looking for bridge transaction with hash: {}", tx_hash);
    
    // Get bridges from source network to find our transaction
    let bridges_response = api_client.get_bridges(config, source_network).await
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Failed to get bridges: {}", e))
        ))?;
    
    let bridges = bridges_response["bridges"].as_array()
        .ok_or_else(|| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed("Invalid bridges response")
        ))?;
    
    // Find our bridge transaction
    let bridge_info = bridges.iter()
        .find(|bridge| bridge["tx_hash"].as_str() == Some(tx_hash))
        .ok_or_else(|| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Bridge transaction {} not found", tx_hash))
        ))?;
    
    let deposit_count = bridge_info["deposit_count"].as_u64()
        .ok_or_else(|| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed("Missing deposit_count in bridge info")
        ))?;
    
    println!("📋 Found bridge with deposit count: {}", deposit_count);
    
    // Get L1 info tree index
    let tree_index_response = api_client.get_l1_info_tree_index(config, source_network, deposit_count).await
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Failed to get L1 info tree index: {}", e))
        ))?;
    
    let leaf_index = tree_index_response["l1_info_tree_index"].as_u64()
        .unwrap_or(tree_index_response.as_u64().unwrap_or(0));
    
    println!("🌳 L1 info tree index: {}", leaf_index);
    
    // Get claim proof
    let proof_response = api_client.get_claim_proof(config, network, leaf_index, deposit_count).await
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Failed to get claim proof: {}", e))
        ))?;
    
    let l1_info_tree_leaf = &proof_response["l1_info_tree_leaf"];
    let mainnet_exit_root = l1_info_tree_leaf["mainnet_exit_root"].as_str()
        .ok_or_else(|| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed("Missing mainnet_exit_root in proof")
        ))?;
    
    let rollup_exit_root = l1_info_tree_leaf["rollup_exit_root"].as_str()
        .ok_or_else(|| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed("Missing rollup_exit_root in proof")
        ))?;
    
    println!("🔐 Got claim proof data");
    
    // Extract bridge parameters
    let origin_network = bridge_info["origin_network"].as_u64()
        .map(|n| n as u32)
        .unwrap_or_else(|| network_id_to_chain_id(config, source_network).unwrap_or(1));
    let origin_address = bridge_info["origin_address"].as_str()
        .ok_or_else(|| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed("Missing origin_address in bridge info")
        ))?;
    let destination_network_id = bridge_info["destination_network"].as_u64()
        .map(|n| n as u32)
        .unwrap_or_else(|| network_id_to_chain_id(config, network).unwrap_or(1101));
    let destination_address = bridge_info["destination_address"].as_str()
        .ok_or_else(|| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed("Missing destination_address in bridge info")
        ))?;
    let amount = bridge_info["amount"].as_str()
        .ok_or_else(|| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed("Missing amount in bridge info")
        ))?;
    let metadata = bridge_info["metadata"].as_str().unwrap_or("0x");
    
    // Convert addresses and amount
    let origin_addr = Address::from_str(origin_address)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid origin address: {}", e))
        ))?;
    let dest_addr = Address::from_str(destination_address)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid destination address: {}", e))
        ))?;
    let amount_wei = U256::from_dec_str(amount)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid amount: {}", e))
        ))?;
    let mainnet_root = H256::from_str(mainnet_exit_root)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid mainnet exit root: {}", e))
        ))?;
    let rollup_root = H256::from_str(rollup_exit_root)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid rollup exit root: {}", e))
        ))?;
    
    // Encode ERC20 token metadata properly for claimAsset
    let metadata_bytes = if !metadata.is_empty() && metadata != "0x" {
        // Use metadata from API if available
        hex::decode(metadata.trim_start_matches("0x"))
            .map_err(|e| crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!("Invalid metadata hex: {}", e))
            ))?
    } else {
        // For first-time bridges, fetch and encode ERC20 token details
        let source_client = get_wallet_with_provider(config, source_network).await?;
        let token_contract = ERC20Contract::new(origin_addr, Arc::new(source_client));
        
        // Fetch token details
        let token_name = token_contract.name().call().await
            .unwrap_or_else(|_| "AggERC20".to_string());
        let token_symbol = token_contract.symbol().call().await
            .unwrap_or_else(|_| "AGGERC20".to_string());
        let token_decimals = token_contract.decimals().call().await
            .unwrap_or(18u8);
        
        info!("Encoding ERC20 metadata: name={}, symbol={}, decimals={}", token_name, token_symbol, token_decimals);
        
        // ABI encode as (string,string,uint8)
        use ethers::abi::{encode, Token};
        let tokens = vec![
            Token::String(token_name),
            Token::String(token_symbol),
            Token::Uint(U256::from(token_decimals)),
        ];
        encode(&tokens)
    };
    
    println!("💰 Claiming {} tokens to {}", amount, destination_address);
    
    // Create empty merkle proofs (in sandbox mode these are not used)
    let empty_proof: [[u8; 32]; 32] = [[0u8; 32]; 32];
    
    // Call claimAsset
    let mut call = bridge.claim_asset(
        empty_proof, // smtProofLocalExitRoot
        empty_proof, // smtProofRollupExitRoot  
        deposit_count.into(), // globalIndex
        mainnet_root.into(),
        rollup_root.into(),
        origin_network,
        origin_addr,
        destination_network_id,
        dest_addr,
        amount_wei,
        metadata_bytes.into(),
    );
    
    if gas_options.gas_limit.is_none() {
        call = call.gas(3_000_000u64); // Default high gas limit for claims
    }
    
    let call = gas_options.apply_to_call_with_return(call);
    
    let tx = call.send().await
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Failed to send claim transaction: {}", e))
        ))?;
    
    println!("✅ Claim transaction submitted: {:#x}", tx.tx_hash());
    println!("🎉 Assets should be available once the transaction is mined!");
    
    Ok(())
}

/// Bridge with contract call (bridgeAndCall)
async fn bridge_message(
    config: &Config,
    source_network: u64,
    destination_network: u64,
    params: BridgeMessageParams,
    gas_options: GasOptions,
) -> Result<()> {
    let client = get_wallet_with_provider(config, source_network).await?;
    let bridge_ext_address = get_bridge_extension_address(config, source_network)?;
    let bridge_ext = BridgeExtensionContract::new(bridge_ext_address, Arc::new(client.clone()));
    
    let destination_chain_id = network_id_to_chain_id(config, destination_network)?;
    
    let target_addr = Address::from_str(&params.target)
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid target address: {}", e))
        ))?;
    
    let fallback_addr = if let Some(addr) = &params.fallback_address {
        Address::from_str(addr)
            .map_err(|e| crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!("Invalid fallback address: {}", e))
            ))?
    } else {
        client.address()
    };
    
    let call_data = hex::decode(params.data.trim_start_matches("0x"))
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Invalid call data hex: {}", e))
        ))?;
    
    let eth_amount = if let Some(amt) = &params.amount {
        U256::from_dec_str(amt)
            .map_err(|e| crate::error::AggSandboxError::Config(
                crate::error::ConfigError::validation_failed(&format!("Invalid amount: {}", e))
            ))?
    } else {
        U256::zero()
    };
    
    info!("Bridging message from network {} to network {} with call to {}", source_network, destination_network, params.target);
    
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
    
    let tx = call.send().await
        .map_err(|e| crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("Failed to send bridge and call transaction: {}", e))
        ))?;
    
    println!("✅ Bridge and call transaction submitted: {:#x}", tx.tx_hash());
    println!("💡 This creates both asset and message bridges. The message should execute automatically when ready.");
    
    Ok(())
}
