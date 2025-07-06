#![allow(dead_code)]

use crate::api_client::OptimizedApiClient;
use crate::config::Config;
use futures::future::join_all;
use serde_json::Value;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, instrument, warn};

/// Configuration for batch processing operations
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of concurrent requests
    pub max_concurrent: usize,
    /// Delay between batch operations to avoid overwhelming the server
    pub batch_delay: Duration,
    /// Timeout for individual requests in a batch
    pub request_timeout: Duration,
    /// Whether to continue processing if some requests fail
    pub continue_on_error: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            batch_delay: Duration::from_millis(100),
            request_timeout: Duration::from_secs(30),
            continue_on_error: true,
        }
    }
}

/// Result of a batch operation
#[derive(Debug)]
pub struct BatchResult<T> {
    pub successful: Vec<T>,
    pub failed: Vec<(usize, crate::error::AggSandboxError)>,
    pub total_processed: usize,
    pub success_rate: f64,
}

impl<T> BatchResult<T> {
    pub fn new(successful: Vec<T>, failed: Vec<(usize, crate::error::AggSandboxError)>) -> Self {
        let total_processed = successful.len() + failed.len();
        let success_rate = if total_processed > 0 {
            successful.len() as f64 / total_processed as f64
        } else {
            0.0
        };

        Self {
            successful,
            failed,
            total_processed,
            success_rate,
        }
    }
}

/// High-performance batch processor for API operations
pub struct BatchProcessor {
    client: Arc<OptimizedApiClient>,
    config: BatchConfig,
}

impl BatchProcessor {
    /// Create a new batch processor with default configuration
    pub fn new() -> Self {
        Self {
            client: OptimizedApiClient::global(),
            config: BatchConfig::default(),
        }
    }

    /// Create a new batch processor with custom configuration
    pub fn with_config(config: BatchConfig) -> Self {
        Self {
            client: OptimizedApiClient::global(),
            config,
        }
    }

    /// Process multiple network IDs for bridges data in batches
    #[instrument(skip(self, api_config, network_ids))]
    pub async fn get_bridges_batch(
        &self,
        api_config: &Config,
        network_ids: Vec<u64>,
    ) -> BatchResult<(u64, Value)> {
        info!(
            count = network_ids.len(),
            max_concurrent = self.config.max_concurrent,
            "Starting batch bridges processing"
        );

        let mut successful = Vec::new();
        let mut failed = Vec::new();

        // Process in chunks to control concurrency
        for (chunk_idx, chunk) in network_ids.chunks(self.config.max_concurrent).enumerate() {
            debug!(
                chunk_index = chunk_idx,
                chunk_size = chunk.len(),
                "Processing bridges chunk"
            );

            // Create futures for this chunk
            let futures: Vec<_> = chunk
                .iter()
                .enumerate()
                .map(|(idx, &network_id)| {
                    let client = Arc::clone(&self.client);
                    let config = api_config.clone();
                    async move {
                        let result = client.get_bridges(&config, network_id).await;
                        (
                            chunk_idx * self.config.max_concurrent + idx,
                            network_id,
                            result,
                        )
                    }
                })
                .collect();

            // Execute futures concurrently
            let results = join_all(futures).await;

            // Process results
            for (original_idx, network_id, result) in results {
                match result {
                    Ok(data) => {
                        successful.push((network_id, data));
                        debug!(network_id = network_id, "Successfully retrieved bridges");
                    }
                    Err(e) => {
                        failed.push((original_idx, e));
                        warn!(
                            network_id = network_id,
                            error = %failed.last().unwrap().1,
                            "Failed to retrieve bridges"
                        );
                    }
                }
            }

            // Add delay between chunks to avoid overwhelming the server
            if chunk_idx + 1 < network_ids.chunks(self.config.max_concurrent).len() {
                sleep(self.config.batch_delay).await;
            }
        }

        let result = BatchResult::new(successful, failed);
        info!(
            total_processed = result.total_processed,
            successful = result.successful.len(),
            failed = result.failed.len(),
            success_rate = result.success_rate,
            "Completed batch bridges processing"
        );

        result
    }

