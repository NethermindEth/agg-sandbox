use crate::error::{ConfigError, Result};
use crate::validation::Validator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Main configuration structure for the CLI application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub networks: NetworkConfig,
    pub accounts: AccountConfig,
    pub contracts: ContractConfig,
}

/// API configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    #[serde(with = "duration_serde")]
    #[allow(dead_code)]
    pub timeout: Duration,
    #[allow(dead_code)]
    pub retry_attempts: u32,
}

/// Network configuration for all supported chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub l1: ChainConfig,
    pub l2: ChainConfig,
    pub l3: Option<ChainConfig>,
}

/// Individual chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub name: String,
    pub chain_id: String,
    pub rpc_url: String,
    #[allow(dead_code)]
    pub fork_url: Option<String>,
}

/// Account configuration with pre-configured test accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub accounts: Vec<String>,
    pub private_keys: Vec<String>,
}

/// Contract addresses configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConfig {
    pub l1_contracts: HashMap<String, String>,
    pub l2_contracts: HashMap<String, String>,
}

/// Custom serialization for Duration to support TOML/YAML
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (duration.as_millis() as u64).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// Configuration file format detection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigFormat {
    Toml,
    Yaml,
}

impl ConfigFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "toml" => Some(ConfigFormat::Toml),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            _ => None,
        }
    }
}

impl Config {
    /// Load configuration with automatic source detection
    /// Tries: config files → environment variables → defaults
    pub fn load() -> Result<Self> {
        // Load .env file if it exists
        if Path::new(".env").exists() {
            dotenv::dotenv().ok();
        }

        // Try to load from configuration files first
        let config_paths = [
            "aggsandbox.toml",
            "aggsandbox.yaml",
            "aggsandbox.yml",
            ".aggsandbox.toml",
            ".aggsandbox.yaml",
            ".aggsandbox.yml",
        ];

        for path_str in &config_paths {
            let path = Path::new(path_str);
            if path.exists() {
                return Self::load_from_file(path);
            }
        }

        // Fallback to environment variables and defaults
        Self::load_from_env()
    }

