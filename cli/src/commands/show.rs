use crate::api;
use crate::config::Config;
use crate::error::Result;

/// Bridge and blockchain data subcommands
#[derive(Debug, clap::Subcommand)]
pub enum ShowCommands {
    /// 🌉 Show bridge information for a specific network
    #[command(
        long_about = "Display bridge information for the specified network.\\n\\nBridges enable cross-chain transfers between L1 and L2 networks.\\nThis command shows active bridges, their configurations, and status.\\n\\nNetwork IDs:\\n  • 0 = Ethereum L1\\n  • 1 = First L2 connected to Agglayer\\n  • 2 = Second L2 (if multi-L2 enabled)\\n\\nExamples:\\n  aggsandbox show bridges                    # Show L1 bridges\\n  aggsandbox show bridges --network-id 1    # Show first L2 bridges\\n  aggsandbox show bridges --json             # Raw JSON output for scripting"
    )]
    Bridges {
        /// Network ID to query (0=L1, 1=first L2, etc.)
        #[arg(
            short,
            long,
            default_value = "0",
            help = "Network ID (0=L1 Ethereum, 1=first L2, 2=second L2, etc.)"
        )]
        network_id: u64,
        /// Output raw JSON without formatting (for scripting)
        #[arg(long, help = "Output raw JSON without decorative formatting")]
        json: bool,
    },
    /// 📋 Show pending claims for a network
    #[command(
        long_about = "Display pending claims that can be executed on the specified network.\\n\\nClaims represent cross-chain transfers waiting to be processed.\\nEach claim contains transfer details and required proof data.\\n\\nTypically:\\n  • L1 claims (network 0): Deposits to be claimed on L2\\n  • L2 claims (network 1): Withdrawals to be claimed on L1\\n\\nExamples:\\n  aggsandbox show claims                     # Show L1 claims\\n  aggsandbox show claims --network-id 1     # Show first L2 claims\\n  aggsandbox show claims --json              # Raw JSON output for scripting"
    )]
    Claims {
        /// Network ID to query for pending claims
        #[arg(
            short,
            long,
            default_value = "1",
            help = "Network ID to query for pending claims"
        )]
        network_id: u64,
        /// Output raw JSON without formatting (for scripting)
        #[arg(long, help = "Output raw JSON without decorative formatting")]
        json: bool,
    },
    /// 🔐 Generate and show claim proof for a specific transaction
    #[command(
        long_about = "Generate a cryptographic proof required to claim a cross-chain transfer.\\n\\nClaim proofs are Merkle proofs that verify a deposit exists in the\\nglobal exit tree, enabling secure cross-chain claims.\\n\\nParameters:\\n  • network_id: The target network for claiming\\n  • leaf_index: Position in the global exit tree\\n  • deposit_count: Number of deposits when the exit was created\\n\\nExamples:\\n  aggsandbox show claim-proof --network-id 0 --leaf-index 0 --deposit-count 1\\n  aggsandbox show claim-proof -n 1 -l 5 -d 10\\n  aggsandbox show claim-proof --json         # Raw JSON output for scripting"
    )]
    ClaimProof {
        /// Target network ID for the claim
        #[arg(
            short,
            long,
            default_value = "0",
            help = "Target network ID for claiming"
        )]
        network_id: u64,
        /// Leaf index in the global exit tree
        #[arg(
            short,
            long,
            default_value = "0",
            help = "Leaf index in the global exit tree"
        )]
        leaf_index: u64,
        /// Deposit count at the time of exit creation
        #[arg(
            short,
            long,
            default_value = "1",
            help = "Number of deposits when exit was created"
        )]
        deposit_count: u64,
        /// Output raw JSON without formatting (for scripting)
        #[arg(long, help = "Output raw JSON without decorative formatting")]
        json: bool,
    },
    /// 🌳 Show L1 info tree index for deposit verification
    #[command(
        long_about = "Retrieve the L1 information tree index for a specific deposit count.\\n\\nThe L1 info tree contains snapshots of L1 state that are used\\nby L2 for deposit verification and cross-chain communication.\\n\\nThis is primarily used for:\\n  • Verifying L1 state on L2\\n  • Resolving deposit transactions\\n  • Cross-chain message verification\\n\\nExamples:\\n  aggsandbox show l1-info-tree-index --network-id 0 --deposit-count 0\\n  aggsandbox show l1-info-tree-index -n 1 -d 5\\n  aggsandbox show l1-info-tree-index --json  # Raw JSON output for scripting"
    )]
    L1InfoTreeIndex {
        /// Network ID to query
        #[arg(short, long, default_value = "0", help = "Network ID to query")]
        network_id: u64,
        /// Deposit count to get L1 info tree index for
        #[arg(
            short,
            long,
            default_value = "0",
            help = "Deposit count to lookup in L1 info tree"
        )]
        deposit_count: u64,
        /// Output raw JSON without formatting (for scripting)
        #[arg(long, help = "Output raw JSON without decorative formatting")]
        json: bool,
    },
}

