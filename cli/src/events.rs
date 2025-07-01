use anyhow::{Context, Result};
use colored::*;
use ethers::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

// Known event signatures for common contracts
fn get_event_signatures() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // ERC20 Events
    m.insert(
        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
        "Transfer(address,address,uint256)",
    );
    m.insert(
        "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925",
        "Approval(address,address,uint256)",
    );
    m.insert(
        "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0",
        "OwnershipTransferred(address,address)",
    );

    // Bridge Events
    m.insert(
        "0x501781209a1f8899323b96b4ef08b168df93e0a90c673d1e4cce39366cb62f9b",
        "BridgeEvent(uint8,uint32,address,uint32,address,uint256,bytes,uint32)",
    );
    m.insert(
        "0x1df3f2a973a00d6635911755c260704e95e8a5876997546798770f76396fda4d",
        "ClaimEvent(uint256,uint32,address,address,uint256)",
    );

    // Initialization Events
    m.insert(
        "0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2",
        "Initialized(uint64)",
    );

    // Emergency State Events
    m.insert(
        "0x2261efe5aef6fedc1fd1550b25facc9181745623049c7901287030b9ad1a5497",
        "EmergencyStateActivated()",
    );
    m.insert(
        "0x1e5e34eea33501aecf2ebec9fe0e884a40804275ea7fe10b2ba084c8374308b3",
        "EmergencyStateDeactivated()",
    );

    // Access Control Events
    m.insert(
        "0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d",
        "RoleGranted(bytes32,address,address)",
    );
    m.insert(
        "0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b",
        "RoleRevoked(bytes32,address,address)",
    );
    m.insert(
        "0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff",
        "RoleAdminChanged(bytes32,bytes32,bytes32)",
    );

    // Rollup Manager Events
    m.insert(
        "0xd490df184152edba8455dac3228134939c71f8cb4c8f310c3145dec9037147ac",
        "AddExistingRollup(uint32,uint64,address,uint64,uint8,uint64,bytes32)",
    );
    m.insert(
        "0x9eaf2ecbddb14889c9e141a63175c55ac25e0cd7cdea312cdfbd0397976b383a",
        "AddNewRollupType(uint32,address,address,uint64,uint8,bytes32,string,bytes32)",
    );
    m.insert(
        "0x194c983456df6701c6a50830b90fe80e72b823411d0d524970c9590dc277a641",
        "CreateNewRollup(uint32,uint32,address,uint64,address)",
    );
    m.insert(
        "0x50cadc0c001f05dd4b81db1e92b98d77e718fd2f103fb7b77293e867d329a4c2",
        "UpdateRollupManagerVersion(string)",
    );

    // Timelock Events
    m.insert(
        "0x11c24f4ead16507c69ac467fbd5e4eed5fb5c699626d2cc6d66421df253886d5",
        "MinDelayChange(uint256,uint256)",
    );
    m.insert(
        "0x4cf4410cc57040e44862ef0f45f3dd5a5e02db8eb8add648d4b0e236f1d07dca",
        "CallScheduled(bytes32,uint256,address,uint256,bytes,bytes32,uint256)",
    );
    m.insert(
        "0xc2617efa69bab66782fa219543714338489c4e9e178271560a91b82c3f612b58",
        "CallExecuted(bytes32,uint256,address,uint256,bytes)",
    );
    m.insert(
        "0xbaa1eb22f2a492ba1a5fea61b8df4d27c6c8b5f3971e63bb58fa14ff72eedb70",
        "Cancelled(bytes32)",
    );

    // Bridge Management Events
    m.insert(
        "0x32cf74f8a6d5f88593984d2cd52be5592bfa6884f5896175801a5069ef09cd67",
        "SetBridgeManager(address)",
    );
    m.insert(
        "0x490e59a1701b938786ac72570a1efeac994a3dbe96e2e883e19e902ace6e6a39",
        "NewWrappedToken(uint32,address,address,bytes)",
    );
    m.insert(
        "0xdbe8a5da6a7a916d9adfda9160167a0f8a3da415ee6610e810e753853597fce7",
        "SetSovereignTokenAddress(uint32,address,address,bool)",
    );

    // Global Exit Root Events
    m.insert(
        "0xb1b866fe5fac68e8f1a4ab2520c7a6b493a954934bbd0f054bd91d6674a4c0d5",
        "InsertGlobalExitRoot(bytes32)",
    );
    m.insert(
        "0x605764d0b65b62ecf05dc90f674a00a2e2531fabaf120fdde65790e407fcb7a2",
        "RemoveLastGlobalExitRoot(bytes32)",
    );

    // L1 Info Tree Events
    m.insert(
        "0xda61aa7823fcd807e37b95aabcbe17f03a6f3efd514176444dae191d27fd66b3",
        "UpdateL1InfoTree(bytes32,bytes32)",
    );
    m.insert(
        "0xaf6c6cd7790e0180a4d22eb8ed846e55846f54ed10e5946db19972b5a0813a59",
        "UpdateL1InfoTreeV2(bytes32,uint32,uint256,uint64)",
    );

    // Global Exit Root Events
    m.insert(
        "0xb1b866fe5fac68e8f1a4ab2520c7a6b493a954934bbd0f054bd91d6674a4c0d5",
        "InsertGlobalExitRoot(bytes32)",
    );

    // Bridge Token Events
    m.insert(
        "0x490e59a1701b938786ac72570a1efeac994a3dbe96e2e883e19e902ace6e6a39",
        "NewWrappedToken(uint32,address,address,bytes)",
    );

    // Sequencer Events
    m.insert(
        "0xf54144f9611984021529f814a1cb6a41e22c58351510a0d9f7e822618abb9cc0",
        "SetTrustedSequencer(address)",
    );
    m.insert(
        "0x61f8fec29495a3078e9271456f05fb0707fd4e41f7661865f80fc437d06681ca",
        "SetTrustedAggregator(address)",
    );
    m.insert(
        "0x303446e6a8cb73c83dff421c0b1d5e5ce0719dab1bff13660fc254e58cc17fce",
        "SequenceBatches(uint64)",
    );
    m.insert(
        "0x648a61dd2438f072f5a1960939abd30f37aea80d2e94c9792ad142d3e0a490a4",
        "SequenceForceBatches(uint64)",
    );

    // Verification Events
    m.insert(
        "0x9c72852172521097ba7e1482e6b44b351323df0155f97f4ea18fcec28e1f5966",
        "VerifyBatches(uint64,bytes32,address)",
    );
    m.insert(
        "0xcb339b570a7f0b25afa7333371ff11192092a0aeace12b671f4c212f2815c6fe",
        "VerifyBatchesTrustedAggregator(uint64,bytes32,address)",
    );
    m.insert(
        "0xd1ec3a1216f08b6eff72e169ceb548b782db18a6614852618d86bb19f3f9b0d3",
        "VerifyBatchesTrustedAggregator(uint32,uint64,bytes32,bytes32,address)",
    );

    // Force Batch Events
    m.insert(
        "0xf94bb37db835f1ab585ee00041849a09b12cd081d77fa15ca070757619cbc931",
        "ForceBatch(uint64,bytes32,address,bytes)",
    );
    m.insert(
        "0xa7eb6cb8a613eb4e8bddc1ac3d61ec6cf10898760f0b187bcca794c6ca6fa40b",
        "SetForceBatchTimeout(uint64)",
    );

    // Admin Events
    m.insert(
        "0x056dc487bbf0795d0bbb1b4f0af523a855503cff740bfb4d5475f7a90c091e8e",
        "AcceptAdminRole(address)",
    );
    m.insert(
        "0xa5b56b7906fd0a20e3f35120dd8343db1e12e037a6c90111c7e42885e82a1ce6",
        "TransferAdminRole(address)",
    );

    m
}