    /// Load configuration from environment variables and defaults
    pub fn load_from_env() -> Result<Self> {
        let api = ApiConfig::load()?;
        let networks = NetworkConfig::load();
        let accounts = AccountConfig::load();
        let contracts = ContractConfig::load();

        Ok(Config {
            api,
            networks,
            accounts,
            contracts,
        })
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            ConfigError::validation_failed(&format!(
                "Failed to read config file {}: {e}",
                path.display()
            ))
        })?;

        let format = ConfigFormat::from_path(path).ok_or_else(|| {
            ConfigError::validation_failed(&format!(
                "Unsupported config file format: {}",
                path.display()
            ))
        })?;

        let mut config: Config = match format {
            ConfigFormat::Toml => toml::from_str(&content).map_err(|e| {
                ConfigError::validation_failed(&format!("Invalid TOML in {}: {e}", path.display()))
            })?,
            ConfigFormat::Yaml => serde_yaml::from_str(&content).map_err(|e| {
                ConfigError::validation_failed(&format!("Invalid YAML in {}: {e}", path.display()))
            })?,
        };

        // Merge with environment variables (env vars take precedence)
        config.merge_from_env();
        config.validate()?;

        Ok(config)
    }

    /// Merge configuration with environment variables
    fn merge_from_env(&mut self) {
        // API configuration overrides
        if let Ok(base_url) = std::env::var("API_BASE_URL") {
            self.api.base_url = base_url;
        }
        if let Ok(timeout_str) = std::env::var("API_TIMEOUT_MS") {
            if let Ok(timeout_ms) = timeout_str.parse::<u64>() {
                self.api.timeout = Duration::from_millis(timeout_ms);
            }
        }
        if let Ok(retry_str) = std::env::var("API_RETRY_ATTEMPTS") {
            if let Ok(retry_attempts) = retry_str.parse::<u32>() {
                self.api.retry_attempts = retry_attempts;
            }
        }

        // Network configuration overrides
        if let Ok(rpc_1) = std::env::var("RPC_1") {
            self.networks.l1.rpc_url = rpc_1;
        }
        if let Ok(rpc_2) = std::env::var("RPC_2") {
            self.networks.l2.rpc_url = rpc_2;
        }
        if let Ok(rpc_3) = std::env::var("RPC_3") {
            if let Some(l3) = &mut self.networks.l3 {
                l3.rpc_url = rpc_3;
            }
        }

        // Chain ID overrides
        if let Ok(chain_id) = std::env::var("CHAIN_ID_MAINNET") {
            self.networks.l1.chain_id = chain_id;
        }
        if let Ok(chain_id) = std::env::var("CHAIN_ID_AGGLAYER_1") {
            self.networks.l2.chain_id = chain_id;
        }
        if let Ok(chain_id) = std::env::var("CHAIN_ID_AGGLAYER_2") {
            if let Some(l3) = &mut self.networks.l3 {
                l3.chain_id = chain_id;
            }
        }
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate API configuration
        Validator::validate_rpc_url(&self.api.base_url)?;

        if self.api.timeout.as_millis() == 0 {
            return Err(ConfigError::validation_failed("API timeout cannot be zero").into());
        }

        // Validate network configurations
        Validator::validate_rpc_url(&self.networks.l1.rpc_url)?;
        Validator::validate_rpc_url(&self.networks.l2.rpc_url)?;

        if let Some(l3) = &self.networks.l3 {
            Validator::validate_rpc_url(&l3.rpc_url)?;
        }

        // Validate accounts
        for account in &self.accounts.accounts {
            Validator::validate_ethereum_address(account)?;
        }

        Ok(())
    }

    /// Save configuration to a file
    #[allow(dead_code)]
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let format = ConfigFormat::from_path(path).ok_or_else(|| {
            ConfigError::validation_failed(&format!(
                "Unsupported config file format: {}",
                path.display()
            ))
        })?;

        let content = match format {
            ConfigFormat::Toml => toml::to_string_pretty(self).map_err(|e| {
                ConfigError::validation_failed(&format!("Failed to serialize TOML: {e}"))
            })?,
            ConfigFormat::Yaml => serde_yaml::to_string(self).map_err(|e| {
                ConfigError::validation_failed(&format!("Failed to serialize YAML: {e}"))
            })?,
        };

        fs::write(path, content).map_err(|e| {
            ConfigError::validation_failed(&format!(
                "Failed to write config file {}: {e}",
                path.display()
            ))
        })?;

        Ok(())
    }

    /// Get chain configuration by name
    #[allow(dead_code)]
    pub fn get_chain(&self, name: &str) -> Option<&ChainConfig> {
        match name {
            "anvil-l1" | "l1" => Some(&self.networks.l1),
            "anvil-l2" | "l2" => Some(&self.networks.l2),
            "anvil-l3" | "l3" => self.networks.l3.as_ref(),
            _ => None,
        }
    }

    /// Get RPC URL for a chain
    #[allow(dead_code)]
    pub fn get_rpc_url(&self, chain: &str) -> Result<String> {
        match chain {
            "anvil-l1" => Ok(self.networks.l1.rpc_url.clone()),
            "anvil-l2" => Ok(self.networks.l2.rpc_url.clone()),
            "anvil-l3" => {
                if let Some(l3) = &self.networks.l3 {
                    Ok(l3.rpc_url.clone())
                } else {
                    Err(ConfigError::missing_required("L3 chain configuration").into())
                }
            }
            _ => Err(ConfigError::invalid_value(
                "chain",
                chain,
                "Supported chains: anvil-l1, anvil-l2, anvil-l3",
            )
            .into()),
        }
    }

    /// Check if multi-L2 mode is available
    #[allow(dead_code)]
    pub fn has_multi_l2(&self) -> bool {
        self.networks.l3.is_some()
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            base_url: "http://localhost:5577".to_string(),
            timeout: Duration::from_millis(30000),
            retry_attempts: 3,
        }
    }
}

