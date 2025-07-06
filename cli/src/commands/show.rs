use crate::api;
use crate::config::Config;
use crate::error::Result;

/// Bridge and blockchain data subcommands
#[derive(Debug, clap::Subcommand)]
pub enum ShowCommands {
    /// 🌉 Show bridge information for a specific network
    #[command(
        long_about = "Display bridge information for the specified network.\\n\\nBridges enable cross-chain transfers between L1 and L2 networks.\\nThis command shows active bridges, their configurations, and status.\\n\\nNetwork IDs:\\n  • 1 = Ethereum L1\\n  • 1101 = Polygon zkEVM L2\\n  • 1102 = Additional L2 (if multi-L2 enabled)\\n\\nExamples:\\n  aggsandbox show bridges                 # Show L1 bridges\\n  aggsandbox show bridges --network 1101  # Show L2 bridges"
    )]
    Bridges {
        /// Network ID to query (1=L1, 1101=L2, etc.)
        #[arg(
            short,
            long,
            default_value = "1",
            help = "Network ID (1=L1 Ethereum, 1101=L2 Polygon zkEVM)"
        )]
        network_id: u64,
    },
    /// 📋 Show pending claims for a network
    #[command(
        long_about = "Display pending claims that can be executed on the specified network.\\n\\nClaims represent cross-chain transfers waiting to be processed.\\nEach claim contains transfer details and required proof data.\\n\\nTypically:\\n  • L1 claims (network 1): Deposits to be claimed on L2\\n  • L2 claims (network 1101): Withdrawals to be claimed on L1\\n\\nExamples:\\n  aggsandbox show claims                  # Show L1 claims\\n  aggsandbox show claims --network 1101  # Show L2 claims"
    )]
    Claims {
        /// Network ID to query for pending claims
        #[arg(
            short,
            long,
            default_value = "1101",
            help = "Network ID to query for pending claims"
        )]
        network_id: u64,
    },
    /// 🔐 Generate and show claim proof for a specific transaction
    #[command(
        long_about = "Generate a cryptographic proof required to claim a cross-chain transfer.\\n\\nClaim proofs are Merkle proofs that verify a deposit exists in the\\nglobal exit tree, enabling secure cross-chain claims.\\n\\nParameters:\\n  • network_id: The target network for claiming\\n  • leaf_index: Position in the global exit tree\\n  • deposit_count: Number of deposits when the exit was created\\n\\nExamples:\\n  aggsandbox show claim-proof --network 1 --leaf-index 0 --deposit-count 1\\n  aggsandbox show claim-proof -n 1101 -l 5 -d 10"
    )]
    ClaimProof {
        /// Target network ID for the claim
        #[arg(
            short,
            long,
            default_value = "1",
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
    },
    /// 🌳 Show L1 info tree index for deposit verification
    #[command(
        long_about = "Retrieve the L1 information tree index for a specific deposit count.\\n\\nThe L1 info tree contains snapshots of L1 state that are used\\nby L2 for deposit verification and cross-chain communication.\\n\\nThis is primarily used for:\\n  • Verifying L1 state on L2\\n  • Resolving deposit transactions\\n  • Cross-chain message verification\\n\\nExamples:\\n  aggsandbox show l1-info-tree-index --network 1 --deposit-count 0\\n  aggsandbox show l1-info-tree-index -n 1101 -d 5"
    )]
    L1InfoTreeIndex {
        /// Network ID to query
        #[arg(short, long, default_value = "1", help = "Network ID to query")]
        network_id: u64,
        /// Deposit count to get L1 info tree index for
        #[arg(
            short,
            long,
            default_value = "0",
            help = "Deposit count to lookup in L1 info tree"
        )]
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