pub async fn fetch_and_display_events(
    chain: &str,
    blocks: u64,
    address: Option<String>,
) -> Result<()> {
    let rpc_url = get_rpc_url(chain)?;

    println!(
        "{}",
        format!("🔍 Fetching events from {chain} chain")
            .cyan()
            .bold()
    );
    println!("{}", format!("📡 RPC URL: {rpc_url}").dimmed());
    println!("{}", format!("📊 Scanning last {blocks} blocks").dimmed());

    if let Some(addr) = &address {
        println!("{}", format!("🎯 Filtering by contract: {addr}").dimmed());
    }

    // Connect to the chain
    let provider =
        Provider::<Http>::try_from(&rpc_url).context("Failed to connect to RPC endpoint")?;

    let client = Arc::new(provider);

    // Get the latest block number
    let latest_block = client
        .get_block_number()
        .await
        .context("Failed to get latest block number")?;

    let from_block = if latest_block.as_u64() >= blocks {
        U64::from(latest_block.as_u64() - blocks + 1)
    } else {
        U64::zero()
    };

    println!(
        "{}",
        format!("🔍 Scanning blocks {from_block} to {latest_block}").green()
    );

    // Create filter for events
    let mut filter = Filter::new().from_block(from_block).to_block(latest_block);

    // Add address filter if provided
    if let Some(addr) = address {
        let address = addr
            .parse::<Address>()
            .context("Invalid contract address format")?;
        filter = filter.address(address);
    }

    // Fetch logs
    let logs = client
        .get_logs(&filter)
        .await
        .context("Failed to fetch logs from chain")?;

    if logs.is_empty() {
        println!("{}", "📭 No events found in the specified range".yellow());
        return Ok(());
    }

    println!(
        "{}",
        format!("📋 Found {} events", logs.len()).green().bold()
    );
    println!("{}", "═".repeat(80).dimmed());

    // Process and display each log
    for (index, log) in logs.iter().enumerate() {
        display_event(index + 1, log, &client).await?;

        if index < logs.len() - 1 {
            println!("{}", "─".repeat(80).dimmed());
        }
    }

    println!("{}", "═".repeat(80).dimmed());
    println!(
        "{}",
        format!("✅ Displayed {} events", logs.len()).green().bold()
    );

    Ok(())
}

