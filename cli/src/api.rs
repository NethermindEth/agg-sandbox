use crate::api_client::OptimizedApiClient;
use crate::error::Result;
use crate::validation::Validator;
use colored::*;
use serde::Deserialize;
use tracing::{debug, info, instrument};

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

#[instrument(fields(network_id = network_id))]
pub async fn get_bridges(config: &Config, network_id: u64) -> Result<BridgeResponse> {
    // Validate network ID
    debug!(network_id = network_id, "Validating network ID");
    let validated_network_id = Validator::validate_network_id(network_id)?;

    info!(
        network_id = validated_network_id,
        "Fetching bridges from API with caching"
    );

    println!(
        "{}",
        format!("🔍 Fetching bridges for network_id: {validated_network_id}").cyan()
    );

    // Use the optimized client with caching and connection pooling
    let client = OptimizedApiClient::global();
    let bridge_data = client.get_bridges(config, validated_network_id).await?;

    info!(
        bridges_count = bridge_data.as_object().map(|o| o.len()).unwrap_or(0),
        "Successfully retrieved bridges"
    );

    Ok(BridgeResponse { data: bridge_data })
}

pub async fn get_claims(config: &Config, network_id: u64) -> Result<ClaimResponse> {
    // Validate network ID
    let validated_network_id = Validator::validate_network_id(network_id)?;

    println!(
        "{}",
        format!("🔍 Fetching claims for network_id: {validated_network_id}").cyan()
    );

    // Use the optimized client with caching and connection pooling
    let client = OptimizedApiClient::global();
    let claim_data = client.get_claims(config, validated_network_id).await?;

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

    println!(
        "{}",
        format!(
            "🔍 Fetching claim proof for network_id: {validated_network_id}, leaf_index: {leaf_index}, deposit_count: {deposit_count}"
        )
        .cyan()
    );

    // Use the optimized client with caching and connection pooling
    let client = OptimizedApiClient::global();
    let proof_data = client
        .get_claim_proof(config, validated_network_id, leaf_index, deposit_count)
        .await?;

    Ok(ClaimProofResponse { data: proof_data })
}

pub async fn get_l1_info_tree_index(
    config: &Config,
    network_id: u64,
    deposit_count: u64,
) -> Result<L1InfoTreeIndexResponse> {
    // Validate network ID
    let validated_network_id = Validator::validate_network_id(network_id)?;

    println!(
        "{}",
        format!(
            "🔍 Fetching L1 info tree index for network_id: {validated_network_id}, deposit_count: {deposit_count}"
        )
        .cyan()
    );

    // Use the optimized client with caching and connection pooling
    let client = OptimizedApiClient::global();
    let info_data = client
        .get_l1_info_tree_index(config, validated_network_id, deposit_count)
        .await?;

    Ok(L1InfoTreeIndexResponse { data: info_data })
}

pub fn print_json_response(title: &str, data: &serde_json::Value) {
    println!("\n{}", format!("📋 {title}").green().bold());
    println!("{}", "═".repeat(60).dimmed());

    let pretty_json = serde_json::to_string_pretty(data).unwrap_or_else(|_| format!("{data:?}"));

    println!("{pretty_json}");
    println!("{}", "═".repeat(60).dimmed());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        AccountConfig, ApiConfig, ChainConfig, Config, ContractConfig, NetworkConfig,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::time::Duration;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_config(base_url: &str) -> Config {
        Config {
            api: ApiConfig {
                base_url: base_url.to_string(),
                timeout: Duration::from_millis(5000),
                retry_attempts: 3,
            },
            networks: NetworkConfig {
                l1: ChainConfig {
                    name: "Test-L1".to_string(),
                    chain_id: "1".to_string(),
                    rpc_url: "http://localhost:8545".to_string(),
                    fork_url: None,
                },
                l2: ChainConfig {
                    name: "Test-L2".to_string(),
                    chain_id: "1101".to_string(),
                    rpc_url: "http://localhost:8546".to_string(),
                    fork_url: None,
                },
                l3: None,
            },
            accounts: AccountConfig {
                accounts: vec!["0xtest".to_string()],
                private_keys: vec!["0xkey".to_string()],
            },
            contracts: ContractConfig {
                l1_contracts: HashMap::new(),
                l2_contracts: HashMap::new(),
            },
        }
    }

    #[tokio::test]
    async fn test_get_bridges_success() {
        // Setup mock server
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        let mock_response = json!({
            "bridges": [
                {
                    "id": "1",
                    "network_id": 1,
                    "address": "0x123"
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .and(query_param("network_id", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        // Test the function
        let result = get_bridges(&config, 1).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data, mock_response);
    }

    #[tokio::test]
    async fn test_get_bridges_invalid_network_id() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        // Test with invalid network ID
        let result = get_bridges(&config, 999).await;

        assert!(result.is_err());
        // Verify it's a validation error for invalid network ID
        match result.unwrap_err() {
            crate::error::AggSandboxError::Config(_) => {} // Expected
            _ => panic!("Expected ConfigError for invalid network ID"),
        }
    }

    #[tokio::test]
    async fn test_get_bridges_server_error() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/bridge/v1/bridges"))
            .and(query_param("network_id", "1"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let result = get_bridges(&config, 1).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::AggSandboxError::Api(api_err) => match api_err {
                crate::error::ApiError::RequestFailed { status, .. } => {
                    assert_eq!(status, 500);
                }
                _ => panic!("Expected RequestFailed error"),
            },
            _ => panic!("Expected ApiError"),
        }
    }

    #[tokio::test]
    async fn test_get_claims_success() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        let mock_response = json!({
            "claims": [
                {
                    "id": "1",
                    "network_id": 1101,
                    "amount": "1000000000000000000"
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/bridge/v1/claims"))
            .and(query_param("network_id", "1101"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let result = get_claims(&config, 1101).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data, mock_response);
    }

    #[tokio::test]
    async fn test_get_claim_proof_success() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        let mock_response = json!({
            "proof": {
                "leaf_index": 0,
                "deposit_count": 1,
                "merkle_proof": ["0xabc", "0xdef"]
            }
        });

        Mock::given(method("GET"))
            .and(path("/bridge/v1/claim-proof"))
            .and(query_param("network_id", "1"))
            .and(query_param("leaf_index", "0"))
            .and(query_param("deposit_count", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let result = get_claim_proof(&config, 1, 0, 1).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data, mock_response);
    }

    #[tokio::test]
    async fn test_get_l1_info_tree_index_success() {
        let mock_server = MockServer::start().await;
        let config = create_test_config(&mock_server.uri());

        let mock_response = json!({
            "l1_info_tree_index": 42,
            "deposit_count": 0
        });

        Mock::given(method("GET"))
            .and(path("/bridge/v1/l1-info-tree-index"))
            .and(query_param("network_id", "1"))
            .and(query_param("deposit_count", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let result = get_l1_info_tree_index(&config, 1, 0).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data, mock_response);
    }

    #[test]
    fn test_print_json_response() {
        let test_data = json!({
            "test": "value",
            "number": 42,
            "array": [1, 2, 3]
        });

        // This test mainly ensures the function doesn't panic
        // In a real scenario, you might want to capture stdout to verify output
        print_json_response("Test Response", &test_data);
    }
}
