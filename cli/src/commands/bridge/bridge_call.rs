use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

use super::{
    get_bridge_extension_address, get_wallet_with_provider, network_id_to_chain_id,
    BridgeExtensionContract, ERC20Contract, GasOptions,
};

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
pub async fn bridge_and_call_with_approval(
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
