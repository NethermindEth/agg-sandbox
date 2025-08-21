use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

use super::{
    get_bridge_extension_address, get_wallet_with_provider, BridgeExtensionContract, ERC20Contract,
    GasOptions,
};

/// Parameters for bridge message operations
///
/// Use the builder pattern to construct this struct:
/// ```rust
/// // Basic usage with required fields
/// let params = BridgeMessageParams::builder()
///     .target("0x1234567890123456789012345678901234567890")
///     .data("0x12345678")
///     .build_with_crate_error()?;
///
/// // Advanced usage with all options
/// let params = BridgeMessageParams::builder()
///     .target("0x1234567890123456789012345678901234567890")
///     .data("0x12345678")
///     .amount("1000000000000000000")
///     .fallback_address("0x0987654321098765432109876543210987654321")
///     .build_with_crate_error()?;
/// ```
#[derive(Debug, Clone)]
pub struct BridgeMessageParams {
    pub target: String,
    pub data: String,
    pub amount: Option<String>,
    pub fallback_address: Option<String>,
}

impl BridgeMessageParams {
    /// Create a new builder instance
    pub fn builder() -> BridgeMessageParamsBuilder {
        BridgeMessageParamsBuilder::default()
    }

    /// Create a builder with common defaults for ETH message bridging
    #[allow(dead_code)]
    pub fn eth_message_builder(target: &str, data: &str) -> BridgeMessageParamsBuilder {
        BridgeMessageParamsBuilder::default()
            .target(target)
            .data(data)
    }

    #[allow(dead_code)]
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

/// Builder for constructing BridgeMessageParams with validation
///
/// The builder pattern provides several benefits:
/// - Type-safe construction with compile-time validation
/// - Fluent API for easy chaining of method calls
/// - Clear separation between required and optional fields
/// - Built-in validation during the build process
/// - Prevents construction of invalid configurations
#[derive(Default)]
pub struct BridgeMessageParamsBuilder {
    target: Option<String>,
    data: Option<String>,
    amount: Option<String>,
    fallback_address: Option<String>,
}

impl BridgeMessageParamsBuilder {
    /// Set the target contract address
    pub fn target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }

    /// Set the call data (hex encoded)
    pub fn data(mut self, data: &str) -> Self {
        self.data = Some(data.to_string());
        self
    }

    /// Set the ETH amount to send with the call in wei (optional)
    pub fn amount(mut self, amount: &str) -> Self {
        self.amount = Some(amount.to_string());
        self
    }

    /// Set the fallback address if the call fails (optional)
    pub fn fallback_address(mut self, fallback_address: &str) -> Self {
        self.fallback_address = Some(fallback_address.to_string());
        self
    }

    /// Build the BridgeMessageParams with validation
    pub fn build(self) -> std::result::Result<BridgeMessageParams, &'static str> {
        let target = self.target.ok_or("Target address is required")?;
        let data = self.data.ok_or("Call data is required")?;

        // Validate target address format
        if Address::from_str(&target).is_err() {
            return Err("Invalid target address format");
        }

        // Validate call data format
        if hex::decode(data.trim_start_matches("0x")).is_err() {
            return Err("Invalid call data hex format");
        }

        // Validate amount if provided
        if let Some(amt) = &self.amount {
            if U256::from_dec_str(amt).is_err() {
                return Err("Invalid amount format");
            }
        }

        // Validate fallback address if provided
        if let Some(addr) = &self.fallback_address {
            if Address::from_str(addr).is_err() {
                return Err("Invalid fallback address format");
            }
        }

        Ok(BridgeMessageParams {
            target,
            data,
            amount: self.amount,
            fallback_address: self.fallback_address,
        })
    }

    /// Build and convert to crate's Result type
    pub fn build_with_crate_error(self) -> Result<BridgeMessageParams> {
        self.build().map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(e))
        })
    }
}

/// Arguments for bridge and call operations with approval
///
/// Use the builder pattern to construct this struct:
/// ```rust
/// let args = BridgeAndCallArgs::builder()
///     .config(&config)
///     .source_network(0)
///     .destination_network(1)
///     .token_address("0x1234567890123456789012345678901234567890")
///     .amount("1000000000000000000")
///     .target("0x0987654321098765432109876543210987654321")
///     .data("0x12345678")
///     .fallback("0x1111111111111111111111111111111111111111")
///     .build_with_crate_error()?;
/// ```
pub struct BridgeAndCallArgs<'a> {
    pub config: &'a Config,
    pub source_network: u64,
    pub destination_network: u64,
    pub token_address: &'a str,
    pub amount: &'a str,
    pub target: &'a str,
    pub data: &'a str,
    pub fallback: &'a str,
    pub gas_options: GasOptions,
    pub private_key: Option<&'a str>,
    pub msg_value: Option<&'a str>,
}

