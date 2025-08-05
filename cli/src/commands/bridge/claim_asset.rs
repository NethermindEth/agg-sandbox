use crate::api_client::OptimizedApiClient;
use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use std::str::FromStr;
use std::sync::Arc;

use super::{
    common::validation_error, get_bridge_contract_address, get_bridge_extension_address,
    get_wallet_with_provider, BridgeContract, ERC20Contract, GasOptions,
};

/// Arguments for claiming bridged assets
///
/// Use the builder pattern to construct this struct:
/// ```rust
/// let args = ClaimAssetArgs::builder()
///     .config(&config)
///     .network(1)
///     .tx_hash("0x1234567890123456789012345678901234567890123456789012345678901234")
///     .source_network(0)
///     .build_with_crate_error()?;
///
/// // Advanced usage with all options
/// let args = ClaimAssetArgs::builder()
///     .config(&config)
///     .network(1)
///     .tx_hash("0x1234567890123456789012345678901234567890123456789012345678901234")
///     .source_network(0)
///     .deposit_count(Some(0))
///     .token_address(Some("0x1234567890123456789012345678901234567890"))
///     .gas_options(gas_options)
///     .private_key("0x1234567890123456789012345678901234567890123456789012345678901234")
///     .custom_data(Some("0x12345678"))
///     .build_with_crate_error()?;
/// ```
pub struct ClaimAssetArgs<'a> {
    pub config: &'a Config,
    pub network: u64,
    pub tx_hash: &'a str,
    pub source_network: u64,
    pub deposit_count: Option<u64>,
    #[allow(dead_code)]
    pub token_address: Option<&'a str>,
    pub gas_options: GasOptions,
    pub private_key: Option<&'a str>,
    pub custom_data: Option<&'a str>,
}

impl<'a> ClaimAssetArgs<'a> {
    /// Create a new builder instance
    pub fn builder() -> ClaimAssetArgsBuilder<'a> {
        ClaimAssetArgsBuilder::default()
    }

    /// Create a builder with common defaults for asset claiming
    #[allow(dead_code)]
    pub fn asset_claim_builder(
        config: &'a Config,
        network: u64,
        tx_hash: &'a str,
        source_network: u64,
    ) -> ClaimAssetArgsBuilder<'a> {
        ClaimAssetArgsBuilder::default()
            .config(config)
            .network(network)
            .tx_hash(tx_hash)
            .source_network(source_network)
    }

    /// Create a builder with common defaults for message claiming
    #[allow(dead_code)]
    pub fn message_claim_builder(
        config: &'a Config,
        network: u64,
        tx_hash: &'a str,
        source_network: u64,
        custom_data: &'a str,
    ) -> ClaimAssetArgsBuilder<'a> {
        ClaimAssetArgsBuilder::default()
            .config(config)
            .network(network)
            .tx_hash(tx_hash)
            .source_network(source_network)
            .custom_data(Some(custom_data))
    }
}

/// Builder for constructing ClaimAssetArgs with validation
///
/// The builder pattern provides several benefits:
/// - Type-safe construction with compile-time validation
/// - Fluent API for easy chaining of method calls
/// - Clear separation between required and optional fields
/// - Built-in validation during the build process
/// - Prevents construction of invalid configurations
pub struct ClaimAssetArgsBuilder<'a> {
    config: Option<&'a Config>,
    network: Option<u64>,
    tx_hash: Option<&'a str>,
    source_network: Option<u64>,
    deposit_count: Option<u64>,
    token_address: Option<&'a str>,
    gas_options: Option<GasOptions>,
    private_key: Option<&'a str>,
    custom_data: Option<&'a str>,
}

impl<'a> Default for ClaimAssetArgsBuilder<'a> {
    fn default() -> Self {
        Self {
            config: None,
            network: None,
            tx_hash: None,
            source_network: None,
            deposit_count: None,
            token_address: None,
            gas_options: Some(GasOptions::new(None, None)),
            private_key: None,
            custom_data: None,
        }
    }
}

