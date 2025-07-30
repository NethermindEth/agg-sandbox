use crate::api_client::OptimizedApiClient;
use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use std::str::FromStr;
use std::sync::Arc;

use super::{
    get_bridge_contract_address, get_bridge_extension_address, get_wallet_with_provider,
    network_id_to_chain_id, BridgeContract, ERC20Contract, GasOptions,
};

/// Claim bridged assets on destination network
pub async fn claim_asset(
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

        super::claim_message::execute_claim_message(
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
pub async fn execute_claim_asset(
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