impl<'a> BridgeAndCallArgs<'a> {
    /// Create a new builder instance
    pub fn builder() -> BridgeAndCallArgsBuilder<'a> {
        BridgeAndCallArgsBuilder::default()
    }
}

/// Builder for constructing BridgeAndCallArgs with validation
pub struct BridgeAndCallArgsBuilder<'a> {
    config: Option<&'a Config>,
    source_network: Option<u64>,
    destination_network: Option<u64>,
    token_address: Option<&'a str>,
    amount: Option<&'a str>,
    target: Option<&'a str>,
    data: Option<&'a str>,
    fallback: Option<&'a str>,
    gas_options: Option<GasOptions>,
    private_key: Option<&'a str>,
    msg_value: Option<&'a str>,
}

impl<'a> Default for BridgeAndCallArgsBuilder<'a> {
    fn default() -> Self {
        Self {
            config: None,
            source_network: None,
            destination_network: None,
            token_address: None,
            amount: None,
            target: None,
            data: None,
            fallback: None,
            gas_options: Some(GasOptions::new(None, None)),
            private_key: None,
            msg_value: None,
        }
    }
}

impl<'a> BridgeAndCallArgsBuilder<'a> {
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

    /// Set the token address
    pub fn token_address(mut self, token_address: &'a str) -> Self {
        self.token_address = Some(token_address);
        self
    }

    /// Set the amount to bridge (in wei)
    pub fn amount(mut self, amount: &'a str) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the target contract address
    pub fn target(mut self, target: &'a str) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the call data (hex encoded)
    pub fn data(mut self, data: &'a str) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the fallback address
    pub fn fallback(mut self, fallback: &'a str) -> Self {
        self.fallback = Some(fallback);
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

    /// Set ETH value to send with the contract call (in wei)
    pub fn msg_value(mut self, msg_value: &'a str) -> Self {
        self.msg_value = Some(msg_value);
        self
    }

    /// Build the BridgeAndCallArgs with validation
    pub fn build(self) -> std::result::Result<BridgeAndCallArgs<'a>, &'static str> {
        let config = self.config.ok_or("Config is required")?;
        let source_network = self.source_network.ok_or("Source network is required")?;
        let destination_network = self
            .destination_network
            .ok_or("Destination network is required")?;
        let token_address = self.token_address.ok_or("Token address is required")?;
        let amount = self.amount.ok_or("Amount is required")?;
        let target = self.target.ok_or("Target address is required")?;
        let data = self.data.ok_or("Call data is required")?;
        let fallback = self.fallback.ok_or("Fallback address is required")?;
        let gas_options = self.gas_options.ok_or("Gas options are required")?;

        // Validate addresses
        if Address::from_str(token_address).is_err() {
            return Err("Invalid token address format");
        }
        if Address::from_str(target).is_err() {
            return Err("Invalid target address format");
        }
        if Address::from_str(fallback).is_err() {
            return Err("Invalid fallback address format");
        }

        // Validate amount format
        if U256::from_dec_str(amount).is_err() {
            return Err("Invalid amount format");
        }

        // Validate call data format
        if hex::decode(data.trim_start_matches("0x")).is_err() {
            return Err("Invalid call data hex format");
        }

        // Validate msg_value if provided
        if let Some(value) = &self.msg_value {
            if U256::from_dec_str(value).is_err() {
                return Err("Invalid msg_value format");
            }
        }

        Ok(BridgeAndCallArgs {
            config,
            source_network,
            destination_network,
            token_address,
            amount,
            target,
            data,
            fallback,
            gas_options,
            private_key: self.private_key,
            msg_value: self.msg_value,
        })
    }

    /// Build and convert to crate's Result type
    pub fn build_with_crate_error(self) -> Result<BridgeAndCallArgs<'a>> {
        self.build().map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(e))
        })
    }
}

/// Bridge with contract call (bridgeAndCall)
pub async fn bridge_message(
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

    let destination_network_id = destination_network as u32;

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
        destination_network_id,
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
        "✅ Bridge message transaction submitted: {:#x}",
        tx.tx_hash()
    );
    println!("💡 This creates a message bridge that will execute automatically when ready.");

    Ok(())
}

