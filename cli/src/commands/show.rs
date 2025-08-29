use crate::api;
use crate::config::Config;
use crate::error::Result;

/// Bridge and blockchain data subcommands
#[derive(Debug, clap::Subcommand)]
pub enum ShowCommands {
    /// 🌉 Show bridge information for a specific network
    #[command(
        long_about = "Display bridge information for the specified network.\\n\\nBridges enable cross-chain transfers between L1 and L2 networks.\\nThis command shows active bridges, their configurations, and status.\\n\\nNetwork IDs:\\n  • 0 = Ethereum L1\\n  • 1 = First L2 connected to Agglayer\\n  • 2 = Second L2 (if multi-L2 enabled)\\n\\nExamples:\\n  `aggsandbox show bridges`                    # Show L1 bridges\\n  `aggsandbox show bridges --network-id 1`    # Show first L2 bridges\\n  `aggsandbox show bridges --json`             # Raw JSON output for scripting"
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
        long_about = "Display claims that can be executed on the specified network.\\n\\nClaims represent cross-chain transfers waiting to be processed.\\nEach claim contains transfer details and required proof data.\\n\\nTypically:\\n  • L1 claims (network 0): Deposits to be claimed on L2\\n  • L2 claims (network 1): Withdrawals to be claimed on L1\\n\\nFiltering:\\n  Use filters to narrow down results when many claims are present.\\n\\nExamples:\\n  `aggsandbox show claims`                                    # Show all L2 claims\\n  `aggsandbox show claims --network-id 0`                     # Show L1 claims\\n  `aggsandbox show claims --bridge-tx-hash 0x123...`          # Filter by bridge transaction\\n  `aggsandbox show claims --claim-tx-hash 0xabc...`           # Filter by claim transaction\\n  `aggsandbox show claims --status pending`                   # Show only pending claims\\n  `aggsandbox show claims --claim-type asset`                 # Show only asset claims\\n  `aggsandbox show claims --address 0xdef...`                 # Filter by destination address\\n  `aggsandbox show claims --json`                             # Raw JSON output for scripting"
    )]
    Claims {
        /// Network ID to query for claims
        #[arg(
            short,
            long,
            default_value = "1",
            help = "Network ID to query for claims"
        )]
        network_id: u64,
        /// Filter by bridge transaction hash
        #[arg(long, help = "Filter claims by bridge transaction hash")]
        bridge_tx_hash: Option<String>,
        /// Filter by claim transaction hash
        #[arg(
            long,
            help = "Filter claims by claim transaction hash (empty for pending claims)"
        )]
        claim_tx_hash: Option<String>,
        /// Filter by claim status (pending, completed)
        #[arg(long, help = "Filter claims by status (pending, completed)")]
        status: Option<String>,
        /// Filter by claim type (asset, message)
        #[arg(long, help = "Filter claims by type (asset, message)")]
        claim_type: Option<String>,
        /// Filter by destination address
        #[arg(long, help = "Filter claims by destination address")]
        address: Option<String>,
        /// Output raw JSON without formatting (for scripting)
        #[arg(long, help = "Output raw JSON without decorative formatting")]
        json: bool,
    },
    /// 🔐 Generate and show claim proof for a specific transaction
    #[command(
        long_about = "Generate a cryptographic proof required to claim a cross-chain transfer.\\n\\nClaim proofs are Merkle proofs that verify a deposit exists in the\\nglobal exit tree, enabling secure cross-chain claims.\\n\\nParameters:\\n  • network_id: The target network for claiming\\n  • leaf_index: Position in the global exit tree\\n  • deposit_count: Number of deposits when the exit was created\\n\\nExamples:\\n  `aggsandbox show claim-proof --network-id 0 --leaf-index 0 --deposit-count 1`\\n  `aggsandbox show claim-proof -n 1 -l 5 -d 10`\\n  `aggsandbox show claim-proof --json`         # Raw JSON output for scripting"
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
        long_about = "Retrieve the L1 information tree index for a specific deposit count.\\n\\nThe L1 info tree contains snapshots of L1 state that are used\\nby L2 for deposit verification and cross-chain communication.\\n\\nThis is primarily used for:\\n  • Verifying L1 state on L2\\n  • Resolving deposit transactions\\n  • Cross-chain message verification\\n\\nExamples:\\n  `aggsandbox show l1-info-tree-index --network-id 0 --deposit-count 0`\\n  `aggsandbox show l1-info-tree-index -n 1 -d 5`\\n  `aggsandbox show l1-info-tree-index --json`  # Raw JSON output for scripting"
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
                let display_data = filter_display_metadata(&response.data);
                api::print_json_response("Bridge Information", &display_data);
            }
        }
        ShowCommands::Claims {
            network_id,
            bridge_tx_hash,
            claim_tx_hash,
            status,
            claim_type,
            address,
            json,
        } => {
            let response = api::get_claims(&config, network_id, json).await?;
            let filtered_data = filter_claims(
                &response.data,
                bridge_tx_hash.as_deref(),
                claim_tx_hash.as_deref(),
                status.as_deref(),
                claim_type.as_deref(),
                address.as_deref(),
            );
            if json {
                api::print_raw_json(&filtered_data);
            } else {
                let display_data = filter_display_metadata(&filtered_data);
                api::print_json_response("Claims Information", &display_data);
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
                let display_data = filter_display_metadata(&response.data);
                api::print_json_response("Claim Proof Information", &display_data);
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
                let display_data = filter_display_metadata(&response.data);
                api::print_json_response("L1 Info Tree Index", &display_data);
            }
        }
    }
    Ok(())
}

