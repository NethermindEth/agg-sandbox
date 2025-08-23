use crate::error::Result;
use ethers::prelude::*;
use ethers::providers::Http;
use ethers::signers::LocalWallet;
use std::sync::Arc;

use super::{BridgeContract, GasOptions};

/// Type alias for the bridge contract with middleware
pub type BridgeContractWithMiddleware<'a> =
    &'a BridgeContract<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>;

/// Arguments for executing claim message operations
///
/// Use the builder pattern to construct this struct:
/// ```rust
/// let args = ClaimMessageArgs::builder()
///     .bridge(&bridge)
///     .deposit_count(1)
///     .mainnet_root(mainnet_root)
///     .rollup_root(rollup_root)
///     .origin_network(1)
///     .origin_addr(origin_addr)
///     .destination_network_id(0)
///     .dest_addr(dest_addr)
///     .amount_wei(amount_wei)
///     .metadata_bytes(metadata_bytes)
///     .gas_options(gas_options)
///     .build_with_crate_error()?;
/// ```
pub struct ClaimMessageArgs<'a> {
    pub bridge: BridgeContractWithMiddleware<'a>,
    pub deposit_count: u64,
    pub mainnet_root: H256,
    pub rollup_root: H256,
    pub origin_network: u32,
    pub origin_addr: Address,
    pub destination_network_id: u32,
    pub dest_addr: Address,
    pub amount_wei: U256,
    pub metadata_bytes: Vec<u8>,
    pub gas_options: &'a GasOptions,
    pub msg_value: Option<U256>,
}

impl<'a> ClaimMessageArgs<'a> {
    /// Create a new builder instance
    pub fn builder() -> ClaimMessageArgsBuilder<'a> {
        ClaimMessageArgsBuilder::default()
    }

    /// Create a builder with common defaults for message claiming
    #[allow(dead_code)]
    pub fn message_claim_builder(
        bridge: BridgeContractWithMiddleware<'a>,
        params: MessageClaimParams,
        gas_options: &'a GasOptions,
    ) -> ClaimMessageArgsBuilder<'a> {
        ClaimMessageArgsBuilder::default()
            .bridge(bridge)
            .deposit_count(params.deposit_count)
            .mainnet_root(params.mainnet_root)
            .rollup_root(params.rollup_root)
            .origin_network(params.origin_network)
            .origin_addr(params.origin_addr)
            .destination_network_id(params.destination_network_id)
            .dest_addr(params.dest_addr)
            .amount_wei(params.amount_wei)
            .metadata_bytes(params.metadata_bytes)
            .gas_options(gas_options)
    }
}

/// Parameters for message claiming operations
#[derive(Debug, Clone)]
pub struct MessageClaimParams {
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

/// Builder for constructing ClaimMessageArgs with validation
///
/// The builder pattern provides several benefits:
/// - Type-safe construction with compile-time validation
/// - Fluent API for easy chaining of method calls
/// - Clear separation between required and optional fields
/// - Built-in validation during the build process
/// - Prevents construction of invalid configurations
#[derive(Default)]
pub struct ClaimMessageArgsBuilder<'a> {
    bridge: Option<BridgeContractWithMiddleware<'a>>,
    deposit_count: Option<u64>,
    mainnet_root: Option<H256>,
    rollup_root: Option<H256>,
    origin_network: Option<u32>,
    origin_addr: Option<Address>,
    destination_network_id: Option<u32>,
    dest_addr: Option<Address>,
    amount_wei: Option<U256>,
    metadata_bytes: Option<Vec<u8>>,
    gas_options: Option<&'a GasOptions>,
    msg_value: Option<U256>,
}

impl<'a> ClaimMessageArgsBuilder<'a> {
    /// Set the bridge contract
    pub fn bridge(mut self, bridge: BridgeContractWithMiddleware<'a>) -> Self {
        self.bridge = Some(bridge);
        self
    }

    /// Set the deposit count
    pub fn deposit_count(mut self, deposit_count: u64) -> Self {
        self.deposit_count = Some(deposit_count);
        self
    }

    /// Set the mainnet exit root
    pub fn mainnet_root(mut self, mainnet_root: H256) -> Self {
        self.mainnet_root = Some(mainnet_root);
        self
    }

    /// Set the rollup exit root
    pub fn rollup_root(mut self, rollup_root: H256) -> Self {
        self.rollup_root = Some(rollup_root);
        self
    }