async fn display_event(index: usize, log: &Log, client: &Arc<Provider<Http>>) -> Result<()> {
    println!("{}", format!("📝 Event #{index}").blue().bold());

    // Get block information
    if let Some(block_number) = log.block_number {
        if let Ok(Some(block)) = client.get_block(block_number).await {
            let timestamp = block.timestamp;
            if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp.as_u64() as i64, 0) {
                println!(
                    "⏰ Time: {}",
                    datetime
                        .format("%Y-%m-%d %H:%M:%S UTC")
                        .to_string()
                        .yellow()
                );
            }
        }
        println!("🧱 Block: {}", block_number.to_string().yellow());
    }

    if let Some(tx_hash) = log.transaction_hash {
        println!("📄 Transaction: {}", format!("0x{tx_hash:x}").yellow());
    }

    println!("📍 Contract: {}", format!("0x{:x}", log.address).yellow());

    // Decode the event
    if !log.topics.is_empty() {
        let event_signature = format!("0x{:x}", log.topics[0]);

        let event_signatures = get_event_signatures();
        if let Some(&event_name) = event_signatures.get(event_signature.as_str()) {
            println!("🎯 Event: {}", event_name.green().bold());
            decode_known_event(event_name, log)?;
        } else {
            println!("🎯 Event: {}", "Unknown Event".red());
            println!("📋 Raw Signature: {}", event_signature.dimmed());
        }
    }

    // Always show raw data for debugging
    println!("🔍 Raw Data:");
    println!("  Topics: {}", format!("{:?}", log.topics).dimmed());
    if !log.data.is_empty() {
        println!(
            "  Data: {}",
            format!("0x{}", hex::encode(&log.data)).dimmed()
        );
    }

    Ok(())
}

