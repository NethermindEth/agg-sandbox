//! Bridge utility functions for querying and calculating bridge-related data
//!
//! This module provides standalone utility functions that can be used both
//! programmatically and via CLI commands for bridge operations.

use super::common::{
    contract, get_network_name, serialize_json, table, validate_address, validate_network_id,
    validation_error,
};
use crate::api_client::{CacheConfig, OptimizedApiClient};
use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Arguments for building claim payloads from transaction hashes
#[derive(Debug, Clone)]
pub struct BuildPayloadArgs<'a> {
    pub config: &'a Config,
    pub tx_hash: &'a str,
    pub source_network: u64,
    pub bridge_index: Option<u64>,
}

/// Complete claim payload data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimPayload {
    pub smt_proof: Vec<String>,
    pub smt_proof_rollup: Option<Vec<String>>,
    pub global_index: String,
    pub mainnet_exit_root: String,
    pub rollup_exit_root: String,
    pub origin_network: u32,
    pub origin_token_address: String,
    pub destination_network: u32,
    pub destination_address: String,
    pub amount: String,
    pub metadata: String,
}

/// Arguments for computing global bridge indices
#[derive(Debug, Clone)]
pub struct ComputeGlobalIndexArgs {
    pub index_local: u64,
    pub source_network_id: u64,
}

/// Arguments for getting mapped token information
#[derive(Debug, Clone)]
pub struct MappedTokenArgs<'a> {
    pub config: &'a Config,
    pub network: u64,
    pub origin_network: u32,
    pub origin_token_address: &'a str,
    pub private_key: Option<&'a str>,
}

/// Arguments for precalculating token addresses
#[derive(Debug, Clone)]
pub struct PrecalculatedTokenArgs<'a> {
    pub config: &'a Config,
    pub network: u64,
    pub origin_network: u32,
    pub origin_token_address: &'a str,
    pub private_key: Option<&'a str>,
}

/// Arguments for getting origin token information
#[derive(Debug, Clone)]
pub struct OriginTokenArgs<'a> {
    pub config: &'a Config,
    pub network: u64,
    pub wrapped_token_address: &'a str,
    pub private_key: Option<&'a str>,
}

/// Origin token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginTokenInfo {
    pub origin_network: u32,
    pub origin_token_address: Address,
}

/// Arguments for checking claim status
#[derive(Debug, Clone)]
pub struct IsClaimedArgs<'a> {
    pub config: &'a Config,
    pub network: u64,
    pub index: u64,
    pub source_bridge_network: u64,
    pub private_key: Option<&'a str>,
}

/// JSON output structure for compute index
#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeIndexOutput {
    pub local_index: u64,
    pub source_network: u64,
    pub global_index: String,
}

/// JSON output structure for mapped token info
#[derive(Debug, Serialize, Deserialize)]
pub struct MappedTokenOutput {
    pub origin_network: u32,
    pub origin_token_address: String,
    pub target_network: u64,
    pub wrapped_token_address: String,
}

/// JSON output structure for precalculated token
#[derive(Debug, Serialize, Deserialize)]
pub struct PrecalculatedTokenOutput {
    pub origin_network: u32,
    pub origin_token_address: String,
    pub target_network: u64,
    pub precalculated_address: String,
}

/// JSON output structure for claim status
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimStatusOutput {
    pub network: u64,
    pub bridge_index: u64,
    pub source_network: u64,
    pub is_claimed: bool,
}