    /// Set the origin network ID
    pub fn origin_network(mut self, origin_network: u32) -> Self {
        self.origin_network = Some(origin_network);
        self
    }

    /// Set the origin address
    pub fn origin_addr(mut self, origin_addr: Address) -> Self {
        self.origin_addr = Some(origin_addr);
        self
    }

    /// Set the destination network ID
    pub fn destination_network_id(mut self, destination_network_id: u32) -> Self {
        self.destination_network_id = Some(destination_network_id);
        self
    }

    /// Set the destination address
    pub fn dest_addr(mut self, dest_addr: Address) -> Self {
        self.dest_addr = Some(dest_addr);
        self
    }

    /// Set the amount in wei
    pub fn amount_wei(mut self, amount_wei: U256) -> Self {
        self.amount_wei = Some(amount_wei);
        self
    }

    /// Set the metadata bytes
    pub fn metadata_bytes(mut self, metadata_bytes: Vec<u8>) -> Self {
        self.metadata_bytes = Some(metadata_bytes);
        self
    }

    /// Set gas options
    pub fn gas_options(mut self, gas_options: &'a GasOptions) -> Self {
        self.gas_options = Some(gas_options);
        self
    }

    /// Set ETH value to send with the claim message transaction
    pub fn msg_value(mut self, msg_value: Option<U256>) -> Self {
        self.msg_value = msg_value;
        self
    }

    /// Build the ClaimMessageArgs with validation
    pub fn build(self) -> std::result::Result<ClaimMessageArgs<'a>, &'static str> {
        let bridge = self.bridge.ok_or("Bridge contract is required")?;
        let deposit_count = self.deposit_count.ok_or("Deposit count is required")?;
        let mainnet_root = self.mainnet_root.ok_or("Mainnet root is required")?;
        let rollup_root = self.rollup_root.ok_or("Rollup root is required")?;
        let origin_network = self.origin_network.ok_or("Origin network is required")?;
        let origin_addr = self.origin_addr.ok_or("Origin address is required")?;
        let destination_network_id = self
            .destination_network_id
            .ok_or("Destination network ID is required")?;
        let dest_addr = self.dest_addr.ok_or("Destination address is required")?;
        let amount_wei = self.amount_wei.ok_or("Amount in wei is required")?;
        let metadata_bytes = self.metadata_bytes.ok_or("Metadata bytes are required")?;
        let gas_options = self.gas_options.ok_or("Gas options are required")?;

        // Validate addresses are not zero
        if origin_addr == Address::zero() {
            return Err("Origin address cannot be zero");
        }
        if dest_addr == Address::zero() {
            return Err("Destination address cannot be zero");
        }

        // Note: Message bridges can have zero ETH amount - this is valid for pure message calls

        // Validate deposit count is reasonable
        if deposit_count > 1_000_000 {
            return Err("Deposit count seems unreasonably high");
        }

        Ok(ClaimMessageArgs {
            bridge,
            deposit_count,
            mainnet_root,
            rollup_root,
            origin_network,
            origin_addr,
            destination_network_id,
            dest_addr,
            amount_wei,
            metadata_bytes,
            gas_options,
            msg_value: self.msg_value,
        })
    }

    /// Build and convert to crate's Result type
    pub fn build_with_crate_error(self) -> Result<ClaimMessageArgs<'a>> {
        self.build().map_err(|e| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(e))
        })
    }
}

/// Execute claimMessage contract call
pub async fn execute_claim_message(args: ClaimMessageArgs<'_>) -> Result<H256> {
    let mut call = args.bridge.claim_message(
        args.deposit_count.into(), // globalIndex
        args.mainnet_root.into(),  // mainnetExitRoot
        args.rollup_root.into(),   // rollupExitRoot
        args.origin_network,
        args.origin_addr, // originAddress for message
        args.destination_network_id,
        args.dest_addr,
        args.amount_wei,
        ethers::types::Bytes::from(args.metadata_bytes), // message data
    );

    // Add ETH value if specified for message bridge claims
    if let Some(value) = args.msg_value {
        call = call.value(value);
    }

    if args.gas_options.gas_limit.is_none() {
        call = call.gas(3_000_000u64); // Default high gas limit for claims
    }

    let call = args.gas_options.apply_to_call_with_return(call);
    let tx = call.send().await.map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Failed to send claim message transaction: {e}"),
        ))
    })?;

    println!("💡 For future claims, ensure you wait at least 5 seconds after bridging to allow AggKit to update the Global Exit Root (GER)");
    Ok(tx.tx_hash())
}
