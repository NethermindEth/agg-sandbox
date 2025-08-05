use crate::error::{ConfigError, EventError, Result};
use regex::Regex;
use url::Url;

/// Supported blockchain networks
#[derive(Debug, Clone, PartialEq)]
pub enum SupportedChain {
    AnvilL1,
    AnvilL2,
    AnvilL3,
}

impl SupportedChain {
    /// Get the string representation of the chain
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedChain::AnvilL1 => "anvil-l1",
            SupportedChain::AnvilL2 => "anvil-l2",
            SupportedChain::AnvilL3 => "anvil-l3",
        }
    }

    /// Get all supported chains as strings
    pub fn all_chains() -> Vec<&'static str> {
        vec!["anvil-l1", "anvil-l2", "anvil-l3"]
    }
}

/// Validation functions for user inputs
pub struct Validator;

impl Validator {
    /// Validate and sanitize chain name
    pub fn validate_chain(chain: &str) -> Result<SupportedChain> {
        let sanitized = Self::sanitize_chain_name(chain);

        match sanitized.as_str() {
            "anvil-l1" => Ok(SupportedChain::AnvilL1),
            "anvil-l2" => Ok(SupportedChain::AnvilL2),
            "anvil-l3" => Ok(SupportedChain::AnvilL3),
            _ => Err(EventError::invalid_chain(&format!(
                "Invalid chain '{chain}'. Supported chains: {}",
                SupportedChain::all_chains().join(", ")
            ))
            .into()),
        }
    }

    /// Sanitize chain name input
    fn sanitize_chain_name(chain: &str) -> String {
        chain.trim().to_lowercase()
    }

    /// Validate network ID for API calls
    pub fn validate_network_id(network_id: u64) -> Result<u64> {
        // Define valid Agglayer network ID ranges
        // 0 = Ethereum L1
        // 1 = First L2 connected to Agglayer
        // 2 = Second L2 (if multi-L2 setup)
        // 3+ = Additional L2 chains
        let valid_ranges = [
            (0, 3),         // Agglayer network IDs: 0 (L1), 1-3 (L2 chains)
            (31337, 31339), // Local development networks (for testing)
        ];

        let is_valid = valid_ranges
            .iter()
            .any(|(min, max)| network_id >= *min && network_id <= *max);

        if is_valid {
            Ok(network_id)
        } else {
            Err(ConfigError::invalid_value(
                "network_id",
                &network_id.to_string(),
                "Must be one of: 0 (Ethereum L1), 1-3 (L2 chains), or 31337-31339 (Local development)",
            )
            .into())
        }
    }

    /// Validate Ethereum address format
    pub fn validate_ethereum_address(address: &str) -> Result<String> {
        let sanitized = Self::sanitize_address(address);

        // Ethereum addresses are 42 characters (0x + 40 hex chars)
        if sanitized.len() != 42 {
            return Err(EventError::invalid_address(&format!(
                "Address '{address}' must be 42 characters long (0x + 40 hex digits)"
            ))
            .into());
        }

        // Must start with 0x
        if !sanitized.starts_with("0x") {
            return Err(EventError::invalid_address(&format!(
                "Address '{address}' must start with '0x'"
            ))
            .into());
        }

        // Must contain only hex characters after 0x
        let hex_part = &sanitized[2..];
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(EventError::invalid_address(&format!(
                "Address '{address}' contains invalid hex characters"
            ))
            .into());
        }