/// Build complete claim payload from transaction hash
///
/// Extracts logic from claim_asset.rs to build the complete payload needed for claiming
pub async fn build_payload_for_claim(args: BuildPayloadArgs<'_>) -> Result<ClaimPayload> {
    let api_client = OptimizedApiClient::new(CacheConfig::default());

    // Determine which network to query for bridge data
    let bridge_tx_network = args.source_network;

    // For cross-network claims, determine the proof source network
    let proof_source_network = if args.source_network == 0 { 1 } else { 0 };

    // Get bridges from the network where the transaction actually occurred
    let bridges_response = api_client
        .get_bridges(args.config, bridge_tx_network)
        .await
        .map_err(|e| validation_error(&format!("Failed to get bridges: {e}")))?;

    let bridges = bridges_response["bridges"]
        .as_array()
        .ok_or_else(|| validation_error("Invalid bridges response"))?;

    // Find our bridge transaction
    let bridge_info = if let Some(specific_deposit_count) = args.bridge_index {
        bridges
            .iter()
            .find(|bridge| {
                bridge["tx_hash"].as_str() == Some(args.tx_hash)
                    && bridge["deposit_count"].as_u64() == Some(specific_deposit_count)
            })
            .ok_or_else(|| {
                validation_error(&format!(
                    "Bridge transaction {} with deposit_count {specific_deposit_count} not found",
                    args.tx_hash
                ))
            })?
    } else {
        bridges
            .iter()
            .find(|bridge| bridge["tx_hash"].as_str() == Some(args.tx_hash))
            .ok_or_else(|| {
                validation_error(&format!("Bridge transaction {} not found", args.tx_hash))
            })?
    };

    let deposit_count = bridge_info["deposit_count"]
        .as_u64()
        .ok_or_else(|| validation_error("Missing deposit_count in bridge info"))?;

    // Get L1 info tree index from the proof source network
    let tree_index_response = api_client
        .get_l1_info_tree_index(args.config, proof_source_network, deposit_count)
        .await
        .map_err(|e| validation_error(&format!("Failed to get L1 info tree index: {e}")))?;

    let leaf_index = tree_index_response["l1_info_tree_index"]
        .as_u64()
        .unwrap_or(tree_index_response.as_u64().unwrap_or(0));

    // Get claim proof from the proof source network
    let proof_response = api_client
        .get_claim_proof(args.config, proof_source_network, leaf_index, deposit_count)
        .await
        .map_err(|e| validation_error(&format!("Failed to get claim proof: {e}")))?;

    let l1_info_tree_leaf = &proof_response["l1_info_tree_leaf"];
    let mainnet_exit_root = l1_info_tree_leaf["mainnet_exit_root"]
        .as_str()
        .ok_or_else(|| validation_error("Missing mainnet_exit_root in proof"))?;

    let rollup_exit_root = l1_info_tree_leaf["rollup_exit_root"]
        .as_str()
        .ok_or_else(|| validation_error("Missing rollup_exit_root in proof"))?;

    // Extract SMT proofs
    let smt_proof = proof_response["smt_proof"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let smt_proof_rollup = proof_response["smt_proof_rollup"].as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    });

    // Extract bridge parameters
    let origin_network = bridge_info["orig_net"].as_u64().unwrap_or(0) as u32;
    let origin_token_address = bridge_info["orig_addr"]
        .as_str()
        .unwrap_or("0x0000000000000000000000000000000000000000")
        .to_string();
    let destination_network = bridge_info["dest_net"].as_u64().unwrap_or(0) as u32;
    let destination_address = bridge_info["dest_addr"]
        .as_str()
        .unwrap_or("0x0000000000000000000000000000000000000000")
        .to_string();
    let amount = bridge_info["amount"].as_str().unwrap_or("0").to_string();
    let metadata = bridge_info["metadata"].as_str().unwrap_or("0x").to_string();

    // Compute global index
    let global_index_args = ComputeGlobalIndexArgs {
        index_local: deposit_count,
        source_network_id: args.source_network,
    };
    let global_index = compute_global_index(global_index_args);

    Ok(ClaimPayload {
        smt_proof,
        smt_proof_rollup,
        global_index: global_index.to_string(),
        mainnet_exit_root: mainnet_exit_root.to_string(),
        rollup_exit_root: rollup_exit_root.to_string(),
        origin_network,
        origin_token_address,
        destination_network,
        destination_address,
        amount,
        metadata,
    })
}

/// Compute global index for bridge operations
///
/// Based on lxly.js implementation:
/// - Mainnet (network 0): globalIndex = localIndex + 2^31
/// - L2+ networks: globalIndex = localIndex + (networkId - 1) * 2^32
pub fn compute_global_index(args: ComputeGlobalIndexArgs) -> U256 {
    if args.source_network_id == 0 {
        // Mainnet: globalIndex = localIndex + 2^31
        U256::from(args.index_local) + U256::from(1u64 << 31)
    } else {
        // L2+ networks: globalIndex = localIndex + (networkId - 1) * 2^32
        U256::from(args.index_local) + U256::from((args.source_network_id - 1) * (1u64 << 32))
    }
}

/// Get wrapped token address for an origin token
pub async fn get_mapped_token_info(args: MappedTokenArgs<'_>) -> Result<Address> {
    let origin_token_address = validate_address(args.origin_token_address, "Origin token address")?;
    let bridge_contract =
        contract::get_bridge_contract(args.config, args.network, args.private_key).await?;

    let wrapped_address = bridge_contract
        .get_token_wrapped_address(args.origin_network, origin_token_address)
        .call()
        .await
        .map_err(|e| validation_error(&format!("Failed to get wrapped token address: {e}")))?;

    Ok(wrapped_address)
}