impl<'a> ClaimAssetArgsBuilder<'a> {
    /// Set the configuration
    pub fn config(mut self, config: &'a Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the destination network ID
    pub fn network(mut self, network: u64) -> Self {
        self.network = Some(network);
        self
    }

    /// Set the transaction hash
    pub fn tx_hash(mut self, tx_hash: &'a str) -> Self {
        self.tx_hash = Some(tx_hash);
        self
    }

    /// Set the source network ID
    pub fn source_network(mut self, source_network: u64) -> Self {
        self.source_network = Some(source_network);
        self
    }

    /// Set the deposit count (optional)
    pub fn deposit_count(mut self, deposit_count: Option<u64>) -> Self {
        self.deposit_count = deposit_count;
        self
    }

    /// Set the token address (optional)
    pub fn token_address(mut self, token_address: Option<&'a str>) -> Self {
        self.token_address = token_address;
        self
    }

    /// Set gas options
    pub fn gas_options(mut self, gas_options: GasOptions) -> Self {
        self.gas_options = Some(gas_options);
        self
    }

    /// Set gas limit
    #[allow(dead_code)]
    pub fn gas_limit(mut self, gas_limit: u64) -> Self {
        let gas_options = self
            .gas_options
            .get_or_insert_with(|| GasOptions::new(None, None));
        gas_options.gas_limit = Some(gas_limit);
        self
    }

    /// Set gas price (as string)
    #[allow(dead_code)]
    pub fn gas_price(mut self, gas_price: &str) -> Self {
        let gas_options = self
            .gas_options
            .get_or_insert_with(|| GasOptions::new(None, None));
        gas_options.gas_price = Some(gas_price.to_string());
        self
    }

    /// Set private key for signing transactions
    pub fn private_key(mut self, private_key: &'a str) -> Self {
        self.private_key = Some(private_key);
        self
    }

    /// Set custom data for message bridge claims
    pub fn custom_data(mut self, custom_data: Option<&'a str>) -> Self {
        self.custom_data = custom_data;
        self
    }

    /// Build the ClaimAssetArgs with validation
    pub fn build(self) -> std::result::Result<ClaimAssetArgs<'a>, &'static str> {
        let config = self.config.ok_or("Config is required")?;
        let network = self.network.ok_or("Network is required")?;
        let tx_hash = self.tx_hash.ok_or("Transaction hash is required")?;
        let source_network = self.source_network.ok_or("Source network is required")?;
        let gas_options = self.gas_options.ok_or("Gas options are required")?;

        // Validate transaction hash format
        if !tx_hash.starts_with("0x") || tx_hash.len() != 66 {
            return Err("Invalid transaction hash format (must be 0x-prefixed hex, 64 chars)");
        }

        // Validate token address if provided
        if let Some(addr) = self.token_address {
            if Address::from_str(addr).is_err() {
                return Err("Invalid token address format");
            }
        }

        // Validate custom data if provided
        if let Some(data) = self.custom_data {
            if !data.is_empty() && !data.starts_with("0x") {
                return Err("Invalid custom data format (must be 0x-prefixed hex)");
            }
        }

        Ok(ClaimAssetArgs {
            config,
            network,
            tx_hash,
            source_network,
            deposit_count: self.deposit_count,
            token_address: self.token_address,
            gas_options,
            private_key: self.private_key,
            custom_data: self.custom_data,
        })
    }

    /// Build and convert to crate's Result type
    pub fn build_with_crate_error(self) -> Result<ClaimAssetArgs<'a>> {
        self.build().map_err(validation_error)
    }
}