        Ok(sanitized)
    }

    /// Sanitize Ethereum address input
    fn sanitize_address(address: &str) -> String {
        address.trim().to_lowercase()
    }

    /// Validate block number range
    pub fn validate_block_count(blocks: u64) -> Result<u64> {
        const MAX_BLOCKS: u64 = 10000; // Reasonable limit to prevent excessive queries
        const MIN_BLOCKS: u64 = 1;

        if blocks < MIN_BLOCKS {
            return Err(ConfigError::invalid_value(
                "blocks",
                &blocks.to_string(),
                &format!("Must be at least {MIN_BLOCKS}"),
            )
            .into());
        }

        if blocks > MAX_BLOCKS {
            return Err(ConfigError::invalid_value(
                "blocks",
                &blocks.to_string(),
                &format!("Must not exceed {MAX_BLOCKS} to prevent excessive load"),
            )
            .into());
        }

        Ok(blocks)
    }

    /// Validate RPC URL format
    #[allow(dead_code)]
    pub fn validate_rpc_url(url: &str) -> Result<String> {
        let sanitized = Self::sanitize_url(url);

        // Parse URL to validate format
        let parsed_url = Url::parse(&sanitized).map_err(|_| {
            ConfigError::invalid_value("rpc_url", url, "Must be a valid URL (http:// or https://)")
        })?;

        // Ensure it's HTTP or HTTPS
        match parsed_url.scheme() {
            "http" | "https" => Ok(sanitized),
            _ => Err(ConfigError::invalid_value(
                "rpc_url",
                url,
                "URL scheme must be http:// or https://",
            )
            .into()),
        }
    }

    /// Sanitize URL input
    #[allow(dead_code)]
    fn sanitize_url(url: &str) -> String {
        url.trim().to_string()
    }

    /// Validate file path for security (prevent path traversal)
    #[allow(dead_code)]
    pub fn validate_file_path(path: &str) -> Result<String> {
        let sanitized = Self::sanitize_file_path(path);

        // Check for path traversal attempts
        if sanitized.contains("..") || sanitized.contains("//") {
            return Err(ConfigError::invalid_value(
                "file_path",
                path,
                "Path contains invalid sequences (.. or //)",
            )
            .into());
        }

        // Ensure it's a relative path (no absolute paths for security)
        if sanitized.starts_with('/') || sanitized.contains(':') {
            return Err(ConfigError::invalid_value(
                "file_path",
                path,
                "Absolute paths are not allowed",
            )
            .into());
        }

        // Validate allowed file extensions for compose files
        let allowed_extensions = ["yml", "yaml"];
        if let Some(extension) = sanitized.split('.').next_back() {
            if !allowed_extensions.contains(&extension) {
                return Err(ConfigError::invalid_value(
                    "file_path",
                    path,
                    &format!(
                        "File extension must be one of: {}",
                        allowed_extensions.join(", ")
                    ),
                )
                .into());
            }
        }

        Ok(sanitized)
    }

    /// Sanitize file path input
    #[allow(dead_code)]
    fn sanitize_file_path(path: &str) -> String {
        path.trim().to_string()
    }

    /// Validate environment variable name
    #[allow(dead_code)]
    #[allow(clippy::disallowed_methods)] // Allow unwrap for hardcoded regex
    pub fn validate_env_var_name(name: &str) -> Result<String> {
        let sanitized = Self::sanitize_env_var_name(name);

        // Environment variable names should only contain uppercase letters, numbers, and underscores
        let env_regex = Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap(); // Safe: hardcoded regex pattern is always valid

        if !env_regex.is_match(&sanitized) {
            return Err(ConfigError::invalid_value(
                "env_var_name",
                name,
                "Must start with a letter and contain only uppercase letters, numbers, and underscores"
            ).into());
        }

        Ok(sanitized)
    }

    /// Sanitize environment variable name
    #[allow(dead_code)]
    fn sanitize_env_var_name(name: &str) -> String {
        name.trim().to_uppercase()
    }

    /// Validate service name for Docker operations
    #[allow(clippy::disallowed_methods)] // Allow unwrap for hardcoded regex
    pub fn validate_service_name(service: &str) -> Result<String> {
        let sanitized = Self::sanitize_service_name(service);

        // Docker service names should be alphanumeric with hyphens and underscores
        let service_regex = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_-]*$").unwrap(); // Safe: hardcoded regex pattern is always valid

        if sanitized.is_empty() {
            return Err(ConfigError::invalid_value(
                "service_name",
                service,
                "Service name cannot be empty",
            )
            .into());
        }

        if !service_regex.is_match(&sanitized) {
            return Err(ConfigError::invalid_value(
                "service_name",
                service,
                "Must start with alphanumeric character and contain only letters, numbers, hyphens, and underscores"
            ).into());
        }

        Ok(sanitized)
    }

    /// Sanitize service name
    fn sanitize_service_name(service: &str) -> String {
        service.trim().to_string()
    }

    /// Validate timeout value in milliseconds
    pub fn validate_timeout_ms(timeout_ms: u64) -> Result<u64> {
        const MIN_TIMEOUT: u64 = 1000; // 1 second minimum
        const MAX_TIMEOUT: u64 = 600000; // 10 minutes maximum

        if timeout_ms < MIN_TIMEOUT {
            return Err(ConfigError::invalid_value(
                "timeout_ms",
                &timeout_ms.to_string(),
                &format!("Must be at least {MIN_TIMEOUT} ms (1 second)"),
            )
            .into());
        }

        if timeout_ms > MAX_TIMEOUT {
            return Err(ConfigError::invalid_value(
                "timeout_ms",
                &timeout_ms.to_string(),
                &format!("Must not exceed {MAX_TIMEOUT} ms (10 minutes)"),
            )
            .into());
        }

        Ok(timeout_ms)
    }

    /// Validate retry attempts count
    pub fn validate_retry_attempts(attempts: u32) -> Result<u32> {
        const MAX_RETRIES: u32 = 10;

        if attempts > MAX_RETRIES {
            return Err(ConfigError::invalid_value(
                "retry_attempts",
                &attempts.to_string(),
                &format!("Must not exceed {MAX_RETRIES} retries"),
            )
            .into());
        }

        Ok(attempts)
    }

    /// Batch validate multiple inputs with early termination on first error
    #[allow(dead_code)]
    pub fn validate_batch<T, F>(items: Vec<T>, validator: F) -> Result<Vec<T>>
    where
        F: Fn(&T) -> Result<T>,
        T: Clone,
    {
        let mut validated = Vec::new();

        for item in items {
            let validated_item = validator(&item)?;
            validated.push(validated_item);
        }

        Ok(validated)
    }
}