/// Pre-calculate wrapped token address before deployment
pub async fn precalculated_mapped_token_info(args: PrecalculatedTokenArgs<'_>) -> Result<Address> {
    let origin_token_address = validate_address(args.origin_token_address, "Origin token address")?;
    let bridge_contract =
        contract::get_bridge_contract(args.config, args.network, args.private_key).await?;

    // For precalculated address, we need to provide token metadata
    // This is a simplified implementation - in practice, we might need to fetch metadata from the origin token
    let precalculated_address = bridge_contract
        .precalculated_wrapper_address(
            args.origin_network,
            origin_token_address,
            "Wrapped Token".to_string(), // Default name
            "WT".to_string(),            // Default symbol
            18u8,                        // Default decimals
        )
        .call()
        .await
        .map_err(|e| {
            validation_error(&format!(
                "Failed to precalculate wrapped token address: {e}"
            ))
        })?;

    Ok(precalculated_address)
}

/// Get original token info from wrapped token address
pub async fn get_origin_token_info(args: OriginTokenArgs<'_>) -> Result<OriginTokenInfo> {
    let wrapped_token_address =
        validate_address(args.wrapped_token_address, "Wrapped token address")?;
    let bridge_contract =
        contract::get_bridge_contract(args.config, args.network, args.private_key).await?;

    let (origin_network, origin_token_address) = bridge_contract
        .wrapped_token_to_token_info(wrapped_token_address)
        .call()
        .await
        .map_err(|e| validation_error(&format!("Failed to get origin token info: {e}")))?;

    Ok(OriginTokenInfo {
        origin_network,
        origin_token_address,
    })
}

/// Check if a bridge has been claimed
pub async fn is_claimed(args: IsClaimedArgs<'_>) -> Result<bool> {
    validate_network_id(args.source_bridge_network, "Source bridge network")?;
    let bridge_contract =
        contract::get_bridge_contract(args.config, args.network, args.private_key).await?;

    let claimed = bridge_contract
        .is_claimed(U256::from(args.index), args.source_bridge_network as u32)
        .call()
        .await
        .map_err(|e| validation_error(&format!("Failed to check claim status: {e}")))?;

    Ok(claimed)
}