fn decode_known_event(event_name: &str, log: &Log) -> Result<()> {
    match event_name {
        "Transfer(address,address,uint256)" => decode_transfer_event(log),
        "Approval(address,address,uint256)" => decode_approval_event(log),
        "OwnershipTransferred(address,address)" => decode_ownership_transferred_event(log),
        "BridgeEvent(uint8,uint32,address,uint32,address,uint256,bytes,uint32)" => {
            decode_bridge_event(log)
        }
        "ClaimEvent(uint256,uint32,address,address,uint256)" => decode_claim_event(log),
        "Initialized(uint64)" => decode_initialized_event(log),
        "RoleGranted(bytes32,address,address)" => decode_role_granted_event(log),
        "RoleRevoked(bytes32,address,address)" => decode_role_revoked_event(log),
        "AddExistingRollup(uint32,uint64,address,uint64,uint8,uint64,bytes32)" => {
            decode_add_existing_rollup_event(log)
        }
        "UpdateRollupManagerVersion(string)" => decode_update_rollup_manager_version_event(log),
        "MinDelayChange(uint256,uint256)" => decode_min_delay_change_event(log),
        "SetTrustedSequencer(address)" => decode_set_trusted_sequencer_event(log),
        "SetTrustedAggregator(address)" => decode_set_trusted_aggregator_event(log),
        "SequenceBatches(uint64)" => decode_sequence_batches_event(log),
        "VerifyBatches(uint64,bytes32,address)" => decode_verify_batches_event(log),
        "UpdateL1InfoTree(bytes32,bytes32)" => decode_update_l1_info_tree_event(log),
        "UpdateL1InfoTreeV2(bytes32,uint32,uint256,uint64)" => {
            decode_update_l1_info_tree_v2_event(log)
        }
        "InsertGlobalExitRoot(bytes32)" => decode_insert_global_exit_root_event(log),
        "NewWrappedToken(uint32,address,address,bytes)" => decode_new_wrapped_token_event(log),
        _ => {
            println!("  ⚠️  Decoding not implemented for this event type");
            Ok(())
        }
    }
}

fn decode_transfer_event(log: &Log) -> Result<()> {
    if log.topics.len() >= 3 && log.data.len() >= 32 {
        let from = Address::from(log.topics[1]);
        let to = Address::from(log.topics[2]);
        let amount = U256::from_big_endian(&log.data[0..32]);

        println!("  📤 From: {}", format!("0x{from:x}").cyan());
        println!("  📥 To: {}", format!("0x{to:x}").cyan());
        println!("  💰 Amount: {} tokens", amount.to_string().green());
    }
    Ok(())
}

fn decode_approval_event(log: &Log) -> Result<()> {
    if log.topics.len() >= 3 && log.data.len() >= 32 {
        let owner = Address::from(log.topics[1]);
        let spender = Address::from(log.topics[2]);
        let amount = U256::from_big_endian(&log.data[0..32]);

        println!("  👤 Owner: {}", format!("0x{owner:x}").cyan());
        println!("  🤝 Spender: {}", format!("0x{spender:x}").cyan());
        println!("  💰 Allowance: {} tokens", amount.to_string().green());
    }
    Ok(())
}

fn decode_bridge_event(log: &Log) -> Result<()> {
    println!("  🌉 Bridge Event Details:");
    if !log.data.is_empty() && log.data.len() >= 256 {
        // Decode the bridge event parameters
        let leaf_type = U256::from(&log.data[0..32]);
        let origin_network = U256::from(&log.data[32..64]);
        let origin_address = format!("0x{}", hex::encode(&log.data[76..96]));
        let destination_network = U256::from(&log.data[96..128]);
        let destination_address = format!("0x{}", hex::encode(&log.data[140..160]));
        let amount = U256::from(&log.data[160..192]);

        println!("  🍃 Leaf Type: {}", leaf_type.to_string().dimmed());
        println!("  🌐 Origin Network: {}", origin_network.to_string().cyan());
        println!("  📍 Origin Address: {}", origin_address.yellow());
        println!(
            "  🎯 Destination Network: {}",
            destination_network.to_string().cyan()
        );
        println!("  📍 Destination Address: {}", destination_address.green());
        println!("  💰 Amount: {}", amount.to_string().green());
    } else {
        println!("  ⚠️  Complex bridge event - showing raw data");
        println!("  📊 Data length: {} bytes", log.data.len());
    }
    Ok(())
}

fn decode_ownership_transferred_event(log: &Log) -> Result<()> {
    println!("  👑 Ownership Transfer:");
    if log.topics.len() >= 3 {
        let previous_owner = format!("0x{}", hex::encode(&log.topics[1][12..]));
        let new_owner = format!("0x{}", hex::encode(&log.topics[2][12..]));
        println!("  📤 Previous Owner: {}", previous_owner.dimmed());
        println!("  📥 New Owner: {}", new_owner.green());
    }
    Ok(())
}

