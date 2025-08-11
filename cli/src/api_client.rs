use crate::config::Config;
use crate::error::{ApiError, Result};
use crate::types::NetworkId;
use dashmap::DashMap;
use lru::LruCache;
use reqwest::{Client, ClientBuilder};
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Cache entry that includes the cached data and its expiration time
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

/// Cache key for API requests
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub endpoint: String,
    pub network_id: Option<u64>,
    pub leaf_index: Option<u64>,
    pub deposit_count: Option<u64>,
}

impl CacheKey {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            network_id: None,
            leaf_index: None,
            deposit_count: None,
        }
    }

    pub fn with_network_id(mut self, network_id: u64) -> Self {
        self.network_id = Some(network_id);
        self
    }

    pub fn with_leaf_index(mut self, leaf_index: u64) -> Self {
        self.leaf_index = Some(leaf_index);
        self
    }

    pub fn with_deposit_count(mut self, deposit_count: u64) -> Self {
        self.deposit_count = Some(deposit_count);
        self
    }
}

/// Configuration for the API client cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Default TTL for cache entries
    pub default_ttl: Duration,
    /// TTL for bridges endpoint
    pub bridges_ttl: Duration,
    /// TTL for claims endpoint
    pub claims_ttl: Duration,
    /// TTL for proof endpoints
    pub proof_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            bridges_ttl: Duration::from_secs(180), // 3 minutes
            claims_ttl: Duration::from_secs(120),  // 2 minutes
            proof_ttl: Duration::from_secs(600),   // 10 minutes (proofs are more stable)
        }
    }
}

/// Response cache using LRU eviction policy
type ResponseCache = Arc<RwLock<LruCache<CacheKey, CacheEntry<serde_json::Value>>>>;

/// Statistics for cache performance monitoring
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub expired: u64,
    pub evictions: u64,
}

impl CacheStats {
    #[allow(dead_code)]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// High-performance API client with caching and connection pooling
pub struct OptimizedApiClient {
    client: Client,
    cache: ResponseCache,
    cache_config: CacheConfig,
    stats: Arc<DashMap<String, CacheStats>>,
}

/// Global client instance for reuse across API calls
static GLOBAL_CLIENT: LazyLock<Arc<OptimizedApiClient>> =
    LazyLock::new(|| Arc::new(OptimizedApiClient::new(CacheConfig::default())));

impl OptimizedApiClient {
    /// Create a new optimized API client (fallible version)
    pub fn try_new(cache_config: CacheConfig) -> crate::error::Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            .gzip(true)
            .brotli(true)
            .build()
            .map_err(|e| {
                crate::error::AggSandboxError::Api(crate::error::ApiError::NetworkError(format!(
                    "Failed to create HTTP client: {e}"
                )))
            })?;

        let cache_size = NonZeroUsize::new(cache_config.max_entries).ok_or_else(|| {
            crate::error::AggSandboxError::Config(crate::error::ConfigError::invalid_value(
                "cache_max_entries",
                &cache_config.max_entries.to_string(),
                "Cache size must be greater than 0",
            ))
        })?;
        let cache = Arc::new(RwLock::new(LruCache::new(cache_size)));

        Ok(Self {
            client,
            cache,
            cache_config,
            stats: Arc::new(DashMap::new()),
        })
    }

    /// Create a new optimized API client (panics on error, for global client)
    pub fn new(cache_config: CacheConfig) -> Self {
        match Self::try_new(cache_config) {
            Ok(client) => client,
            Err(e) => {
                // For global client, we need to panic as there's no way to return an error
                // This should never happen with default config
                panic!("Failed to create global API client: {e}");
            }
        }
    }

    /// Get the global shared client instance
    pub fn global() -> Arc<OptimizedApiClient> {
        GLOBAL_CLIENT.clone()
    }

    /// Get cache statistics for monitoring
    #[allow(dead_code)]
    pub fn get_cache_stats(&self, endpoint: &str) -> CacheStats {
        self.stats
            .get(endpoint)
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }

