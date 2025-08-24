use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, info};

use super::{
    common::validation_error, get_bridge_contract_address, get_wallet_with_provider,
    is_eth_address, BridgeContract, ERC20Contract,
};

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

/// Arguments for bridging assets between networks
///
/// Use the builder pattern to construct this struct:
/// ```rust
/// // Basic usage with required fields
/// let args = BridgeAssetArgs::builder()
///     .config(&config)
///     .source_network(0)
///     .destination_network(1)
///     .amount("1000000000000000000")
///     .token_address("0x0000000000000000000000000000000000000000")
///     .build_with_crate_error()?;
///
/// // Advanced usage with optional fields
/// let args = BridgeAssetArgs::builder()
///     .config(&config)
///     .source_network(0)
///     .destination_network(1)
///     .amount("1000000000000000000")
///     .token_address("0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC")
///     .to_address("0x1234567890123456789012345678901234567890")
///     .gas_limit(300000)
///     .gas_price("20000000000")
///     .private_key("0x1234567890123456789012345678901234567890123456789012345678901234")
///     .build_with_crate_error()?;
/// ```
pub struct BridgeAssetArgs<'a> {
    pub config: &'a Config,
    pub source_network: u64,
    pub destination_network: u64,
    pub amount: &'a str,
    pub token_address: &'a str,
    pub to_address: Option<&'a str>,
    pub gas_options: GasOptions,
    pub private_key: Option<&'a str>,
}

impl<'a> BridgeAssetArgs<'a> {
    /// Create a new builder instance
    pub fn builder() -> BridgeAssetArgsBuilder<'a> {
        BridgeAssetArgsBuilder::default()
    }

    /// Create a builder with common defaults for ETH bridging
    #[allow(dead_code)]
    pub fn eth_bridge_builder(
        config: &'a Config,
        source_network: u64,
        destination_network: u64,
        amount: &'a str,
    ) -> BridgeAssetArgsBuilder<'a> {
        BridgeAssetArgsBuilder::default()
            .config(config)
            .source_network(source_network)
            .destination_network(destination_network)
            .amount(amount)
            .token_address("0x0000000000000000000000000000000000000000")
    }
}

/// Builder for constructing BridgeAssetArgs with validation
///
/// The builder pattern provides several benefits:
/// - Type-safe construction with compile-time validation
/// - Fluent API for easy chaining of method calls
/// - Clear separation between required and optional fields
/// - Built-in validation during the build process
/// - Prevents construction of invalid configurations
pub struct BridgeAssetArgsBuilder<'a> {
    config: Option<&'a Config>,
    source_network: Option<u64>,
    destination_network: Option<u64>,
    amount: Option<&'a str>,
    token_address: Option<&'a str>,
    to_address: Option<&'a str>,
    gas_options: Option<GasOptions>,
    private_key: Option<&'a str>,
}

impl<'a> Default for BridgeAssetArgsBuilder<'a> {
    fn default() -> Self {
        Self {
            config: None,
            source_network: None,
            destination_network: None,
            amount: None,
            token_address: None,
            to_address: None,
            gas_options: Some(GasOptions::new(None, None)),
            private_key: None,
        }
    }
}

