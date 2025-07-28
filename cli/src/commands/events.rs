use crate::error::Result;
use crate::events;
use crate::validation::Validator;

/// Handle the events command
pub async fn handle_events(
    network_id: Option<u64>,
    chain: Option<String>,
    blocks: u64,
    address: Option<String>,
) -> Result<()> {
    // Determine which parameter to use and validate
    let resolved_chain = match (network_id, chain) {
        (Some(net_id), Some(_chain_name)) => {
            // Both provided - prefer network_id and warn about chain
            eprintln!("⚠️  Both --network-id and --chain provided. Using --network-id={net_id}. Please use --network-id only as --chain is deprecated.");
            let validated_net_id = Validator::validate_network_id(net_id)?;
            network_id_to_chain(validated_net_id)?
        }
        (Some(net_id), None) => {
            // Network ID provided - validate and convert
            let validated_net_id = Validator::validate_network_id(net_id)?;
            network_id_to_chain(validated_net_id)?
        }
        (None, Some(chain_name)) => {
            // Chain provided - warn about deprecation
            eprintln!("⚠️  --chain parameter is deprecated. Please use --network-id instead (0=L1, 1=L2, 2=L3).");
            chain_name
        }
        (None, None) => {
            return Err(crate::error::ConfigError::missing_required(
                "Either --network-id or --chain must be provided",
            )
            .into());
        }
    };

    events::fetch_and_display_events(&resolved_chain, blocks, address).await
}

/// Convert network ID to chain name
fn network_id_to_chain(network_id: u64) -> Result<String> {
    match network_id {
        0 => Ok("anvil-l1".to_string()),     // L1 Ethereum
        1 => Ok("anvil-l2".to_string()),     // First L2
        2 => Ok("anvil-l3".to_string()),     // Second L2
        3 => Ok("anvil-l3".to_string()),     // Third L2 (maps to same chain for now)
        31337 => Ok("anvil-l1".to_string()), // Local dev L1
        31338 => Ok("anvil-l2".to_string()), // Local dev L2
        31339 => Ok("anvil-l3".to_string()), // Local dev L3
        _ => Err(crate::error::ConfigError::invalid_value(
            "network_id",
            &network_id.to_string(),
            "Unsupported network ID for events command. Use 0 (L1), 1 (L2), or 2-3 (L3)",
        )
        .into()),
    }
}