/// Filter claims based on provided criteria
///
/// Filters claims array based on bridge_tx_hash, claim_tx_hash, status, type, and destination address.
/// If no filters are provided, returns the original data unchanged.
fn filter_claims(
    data: &serde_json::Value,
    bridge_tx_hash_filter: Option<&str>,
    claim_tx_hash_filter: Option<&str>,
    status_filter: Option<&str>,
    type_filter: Option<&str>,
    address_filter: Option<&str>,
) -> serde_json::Value {
    use serde_json::Value;

    // If no filters are provided, return original data
    if bridge_tx_hash_filter.is_none()
        && claim_tx_hash_filter.is_none()
        && status_filter.is_none()
        && type_filter.is_none()
        && address_filter.is_none()
    {
        return data.clone();
    }

    let mut result = data.clone();

    // Extract claims array if it exists
    if let Some(claims_array) = data.get("claims").and_then(|v| v.as_array()) {
        let filtered_claims: Vec<Value> = claims_array
            .iter()
            .filter(|claim| {
                // Filter by bridge transaction hash
                if let Some(bridge_hash) = bridge_tx_hash_filter {
                    if let Some(bridge_tx_hash) =
                        claim.get("bridge_tx_hash").and_then(|v| v.as_str())
                    {
                        if !bridge_tx_hash.eq_ignore_ascii_case(bridge_hash) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Filter by claim transaction hash
                if let Some(claim_hash) = claim_tx_hash_filter {
                    if let Some(claim_tx_hash) = claim.get("claim_tx_hash").and_then(|v| v.as_str())
                    {
                        // Handle empty claim_tx_hash for pending claims
                        if claim_tx_hash.is_empty() {
                            return false; // Can't match empty claim_tx_hash
                        }
                        if !claim_tx_hash.eq_ignore_ascii_case(claim_hash) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Filter by status
                if let Some(status) = status_filter {
                    if let Some(claim_status) = claim.get("status").and_then(|v| v.as_str()) {
                        if !claim_status.eq_ignore_ascii_case(status) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Filter by type
                if let Some(claim_type) = type_filter {
                    if let Some(claim_type_value) = claim.get("type").and_then(|v| v.as_str()) {
                        if !claim_type_value.eq_ignore_ascii_case(claim_type) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Filter by destination address
                if let Some(addr) = address_filter {
                    if let Some(dest_address) = claim.get("dest_address").and_then(|v| v.as_str()) {
                        if !dest_address.eq_ignore_ascii_case(addr) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

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

/// Remove sandbox_metadata from API response for cleaner display output
///
/// Recursively filters out sandbox_metadata at any level while preserving all other data.
/// This is used for display output only - JSON output retains full metadata.
fn filter_display_metadata(data: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;

    match data {
        Value::Object(obj) => {
            let mut filtered_obj = serde_json::Map::new();

            for (key, value) in obj {
                // Skip sandbox_metadata keys at any level
                if key != "sandbox_metadata" {
                    // Recursively filter nested values
                    filtered_obj.insert(key.clone(), filter_display_metadata(value));
                }
            }

            Value::Object(filtered_obj)
        }
        Value::Array(arr) => {
            // Recursively filter array elements
            let filtered_arr: Vec<Value> = arr.iter().map(filter_display_metadata).collect();
            Value::Array(filtered_arr)
        }
        // For primitive values (String, Number, Bool, Null), return as-is
        _ => data.clone(),
    }
}
