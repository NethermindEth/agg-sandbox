//! Type-safe wrappers for domain primitives
//!
//! This module provides newtype wrappers around primitive types to prevent
//! type confusion and enable compile-time safety for domain-specific values.

use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Type-safe wrapper for blockchain chain IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainId(String);

impl ChainId {
    /// Create a new ChainId from a string
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id_str = id.into();

        // Validate that it's a valid chain ID (positive integer as string)
        if id_str.trim().is_empty() {
            return Err(ConfigError::invalid_value("chain_id", &id_str, "cannot be empty").into());
        }

        // Try to parse as number to validate format
        id_str.parse::<u64>().map_err(|_| {
            ConfigError::invalid_value("chain_id", &id_str, "must be a valid positive integer")
        })?;

        Ok(ChainId(id_str))
    }

    /// Get the inner string value
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to u64 for numeric operations
    #[allow(dead_code)]
    pub fn as_u64(&self) -> Result<u64> {
        self.0.parse().map_err(|_| {
            ConfigError::invalid_value("chain_id", &self.0, "failed to parse as number").into()
        })
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ChainId {
    type Err = crate::error::AggSandboxError;

    fn from_str(s: &str) -> Result<Self> {
        ChainId::new(s)
    }
}

/// Type-safe wrapper for network IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(u64);

impl NetworkId {
    /// Create a new NetworkId with validation
    pub fn new(id: u64) -> Result<Self> {
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
            .any(|(min, max)| id >= *min && id <= *max);

        if is_valid {
            Ok(NetworkId(id))
        } else {
            Err(ConfigError::invalid_value(
                "network_id",
                &id.to_string(),
                "Must be one of: 0 (Ethereum L1), 1-3 (L2 chains), or 31337-31339 (Local development)",
            )
            .into())
        }
    }

    /// Get the inner u64 value
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Check if this is an L2/L3 network (non-L1)
    pub fn is_l3(&self) -> bool {
        matches!(self.0, 1..=3)
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<NetworkId> for u64 {
    fn from(network_id: NetworkId) -> Self {
        network_id.0
    }
}

/// Type-safe wrapper for Ethereum addresses
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EthereumAddress(String);

impl EthereumAddress {
    /// Create a new EthereumAddress with validation
    pub fn new(address: impl Into<String>) -> Result<Self> {
        let addr_str = address.into();

        // Basic validation - must start with 0x and be 42 chars total
        if !addr_str.starts_with("0x") {
            return Err(ConfigError::invalid_value(
                "ethereum_address",
                &addr_str,
                "must start with 0x",
            )
            .into());
        }

        if addr_str.len() != 42 {
            return Err(ConfigError::invalid_value(
                "ethereum_address",
                &addr_str,
                "must be exactly 42 characters (0x + 40 hex chars)",
            )
            .into());
        }

        // Validate hex characters after 0x
        let hex_part = &addr_str[2..];
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ConfigError::invalid_value(
                "ethereum_address",
                &addr_str,
                "must contain only hexadecimal characters after 0x",
            )
            .into());
        }

        Ok(EthereumAddress(addr_str))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EthereumAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for EthereumAddress {
    type Err = crate::error::AggSandboxError;

    fn from_str(s: &str) -> Result<Self> {
        EthereumAddress::new(s)
    }
}

/// Type-safe wrapper for RPC URLs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcUrl(String);

impl RpcUrl {
    /// Create a new RpcUrl with validation
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let url_str = url.into();

        // Basic URL validation
        if url_str.trim().is_empty() {
            return Err(ConfigError::invalid_value("rpc_url", &url_str, "cannot be empty").into());
        }

        // Must start with http:// or https://
        if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
            return Err(ConfigError::invalid_value(
                "rpc_url",
                &url_str,
                "must start with http:// or https://",
            )
            .into());
        }

        // Validate URL format
        url::Url::parse(&url_str).map_err(|e| {
            ConfigError::invalid_value("rpc_url", &url_str, &format!("invalid URL format: {e}"))
        })?;

        Ok(RpcUrl(url_str))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RpcUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RpcUrl {
    type Err = crate::error::AggSandboxError;

    fn from_str(s: &str) -> Result<Self> {
        RpcUrl::new(s)
    }
}

/// Type-safe wrapper for contract names
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractName(String);

impl ContractName {
    /// Create a new ContractName
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name_str = name.into();

        if name_str.trim().is_empty() {
            return Err(
                ConfigError::invalid_value("contract_name", &name_str, "cannot be empty").into(),
            );
        }

        // Contract names should be valid identifiers
        if !name_str.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ConfigError::invalid_value(
                "contract_name",
                &name_str,
                "must contain only alphanumeric characters and underscores",
            )
            .into());
        }

        Ok(ContractName(name_str))
    }

    /// Get the inner string value
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContractName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ContractName {
    type Err = crate::error::AggSandboxError;

    fn from_str(s: &str) -> Result<Self> {
        ContractName::new(s)
    }
}

// Implement AsRef<str> for easier access to the inner string for coloring
impl AsRef<str> for ChainId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for EthereumAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for RpcUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ContractName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_valid() {
        let chain_id = ChainId::new("1").unwrap();
        assert_eq!(chain_id.as_str(), "1");
        assert_eq!(chain_id.as_u64().unwrap(), 1);
        assert_eq!(chain_id.to_string(), "1");
    }

    #[test]
    fn test_chain_id_invalid() {
        assert!(ChainId::new("").is_err());
        assert!(ChainId::new("abc").is_err());
        assert!(ChainId::new("-1").is_err());
    }

    #[test]
    fn test_network_id_valid() {
        let l1_network = NetworkId::new(0).unwrap();
        assert_eq!(l1_network.as_u64(), 0);
        assert!(!l1_network.is_l3());

        let l2_network = NetworkId::new(1).unwrap();
        assert_eq!(l2_network.as_u64(), 1);
        assert!(l2_network.is_l3());

        let l3_network = NetworkId::new(2).unwrap();
        assert!(l3_network.is_l3());

        let dev_network = NetworkId::new(31337).unwrap();
        assert_eq!(dev_network.as_u64(), 31337);
    }

    #[test]
    fn test_network_id_invalid() {
        assert!(NetworkId::new(4).is_err()); // Beyond L2 range
        assert!(NetworkId::new(137).is_err()); // Old chain ID format
        assert!(NetworkId::new(1101).is_err()); // Old chain ID format
        assert!(NetworkId::new(999).is_err());
        assert!(NetworkId::new(2000).is_err());
    }

    #[test]
    fn test_ethereum_address_valid() {
        let addr = EthereumAddress::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        assert_eq!(addr.as_str(), "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
    }

    #[test]
    fn test_ethereum_address_invalid() {
        assert!(EthereumAddress::new("").is_err());
        assert!(EthereumAddress::new("0x123").is_err()); // too short
        assert!(EthereumAddress::new("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266").is_err()); // missing 0x
        assert!(EthereumAddress::new("0xGGGGd6e51aad88F6F4ce6aB8827279cffFb92266").is_err());
        // invalid hex
    }

    #[test]
    fn test_rpc_url_valid() {
        let url = RpcUrl::new("http://localhost:8545").unwrap();
        assert_eq!(url.as_str(), "http://localhost:8545");

        let https_url = RpcUrl::new("https://api.example.com").unwrap();
        assert_eq!(https_url.as_str(), "https://api.example.com");
    }

    #[test]
    fn test_rpc_url_invalid() {
        assert!(RpcUrl::new("").is_err());
        assert!(RpcUrl::new("ftp://localhost").is_err());
        assert!(RpcUrl::new("not-a-url").is_err());
        assert!(RpcUrl::new("http://").is_err());
    }

    #[test]
    fn test_contract_name_valid() {
        let name = ContractName::new("PolygonZkEVM").unwrap();
        assert_eq!(name.as_str(), "PolygonZkEVM");

        let underscore_name = ContractName::new("Bridge_Extension").unwrap();
        assert_eq!(underscore_name.as_str(), "Bridge_Extension");
    }

    #[test]
    fn test_contract_name_invalid() {
        assert!(ContractName::new("").is_err());
        assert!(ContractName::new("Contract-Name").is_err()); // hyphen not allowed
        assert!(ContractName::new("Contract Name").is_err()); // space not allowed
        assert!(ContractName::new("Contract@Name").is_err()); // special char not allowed
    }

    #[test]
    fn test_serde_serialization() {
        let chain_id = ChainId::new("1101").unwrap();
        let json = serde_json::to_string(&chain_id).unwrap();
        assert_eq!(json, "\"1101\"");

        let deserialized: ChainId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, chain_id);
    }
}