/// Claim bridged assets on destination network
pub async fn claim_asset(args: ClaimAssetArgs<'_>) -> Result<()> {
    let client = get_wallet_with_provider(args.config, args.network, args.private_key).await?;
    let bridge_address = get_bridge_contract_address(args.config, args.network)?;
    let bridge = BridgeContract::new(bridge_address, Arc::new(client.clone()));
    let api_client = OptimizedApiClient::global();

    println!(
        "🔍 Looking for bridge transaction with hash: {}",
        args.tx_hash
    );

    // For bridge-back scenarios (L2→L1), we need special logic:
    // - The bridge transaction is on the intermediate network (L2)
    // - But the proof data comes from the original token's network (L1)
    // Try to detect this by checking if we're claiming on L1 with source_network=0
    let (bridge_tx_network, proof_source_network) = if args.network == 0 && args.source_network == 0
    {
        // Potential bridge-back scenario: L2→L1 claim
        // First try to find the transaction on L2 (network 1)
        let l2_bridges = api_client.get_bridges(args.config, 1).await.ok();
        if let Some(l2_response) = l2_bridges {
            if let Some(bridges) = l2_response["bridges"].as_array() {
                if bridges
                    .iter()
                    .any(|bridge| bridge["tx_hash"].as_str() == Some(args.tx_hash))
                {
                    println!("🔍 Detected bridge-back scenario: transaction found on L2, using L2 for proof data");
                    (1u64, 1u64) // Bridge tx is on L2, proof data from L2
                } else {
                    (args.source_network, args.source_network) // Normal scenario
                }
            } else {
                (args.source_network, args.source_network) // Normal scenario
            }
        } else {
            (args.source_network, args.source_network) // Normal scenario
        }
    } else {
        (args.source_network, args.source_network) // Normal scenario
    };

    // Get bridges from the network where the transaction actually occurred
    let bridges_response = api_client
        .get_bridges(args.config, bridge_tx_network)
        .await
        .map_err(|e| validation_error(&format!("Failed to get bridges: {e}")))?;

    let bridges = bridges_response["bridges"]
        .as_array()
        .ok_or_else(|| validation_error("Invalid bridges response"))?;

    // Find our bridge transaction
    let bridge_info = if let Some(specific_deposit_count) = args.deposit_count {
        println!(
            "🔍 Looking for bridge with tx_hash: {} and deposit_count: {specific_deposit_count}",
            args.tx_hash
        );
        bridges
            .iter()
            .find(|bridge| {
                bridge["tx_hash"].as_str() == Some(args.tx_hash)
                    && bridge["deposit_count"].as_u64() == Some(specific_deposit_count)
            })
            .ok_or_else(|| {
                crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                    &format!(
                        "Bridge transaction {} with deposit_count {specific_deposit_count} not found",
                        args.tx_hash
                    ),
                ))
            })?
    } else {
        println!(
            "🔍 Looking for bridge with tx_hash: {} (deposit_count auto-detected)",
            args.tx_hash
        );
        bridges
            .iter()
            .find(|bridge| bridge["tx_hash"].as_str() == Some(args.tx_hash))
            .ok_or_else(|| {
                crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                    &format!("Bridge transaction {} not found", args.tx_hash),
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
    println!("🔍 Bridge leaf type: {leaf_type} (0=Asset, 1=Message)");

    // Get L1 info tree index from the proof source network
    // For bridge-back scenarios, this uses L2 (where the bridge tx occurred)
    let tree_index_response = api_client
        .get_l1_info_tree_index(args.config, proof_source_network, deposit_count)
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
        .get_claim_proof(args.config, proof_source_network, leaf_index, deposit_count)
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
        .unwrap_or_else(|| args.source_network as u32);
    let destination_network_id = bridge_info["destination_network"]
        .as_u64()
        .map(|n| n as u32)
        .unwrap_or_else(|| args.network as u32);

    // For message bridges (leaf_type = 1), use BridgeExtension addresses
    // For asset bridges (leaf_type = 0), use the original bridge addresses
    let (origin_address, destination_address) = if leaf_type == 1 {
        // Message bridge - use BridgeExtension addresses
        let origin_bridge_ext = get_bridge_extension_address(args.config, args.source_network)?;
        let dest_bridge_ext = get_bridge_extension_address(args.config, args.network)?;
        println!("🔗 Using BridgeExtension addresses for message bridge:");
        println!(
            "   Origin: {origin_bridge_ext:#x} (network {})",
            args.source_network
        );
        println!(
            "   Destination: {dest_bridge_ext:#x} (network {})",
            args.network
        );
        (
            format!("{origin_bridge_ext:#x}"),
            format!("{dest_bridge_ext:#x}"),
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
        println!("   Origin: {origin_addr} (network {})", args.source_network);
        println!("   Destination: {dest_addr} (network {})", args.network);
        (origin_addr.to_string(), dest_addr.to_string())
    };
    let amount = bridge_info["amount"].as_str().ok_or_else(|| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            "Missing amount in bridge info",
        ))
    })?;
    let metadata = if let Some(custom) = args.custom_data {
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
        let source_client =
            get_wallet_with_provider(args.config, proof_source_network, None).await?;
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

        tracing::info!(
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

        let asset_params = AssetClaimParams {
            deposit_count,
            mainnet_root,
            rollup_root,
            origin_network,
            origin_addr,
            destination_network_id,
            dest_addr,
            amount_wei,
            metadata_bytes: metadata_bytes.clone(),
        };

        execute_claim_asset(&bridge, asset_params, &args.gas_options).await?
    } else {
        // Message bridge - call claimMessage
        println!("📨 Claiming message bridge to trigger contract execution");

        let claim_message_args = super::claim_message::ClaimMessageArgs::builder()
            .bridge(&bridge)
            .deposit_count(deposit_count)
            .mainnet_root(mainnet_root)
            .rollup_root(rollup_root)
            .origin_network(origin_network)
            .origin_addr(origin_addr)
            .destination_network_id(destination_network_id)
            .dest_addr(dest_addr)
            .amount_wei(amount_wei)
            .metadata_bytes(metadata_bytes)
            .gas_options(&args.gas_options)
            .build_with_crate_error()?;

        super::claim_message::execute_claim_message(claim_message_args).await?
    };

    println!("✅ Claim transaction submitted: {tx_hash:#x}");
    if leaf_type == 0 {
        println!("🎉 Assets should be available once the transaction is mined!");
    } else {
        println!("🎉 Message bridge claimed! Contract call should execute automatically.");
    }

    Ok(())
}

/// Parameters for asset claiming operations
#[derive(Debug, Clone)]
pub struct AssetClaimParams {
    pub deposit_count: u64,
    pub mainnet_root: H256,
    pub rollup_root: H256,
    pub origin_network: u32,
    pub origin_addr: Address,
    pub destination_network_id: u32,
    pub dest_addr: Address,
    pub amount_wei: U256,
    pub metadata_bytes: Vec<u8>,
}

/// Execute claimAsset contract call
pub async fn execute_claim_asset(
    bridge: &BridgeContract<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>,
    params: AssetClaimParams,
    gas_options: &GasOptions,
) -> Result<H256> {
    let mut call = bridge.claim_asset(
        params.deposit_count.into(), // globalIndex
        params.mainnet_root.into(),  // mainnetExitRoot
        params.rollup_root.into(),   // rollupExitRoot
        params.origin_network,
        params.origin_addr,
        params.destination_network_id,
        params.dest_addr,
        params.amount_wei,
        ethers::types::Bytes::from(params.metadata_bytes), // metadata
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