/// Get precalculated L2 token address
#[allow(dead_code)]
pub async fn get_precalculated_l2_token_address(
    config: &Config,
    destination_network: u64,
    token_address: &str,
    private_key: Option<&str>,
) -> Result<Address> {
    let client = get_wallet_with_provider(config, destination_network, private_key).await?;
    let bridge_address = super::get_bridge_contract_address(config, destination_network)?;
    let bridge = super::BridgeContract::new(bridge_address, Arc::new(client.clone()));

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
pub async fn bridge_and_call_with_approval(args: BridgeAndCallArgs<'_>) -> Result<()> {
    let client =
        get_wallet_with_provider(args.config, args.source_network, args.private_key).await?;
    let bridge_ext_address = get_bridge_extension_address(args.config, args.source_network)?;
    let bridge_ext = BridgeExtensionContract::new(bridge_ext_address, Arc::new(client.clone()));

    let destination_network_id = args.destination_network as u32;

    // Parse addresses and amounts
    let token_addr = Address::from_str(args.token_address).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid token address: {e}"),
        ))
    })?;

    let target_addr = Address::from_str(args.target).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid target address: {e}"),
        ))
    })?;

    let fallback_addr = Address::from_str(args.fallback).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid fallback address: {e}"),
        ))
    })?;

    let amount_wei = U256::from_dec_str(args.amount).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid amount: {e}"),
        ))
    })?;

    let call_data_bytes = hex::decode(args.data.trim_start_matches("0x")).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid call data hex: {e}"),
        ))
    })?;

    println!("🔧 Bridge and Call Debug:");
    println!("  - Token address: {}", args.token_address);
    println!("  - Amount: {} (Wei: {amount_wei})", args.amount);
    println!("  - Target: {}", args.target);
    println!("  - Fallback: {}", args.fallback);
    println!("  - Destination network ID: {destination_network_id}");
    println!("  - Bridge Extension: {bridge_ext_address:?}");
    let msg_value_wei = if let Some(msg_val) = args.msg_value {
        let msg_val_wei = U256::from_dec_str(msg_val).map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
                &format!("Invalid msg_value: {e}"),
            ))
        })?;
        println!("  - Msg Value: {msg_val} wei");

        // For ETH bridges, warn if msg_value != amount
        if super::is_eth_address(args.token_address) && msg_val_wei != amount_wei {
            println!(
                "⚠️  Warning: msg_value ({msg_val_wei}) differs from bridged amount ({amount_wei})"
            );
            println!("    The target contract may fail if it expects msg.value == bridged amount");
        }

        msg_val_wei
    } else {
        println!("  - Msg Value: Using bridged amount ({amount_wei} wei)");
        amount_wei
    };

    // Step 1: Check and approve bridge extension to spend tokens (skip for ETH)
    if !super::is_eth_address(args.token_address) {
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
                args.amount
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
    } else {
        println!("🔧 Skipping allowance check for ETH (native token)");
    }

    // Step 2: Execute bridgeAndCall
    println!("🔧 Executing bridgeAndCall...");

    let mut call = bridge_ext.bridge_and_call(
        token_addr,
        amount_wei,
        destination_network_id,
        target_addr,
        fallback_addr,
        call_data_bytes.into(),
        true, // forceUpdateGlobalExitRoot
    );

    // Add ETH value for native token transfers
    if super::is_eth_address(args.token_address) {
        call = call.value(msg_value_wei);
    }

    if args.gas_options.gas_limit.is_none() {
        call = call.gas(3_000_000u64); // Default high gas limit
    }

    let call = args.gas_options.apply_to_call_with_return(call);

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
    println!("   1. Asset bridge (leaf_type: 0) - bridges tokens to PolygonBridge");
    println!("   2. Message bridge (leaf_type: 1) - execute calldata from JumpPoint");
    println!();
    println!("💡 To complete the process, you need to claim both bridges:");
    println!(
        "   1. First check bridges: aggsandbox show bridges --network-id {}",
        args.source_network
    );
    println!("   2. Find entries with tx_hash: {:#x}", tx.tx_hash());
    println!("   3. Note the deposit_count for asset bridge (leaf_type: 0)");
    println!("   4. Note the deposit_count for message bridge (leaf_type: 1, has calldata)");
    println!("   5. Claim asset: aggsandbox bridge claim --network {} --tx-hash {:#x} --source-network {} --deposit-count <asset_deposit_count>", args.destination_network, tx.tx_hash(), args.source_network);
    println!("   6. Claim message: aggsandbox bridge claim --network {} --tx-hash {:#x} --source-network {} --deposit-count <message_deposit_count>", args.destination_network, tx.tx_hash(), args.source_network);

    Ok(())
}