/// Input sanitization utilities
#[allow(dead_code)]
pub struct Sanitizer;

impl Sanitizer {
    /// Remove potentially dangerous characters from user input
    #[allow(dead_code)]
    pub fn sanitize_user_input(input: &str) -> String {
        // Remove null bytes, control characters, and excessive whitespace
        let cleaned: String = input
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect();

        // Normalize whitespace
        let normalized = cleaned.split_whitespace().collect::<Vec<&str>>().join(" ");

        // Trim and limit length
        let max_length = 1000; // Reasonable limit for most inputs
        if normalized.len() > max_length {
            normalized[..max_length].to_string()
        } else {
            normalized
        }
    }

    /// Sanitize file names to prevent directory traversal
    #[allow(dead_code)]
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_'))
            .collect()
    }

    /// Sanitize log output to prevent log injection
    #[allow(dead_code)]
    pub fn sanitize_log_message(message: &str) -> String {
        message
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_chain_valid() {
        assert_eq!(
            Validator::validate_chain("anvil-l1").unwrap(),
            SupportedChain::AnvilL1
        );
        assert_eq!(
            Validator::validate_chain("ANVIL-L2").unwrap(),
            SupportedChain::AnvilL2
        );
        assert_eq!(
            Validator::validate_chain("  anvil-l3  ").unwrap(),
            SupportedChain::AnvilL3
        );
    }

    #[test]
    fn test_validate_chain_invalid() {
        assert!(Validator::validate_chain("invalid-chain").is_err());
        assert!(Validator::validate_chain("ethereum").is_err());
        assert!(Validator::validate_chain("").is_err());
    }

    #[test]
    fn test_validate_network_id_valid() {
        assert_eq!(Validator::validate_network_id(0).unwrap(), 0); // L1 Ethereum
        assert_eq!(Validator::validate_network_id(1).unwrap(), 1); // First L2
        assert_eq!(Validator::validate_network_id(2).unwrap(), 2); // Second L2
        assert_eq!(Validator::validate_network_id(3).unwrap(), 3); // Third L2
        assert_eq!(Validator::validate_network_id(31337).unwrap(), 31337); // Local dev
    }

    #[test]
    fn test_validate_network_id_invalid() {
        assert!(Validator::validate_network_id(4).is_err()); // Beyond L2 range
        assert!(Validator::validate_network_id(137).is_err()); // Old chain ID format
        assert!(Validator::validate_network_id(1101).is_err()); // Old chain ID format
        assert!(Validator::validate_network_id(999).is_err()); // Invalid range
        assert!(Validator::validate_network_id(2000).is_err()); // Invalid range
    }

    #[test]
    fn test_validate_ethereum_address_valid() {
        let valid_address = "0x742d35cc6965c592342c6c16fb8eaeb90a23b5c0";
        assert_eq!(
            Validator::validate_ethereum_address(valid_address).unwrap(),
            valid_address
        );

        // Test with uppercase
        let uppercase_address = "0x742D35CC6965C592342C6C16FB8EAEB90A23B5C0";
        assert_eq!(
            Validator::validate_ethereum_address(uppercase_address).unwrap(),
            valid_address // Should be normalized to lowercase
        );
    }

    #[test]
    fn test_validate_ethereum_address_invalid() {
        // Too short
        assert!(Validator::validate_ethereum_address("0x123").is_err());

        // Missing 0x prefix
        assert!(
            Validator::validate_ethereum_address("742d35cc6965c592342c6c16fb8eaeb90a23b5c0")
                .is_err()
        );

        // Invalid hex characters
        assert!(
            Validator::validate_ethereum_address("0x742d35cc6965c592342c6c16fb8eaeb90a23b5cg")
                .is_err()
        );
    }

    #[test]
    fn test_validate_block_count_valid() {
        assert_eq!(Validator::validate_block_count(1).unwrap(), 1);
        assert_eq!(Validator::validate_block_count(100).unwrap(), 100);
        assert_eq!(Validator::validate_block_count(10000).unwrap(), 10000);
    }

    #[test]
    fn test_validate_block_count_invalid() {
        assert!(Validator::validate_block_count(0).is_err());
        assert!(Validator::validate_block_count(10001).is_err());
    }

    #[test]
    fn test_validate_rpc_url_valid() {
        assert!(Validator::validate_rpc_url("http://localhost:8545").is_ok());
        assert!(Validator::validate_rpc_url("https://rpc.example.com").is_ok());
    }

    #[test]
    fn test_validate_rpc_url_invalid() {
        assert!(Validator::validate_rpc_url("ftp://example.com").is_err());
        assert!(Validator::validate_rpc_url("not-a-url").is_err());
        assert!(Validator::validate_rpc_url("").is_err());
    }

    #[test]
    fn test_validate_file_path_valid() {
        assert!(Validator::validate_file_path("docker-compose.yml").is_ok());
        assert!(Validator::validate_file_path("configs/test.yaml").is_ok());
    }

    #[test]
    fn test_validate_file_path_invalid() {
        assert!(Validator::validate_file_path("../etc/passwd").is_err());
        assert!(Validator::validate_file_path("/absolute/path.yml").is_err());
        assert!(Validator::validate_file_path("file.txt").is_err()); // Wrong extension
    }

    #[test]
    fn test_validate_env_var_name_valid() {
        assert_eq!(
            Validator::validate_env_var_name("API_BASE_URL").unwrap(),
            "API_BASE_URL"
        );
        assert_eq!(
            Validator::validate_env_var_name("fork_url_mainnet").unwrap(),
            "FORK_URL_MAINNET"
        );
    }

    #[test]
    fn test_validate_env_var_name_invalid() {
        assert!(Validator::validate_env_var_name("123_INVALID").is_err());
        assert!(Validator::validate_env_var_name("invalid-name").is_err());
        assert!(Validator::validate_env_var_name("").is_err());
    }

    #[test]
    fn test_validate_service_name_valid() {
        assert!(Validator::validate_service_name("anvil-l1").is_ok());
        assert!(Validator::validate_service_name("service_name").is_ok());
        assert!(Validator::validate_service_name("service123").is_ok());
    }

    #[test]
    fn test_validate_service_name_invalid() {
        assert!(Validator::validate_service_name("").is_err());
        assert!(Validator::validate_service_name("-invalid").is_err());
        assert!(Validator::validate_service_name("invalid@service").is_err());
    }

    #[test]
    fn test_validate_timeout_ms_valid() {
        assert_eq!(Validator::validate_timeout_ms(5000).unwrap(), 5000);
        assert_eq!(Validator::validate_timeout_ms(1000).unwrap(), 1000);
        assert_eq!(Validator::validate_timeout_ms(600000).unwrap(), 600000);
    }

    #[test]
    fn test_validate_timeout_ms_invalid() {
        assert!(Validator::validate_timeout_ms(500).is_err()); // Too low
        assert!(Validator::validate_timeout_ms(700000).is_err()); // Too high
    }

    #[test]
    fn test_validate_retry_attempts_valid() {
        assert_eq!(Validator::validate_retry_attempts(3).unwrap(), 3);
        assert_eq!(Validator::validate_retry_attempts(0).unwrap(), 0);
        assert_eq!(Validator::validate_retry_attempts(10).unwrap(), 10);
    }

    #[test]
    fn test_validate_retry_attempts_invalid() {
        assert!(Validator::validate_retry_attempts(11).is_err());
    }

    #[test]
    fn test_sanitize_user_input() {
        assert_eq!(
            Sanitizer::sanitize_user_input("  hello   world  \n"),
            "hello world"
        );
        assert_eq!(Sanitizer::sanitize_user_input("test\x00input"), "testinput");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            Sanitizer::sanitize_filename("file@name#.txt"),
            "filename.txt"
        );
        assert_eq!(
            Sanitizer::sanitize_filename("valid-file_123.yml"),
            "valid-file_123.yml"
        );
    }

    #[test]
    fn test_sanitize_log_message() {
        assert_eq!(
            Sanitizer::sanitize_log_message("message\nwith\nnewlines"),
            "message\\nwith\\nnewlines"
        );
        assert_eq!(Sanitizer::sanitize_log_message("tab\there"), "tab\\there");
    }

    #[test]
    fn test_validate_batch() {
        let network_ids = vec![0u64, 1u64, 2u64];

        let result = Validator::validate_batch(network_ids.clone(), |&id| {
            Validator::validate_network_id(id)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);

        // Test with invalid network ID
        let invalid_ids = vec![1u64, 999u64];

        let result =
            Validator::validate_batch(invalid_ids, |&id| Validator::validate_network_id(id));

        assert!(result.is_err());
    }
}
