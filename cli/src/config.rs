use crate::error::{ConfigError, Result};
use crate::types::{ChainId, EthereumAddress, NetworkId, RpcUrl};
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
    pub base_url: RpcUrl,
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
    pub chain_id: ChainId,
    pub rpc_url: RpcUrl,
    #[allow(dead_code)]
    pub fork_url: Option<RpcUrl>,
}

/// Account configuration with pre-configured test accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub accounts: Vec<EthereumAddress>,
    pub private_keys: Vec<String>, // Keep as String since private keys have different format
}

/// Contract addresses configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConfig {
    pub l1_contracts: HashMap<String, EthereumAddress>,
    pub l2_contracts: HashMap<String, EthereumAddress>,
    pub l3_contracts: HashMap<String, EthereumAddress>,
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
    /// Get the appropriate API base URL for a given network ID
    pub fn get_api_base_url(&self, network_id: NetworkId) -> String {
        match network_id.as_u64() {
            // Network ID 2+ served by aggkit-l3 (port 5578)
            2..=3 => {
                // Replace port 5577 with 5578 for aggkit-l3
                let base_url = self.api.base_url.as_str();
                if base_url.contains("5577") {
                    base_url.replace("5577", "5578")
                } else {
                    // If custom base_url doesn't contain 5577, construct l3 URL
                    let base = if base_url.starts_with("http://") {
                        base_url.strip_prefix("http://").unwrap_or("localhost")
                    } else if base_url.starts_with("https://") {
                        base_url.strip_prefix("https://").unwrap_or("localhost")
                    } else {
                        base_url
                    };

                    let host = base.split(':').next().unwrap_or("localhost");
                    format!("http://{host}:5578")
                }
            }
            // Network ID 0 (L1), 1 (L2), and dev networks served by aggkit-l2 (port 5577) - default
            _ => self.api.base_url.as_str().to_string(),
        }
    }

    /// Load configuration with automatic source detection
    /// Tries: config files → environment variables → defaults
    pub fn load() -> Result<Self> {
        Self::load_with_env_refresh(false)
    }

    /// Parse .env file directly into a HashMap
    fn parse_env_file() -> Option<std::collections::HashMap<String, String>> {
        use std::fs;
        if let Ok(content) = fs::read_to_string(".env") {
            let mut env_map = std::collections::HashMap::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    env_map.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
            Some(env_map)
        } else {
            None
        }
    }

    /// Load configuration with explicit environment refresh
    pub fn load_with_env_refresh(force_env_refresh: bool) -> Result<Self> {
        // Load .env file if it exists
        if Path::new(".env").exists() {
            if force_env_refresh {
                // Parse .env file directly instead of relying on environment variables
                let env_map = Self::parse_env_file();

                // Load the rest of the config normally
                return Self::load_with_env_map(env_map);
            } else {
                dotenv::dotenv().ok();
            }
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

    /// Load configuration with a custom environment map
    fn load_with_env_map(
        env_map: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Self> {
        let api = ApiConfig::load()?;
        let networks = NetworkConfig::load();
        let accounts = AccountConfig::load();
        let contracts = if let Some(ref env_vars) = env_map {
            ContractConfig::load_with_env_override(Some(env_vars.clone()))
        } else {
            ContractConfig::load()
        };

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
            if let Ok(rpc_url) = RpcUrl::new(base_url) {
                self.api.base_url = rpc_url;
            }
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
            if let Ok(url) = RpcUrl::new(rpc_1) {
                self.networks.l1.rpc_url = url;
            }
        }
        if let Ok(rpc_2) = std::env::var("RPC_2") {
            if let Ok(url) = RpcUrl::new(rpc_2) {
                self.networks.l2.rpc_url = url;
            }
        }
        if let Ok(rpc_3) = std::env::var("RPC_3") {
            if let Some(l3) = &mut self.networks.l3 {
                if let Ok(url) = RpcUrl::new(rpc_3) {
                    l3.rpc_url = url;
                }
            }
        }

        // Chain ID overrides
        if let Ok(chain_id) = std::env::var("CHAIN_ID_MAINNET") {
            if let Ok(id) = ChainId::new(chain_id) {
                self.networks.l1.chain_id = id;
            }
        }
        if let Ok(chain_id) = std::env::var("CHAIN_ID_AGGLAYER_1") {
            if let Ok(id) = ChainId::new(chain_id) {
                self.networks.l2.chain_id = id;
            }
        }
        if let Ok(chain_id) = std::env::var("CHAIN_ID_AGGLAYER_2") {
            if let Some(l3) = &mut self.networks.l3 {
                if let Ok(id) = ChainId::new(chain_id) {
                    l3.chain_id = id;
                }
            }
        }
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // API configuration is already validated by RpcUrl constructor
        if self.api.timeout.as_millis() == 0 {
            return Err(ConfigError::validation_failed("API timeout cannot be zero").into());
        }

        // Network configurations are already validated by RpcUrl constructor
        // Chain IDs are already validated by ChainId constructor
        // Accounts are already validated by EthereumAddress constructor

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
            "anvil-l1" => Ok(self.networks.l1.rpc_url.as_str().to_string()),
            "anvil-l2" => Ok(self.networks.l2.rpc_url.as_str().to_string()),
            "anvil-l3" => {
                if let Some(l3) = &self.networks.l3 {
                    Ok(l3.rpc_url.as_str().to_string())
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
    #[allow(clippy::disallowed_methods)] // Allow unwrap for hardcoded defaults
    fn default() -> Self {
        ApiConfig {
            base_url: RpcUrl::new("http://localhost:5577").unwrap(), // Safe: hardcoded default URL
            timeout: Duration::from_millis(30000),
            retry_attempts: 3,
        }
    }
}

impl ApiConfig {
    fn load() -> Result<Self> {
        let base_url_str = get_env_var("API_BASE_URL", "http://localhost:5577");
        let base_url = RpcUrl::new(base_url_str)?;

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
    #[allow(clippy::disallowed_methods)] // Allow unwrap for hardcoded defaults
    fn load() -> Self {
        let l1 = ChainConfig {
            name: "Ethereum-L1".to_string(),
            chain_id: ChainId::new(get_env_var("CHAIN_ID_MAINNET", "1")).unwrap(), // Safe: hardcoded default value "1" is always valid
            rpc_url: RpcUrl::new(get_env_var("RPC_1", "http://localhost:8545")).unwrap(), // Safe: hardcoded default URL is always valid
            fork_url: std::env::var("FORK_URL_MAINNET")
                .ok()
                .and_then(|url| RpcUrl::new(url).ok()),
        };

        let l2 = ChainConfig {
            name: "Polygon-zkEVM".to_string(),
            chain_id: ChainId::new(get_env_var("CHAIN_ID_AGGLAYER_1", "1101")).unwrap(), // Safe: hardcoded default value "1101" is always valid
            rpc_url: RpcUrl::new(get_env_var("RPC_2", "http://localhost:8546")).unwrap(), // Safe: hardcoded default URL is always valid
            fork_url: std::env::var("FORK_URL_AGGLAYER_1")
                .ok()
                .and_then(|url| RpcUrl::new(url).ok()),
        };

        // L3 is optional for multi-L2 mode
        let l3 = if Path::new("docker-compose.multi-l2.yml").exists() {
            Some(ChainConfig {
                name: "Agglayer-2".to_string(),
                chain_id: ChainId::new(get_env_var("CHAIN_ID_AGGLAYER_2", "1102")).unwrap(), // Safe: hardcoded default value "1102" is always valid
                rpc_url: RpcUrl::new(get_env_var("RPC_3", "http://localhost:8547")).unwrap(), // Safe: hardcoded default URL is always valid
                fork_url: std::env::var("FORK_URL_AGGLAYER_2")
                    .ok()
                    .and_then(|url| RpcUrl::new(url).ok()),
            })
        } else {
            None
        };

        NetworkConfig { l1, l2, l3 }
    }
}

impl AccountConfig {
    #[allow(clippy::disallowed_methods)] // Allow unwrap for hardcoded test addresses
    fn load() -> Self {
        // Pre-configured test accounts (same as in logs.rs)
        let accounts = vec![
            EthereumAddress::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap(), // Safe: hardcoded valid test address
            EthereumAddress::new("0x70997970C51812dc3A010C7d01b50e0d17dc79C8").unwrap(), // Safe: hardcoded valid test address
            EthereumAddress::new("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC").unwrap(), // Safe: hardcoded valid test address
            EthereumAddress::new("0x90F79bf6EB2c4f870365E785982E1f101E93b906").unwrap(), // Safe: hardcoded valid test address
            EthereumAddress::new("0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65").unwrap(), // Safe: hardcoded valid test address
            EthereumAddress::new("0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc").unwrap(), // Safe: hardcoded valid test address
            EthereumAddress::new("0x976EA74026E726554dB657fA54763abd0C3a0aa9").unwrap(), // Safe: hardcoded valid test address
            EthereumAddress::new("0x14dC79964da2C08b23698B3D3cc7Ca32193d9955").unwrap(), // Safe: hardcoded valid test address
            EthereumAddress::new("0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f").unwrap(), // Safe: hardcoded valid test address
            EthereumAddress::new("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720").unwrap(), // Safe: hardcoded valid test address
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
        Self::load_with_env_override(None)
    }

    /// Load contract config with optional direct .env file reading
    fn load_with_env_override(
        env_override: Option<std::collections::HashMap<String, String>>,
    ) -> Self {
        let mut l1_contracts = HashMap::new();
        let mut l2_contracts = HashMap::new();

        // Helper function to add contract if valid address
        let add_contract =
            |contracts: &mut HashMap<String, EthereumAddress>,
             env_var: &str,
             name: &str,
             env_map: &Option<std::collections::HashMap<String, String>>| {
                let addr = if let Some(env_map) = env_map {
                    // Use direct .env file values if provided
                    env_map.get(env_var).cloned().unwrap_or_default()
                } else {
                    // Fall back to environment variables
                    std::env::var(env_var).unwrap_or_default()
                };

                if !addr.is_empty() {
                    if let Ok(eth_addr) = EthereumAddress::new(addr) {
                        contracts.insert(name.to_string(), eth_addr);
                    }
                }
            };

        // L1 contracts
        add_contract(
            &mut l1_contracts,
            "FFLONK_VERIFIER_L1",
            "FflonkVerifier",
            &env_override,
        );
        add_contract(
            &mut l1_contracts,
            "POLYGON_ZKEVM_L1",
            "PolygonZkEVM",
            &env_override,
        );
        add_contract(
            &mut l1_contracts,
            "POLYGON_ZKEVM_BRIDGE_L1",
            "PolygonZkEVMBridge",
            &env_override,
        );
        add_contract(
            &mut l1_contracts,
            "POLYGON_ZKEVM_TIMELOCK_L1",
            "PolygonZkEVMTimelock",
            &env_override,
        );
        add_contract(
            &mut l1_contracts,
            "POLYGON_ZKEVM_GLOBAL_EXIT_ROOT_L1",
            "PolygonZkEVMGlobalExitRoot",
            &env_override,
        );
        add_contract(
            &mut l1_contracts,
            "POLYGON_ROLLUP_MANAGER_L1",
            "PolygonRollupManager",
            &env_override,
        );
        add_contract(&mut l1_contracts, "AGG_ERC20_L1", "AggERC20", &env_override);
        add_contract(
            &mut l1_contracts,
            "BRIDGE_EXTENSION_L1",
            "BridgeExtension",
            &env_override,
        );
        add_contract(
            &mut l1_contracts,
            "POLYGON_ZKEVM_GLOBAL_EXIT_ROOT_L1",
            "GlobalExitRootManager",
            &env_override,
        );

        // L2 contracts
        add_contract(
            &mut l2_contracts,
            "POLYGON_ZKEVM_BRIDGE_L2",
            "PolygonZkEVMBridge",
            &env_override,
        );
        add_contract(
            &mut l2_contracts,
            "POLYGON_ZKEVM_TIMELOCK_L2",
            "PolygonZkEVMTimelock",
            &env_override,
        );
        add_contract(&mut l2_contracts, "AGG_ERC20_L2", "AggERC20", &env_override);
        add_contract(
            &mut l2_contracts,
            "BRIDGE_EXTENSION_L2",
            "BridgeExtension",
            &env_override,
        );
        add_contract(
            &mut l2_contracts,
            "GLOBAL_EXIT_ROOT_MANAGER_L2",
            "GlobalExitRootManager",
            &env_override,
        );

        // L3 contracts
        let mut l3_contracts = HashMap::new();
        add_contract(
            &mut l3_contracts,
            "POLYGON_ZKEVM_BRIDGE_L3",
            "PolygonZkEVMBridge",
            &env_override,
        );
        add_contract(
            &mut l3_contracts,
            "POLYGON_ZKEVM_TIMELOCK_L3",
            "PolygonZkEVMTimelock",
            &env_override,
        );
        add_contract(&mut l3_contracts, "AGG_ERC20_L3", "AggERC20", &env_override);
        add_contract(
            &mut l3_contracts,
            "BRIDGE_EXTENSION_L3",
            "BridgeExtension",
            &env_override,
        );
        add_contract(
            &mut l3_contracts,
            "GLOBAL_EXIT_ROOT_MANAGER_L3",
            "GlobalExitRootManager",
            &env_override,
        );

        ContractConfig {
            l1_contracts,
            l2_contracts,
            l3_contracts,
        }
    }

    /// Get contract address with fallback to "Not deployed"
    pub fn get_contract(&self, layer: &str, name: &str) -> String {
        match layer {
            "l1" => self
                .l1_contracts
                .get(name)
                .map(|addr| addr.as_str().to_string())
                .unwrap_or_else(|| "Not deployed".to_string()),
            "l2" => self
                .l2_contracts
                .get(name)
                .map(|addr| addr.as_str().to_string())
                .unwrap_or_else(|| "Not deployed".to_string()),
            "l3" => self
                .l3_contracts
                .get(name)
                .map(|addr| addr.as_str().to_string())
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
            urls.push(("Mainnet".to_string(), url.as_str().to_string()));
        }

        if let Some(url) = &self.networks.l2.fork_url {
            urls.push(("Agglayer 1".to_string(), url.as_str().to_string()));
        }

        if multi_l2 {
            if let Some(l3) = &self.networks.l3 {
                if let Some(url) = &l3.fork_url {
                    urls.push(("Agglayer 2".to_string(), url.as_str().to_string()));
                }
            }
        }

        urls
    }
}

impl Default for Config {
    #[allow(clippy::disallowed_methods)] // Allow unwrap for hardcoded defaults
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
        assert_eq!(config.networks.l1.chain_id.as_str(), "1");
        assert_eq!(config.networks.l2.chain_id.as_str(), "1101");
        assert_eq!(config.api.base_url.as_str(), "http://localhost:5577");
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
        assert!(accounts.accounts[0].as_str().starts_with("0x"));
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
        assert_eq!(api.base_url.as_str(), "http://localhost:5577");
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

    #[test]
    fn test_get_api_base_url() {
        let config = Config::load().unwrap();

        // Test L1 (network 0) and L2 (network 1) go to port 5577
        assert_eq!(
            config.get_api_base_url(NetworkId::new(0).unwrap()),
            "http://localhost:5577"
        );
        assert_eq!(
            config.get_api_base_url(NetworkId::new(1).unwrap()),
            "http://localhost:5577"
        );
        assert_eq!(
            config.get_api_base_url(NetworkId::new(31337).unwrap()),
            "http://localhost:5577"
        );

        // Test network ID 2+ goes to port 5578 (aggkit-l3)
        assert_eq!(
            config.get_api_base_url(NetworkId::new(2).unwrap()),
            "http://localhost:5578"
        );
        assert_eq!(
            config.get_api_base_url(NetworkId::new(3).unwrap()),
            "http://localhost:5578"
        );
    }

    #[test]
    fn test_get_api_base_url_custom_host() {
        let mut config = Config::load().unwrap();
        config.api.base_url = RpcUrl::new("https://custom.host.com:5577").unwrap();

        // Test L1 (network 0) and L2 (network 1) use custom host with port 5577
        assert_eq!(
            config.get_api_base_url(NetworkId::new(0).unwrap()),
            "https://custom.host.com:5577"
        );
        assert_eq!(
            config.get_api_base_url(NetworkId::new(1).unwrap()),
            "https://custom.host.com:5577"
        );

        // Test network ID 2+ goes to port 5578 on custom host
        assert_eq!(
            config.get_api_base_url(NetworkId::new(2).unwrap()),
            "https://custom.host.com:5578"
        );
        assert_eq!(
            config.get_api_base_url(NetworkId::new(3).unwrap()),
            "https://custom.host.com:5578"
        );
    }

    #[test]
    fn test_get_api_base_url_custom_without_port() {
        let mut config = Config::load().unwrap();
        config.api.base_url = RpcUrl::new("https://api.example.com").unwrap();

        // Test L1 (network 0) and L2 (network 1) use original URL
        assert_eq!(
            config.get_api_base_url(NetworkId::new(0).unwrap()),
            "https://api.example.com"
        );
        assert_eq!(
            config.get_api_base_url(NetworkId::new(1).unwrap()),
            "https://api.example.com"
        );

        // Test network ID 2+ constructs new URL with port 5578
        assert_eq!(
            config.get_api_base_url(NetworkId::new(2).unwrap()),
            "http://api.example.com:5578"
        );
        assert_eq!(
            config.get_api_base_url(NetworkId::new(3).unwrap()),
            "http://api.example.com:5578"
        );
    }
}