impl ApiConfig {
    fn load() -> Result<Self> {
        let base_url_str = get_env_var("API_BASE_URL", "http://localhost:5577");
        let base_url = Validator::validate_rpc_url(&base_url_str)?;

        let timeout_ms_str = get_env_var("API_TIMEOUT_MS", "30000");
        let timeout_ms = timeout_ms_str.parse::<u64>().map_err(|_| {
            ConfigError::invalid_value(
                "API_TIMEOUT_MS",
                &timeout_ms_str,
                "must be a valid number in milliseconds",
            )
        })?;
        let validated_timeout_ms = Validator::validate_timeout_ms(timeout_ms)?;

        let retry_attempts_str = get_env_var("API_RETRY_ATTEMPTS", "3");
        let retry_attempts = retry_attempts_str.parse::<u32>().map_err(|_| {
            ConfigError::invalid_value(
                "API_RETRY_ATTEMPTS",
                &retry_attempts_str,
                "must be a valid positive number",
            )
        })?;
        let validated_retry_attempts = Validator::validate_retry_attempts(retry_attempts)?;

        Ok(ApiConfig {
            base_url,
            timeout: Duration::from_millis(validated_timeout_ms),
            retry_attempts: validated_retry_attempts,
        })
    }
}

impl NetworkConfig {
    fn load() -> Self {
        let l1 = ChainConfig {
            name: "Ethereum-L1".to_string(),
            chain_id: get_env_var("CHAIN_ID_MAINNET", "1"),
            rpc_url: get_env_var("RPC_1", "http://localhost:8545"),
            fork_url: std::env::var("FORK_URL_MAINNET").ok(),
        };

        let l2 = ChainConfig {
            name: "Polygon-zkEVM".to_string(),
            chain_id: get_env_var("CHAIN_ID_AGGLAYER_1", "1101"),
            rpc_url: get_env_var("RPC_2", "http://localhost:8546"),
            fork_url: std::env::var("FORK_URL_AGGLAYER_1").ok(),
        };

        // L3 is optional for multi-L2 mode
        let l3 = if Path::new("docker-compose.multi-l2.yml").exists() {
            Some(ChainConfig {
                name: "AggLayer-2".to_string(),
                chain_id: get_env_var("CHAIN_ID_AGGLAYER_2", "1102"),
                rpc_url: get_env_var("RPC_3", "http://localhost:8547"),
                fork_url: std::env::var("FORK_URL_AGGLAYER_2").ok(),
            })
        } else {
            None
        };

        NetworkConfig { l1, l2, l3 }
    }
}

impl AccountConfig {
    fn load() -> Self {
        // Pre-configured test accounts (same as in logs.rs)
        let accounts = vec![
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string(),
            "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC".to_string(),
            "0x90F79bf6EB2c4f870365E785982E1f101E93b906".to_string(),
            "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65".to_string(),
            "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc".to_string(),
            "0x976EA74026E726554dB657fA54763abd0C3a0aa9".to_string(),
            "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955".to_string(),
            "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f".to_string(),
            "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720".to_string(),
        ];

        let private_keys = vec![
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
            "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d".to_string(),
            "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a".to_string(),
            "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6".to_string(),
            "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a".to_string(),
            "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba".to_string(),
            "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e".to_string(),
            "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356".to_string(),
            "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97".to_string(),
            "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6".to_string(),
        ];

        AccountConfig {
            accounts,
            private_keys,
        }
    }
}