    /// Clear the cache (useful for testing or manual cache invalidation)
    #[allow(dead_code, clippy::disallowed_methods)] // Allow for tracing macro expansion
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Cache cleared");
    }

    /// Get TTL for a specific endpoint
    fn get_ttl_for_endpoint(&self, endpoint: &str) -> Duration {
        match endpoint {
            "bridges" => self.cache_config.bridges_ttl,
            "claims" => self.cache_config.claims_ttl,
            "claim-proof" | "l1-info-tree-index" => self.cache_config.proof_ttl,
            _ => self.cache_config.default_ttl,
        }
    }

    /// Check if a cache entry is expired
    fn is_expired(entry: &CacheEntry<serde_json::Value>) -> bool {
        Instant::now() > entry.expires_at
    }

    /// Update cache statistics
    fn update_stats(&self, endpoint: &str, hit: bool, expired: bool) {
        let mut stats = self.stats.entry(endpoint.to_string()).or_default();
        if hit {
            if expired {
                stats.expired += 1;
                stats.misses += 1;
            } else {
                stats.hits += 1;
            }
        } else {
            stats.misses += 1;
        }
    }

    /// Get data from cache or fetch from API
    #[allow(clippy::disallowed_methods)] // Allow for tracing macro expansion
    #[instrument(fields(cache_key = ?cache_key), skip(self, fetch_fn))]
    pub async fn get_cached_or_fetch<F, Fut>(
        &self,
        cache_key: CacheKey,
        fetch_fn: F,
    ) -> Result<serde_json::Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<serde_json::Value>>,
    {
        let endpoint = cache_key.endpoint.clone();

        // Try to get from cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.peek(&cache_key) {
                if !Self::is_expired(entry) {
                    debug!(cache_key = ?cache_key, "Cache hit");
                    self.update_stats(&endpoint, true, false);
                    return Ok(entry.data.clone());
                } else {
                    debug!(cache_key = ?cache_key, "Cache entry expired");
                    self.update_stats(&endpoint, true, true);
                }
            } else {
                debug!(cache_key = ?cache_key, "Cache miss");
                self.update_stats(&endpoint, false, false);
            }
        }

        // Fetch from API
        debug!(cache_key = ?cache_key, "Fetching from API");
        let data = fetch_fn().await?;

        // Store in cache
        {
            let mut cache = self.cache.write().await;
            let ttl = self.get_ttl_for_endpoint(&endpoint);
            let expires_at = Instant::now() + ttl;

            if let Some(_evicted) = cache.put(
                cache_key.clone(),
                CacheEntry {
                    data: data.clone(),
                    expires_at,
                },
            ) {
                debug!(cache_key = ?cache_key, "Cache entry evicted due to capacity");
                self.stats.entry(endpoint).or_default().evictions += 1;
            }

            debug!(
                cache_key = ?cache_key,
                ttl = ?ttl,
                "Cached response"
            );
        }

        Ok(data)
    }

    /// Make an HTTP GET request with specified timeout
    #[allow(clippy::disallowed_methods)] // Allow for tracing macro expansion
    #[instrument(fields(url = %url), skip(self))]
    pub async fn get_with_timeout(
        &self,
        url: &str,
        timeout: Duration,
    ) -> Result<serde_json::Value> {
        debug!(url = %url, "Making HTTP GET request");

        let response = self
            .client
            .get(url)
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| {
                warn!(url = %url, error = %e, "HTTP request failed");
                ApiError::network_error(&e.to_string())
            })?;

        let status = response.status();
        debug!(url = %url, status = %status, "Received HTTP response");

        if !status.is_success() {
            warn!(
                url = %url,
                status = %status,
                "API request failed with non-success status"
            );
            return Err(
                ApiError::request_failed(url, status.as_u16(), "API request failed").into(),
            );
        }

        let data: serde_json::Value = response.json().await.map_err(|e| {
            warn!(url = %url, error = %e, "Failed to parse JSON response");
            ApiError::json_parse_error(&e.to_string())
        })?;

        debug!(
            url = %url,
            size = data.to_string().len(),
            "Successfully parsed JSON response"
        );

        Ok(data)
    }

    /// Generic POST helper that sends JSON and parses the JSON response.
    async fn post_json_with_timeout(
        &self,
        url: &str,
        body: &serde_json::Value,
        timeout: Duration,
    ) -> Result<serde_json::Value> {
        debug!(url, "HTTP POST request");

        let response = self
            .client
            .post(url)
            .json(body)
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| {
                warn!(url = %url, error = %e, "HTTP request failed");
                ApiError::network_error(&e.to_string())
            })?;

        let status = response.status();
        debug!(url = %url, status = %status, "Received HTTP response");

        if !status.is_success() {
            warn!(
                url = %url,
                status = %status,
                "POST request failed with non-success status"
            );
            return Err(
                ApiError::request_failed(url, status.as_u16(), "POST request failed").into(),
            );
        }

        let data: serde_json::Value = response.json().await.map_err(|e| {
            warn!(url = %url, error = %e, "Failed to parse JSON response");
            ApiError::json_parse_error(&e.to_string())
        })?;

        debug!(
            url = %url,
            size = data.to_string().len(),
            "Successfully parsed JSON response"
        );

        Ok(data)
    }

    /// Get bridges with caching
    #[allow(clippy::disallowed_methods)] // Allow for tracing macro expansion
    #[instrument(fields(network_id = network_id), skip(self, config))]
    pub async fn get_bridges(&self, config: &Config, network_id: u64) -> Result<serde_json::Value> {
        let cache_key = CacheKey::new("bridges".to_string()).with_network_id(network_id);

        let base_url = config.get_api_base_url(NetworkId::new(network_id)?);
        let url = format!("{base_url}/bridge/v1/bridges?network_id={network_id}");

        let timeout = config.api.timeout;

        self.get_cached_or_fetch(cache_key, || async {
            self.get_with_timeout(&url, timeout).await
        })
        .await
    }

    /// Get claims with caching
    #[allow(clippy::disallowed_methods)] // Allow for tracing macro expansion
    #[instrument(fields(network_id = network_id), skip(self, config))]
    pub async fn get_claims(&self, config: &Config, network_id: u64) -> Result<serde_json::Value> {
        let cache_key = CacheKey::new("claims".to_string()).with_network_id(network_id);

        let base_url = config.get_api_base_url(NetworkId::new(network_id)?);
        let url = format!("{base_url}/bridge/v1/claims?network_id={network_id}");

        let timeout = config.api.timeout;

        self.get_cached_or_fetch(cache_key, || async {
            self.get_with_timeout(&url, timeout).await
        })
        .await
    }

    /// Get claim proof with caching
    #[allow(clippy::disallowed_methods)] // Allow for tracing macro expansion
    #[instrument(
        fields(network_id = network_id, leaf_index = leaf_index, deposit_count = deposit_count),
        skip(self, config)
    )]
    pub async fn get_claim_proof(
        &self,
        config: &Config,
        network_id: u64,
        leaf_index: u64,
        deposit_count: u64,
    ) -> Result<serde_json::Value> {
        let cache_key = CacheKey::new("claim-proof".to_string())
            .with_network_id(network_id)
            .with_leaf_index(leaf_index)
            .with_deposit_count(deposit_count);

        let base_url = config.get_api_base_url(NetworkId::new(network_id)?);
        let url = format!("{base_url}/bridge/v1/claim-proof?network_id={network_id}&leaf_index={leaf_index}&deposit_count={deposit_count}");

        let timeout = config.api.timeout;

        self.get_cached_or_fetch(cache_key, || async {
            self.get_with_timeout(&url, timeout).await
        })
        .await
    }

    /// Get L1 info tree index with caching
    #[allow(clippy::disallowed_methods)] // Allow for tracing macro expansion
    #[instrument(
        fields(network_id = network_id, deposit_count = deposit_count),
        skip(self, config)
    )]
    pub async fn get_l1_info_tree_index(
        &self,
        config: &Config,
        network_id: u64,
        deposit_count: u64,
    ) -> Result<serde_json::Value> {
        let cache_key = CacheKey::new("l1-info-tree-index".to_string())
            .with_network_id(network_id)
            .with_deposit_count(deposit_count);

        let base_url = config.get_api_base_url(NetworkId::new(network_id)?);
        let url = format!("{base_url}/bridge/v1/l1-info-tree-index?network_id={network_id}&deposit_count={deposit_count}");

        let timeout = config.api.timeout;

        self.get_cached_or_fetch(cache_key, || async {
            self.get_with_timeout(&url, timeout).await
        })
        .await
    }
    /// Call POST /bridge/v1/sponsor-claim (no caching).
    pub async fn post_sponsor_claim(
        &self,
        config: &Config,
        body: &serde_json::Value,
        network_id: u64,
    ) -> Result<serde_json::Value> {
        let base_url = config.get_api_base_url(NetworkId::new(network_id)?);
        let url = format!("{base_url}/bridge/v1/sponsor-claim");
        let timeout = config.api.timeout;
        self.post_json_with_timeout(&url, body, timeout).await
    }

    /// Get a sponsored claim status (no caching)
    pub async fn get_sponsored_claim_status(
        &self,
        config: &Config,
        global_index: u64,
        network_id: u64,
    ) -> Result<serde_json::Value> {
        let base_url = config.get_api_base_url(NetworkId::new(network_id)?);
        let url = format!("{base_url}/bridge/v1/sponsored-claim-status?global_index={global_index}&network_id={network_id}");
        let timeout = config.api.timeout;
        self.get_with_timeout(&url, timeout).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_creation() {
        let key = CacheKey::new("test-endpoint".to_string())
            .with_network_id(1)
            .with_leaf_index(10)
            .with_deposit_count(5);

        assert_eq!(key.endpoint, "test-endpoint");
        assert_eq!(key.network_id, Some(1));
        assert_eq!(key.leaf_index, Some(10));
        assert_eq!(key.deposit_count, Some(5));
    }

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            hits: 80,
            misses: 20,
            expired: 5,
            evictions: 2,
        };

        assert_eq!(stats.hit_rate(), 0.8);
    }

    #[test]
    fn test_cache_stats_empty() {
        let stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let _client = OptimizedApiClient::new(CacheConfig::default());

        // Create an expired entry
        let expired_entry = CacheEntry {
            data: serde_json::json!({"test": "data"}),
            expires_at: Instant::now() - Duration::from_secs(1),
        };

        assert!(OptimizedApiClient::is_expired(&expired_entry));

        // Create a non-expired entry
        let valid_entry = CacheEntry {
            data: serde_json::json!({"test": "data"}),
            expires_at: Instant::now() + Duration::from_secs(60),
        };

        assert!(!OptimizedApiClient::is_expired(&valid_entry));
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let client = OptimizedApiClient::new(CacheConfig::default());

        // Add an entry to cache
        {
            let mut cache = client.cache.write().await;
            let key = CacheKey::new("test".to_string());
            let entry = CacheEntry {
                data: serde_json::json!({"test": "data"}),
                expires_at: Instant::now() + Duration::from_secs(60),
            };
            cache.put(key, entry);
            assert_eq!(cache.len(), 1);
        }

        // Clear cache
        client.clear_cache().await;

        // Verify cache is empty
        {
            let cache = client.cache.read().await;
            assert_eq!(cache.len(), 0);
        }
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_entries, 1000);
        assert_eq!(config.default_ttl, Duration::from_secs(300));
        assert!(config.bridges_ttl < config.proof_ttl);
    }

    #[test]
    fn test_ttl_for_endpoint() {
        let client = OptimizedApiClient::new(CacheConfig::default());

        assert_eq!(
            client.get_ttl_for_endpoint("bridges"),
            client.cache_config.bridges_ttl
        );
        assert_eq!(
            client.get_ttl_for_endpoint("claims"),
            client.cache_config.claims_ttl
        );
        assert_eq!(
            client.get_ttl_for_endpoint("claim-proof"),
            client.cache_config.proof_ttl
        );
        assert_eq!(
            client.get_ttl_for_endpoint("unknown"),
            client.cache_config.default_ttl
        );
    }
}
