use ethers::types::{Address, Bytes, H256};
use serde::{Deserialize, Serialize};

/// Body expected by POST /bridge/v1/sponsor-claim
#[derive(Serialize)]
pub struct ClaimBody {
    #[serde(rename = "LeafType")]
    pub leaf_type: u8,
    #[serde(rename = "GlobalIndex")]
    pub global_index: u64,
    #[serde(rename = "MainnetExitRoot")]
    pub mainnet_exit_root: H256,
    #[serde(rename = "RollupExitRoot")]
    pub rollup_exit_root: H256,
    #[serde(rename = "OriginNetwork")]
    pub origin_network: u64,
    #[serde(rename = "OriginTokenAddress")]
    pub origin_token_address: Address,
    #[serde(rename = "DestinationNetwork")]
    pub destination_network: u64,
    #[serde(rename = "DestinationAddress")]
    pub destination_address: Address,
    #[serde(rename = "Amount")]
    pub amount: u64,
    #[serde(rename = "Metadata")]
    pub metadata: Bytes,
}
#[derive(Debug, Deserialize)]
pub struct BridgeInfo {
    // Only added fields needed for the claim body
    #[serde(rename = "leaf_type")]
    pub leaf_type: u8,

    #[serde(rename = "origin_network")]
    pub origin_network: u64,

    #[serde(rename = "origin_address")]
    pub origin_address: Address,

    #[serde(rename = "destination_network")]
    pub destination_network: u64,

    #[serde(rename = "destination_address")]
    pub destination_address: Address,

    #[serde(rename = "amount")]
    pub amount: String,

    #[serde(rename = "metadata")]
    pub metadata: Bytes,

    #[serde(rename = "deposit_count")]
    pub deposit_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct BridgesWrapper {
    pub bridges: Vec<BridgeInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ClaimProofWrapper {
    #[serde(rename = "l1_info_tree_leaf")]
    pub leaf: ClaimLeaf,
}

#[derive(Debug, Deserialize)]
pub struct ClaimLeaf {
    #[serde(rename = "mainnet_exit_root")]
    pub mainnet_exit_root: H256,
    #[serde(rename = "rollup_exit_root")]
    pub rollup_exit_root: H256,
}