/// Handle the show command
pub async fn handle_show(subcommand: ShowCommands) -> Result<()> {
    let config = Config::load()?;

    match subcommand {
        ShowCommands::Bridges { network_id, json } => {
            let response = api::get_bridges(&config, network_id, json).await?;
            if json {
                api::print_raw_json(&response.data);
            } else {
                api::print_json_response("Bridge Information", &response.data);
            }
        }
        ShowCommands::Claims { network_id, json } => {
            let response = api::get_claims(&config, network_id, json).await?;
            let filtered_data = filter_duplicate_claims(&response.data);
            if json {
                api::print_raw_json(&filtered_data);
            } else {
                api::print_json_response("Claims Information", &filtered_data);
            }
        }
        ShowCommands::ClaimProof {
            network_id,
            leaf_index,
            deposit_count,
            json,
        } => {
            let response =
                api::get_claim_proof(&config, network_id, leaf_index, deposit_count, json).await?;
            if json {
                api::print_raw_json(&response.data);
            } else {
                api::print_json_response("Claim Proof Information", &response.data);
            }
        }
        ShowCommands::L1InfoTreeIndex {
            network_id,
            deposit_count,
            json,
        } => {
            let response =
                api::get_l1_info_tree_index(&config, network_id, deposit_count, json).await?;
            if json {
                api::print_raw_json(&response.data);
            } else {
                api::print_json_response("L1 Info Tree Index", &response.data);
            }
        }
    }
    Ok(())
}

/// Filter out duplicate pending claims that have already been completed
///
/// Groups claims by bridge_tx_hash AND type, removing "pending" claims if there are
/// corresponding "completed" claims for the same bridge transaction and type.
/// This handles bridge-and-call operations where the same bridge_tx_hash can have
/// both "asset" and "message" type claims that need separate processing.
fn filter_duplicate_claims(data: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    use std::collections::HashMap;

    let mut result = data.clone();

    // Extract claims array if it exists
    if let Some(claims_array) = data.get("claims").and_then(|v| v.as_array()) {
        // Group claims by (bridge_tx_hash, type) composite key
        let mut grouped_claims: HashMap<(String, String), Vec<&Value>> = HashMap::new();

        for claim in claims_array {
            if let (Some(bridge_tx_hash), Some(claim_type)) = (
                claim.get("bridge_tx_hash").and_then(|v| v.as_str()),
                claim.get("type").and_then(|v| v.as_str()),
            ) {
                let composite_key = (bridge_tx_hash.to_string(), claim_type.to_string());
                grouped_claims.entry(composite_key).or_default().push(claim);
            }
        }

        // Filter claims: remove pending if completed exists for same (bridge_tx_hash, type)
        let mut filtered_claims = Vec::new();

        for (_composite_key, mut claims_group) in grouped_claims {
            // Check if we have both pending and completed claims for this specific type
            let has_completed = claims_group
                .iter()
                .any(|claim| claim.get("status").and_then(|s| s.as_str()) == Some("completed"));

            if has_completed {
                // Keep only completed claims for this (bridge_tx_hash, type) combination
                claims_group.retain(|claim| {
                    claim.get("status").and_then(|s| s.as_str()) == Some("completed")
                });
            }
            // If no completed claims exist, keep all (including pending ones)

            // Add filtered claims to result
            for claim in claims_group {
                filtered_claims.push(claim.clone());
            }
        }

        // Update the result with filtered claims
        if let Some(result_obj) = result.as_object_mut() {
            let filtered_count = filtered_claims.len();
            result_obj.insert("claims".to_string(), Value::Array(filtered_claims));

            // Update count to reflect the filtered number
            result_obj.insert(
                "count".to_string(),
                Value::Number(serde_json::Number::from(filtered_count)),
            );
        }
    }

    result
}
