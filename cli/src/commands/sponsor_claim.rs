use crate::api;
use crate::config::Config;
use crate::error::Result;
use crate::utils::{BridgeInfo,BridgesWrapper, ClaimBody, ClaimProofWrapper};
use anyhow::anyhow;


pub async fn handle_sponsor_claim(deposit: u32, origin_network: u64, destination_network: u64) -> Result<()> {
    let config = Config::load()?;

    // Let's asume for the moment that the bridge transaction comes from the L1, in which case global index is equal to deposit
    // TODO: Investigate how would the transaction from L2 to L1 work
    let global_index = deposit as u64;

    // Get Bridges information
    let bridges_resp = api::get_bridges(&config, origin_network, false).await?;
    let BridgesWrapper { bridges } : BridgesWrapper = serde_json::from_value(bridges_resp.data)?;

    // Find the bridge whose deposit_count matches `deposit`
    let bridge: &BridgeInfo = bridges
        .iter()
        .find(|b| b.deposit_count == deposit)
        .ok_or_else(|| anyhow!("bridge with deposit #{deposit} not found on network {origin_network}"))?;

    // Get claim_proof information in order to extract mainnet_exit_root and rollup_exit_root
    let proof_resp = api::get_claim_proof(&config, origin_network, deposit as u64, deposit as u64, false).await?;
    let ClaimProofWrapper { leaf } : ClaimProofWrapper = serde_json::from_value(proof_resp.data)?;

    // Parse amount from string to u64
    let amount: u64 = bridge.amount
        .parse()
        .map_err(|e| anyhow!("parsing amount {}: {}", bridge.amount, e))?;

    // Build the request body
    let body = ClaimBody {
        leaf_type:            bridge.leaf_type,
        global_index:         global_index,
        mainnet_exit_root:    leaf.mainnet_exit_root,
        rollup_exit_root:     leaf.rollup_exit_root,
        origin_network:       origin_network,
        origin_token_address: bridge.origin_address,
        destination_network:  destination_network,
        destination_address:  bridge.destination_address,
        amount:               amount,
        metadata:             bridge.metadata.clone(),
    };

    println!(
        "Claim body for POST request:\n{}",
        serde_json::to_string_pretty(&body)?
    );

    // POST /bridge/v1/sponsor-claim
    api::post_sponsor_claim(&config, &body).await?;

    println!("✅  Sponsor-claim submitted (globalIndex = {global_index})");

    Ok(())

}

pub async fn handle_claim_status(global_index: u64) -> Result<()> {
    let config = Config::load()?;

    let resp = api::get_sponsored_claim_status(&config, global_index).await?;

    api::print_json_response("Sponsored Claim Status", &resp.data);

    Ok(())
}