impl<'a> BridgeAssetArgsBuilder<'a> {
    /// Set the configuration
    pub fn config(mut self, config: &'a Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the source network ID
    pub fn source_network(mut self, source_network: u64) -> Self {
        self.source_network = Some(source_network);
        self
    }

    /// Set the destination network ID
    pub fn destination_network(mut self, destination_network: u64) -> Self {
        self.destination_network = Some(destination_network);
        self
    }

    /// Set the amount to bridge (in wei)
    pub fn amount(mut self, amount: &'a str) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the token address to bridge
    pub fn token_address(mut self, token_address: &'a str) -> Self {
        self.token_address = Some(token_address);
        self
    }

    /// Set the recipient address (optional)
    pub fn recipient_address(mut self, to_address: &'a str) -> Self {
        self.to_address = Some(to_address);
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

    pub fn build(self) -> std::result::Result<BridgeAssetArgs<'a>, &'static str> {
        let config = self.config.ok_or("Config is required")?;
        let source_network = self.source_network.ok_or("Source network is required")?;
        let destination_network = self
            .destination_network
            .ok_or("Destination network is required")?;
        let amount = self.amount.ok_or("Amount is required")?;
        let token_address = self.token_address.ok_or("Token address is required")?;
        let gas_options = self.gas_options.ok_or("Gas options are required")?;

        // Validate amount format
        if U256::from_dec_str(amount).is_err() {
            return Err("Invalid amount format");
        }

        // Validate token address format
        if Address::from_str(token_address).is_err() {
            return Err("Invalid token address format");
        }

        // Validate to_address if provided
        if let Some(addr) = self.to_address {
            if Address::from_str(addr).is_err() {
                return Err("Invalid recipient address format");
            }
        }

        Ok(BridgeAssetArgs {
            config,
            source_network,
            destination_network,
            amount,
            token_address,
            to_address: self.to_address,
            gas_options,
            private_key: self.private_key,
        })
    }

    /// Build and convert to crate's Result type
    pub fn build_with_crate_error(self) -> Result<BridgeAssetArgs<'a>> {
        self.build().map_err(validation_error)
    }
}

/// Bridge assets between networks
#[allow(clippy::disallowed_methods)] // Allow tracing macros
pub async fn bridge_asset(args: BridgeAssetArgs<'_>) -> Result<()> {
    let client =
        get_wallet_with_provider(args.config, args.source_network, args.private_key).await?;
    let bridge_address = get_bridge_contract_address(args.config, args.source_network)?;
    let bridge = BridgeContract::new(bridge_address, Arc::new(client.clone()));

    let destination_network_id = args.destination_network as u32;

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
    let tx_hash_for_claim = if is_eth_address(args.token_address) {
        info!(
            "Bridging ETH from network {} to network {}",
            args.source_network, args.destination_network
        );
        debug!("Bridging ETH - amount: {amount_wei} wei, destination_network_id: {destination_network_id}, recipient: {recipient:?}");

        let call = bridge
            .bridge_asset(
                destination_network_id,
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

        let tx_hash = tx.tx_hash();
        println!("✅ Bridge transaction submitted: {tx_hash:#x}");
        tx_hash
    } else {
        info!(
            "Bridging ERC20 token {} from network {} to network {}",
            args.token_address, args.source_network, args.destination_network
        );
        debug!("ERC20 Bridge Debug:");
        debug!("  - Token address: {}", args.token_address);
        debug!("  - Token address (parsed): {token_addr:?}");
        debug!("  - From address: {:?}", client.address());
        debug!("  - Bridge address: {bridge_address:?}");
        debug!("  - Amount: {} (Wei: {amount_wei})", args.amount);
        debug!("  - Destination network ID: {destination_network_id}");
        debug!("  - Recipient: {recipient:?}");

        // First check and approve if needed
        let token = ERC20Contract::new(token_addr, Arc::new(client.clone()));

        debug!(
            "Checking allowance: token.allowance({:?}, {bridge_address:?})",
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

        debug!("Current allowance: {allowance}, Required: {amount_wei}");

        if allowance < amount_wei {
            info!("Approving bridge contract to spend {} tokens", args.amount);
            debug!("Calling approve: token.approve({bridge_address:?}, {amount_wei})");
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
        debug!("Calling bridgeAsset:");
        debug!("  - destination_network_id: {destination_network_id}");
        debug!("  - recipient: {recipient:?}");
        debug!("  - amount_wei: {amount_wei}");
        debug!("  - token_addr: {token_addr:?}");
        debug!("  - forceUpdateGlobalExitRoot: true");

        let call = bridge.bridge_asset(
            destination_network_id,
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

        let tx_hash = tx.tx_hash();
        println!("✅ Bridge transaction submitted: {tx_hash:#x}");
        tx_hash
    };

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

    println!("💡 Use `aggsandbox bridge claim --network-id {} --tx-hash {tx_hash_for_claim:#x} --source-network-id {claim_source_network}` to claim assets", args.destination_network);
    println!("⏰ Wait at least 5 seconds after bridging before claiming to allow AggKit to update the Global Exit Root (GER)");

    Ok(())
}