fn decode_claim_event(log: &Log) -> Result<()> {
    println!("  🎯 Claim Event:");
    if !log.data.is_empty() && log.data.len() >= 160 {
        // Decode ClaimEvent(uint256,uint32,address,address,uint256)
        let global_index = U256::from(&log.data[0..32]);
        let origin_network = U256::from(&log.data[32..64]);
        let origin_address = format!("0x{}", hex::encode(&log.data[76..96]));
        let destination_address = format!("0x{}", hex::encode(&log.data[108..128]));
        let amount = U256::from(&log.data[128..160]);

        println!("  🌍 Global Index: {}", global_index.to_string().cyan());
        println!("  🌐 Origin Network: {}", origin_network.to_string().cyan());
        println!("  📍 Origin Address: {}", origin_address.yellow());
        println!("  📍 Destination Address: {}", destination_address.green());
        println!("  💰 Amount: {}", amount.to_string().green());
    } else {
        println!("  ⚠️  Complex claim event - showing raw data");
    }
    Ok(())
}

fn decode_initialized_event(log: &Log) -> Result<()> {
    println!("  🚀 Contract Initialized:");
    if !log.data.is_empty() && log.data.len() >= 32 {
        let version = U256::from(&log.data[0..32]);
        println!("  📊 Version: {}", version.to_string().cyan());
    }
    Ok(())
}

fn decode_role_granted_event(log: &Log) -> Result<()> {
    println!("  🔐 Role Granted:");
    if log.topics.len() >= 4 {
        let role = format!("0x{}", hex::encode(log.topics[1]));
        let account = format!("0x{}", hex::encode(&log.topics[2][12..]));
        let sender = format!("0x{}", hex::encode(&log.topics[3][12..]));
        println!("  🎭 Role: {}", role.yellow());
        println!("  👤 Account: {}", account.green());
        println!("  📤 Granted by: {}", sender.dimmed());
    }
    Ok(())
}

fn decode_role_revoked_event(log: &Log) -> Result<()> {
    println!("  🚫 Role Revoked:");
    if log.topics.len() >= 4 {
        let role = format!("0x{}", hex::encode(log.topics[1]));
        let account = format!("0x{}", hex::encode(&log.topics[2][12..]));
        let sender = format!("0x{}", hex::encode(&log.topics[3][12..]));
        println!("  🎭 Role: {}", role.yellow());
        println!("  👤 Account: {}", account.red());
        println!("  📤 Revoked by: {}", sender.dimmed());
    }
    Ok(())
}

fn decode_add_existing_rollup_event(log: &Log) -> Result<()> {
    println!("  🔄 Add Existing Rollup:");
    if log.topics.len() >= 2 {
        let rollup_id = U256::from(log.topics[1].as_bytes());
        println!("  🆔 Rollup ID: {}", rollup_id.to_string().cyan());
    }
    if !log.data.is_empty() {
        println!("  ⚠️  Complex rollup data - showing summary only");
    }
    Ok(())
}

fn decode_update_rollup_manager_version_event(log: &Log) -> Result<()> {
    println!("  📦 Rollup Manager Version Update:");
    if !log.data.is_empty() && log.data.len() >= 64 {
        // Try to decode string from data
        println!("  📊 New version updated");
    }
    Ok(())
}

fn decode_min_delay_change_event(log: &Log) -> Result<()> {
    println!("  ⏱️  Min Delay Change:");
    if !log.data.is_empty() && log.data.len() >= 64 {
        let old_delay = U256::from(&log.data[0..32]);
        let new_delay = U256::from(&log.data[32..64]);
        println!("  📤 Old Delay: {} seconds", old_delay.to_string().dimmed());
        println!("  📥 New Delay: {} seconds", new_delay.to_string().green());
    }
    Ok(())
}

fn decode_set_trusted_sequencer_event(log: &Log) -> Result<()> {
    println!("  🔗 Set Trusted Sequencer:");
    if log.topics.len() >= 2 {
        let sequencer = format!("0x{}", hex::encode(&log.topics[1][12..]));
        println!("  👤 New Sequencer: {}", sequencer.green());
    }
    Ok(())
}

fn decode_set_trusted_aggregator_event(log: &Log) -> Result<()> {
    println!("  🔗 Set Trusted Aggregator:");
    if log.topics.len() >= 2 {
        let aggregator = format!("0x{}", hex::encode(&log.topics[1][12..]));
        println!("  👤 New Aggregator: {}", aggregator.green());
    }
    Ok(())
}

