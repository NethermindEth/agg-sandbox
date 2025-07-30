//! Common utilities and shared functionality for bridge operations
//!
//! This module contains shared validation, formatting, error handling, and
//! contract interaction utilities used across all bridge commands.

use crate::config::Config;
use crate::error::Result;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Validate Ethereum address format
pub fn validate_address(address: &str, field_name: &str) -> Result<Address> {
    if address.is_empty() {
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!("{field_name} cannot be empty")),
        ));
    }

    if !address.starts_with("0x") {
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!(
                "{field_name} must start with '0x'"
            )),
        ));
    }

    if address.len() != 42 {
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(&format!(
                "{field_name} must be 42 characters long (including '0x')"
            )),
        ));
    }

    Address::from_str(address).map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Invalid {field_name} format: {e}"),
        ))
    })
}

/// Validate network ID
pub fn validate_network_id(network_id: u64, field_name: &str) -> Result<()> {
    if network_id > 2 {
        return Err(crate::error::AggSandboxError::Config(
            crate::error::ConfigError::validation_failed(
                &format!("{field_name} must be 0 (Mainnet), 1 (AggLayer-1), or 2 (AggLayer-2), got: {network_id}"),
            ),
        ));
    }
    Ok(())
}

/// Get network display name
pub fn get_network_name(network_id: u64) -> &'static str {
    match network_id {
        0 => "Mainnet",
        1 => "AggLayer-1",
        2 => "AggLayer-2",
        _ => "Unknown",
    }
}

/// Create validation error with consistent formatting
pub fn validation_error(message: &str) -> crate::error::AggSandboxError {
    crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(message))
}

/// Serialize JSON output with error handling
pub fn serialize_json<T: Serialize>(data: &T) -> Result<String> {
    serde_json::to_string_pretty(data)
        .map_err(|e| validation_error(&format!("Failed to serialize output to JSON: {e}")))
}

/// Table formatting utilities
pub mod table {
    /// Print table header
    pub fn print_header(title: &str) {
        println!("{title}");
        println!("┌────────────────────────┬─────────────────────────────────────────────┐");
    }

    /// Print table row
    pub fn print_row(key: &str, value: &str) {
        println!("│ {key:<22} │ {value:<43} │");
    }

    /// Print table footer
    pub fn print_footer() {
        println!("└────────────────────────┴─────────────────────────────────────────────┘");
    }

    /// Print complete table with data
    pub fn print_table(title: &str, rows: &[(&str, &str)]) {
        print_header(title);
        for (key, value) in rows {
            print_row(key, value);
        }
        print_footer();
    }
}

/// Contract interaction utilities
pub mod contract {
    use super::*;
    use crate::commands::bridge::{
        get_bridge_contract_address, get_wallet_with_provider, BridgeContract,
    };
    use ethers::middleware::SignerMiddleware;
    use ethers::providers::{Http, Provider};
    use ethers::signers::LocalWallet;
    use std::sync::Arc;

    /// Get bridge contract instance with validation
    pub async fn get_bridge_contract(
        config: &Config,
        network: u64,
        private_key: Option<&str>,
    ) -> Result<BridgeContract<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>> {
        validate_network_id(network, "Network")?;
        let client = get_wallet_with_provider(config, network, private_key).await?;
        let bridge_address = get_bridge_contract_address(config, network)?;
        Ok(BridgeContract::new(bridge_address, client.into()))
    }
}

/// JSON output structures for utilities
#[derive(Debug, Serialize, Deserialize)]
pub struct UtilityOutput<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_address_valid() {
        let result = validate_address("0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC", "Test address");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_address_invalid_format() {
        let result = validate_address("invalid", "Test address");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_network_id_valid() {
        assert!(validate_network_id(0, "Network").is_ok());
        assert!(validate_network_id(1, "Network").is_ok());
        assert!(validate_network_id(2, "Network").is_ok());
    }

    #[test]
    fn test_validate_network_id_invalid() {
        assert!(validate_network_id(3, "Network").is_err());
    }

    #[test]
    fn test_get_network_name() {
        assert_eq!(get_network_name(0), "Mainnet");
        assert_eq!(get_network_name(1), "AggLayer-1");
        assert_eq!(get_network_name(2), "AggLayer-2");
        assert_eq!(get_network_name(99), "Unknown");
    }
}
