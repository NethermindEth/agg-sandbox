use anyhow::{Context, Result};
use colored::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BridgeResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ClaimResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ClaimProofResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

const BASE_URL: &str = "http://localhost:5577";

pub async fn get_bridges(network_id: u64) -> Result<BridgeResponse> {
    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}/bridge/v1/bridges?network_id={network_id}");

    println!(
        "{}",
        format!("🔍 Fetching bridges for network_id: {network_id}").cyan()
    );
    println!("{}", format!("📡 URL: {url}").dimmed());

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request to bridges endpoint")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "API request failed with status: {}",
            response.status()
        ));
    }

    let bridge_data: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse bridges response as JSON")?;

    Ok(BridgeResponse { data: bridge_data })
}

pub async fn get_claims(network_id: u64) -> Result<ClaimResponse> {
    let client = reqwest::Client::new();
    let url = format!("{BASE_URL}/bridge/v1/claims?network_id={network_id}");

    println!(
        "{}",
        format!("🔍 Fetching claims for network_id: {network_id}").cyan()
    );
    println!("{}", format!("📡 URL: {url}").dimmed());

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request to claims endpoint")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "API request failed with status: {}",
            response.status()
        ));
    }

    let claim_data: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse claims response as JSON")?;

    Ok(ClaimResponse { data: claim_data })
}

pub async fn get_claim_proof(
    network_id: u64,
    leaf_index: u64,
    deposit_count: u64,
) -> Result<ClaimProofResponse> {
    let client = reqwest::Client::new();
    let url = format!(
        "{BASE_URL}/bridge/v1/claim-proof?network_id={network_id}&leaf_index={leaf_index}&deposit_count={deposit_count}"
    );

    println!(
        "{}",
        format!(
            "🔍 Fetching claim proof for network_id: {network_id}, leaf_index: {leaf_index}, deposit_count: {deposit_count}"
        )
        .cyan()
    );
    println!("{}", format!("📡 URL: {url}").dimmed());

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request to claim-proof endpoint")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "API request failed with status: {}",
            response.status()
        ));
    }

    let proof_data: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse claim-proof response as JSON")?;

    Ok(ClaimProofResponse { data: proof_data })
}

pub fn print_json_response(title: &str, data: &serde_json::Value) {
    println!("\n{}", format!("📋 {title}").green().bold());
    println!("{}", "═".repeat(60).dimmed());

    let pretty_json = serde_json::to_string_pretty(data).unwrap_or_else(|_| format!("{data:?}"));

    println!("{pretty_json}");
    println!("{}", "═".repeat(60).dimmed());
}