/// Bridge utility commands
#[derive(Debug, clap::Subcommand)]
pub enum UtilityCommands {
    /// Build claim payload from transaction hash
    ///
    /// Extract complete claim payload data from a bridge transaction hash.
    /// This includes SMT proofs, exit roots, and all parameters needed for claiming.
    ///
    /// Examples:
    ///   aggsandbox bridge utils build-payload -t 0xabc123... -s 0
    ///   aggsandbox bridge utils build-payload -t 0xdef456... -s 0 --bridge-index 1 --json
    BuildPayload {
        #[arg(short, long, help = "Bridge transaction hash")]
        tx_hash: String,
        #[arg(short, long, help = "Source network ID")]
        source_network: u64,
        #[arg(long, help = "Bridge index for multi-bridge transactions")]
        bridge_index: Option<u64>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    /// Calculate global index
    ///
    /// Calculate the global bridge index from local index and source network.
    /// Uses lxly.js-compatible algorithm: L1 = localIndex + 2^31, L2+ = localIndex + (networkId-1) * 2^32
    ///
    /// Examples:
    ///   aggsandbox bridge utils compute-index --local-index 42 --source-network 0
    ///   aggsandbox bridge utils compute-index --local-index 100 --source-network 1 --json
    ComputeIndex {
        #[arg(long, help = "Local deposit index")]
        local_index: u64,
        #[arg(long, help = "Source network ID")]
        source_network: u64,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    /// Get wrapped token address
    ///
    /// Query the bridge contract to get the wrapped token address for an origin token.
    /// Returns the actual deployed wrapped token address on the target network.
    ///
    /// Examples:
    ///   aggsandbox bridge utils get-mapped -n 1 --origin-network 0 --origin-token 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC
    ///   aggsandbox bridge utils get-mapped -n 1 --origin-network 0 --origin-token 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC --json
    GetMapped {
        #[arg(short, long, help = "Target network ID")]
        network: u64,
        #[arg(long, help = "Origin network ID")]
        origin_network: u32,
        #[arg(long, help = "Origin token address")]
        origin_token: String,
        #[arg(long, help = "Private key (optional)")]
        private_key: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    /// Pre-calculate token address
    ///
    /// Calculate what the wrapped token address will be before deployment.
    /// Useful for knowing the token address before any tokens are bridged.
    ///
    /// Examples:
    ///   aggsandbox bridge utils precalculate -n 1 --origin-network 0 --origin-token 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC
    ///   aggsandbox bridge utils precalculate -n 1 --origin-network 0 --origin-token 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC --json
    Precalculate {
        #[arg(short, long, help = "Target network ID")]
        network: u64,
        #[arg(long, help = "Origin network ID")]
        origin_network: u32,
        #[arg(long, help = "Origin token address")]
        origin_token: String,
        #[arg(long, help = "Private key (optional)")]
        private_key: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    /// Get origin token info
    ///
    /// Get original token information from a wrapped token address.
    /// Returns the origin network ID and original token contract address.
    ///
    /// Examples:
    ///   aggsandbox bridge utils get-origin -n 1 --wrapped-token 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0
    ///   aggsandbox bridge utils get-origin -n 1 --wrapped-token 0x742d35Cc6965C592342c6c16fb8eaeb90a23b5C0 --json
    GetOrigin {
        #[arg(short, long, help = "Network ID")]
        network: u64,
        #[arg(long, help = "Wrapped token address")]
        wrapped_token: String,
        #[arg(long, help = "Private key (optional)")]
        private_key: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },

    /// Check if bridge is claimed
    ///
    /// Check if a specific bridge has been claimed on the destination network.
    /// Returns true if claimed, false if still pending.
    ///
    /// Examples:
    ///   aggsandbox bridge utils is-claimed -n 1 --index 42 --source-network 0
    ///   aggsandbox bridge utils is-claimed -n 1 --index 42 --source-network 0 --json
    IsClaimed {
        #[arg(short, long, help = "Network ID")]
        network: u64,
        #[arg(long, help = "Bridge index")]
        index: u64,
        #[arg(long, help = "Source bridge network ID")]
        source_network: u64,
        #[arg(long, help = "Private key (optional)")]
        private_key: Option<String>,
        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
}

/// Handle utility commands
pub async fn handle_utility_command(config: &Config, command: UtilityCommands) -> Result<()> {
    match command {
        UtilityCommands::BuildPayload {
            tx_hash,
            source_network,
            bridge_index,
            json,
        } => {
            info!(
                tx_hash = %tx_hash,
                source_network = source_network,
                bridge_index = ?bridge_index,
                "Building claim payload"
            );

            let args = BuildPayloadArgs {
                config,
                tx_hash: &tx_hash,
                source_network,
                bridge_index,
            };

            let payload = build_payload_for_claim(args).await?;

            if json {
                let json_str = serialize_json(&payload)?;
                println!("{json_str}");
            } else {
                let origin_network_str = format!(
                    "{} ({})",
                    payload.origin_network,
                    get_network_name(payload.origin_network as u64)
                );
                let destination_network_str = format!(
                    "{} ({})",
                    payload.destination_network,
                    get_network_name(payload.destination_network as u64)
                );
                let smt_proof_len_str = payload.smt_proof.len().to_string();
                let mut rows = vec![
                    ("Global Index", payload.global_index.as_str()),
                    ("Origin Network", origin_network_str.as_str()),
                    ("Destination Network", destination_network_str.as_str()),
                    ("Origin Token", payload.origin_token_address.as_str()),
                    ("Destination Address", payload.destination_address.as_str()),
                    ("Amount", payload.amount.as_str()),
                    ("Mainnet Exit Root", payload.mainnet_exit_root.as_str()),
                    ("Rollup Exit Root", payload.rollup_exit_root.as_str()),
                    ("SMT Proof Length", smt_proof_len_str.as_str()),
                ];
                let rollup_proof_len_str;
                if let Some(rollup_proof) = &payload.smt_proof_rollup {
                    rollup_proof_len_str = rollup_proof.len().to_string();
                    rows.push(("SMT Rollup Proof Len", rollup_proof_len_str.as_str()));
                }
                table::print_table("🔍 Bridge Claim Payload", &rows);
            }

            Ok(())
        }
        UtilityCommands::ComputeIndex {
            local_index,
            source_network,
            json,
        } => {
            validate_network_id(source_network, "Source network")?;

            info!(
                local_index = local_index,
                source_network = source_network,
                "Computing global index"
            );

            let args = ComputeGlobalIndexArgs {
                index_local: local_index,
                source_network_id: source_network,
            };

            let global_index = compute_global_index(args);

            if json {
                let output = ComputeIndexOutput {
                    local_index,
                    source_network,
                    global_index: global_index.to_string(),
                };
                let json_str = serialize_json(&output)?;
                println!("{json_str}");
            } else {
                let local_index_str = local_index.to_string();
                let source_network_str =
                    format!("{source_network} ({})", get_network_name(source_network));
                let global_index_str = global_index.to_string();
                let rows = vec![
                    ("Local Index", local_index_str.as_str()),
                    ("Source Network", source_network_str.as_str()),
                    ("Global Index", global_index_str.as_str()),
                ];
                table::print_table("🧮 Global Index Calculation", &rows);
            }

            Ok(())
        }
        UtilityCommands::GetMapped {
            network,
            origin_network,
            origin_token,
            private_key,
            json,
        } => {
            info!(
                network = network,
                origin_network = origin_network,
                origin_token = %origin_token,
                "Getting mapped token address"
            );

            let args = MappedTokenArgs {
                config,
                network,
                origin_network,
                origin_token_address: &origin_token,
                private_key: private_key.as_deref(),
            };

            let mapped_address = get_mapped_token_info(args).await?;

            if json {
                let output = MappedTokenOutput {
                    origin_network,
                    origin_token_address: origin_token.clone(),
                    target_network: network,
                    wrapped_token_address: format!("{mapped_address:?}"),
                };
                let json_str = serialize_json(&output)?;
                println!("{json_str}");
            } else {
                let origin_network_str = format!(
                    "{origin_network} ({})",
                    get_network_name(origin_network as u64)
                );
                let target_network_str = format!("{network} ({})", get_network_name(network));
                let wrapped_address_str = format!("{mapped_address:?}");
                let rows = vec![
                    ("Origin Network", origin_network_str.as_str()),
                    ("Origin Token", origin_token.as_str()),
                    ("Target Network", target_network_str.as_str()),
                    ("Wrapped Token Address", wrapped_address_str.as_str()),
                ];
                table::print_table("🔗 Mapped Token Information", &rows);
            }

            Ok(())
        }
        UtilityCommands::Precalculate {
            network,
            origin_network,
            origin_token,
            private_key,
            json,
        } => {
            info!(
                network = network,
                origin_network = origin_network,
                origin_token = %origin_token,
                "Precalculating token address"
            );

            let args = PrecalculatedTokenArgs {
                config,
                network,
                origin_network,
                origin_token_address: &origin_token,
                private_key: private_key.as_deref(),
            };

            let precalculated_address = precalculated_mapped_token_info(args).await?;

            if json {
                let output = PrecalculatedTokenOutput {
                    origin_network,
                    origin_token_address: origin_token.clone(),
                    target_network: network,
                    precalculated_address: format!("{precalculated_address:?}"),
                };
                let json_str = serialize_json(&output)?;
                println!("{json_str}");
            } else {
                let origin_network_str = format!(
                    "{origin_network} ({})",
                    get_network_name(origin_network as u64)
                );
                let target_network_str = format!("{network} ({})", get_network_name(network));
                let precalculated_address_str = format!("{precalculated_address:?}");
                let rows = vec![
                    ("Origin Network", origin_network_str.as_str()),
                    ("Origin Token", origin_token.as_str()),
                    ("Target Network", target_network_str.as_str()),
                    ("Precalculated Address", precalculated_address_str.as_str()),
                ];
                table::print_table("🧮 Precalculated Token Address", &rows);
            }

            Ok(())
        }
        UtilityCommands::GetOrigin {
            network,
            wrapped_token,
            private_key,
            json,
        } => {
            info!(
                network = network,
                wrapped_token = %wrapped_token,
                "Getting origin token info"
            );

            let args = OriginTokenArgs {
                config,
                network,
                wrapped_token_address: &wrapped_token,
                private_key: private_key.as_deref(),
            };

            let origin_info = get_origin_token_info(args).await?;

            if json {
                let json_str = serialize_json(&origin_info)?;
                println!("{json_str}");
            } else {
                let network_str = format!("{network} ({})", get_network_name(network));
                let origin_network_str = format!(
                    "{} ({})",
                    origin_info.origin_network,
                    get_network_name(origin_info.origin_network as u64)
                );
                let origin_token_address_str = format!("{:?}", origin_info.origin_token_address);
                let rows = vec![
                    ("Network", network_str.as_str()),
                    ("Wrapped Token", wrapped_token.as_str()),
                    ("Origin Network", origin_network_str.as_str()),
                    ("Origin Token Address", origin_token_address_str.as_str()),
                ];
                table::print_table("🔍 Origin Token Information", &rows);
            }

            Ok(())
        }
        UtilityCommands::IsClaimed {
            network,
            index,
            source_network,
            private_key,
            json,
        } => {
            info!(
                network = network,
                index = index,
                source_network = source_network,
                "Checking claim status"
            );

            let args = IsClaimedArgs {
                config,
                network,
                index,
                source_bridge_network: source_network,
                private_key: private_key.as_deref(),
            };

            let claimed = is_claimed(args).await?;

            if json {
                let output = ClaimStatusOutput {
                    network,
                    bridge_index: index,
                    source_network,
                    is_claimed: claimed,
                };
                let json_str = serialize_json(&output)?;
                println!("{json_str}");
            } else {
                let network_str = format!("{network} ({})", get_network_name(network));
                let index_str = index.to_string();
                let source_network_str =
                    format!("{source_network} ({})", get_network_name(source_network));
                let claimed_status = if claimed {
                    "✅ CLAIMED"
                } else {
                    "❌ NOT CLAIMED"
                };
                let rows = vec![
                    ("Network", network_str.as_str()),
                    ("Bridge Index", index_str.as_str()),
                    ("Source Network", source_network_str.as_str()),
                    ("Claimed Status", claimed_status),
                ];
                table::print_table("🔍 Bridge Claim Status", &rows);
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_global_index_mainnet() {
        let args = ComputeGlobalIndexArgs {
            index_local: 42,
            source_network_id: 0,
        };
        let result = compute_global_index(args);
        let expected = U256::from(42) + U256::from(1u64 << 31); // 42 + 2147483648
        assert_eq!(result, expected);
    }

    #[test]
    fn test_compute_global_index_l2() {
        let args = ComputeGlobalIndexArgs {
            index_local: 42,
            source_network_id: 1,
        };
        let result = compute_global_index(args);
        let expected = U256::from(42) + U256::from(0 * (1u64 << 32)); // 42 + 0
        assert_eq!(result, expected);
    }

    #[test]
    fn test_compute_global_index_l3() {
        let args = ComputeGlobalIndexArgs {
            index_local: 42,
            source_network_id: 2,
        };
        let result = compute_global_index(args);
        let expected = U256::from(42) + U256::from(1 * (1u64 << 32)); // 42 + 4294967296
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_payload_args_structure() {
        // Test that BuildPayloadArgs can be created correctly
        let config = &Config::default();
        let args = BuildPayloadArgs {
            config,
            tx_hash: "0x123",
            source_network: 0,
            bridge_index: Some(1),
        };

        assert_eq!(args.tx_hash, "0x123");
        assert_eq!(args.source_network, 0);
        assert_eq!(args.bridge_index, Some(1));
    }

    #[test]
    fn test_mapped_token_args_structure() {
        // Test that MappedTokenArgs can be created correctly
        let config = &Config::default();
        let args = MappedTokenArgs {
            config,
            network: 1,
            origin_network: 0,
            origin_token_address: "0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC",
            private_key: None,
        };

        assert_eq!(args.network, 1);
        assert_eq!(args.origin_network, 0);
        assert_eq!(
            args.origin_token_address,
            "0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC"
        );
        assert!(args.private_key.is_none());
    }

    #[test]
    fn test_origin_token_info_serialization() {
        // Test that OriginTokenInfo can be serialized/deserialized
        let info = OriginTokenInfo {
            origin_network: 0,
            origin_token_address: Address::zero(),
        };

        let json = serde_json::to_string(&info).expect("Should serialize");
        let deserialized: OriginTokenInfo =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.origin_network, 0);
        assert_eq!(deserialized.origin_token_address, Address::zero());
    }

    #[test]
    fn test_is_claimed_args_structure() {
        // Test that IsClaimedArgs can be created correctly
        let config = &Config::default();
        let args = IsClaimedArgs {
            config,
            network: 1,
            index: 42,
            source_bridge_network: 0,
            private_key: Some("0x123"),
        };

        assert_eq!(args.network, 1);
        assert_eq!(args.index, 42);
        assert_eq!(args.source_bridge_network, 0);
        assert_eq!(args.private_key, Some("0x123"));
    }
}
