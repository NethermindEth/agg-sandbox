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

    // Bridge Events
    m.insert(
        "0x501781209a1f8899323b96b4ef08b168df93e0a90c673d1e4cce39366cb62f9b",
        "BridgeEvent(uint8,uint32,address,uint32,address,uint256,bytes,uint32)",
    );
    m.insert(
        "0x25308c93ceeed775b33ab0a0e25cd929e1c5f1b82b6e7c5b9e8b4b4b6e7f8e9",
        "ClaimEvent(bytes32,uint32,address,address,uint256)",
    );

    // Deposit/Withdrawal Events
    m.insert(
        "0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c",
        "Deposit(address,uint256)",
    );
    m.insert(
        "0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65",
        "Withdrawal(address,uint256)",
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
        format!("🔍 Fetching events from {} chain", chain)
            .cyan()
            .bold()
    );
    println!("{}", format!("📡 RPC URL: {}", rpc_url).dimmed());
    println!("{}", format!("📊 Scanning last {} blocks", blocks).dimmed());

    if let Some(addr) = &address {
        println!("{}", format!("🎯 Filtering by contract: {}", addr).dimmed());
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
        format!("🔍 Scanning blocks {} to {}", from_block, latest_block).green()
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
    println!("{}", format!("📝 Event #{}", index).blue().bold());

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
        println!("📄 Transaction: {}", format!("0x{:x}", tx_hash).yellow());
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
        "BridgeEvent(uint8,uint32,address,uint32,address,uint256,bytes,uint32)" => {
            decode_bridge_event(log)
        }
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

        println!("  📤 From: {}", format!("0x{:x}", from).cyan());
        println!("  📥 To: {}", format!("0x{:x}", to).cyan());
        println!("  💰 Amount: {} tokens", amount.to_string().green());

        // Convert to human readable if it's a reasonable number
        if amount < U256::from(1_000_000_000_000_000_000_000_000u128) {
            let eth_amount = amount.as_u128() as f64 / 1e18;
            if eth_amount > 0.0 {
                println!(
                    "  💰 Amount (18 decimals): {:.6} tokens",
                    eth_amount.to_string().green()
                );
            }
        }
    }
    Ok(())
}

fn decode_approval_event(log: &Log) -> Result<()> {
    if log.topics.len() >= 3 && log.data.len() >= 32 {
        let owner = Address::from(log.topics[1]);
        let spender = Address::from(log.topics[2]);
        let amount = U256::from_big_endian(&log.data[0..32]);

        println!("  👤 Owner: {}", format!("0x{:x}", owner).cyan());
        println!("  🤝 Spender: {}", format!("0x{:x}", spender).cyan());
        println!("  💰 Allowance: {} tokens", amount.to_string().green());

        // Convert to human readable if it's a reasonable number
        if amount < U256::from(1_000_000_000_000_000_000_000_000u128) {
            let eth_amount = amount.as_u128() as f64 / 1e18;
            if eth_amount > 0.0 {
                println!(
                    "  💰 Allowance (18 decimals): {:.6} tokens",
                    eth_amount.to_string().green()
                );
            }
        }
    }
    Ok(())
}

fn decode_bridge_event(log: &Log) -> Result<()> {
    println!("  🌉 Bridge Event Details:");
    if log.topics.len() >= 1 && !log.data.is_empty() {
        // This is a complex event, just show that it's a bridge event for now
        println!("  ⚠️  Complex bridge event - showing raw data");
        println!("  📊 Data length: {} bytes", log.data.len());
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