    /// Process multiple network IDs for claims data in batches
    #[instrument(skip(self, api_config, network_ids))]
    pub async fn get_claims_batch(
        &self,
        api_config: &Config,
        network_ids: Vec<u64>,
    ) -> BatchResult<(u64, Value)> {
        info!(
            count = network_ids.len(),
            max_concurrent = self.config.max_concurrent,
            "Starting batch claims processing"
        );

        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for (chunk_idx, chunk) in network_ids.chunks(self.config.max_concurrent).enumerate() {
            debug!(
                chunk_index = chunk_idx,
                chunk_size = chunk.len(),
                "Processing claims chunk"
            );

            let futures: Vec<_> = chunk
                .iter()
                .enumerate()
                .map(|(idx, &network_id)| {
                    let client = Arc::clone(&self.client);
                    let config = api_config.clone();
                    async move {
                        let result = client.get_claims(&config, network_id).await;
                        (
                            chunk_idx * self.config.max_concurrent + idx,
                            network_id,
                            result,
                        )
                    }
                })
                .collect();

            let results = join_all(futures).await;

            for (original_idx, network_id, result) in results {
                match result {
                    Ok(data) => {
                        successful.push((network_id, data));
                        debug!(network_id = network_id, "Successfully retrieved claims");
                    }
                    Err(e) => {
                        failed.push((original_idx, e));
                        warn!(
                            network_id = network_id,
                            error = %failed.last().unwrap().1,
                            "Failed to retrieve claims"
                        );
                    }
                }
            }

            if chunk_idx + 1 < network_ids.chunks(self.config.max_concurrent).len() {
                sleep(self.config.batch_delay).await;
            }
        }

        let result = BatchResult::new(successful, failed);
        info!(
            total_processed = result.total_processed,
            successful = result.successful.len(),
            failed = result.failed.len(),
            success_rate = result.success_rate,
            "Completed batch claims processing"
        );

        result
    }

    /// Process multiple proof requests in batches
    #[instrument(skip(self, api_config, requests))]
    pub async fn get_proofs_batch(
        &self,
        api_config: &Config,
        requests: Vec<(u64, u64, u64)>, // (network_id, leaf_index, deposit_count)
    ) -> BatchResult<((u64, u64, u64), Value)> {
        info!(
            count = requests.len(),
            max_concurrent = self.config.max_concurrent,
            "Starting batch proof processing"
        );

        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for (chunk_idx, chunk) in requests.chunks(self.config.max_concurrent).enumerate() {
            debug!(
                chunk_index = chunk_idx,
                chunk_size = chunk.len(),
                "Processing proof chunk"
            );

            let futures: Vec<_> = chunk
                .iter()
                .enumerate()
                .map(|(idx, &(network_id, leaf_index, deposit_count))| {
                    let client = Arc::clone(&self.client);
                    let config = api_config.clone();
                    async move {
                        let result = client
                            .get_claim_proof(&config, network_id, leaf_index, deposit_count)
                            .await;
                        (
                            chunk_idx * self.config.max_concurrent + idx,
                            (network_id, leaf_index, deposit_count),
                            result,
                        )
                    }
                })
                .collect();

            let results = join_all(futures).await;

            for (original_idx, request_params, result) in results {
                match result {
                    Ok(data) => {
                        successful.push((request_params, data));
                        debug!(
                            network_id = request_params.0,
                            leaf_index = request_params.1,
                            deposit_count = request_params.2,
                            "Successfully retrieved proof"
                        );
                    }
                    Err(e) => {
                        failed.push((original_idx, e));
                        warn!(
                            network_id = request_params.0,
                            leaf_index = request_params.1,
                            deposit_count = request_params.2,
                            error = %failed.last().unwrap().1,
                            "Failed to retrieve proof"
                        );
                    }
                }
            }

            if chunk_idx + 1 < requests.chunks(self.config.max_concurrent).len() {
                sleep(self.config.batch_delay).await;
            }
        }

        let result = BatchResult::new(successful, failed);
        info!(
            total_processed = result.total_processed,
            successful = result.successful.len(),
            failed = result.failed.len(),
            success_rate = result.success_rate,
            "Completed batch proof processing"
        );

        result
    }