fn decode_sequence_batches_event(log: &Log) -> Result<()> {
    println!("  📦 Sequence Batches:");
    if log.topics.len() >= 2 {
        let batch_num = U256::from(log.topics[1].as_bytes());
        println!("  🔢 Batch Number: {}", batch_num.to_string().cyan());
    }
    Ok(())
}

fn decode_verify_batches_event(log: &Log) -> Result<()> {
    println!("  ✅ Verify Batches:");
    if log.topics.len() >= 2 {
        let batch_num = U256::from(log.topics[1].as_bytes());
        println!("  🔢 Batch Number: {}", batch_num.to_string().cyan());
    }
    if log.topics.len() >= 4 {
        let aggregator = format!("0x{}", hex::encode(&log.topics[3][12..]));
        println!("  👤 Aggregator: {}", aggregator.green());
    }
    Ok(())
}

fn decode_update_l1_info_tree_event(log: &Log) -> Result<()> {
    println!("  🌳 Update L1 Info Tree:");
    if log.topics.len() >= 3 {
        let main_exit_root = format!("0x{}", hex::encode(log.topics[1]));
        let rollup_exit_root = format!("0x{}", hex::encode(log.topics[2]));
        println!("  🔗 Main Exit Root: {}", main_exit_root.cyan());
        println!("  🔗 Rollup Exit Root: {}", rollup_exit_root.yellow());
    }
    Ok(())
}

fn decode_update_l1_info_tree_v2_event(log: &Log) -> Result<()> {
    println!("  🌳 Update L1 Info Tree V2:");
    if log.topics.len() >= 2 {
        let current_l1_info_root = format!("0x{}", hex::encode(log.topics[1]));
        println!("  🔗 Current L1 Info Root: {}", current_l1_info_root.cyan());
    }
    if !log.data.is_empty() && log.data.len() >= 96 {
        let leaf_count = U256::from(&log.data[0..32]);
        let block_hash = format!("0x{}", hex::encode(&log.data[32..64]));
        let timestamp = U256::from(&log.data[64..96]);
        println!("  📊 Leaf Count: {}", leaf_count.to_string().green());
        println!("  🧱 Block Hash: {}", block_hash.yellow());
        println!("  ⏰ Timestamp: {}", timestamp.to_string().dimmed());
    }
    Ok(())
}

fn decode_insert_global_exit_root_event(log: &Log) -> Result<()> {
    println!("  🌍 Insert Global Exit Root:");
    if log.topics.len() >= 2 {
        let global_exit_root = format!("0x{}", hex::encode(log.topics[1]));
        println!("  🔗 Global Exit Root: {}", global_exit_root.cyan());
    }
    Ok(())
}

fn decode_new_wrapped_token_event(log: &Log) -> Result<()> {
    println!("  🪙 New Wrapped Token:");
    if !log.data.is_empty() && log.data.len() >= 128 {
        let origin_network = U256::from(&log.data[0..32]);
        let origin_token_address = format!("0x{}", hex::encode(&log.data[44..64]));
        let wrapped_token_address = format!("0x{}", hex::encode(&log.data[76..96]));

        println!("  🌐 Origin Network: {}", origin_network.to_string().cyan());
        println!("  📍 Origin Token: {}", origin_token_address.yellow());
        println!("  🎁 Wrapped Token: {}", wrapped_token_address.green());

        // Try to decode metadata if present
        if log.data.len() > 128 {
            // Skip the first 128 bytes (4 * 32) and try to find the metadata offset
            if let Ok(metadata_offset) = std::str::from_utf8(&log.data[128..]) {
                if !metadata_offset.trim().is_empty() {
                    println!("  📋 Metadata: Available");
                }
            }
        }
    }
    Ok(())
}

fn get_rpc_url(chain: &str) -> Result<String> {
    let rpc_url = match chain {
        "anvil-l1" => {
            std::env::var("RPC_1").unwrap_or_else(|_| "http://localhost:8545".to_string())
        }
        "anvil-l2" => {
            std::env::var("RPC_2").unwrap_or_else(|_| "http://localhost:8546".to_string())
        }
        "anvil-l3" => {
            std::env::var("RPC_3").unwrap_or_else(|_| "http://localhost:8547".to_string())
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid chain '{}'. Supported chains: anvil-l1, anvil-l2, anvil-l3",
                chain
            ));
        }
    };

    Ok(rpc_url)
}
