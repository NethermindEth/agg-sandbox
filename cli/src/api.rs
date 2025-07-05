use crate::error::{ApiError, Result};
use crate::validation::Validator;
use colored::*;
use serde::Deserialize;

use super::config::Config;

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

#[derive(Debug, Deserialize)]
pub struct L1InfoTreeIndexResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

pub async fn get_bridges(config: &Config, network_id: u64) -> Result<BridgeResponse> {
    // Validate network ID
    let validated_network_id = Validator::validate_network_id(network_id)?;

    let client = reqwest::Client::new();
    let url = format!(
        "{}/bridge/v1/bridges?network_id={validated_network_id}",
        config.api.base_url
    );

    println!(
        "{}",
        format!("🔍 Fetching bridges for network_id: {validated_network_id}").cyan()
    );
    println!("{}", format!("📡 URL: {url}").dimmed());

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::network_error(&e.to_string()))?;

    if !response.status().is_success() {
        return Err(ApiError::request_failed(
            &url,
            response.status().as_u16(),
            "Bridges endpoint request failed",
        )
        .into());
    }

    let bridge_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ApiError::json_parse_error(&e.to_string()))?;

    Ok(BridgeResponse { data: bridge_data })
}

pub async fn get_claims(config: &Config, network_id: u64) -> Result<ClaimResponse> {
    // Validate network ID
    let validated_network_id = Validator::validate_network_id(network_id)?;

    let client = reqwest::Client::new();
    let url = format!(
        "{}/bridge/v1/claims?network_id={validated_network_id}",
        config.api.base_url
    );

    println!(
        "{}",
        format!("🔍 Fetching claims for network_id: {validated_network_id}").cyan()
    );
    println!("{}", format!("📡 URL: {url}").dimmed());

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::network_error(&e.to_string()))?;

    if !response.status().is_success() {
        return Err(ApiError::request_failed(
            &url,
            response.status().as_u16(),
            "Claims endpoint request failed",
        )
        .into());
    }

    let claim_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ApiError::json_parse_error(&e.to_string()))?;

    Ok(ClaimResponse { data: claim_data })
}

pub async fn get_claim_proof(
    config: &Config,
    network_id: u64,
    leaf_index: u64,
    deposit_count: u64,
) -> Result<ClaimProofResponse> {
    // Validate network ID
    let validated_network_id = Validator::validate_network_id(network_id)?;

    let client = reqwest::Client::new();
    let url = format!(
        "{}/bridge/v1/claim-proof?network_id={validated_network_id}&leaf_index={leaf_index}&deposit_count={deposit_count}",
        config.api.base_url
    );

    println!(
        "{}",
        format!(
            "🔍 Fetching claim proof for network_id: {validated_network_id}, leaf_index: {leaf_index}, deposit_count: {deposit_count}"
        )
        .cyan()
    );
    println!("{}", format!("📡 URL: {url}").dimmed());

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::network_error(&e.to_string()))?;

    if !response.status().is_success() {
        return Err(ApiError::request_failed(
            &url,
            response.status().as_u16(),
            "Claim-proof endpoint request failed",
        )
        .into());
    }

    let proof_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ApiError::json_parse_error(&e.to_string()))?;

    Ok(ClaimProofResponse { data: proof_data })
}

pub async fn get_l1_info_tree_index(
    config: &Config,
    network_id: u64,
    deposit_count: u64,
) -> Result<L1InfoTreeIndexResponse> {
    // Validate network ID
    let validated_network_id = Validator::validate_network_id(network_id)?;

    let client = reqwest::Client::new();
    let url = format!(
        "{}/bridge/v1/l1-info-tree-index?network_id={validated_network_id}&deposit_count={deposit_count}",
        config.api.base_url
    );

    println!(
        "{}",
        format!(
            "🔍 Fetching L1 info tree index for network_id: {validated_network_id}, deposit_count: {deposit_count}"
        )
        .cyan()
    );
    println!("{}", format!("📡 URL: {url}").dimmed());

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::network_error(&e.to_string()))?;

    if !response.status().is_success() {
        return Err(ApiError::request_failed(
            &url,
            response.status().as_u16(),
            "L1-info-tree-index endpoint request failed",
        )
        .into());
    }

    let info_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ApiError::json_parse_error(&e.to_string()))?;

    Ok(L1InfoTreeIndexResponse { data: info_data })
}

pub fn print_json_response(title: &str, data: &serde_json::Value) {
    println!("\n{}", format!("📋 {title}").green().bold());
    println!("{}", "═".repeat(60).dimmed());

    let pretty_json = serde_json::to_string_pretty(data).unwrap_or_else(|_| format!("{data:?}"));

    println!("{pretty_json}");
    println!("{}", "═".repeat(60).dimmed());
}