    /// Process requests with automatic retry on failure
    #[instrument(skip(self, api_config, network_ids))]
    pub async fn get_bridges_with_retry(
        &self,
        api_config: &Config,
        network_ids: Vec<u64>,
        max_retries: usize,
    ) -> BatchResult<(u64, Value)> {
        let mut remaining_network_ids = network_ids;
        let mut all_successful = Vec::new();
        let mut all_failed = Vec::new();

        for retry_attempt in 0..=max_retries {
            if remaining_network_ids.is_empty() {
                break;
            }

            info!(
                retry_attempt = retry_attempt,
                remaining_count = remaining_network_ids.len(),
                "Processing batch with retry"
            );

            let result = self
                .get_bridges_batch(api_config, remaining_network_ids.clone())
                .await;

            // Add successful results to our collection
            all_successful.extend(result.successful);

            // Prepare for retry with failed items
            if retry_attempt < max_retries {
                remaining_network_ids = result
                    .failed
                    .into_iter()
                    .filter_map(|(idx, _)| {
                        if idx < remaining_network_ids.len() {
                            Some(remaining_network_ids[idx])
                        } else {
                            None
                        }
                    })
                    .collect();

                if !remaining_network_ids.is_empty() {
                    let retry_delay = Duration::from_millis(500 * (retry_attempt as u64 + 1));
                    info!(
                        retry_delay = ?retry_delay,
                        remaining_count = remaining_network_ids.len(),
                        "Waiting before retry"
                    );
                    sleep(retry_delay).await;
                }
            } else {
                // Last attempt, add remaining failures
                all_failed.extend(result.failed);
            }
        }

        BatchResult::new(all_successful, all_failed)
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.batch_delay, Duration::from_millis(100));
        assert!(config.continue_on_error);
    }

    #[test]
    fn test_batch_result_creation() {
        let successful = vec![1, 2, 3];
        let failed = vec![(
            0,
            crate::error::AggSandboxError::Api(crate::error::ApiError::network_error("test error")),
        )];

        let result = BatchResult::new(successful, failed);
        assert_eq!(result.total_processed, 4);
        assert_eq!(result.success_rate, 0.75);
        assert_eq!(result.successful.len(), 3);
        assert_eq!(result.failed.len(), 1);
    }

    #[test]
    fn test_batch_result_empty() {
        let result: BatchResult<i32> = BatchResult::new(Vec::new(), Vec::new());
        assert_eq!(result.total_processed, 0);
        assert_eq!(result.success_rate, 0.0);
    }

    #[test]
    fn test_batch_processor_creation() {
        let processor = BatchProcessor::new();
        assert_eq!(processor.config.max_concurrent, 10);

        let custom_config = BatchConfig {
            max_concurrent: 5,
            batch_delay: Duration::from_millis(200),
            request_timeout: Duration::from_secs(60),
            continue_on_error: false,
        };

        let custom_processor = BatchProcessor::with_config(custom_config.clone());
        assert_eq!(custom_processor.config.max_concurrent, 5);
        assert_eq!(
            custom_processor.config.batch_delay,
            Duration::from_millis(200)
        );
        assert!(!custom_processor.config.continue_on_error);
    }

    #[tokio::test]
    async fn test_batch_result_success_rate() {
        let successful = vec![(1, serde_json::json!({"test": "data"}))];
        let failed = vec![];

        let result = BatchResult::new(successful, failed);
        assert_eq!(result.success_rate, 1.0);
    }
}
