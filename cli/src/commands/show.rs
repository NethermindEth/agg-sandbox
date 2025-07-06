use crate::api;
use crate::config::Config;
use crate::error::Result;

/// Show subcommands enum (moved from main.rs)
#[derive(Debug, clap::Subcommand)]
pub enum ShowCommands {
    /// Show bridges for a network
    Bridges {
        /// Network ID to query bridges for
        #[arg(short, long, default_value = "1")]
        network_id: u64,
    },
    /// Show claims for a network
    Claims {
        /// Network ID to query claims for
        #[arg(short, long, default_value = "1101")]
        network_id: u64,
    },
    /// Show claim proof
    ClaimProof {
        /// Network ID
        #[arg(short, long, default_value = "1")]
        network_id: u64,
        /// Leaf index
        #[arg(short, long, default_value = "0")]
        leaf_index: u64,
        /// Deposit count
        #[arg(short, long, default_value = "1")]
        deposit_count: u64,
    },
    /// Show L1 info tree index
    L1InfoTreeIndex {
        /// Network ID
        #[arg(short, long, default_value = "1")]
        network_id: u64,
        /// Deposit count
        #[arg(short, long, default_value = "0")]
        deposit_count: u64,
    },
}

/// Handle the show command
pub async fn handle_show(subcommand: ShowCommands) -> Result<()> {
    let config = Config::load()?;

    match subcommand {
        ShowCommands::Bridges { network_id } => {
            let response = api::get_bridges(&config, network_id).await?;
            api::print_json_response("Bridge Information", &response.data);
        }
        ShowCommands::Claims { network_id } => {
            let response = api::get_claims(&config, network_id).await?;
            api::print_json_response("Claims Information", &response.data);
        }
        ShowCommands::ClaimProof {
            network_id,
            leaf_index,
            deposit_count,
        } => {
            let response =
                api::get_claim_proof(&config, network_id, leaf_index, deposit_count).await?;
            api::print_json_response("Claim Proof Information", &response.data);
        }
        ShowCommands::L1InfoTreeIndex {
            network_id,
            deposit_count,
        } => {
            let response = api::get_l1_info_tree_index(&config, network_id, deposit_count).await?;
            api::print_json_response("L1 Info Tree Index", &response.data);
        }
    }
    Ok(())
}