impl ContractConfig {
    fn load() -> Self {
        let mut l1_contracts = HashMap::new();
        let mut l2_contracts = HashMap::new();

        // L1 contracts
        if let Ok(addr) = std::env::var("FFLONK_VERIFIER_L1") {
            l1_contracts.insert("FflonkVerifier".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("POLYGON_ZKEVM_L1") {
            l1_contracts.insert("PolygonZkEVM".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("POLYGON_ZKEVM_BRIDGE_L1") {
            l1_contracts.insert("PolygonZkEVMBridge".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("POLYGON_ZKEVM_TIMELOCK_L1") {
            l1_contracts.insert("PolygonZkEVMTimelock".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("POLYGON_ZKEVM_GLOBAL_EXIT_ROOT_L1") {
            l1_contracts.insert("PolygonZkEVMGlobalExitRoot".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("POLYGON_ROLLUP_MANAGER_L1") {
            l1_contracts.insert("PolygonRollupManager".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("AGG_ERC20_L1") {
            l1_contracts.insert("AggERC20".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("BRIDGE_EXTENSION_L1") {
            l1_contracts.insert("BridgeExtension".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("GLOBAL_EXIT_ROOT_MANAGER_L1") {
            l1_contracts.insert("GlobalExitRootManager".to_string(), addr);
        }

        // L2 contracts
        if let Ok(addr) = std::env::var("POLYGON_ZKEVM_BRIDGE_L2") {
            l2_contracts.insert("PolygonZkEVMBridge".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("POLYGON_ZKEVM_TIMELOCK_L2") {
            l2_contracts.insert("PolygonZkEVMTimelock".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("AGG_ERC20_L2") {
            l2_contracts.insert("AggERC20".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("BRIDGE_EXTENSION_L2") {
            l2_contracts.insert("BridgeExtension".to_string(), addr);
        }
        if let Ok(addr) = std::env::var("GLOBAL_EXIT_ROOT_MANAGER_L2") {
            l2_contracts.insert("GlobalExitRootManager".to_string(), addr);
        }

        ContractConfig {
            l1_contracts,
            l2_contracts,
        }
    }

    /// Get contract address with fallback to "Not deployed"
    pub fn get_contract(&self, layer: &str, name: &str) -> String {
        match layer {
            "l1" => self
                .l1_contracts
                .get(name)
                .cloned()
                .unwrap_or_else(|| "Not deployed".to_string()),
            "l2" => self
                .l2_contracts
                .get(name)
                .cloned()
                .unwrap_or_else(|| "Not deployed".to_string()),
            _ => "Not deployed".to_string(),
        }
    }
}

/// Helper function to get environment variable with fallback
fn get_env_var(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

/// Validation helpers
impl Config {
    /// Validate fork mode configuration
    #[allow(dead_code)]
    pub fn validate_fork_mode(&self, multi_l2: bool) -> Result<()> {
        if self.networks.l1.fork_url.is_none() {
            return Err(ConfigError::env_var_not_found("FORK_URL_MAINNET").into());
        }

        if self.networks.l2.fork_url.is_none() {
            return Err(ConfigError::env_var_not_found("FORK_URL_AGGLAYER_1").into());
        }

        if multi_l2 {
            if let Some(l3) = &self.networks.l3 {
                if l3.fork_url.is_none() {
                    return Err(ConfigError::env_var_not_found("FORK_URL_AGGLAYER_2").into());
                }
            } else {
                return Err(ConfigError::validation_failed(
                    "Multi-L2 mode requested but L3 configuration not available",
                )
                .into());
            }
        }

        Ok(())
    }

    /// Get fork URLs for display
    #[allow(dead_code)]
    pub fn get_fork_urls(&self, multi_l2: bool) -> Vec<(String, String)> {
        let mut urls = Vec::new();

        if let Some(url) = &self.networks.l1.fork_url {
            urls.push(("Mainnet".to_string(), url.clone()));
        }

        if let Some(url) = &self.networks.l2.fork_url {
            urls.push(("AggLayer 1".to_string(), url.clone()));
        }

        if multi_l2 {
            if let Some(l3) = &self.networks.l3 {
                if let Some(url) = &l3.fork_url {
                    urls.push(("AggLayer 2".to_string(), url.clone()));
                }
            }
        }

        urls
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api: ApiConfig::default(),
            networks: NetworkConfig::load(),
            accounts: AccountConfig::load(),
            contracts: ContractConfig::load(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::load();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.networks.l1.chain_id, "1");
        assert_eq!(config.networks.l2.chain_id, "1101");
        assert_eq!(config.api.base_url, "http://localhost:5577");
    }

    #[test]
    fn test_get_chain() {
        let config = Config::load().unwrap();

        assert!(config.get_chain("anvil-l1").is_some());
        assert!(config.get_chain("l1").is_some());
        assert!(config.get_chain("anvil-l2").is_some());
        assert!(config.get_chain("l2").is_some());
        assert!(config.get_chain("invalid").is_none());
    }

    #[test]
    fn test_get_rpc_url() {
        let config = Config::load().unwrap();

        assert!(config.get_rpc_url("anvil-l1").is_ok());
        assert!(config.get_rpc_url("anvil-l2").is_ok());
        assert!(config.get_rpc_url("invalid").is_err());
    }

    #[test]
    fn test_account_config() {
        let accounts = AccountConfig::load();
        assert_eq!(accounts.accounts.len(), 10);
        assert_eq!(accounts.private_keys.len(), 10);
        assert!(accounts.accounts[0].starts_with("0x"));
        assert!(accounts.private_keys[0].starts_with("0x"));
    }

    #[test]
    fn test_contract_config() {
        let contracts = ContractConfig::load();
        assert_eq!(contracts.get_contract("l1", "NonExistent"), "Not deployed");
    }

    #[test]
    fn test_api_config_defaults() {
        let api = ApiConfig::load().unwrap();
        assert_eq!(api.base_url, "http://localhost:5577");
        assert_eq!(api.timeout, Duration::from_millis(30000));
        assert_eq!(api.retry_attempts, 3);
    }

    #[test]
    fn test_config_format_detection() {
        use std::path::Path;

        assert_eq!(
            ConfigFormat::from_path(Path::new("config.toml")),
            Some(ConfigFormat::Toml)
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.yaml")),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.yml")),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(ConfigFormat::from_path(Path::new("config.json")), None);
    }

    #[test]
    fn test_toml_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[api]"));
        assert!(toml_str.contains("[networks.l1]"));
        assert!(toml_str.contains("base_url"));

        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.api.base_url, config.api.base_url);
    }

    #[test]
    fn test_yaml_serialization() {
        let config = Config::default();
        let yaml_str = serde_yaml::to_string(&config).unwrap();
        assert!(yaml_str.contains("api:"));
        assert!(yaml_str.contains("networks:"));
        assert!(yaml_str.contains("base_url:"));

        let deserialized: Config = serde_yaml::from_str(&yaml_str).unwrap();
        assert_eq!(deserialized.api.base_url, config.api.base_url);
    }

    #[test]
    fn test_duration_serialization() {
        let config = Config::default();

        // Test TOML serialization preserves duration
        let toml_str = toml::to_string(&config).unwrap();
        let toml_config: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(toml_config.api.timeout, config.api.timeout);

        // Test YAML serialization preserves duration
        let yaml_str = serde_yaml::to_string(&config).unwrap();
        let yaml_config: Config = serde_yaml::from_str(&yaml_str).unwrap();
        assert_eq!(yaml_config.api.timeout, config.api.timeout);
    }

    #[test]
    fn test_config_save_and_load() {
        use tempfile::NamedTempFile;

        let original_config = Config::default();

        // Test TOML save/load
        let toml_file = NamedTempFile::with_suffix(".toml").unwrap();
        original_config.save_to_file(toml_file.path()).unwrap();
        let loaded_toml = Config::load_from_file(toml_file.path()).unwrap();
        assert_eq!(loaded_toml.api.base_url, original_config.api.base_url);

        // Test YAML save/load
        let yaml_file = NamedTempFile::with_suffix(".yaml").unwrap();
        original_config.save_to_file(yaml_file.path()).unwrap();
        let loaded_yaml = Config::load_from_file(yaml_file.path()).unwrap();
        assert_eq!(loaded_yaml.api.base_url, original_config.api.base_url);
    }
}
