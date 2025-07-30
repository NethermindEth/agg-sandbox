use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

use super::{
    get_bridge_contract_address, get_wallet_with_provider, is_eth_address, network_id_to_chain_id,
    BridgeContract, ERC20Contract,
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

/// Bridge assets between networks
#[allow(clippy::disallowed_methods)] // Allow tracing macros
pub async fn bridge_asset(args: BridgeAssetArgs<'_>) -> Result<()> {
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
