use std::fmt;

/// Main error type for the AggSandbox CLI
#[derive(Debug)]
pub enum AggSandboxError {
    /// Configuration-related errors
    Config(ConfigError),
    /// Docker-related errors
    Docker(DockerError),
    /// API-related errors
    Api(ApiError),
    /// Event processing errors
    Events(EventError),
    /// I/O errors
    Io(std::io::Error),
    /// Generic errors with context
    Other(String),
}

/// Configuration-specific errors
#[derive(Debug)]
pub enum ConfigError {
    /// Environment variable not found
    EnvVarNotFound(String),
    /// Invalid configuration value
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },
    /// Missing required configuration
    MissingRequired(String),
    /// Configuration validation failed
    ValidationFailed(String),
}

/// Docker-related errors
#[derive(Debug)]
pub enum DockerError {
    /// Docker compose file not found
    #[allow(dead_code)]
    ComposeFileNotFound(String),
    /// Docker command execution failed
    CommandFailed { command: String, stderr: String },
    /// Docker service not running
    #[allow(dead_code)]
    ServiceNotRunning(String),
    /// Docker compose validation failed
    ComposeValidationFailed(String),
}

/// API-related errors
#[derive(Debug)]
pub enum ApiError {
    /// HTTP request failed
    RequestFailed {
        url: String,
        status: u16,
        message: String,
    },
    /// Network connection failed
    NetworkError(String),
    /// JSON parsing failed
    JsonParseError(String),
    /// API response validation failed
    #[allow(dead_code)]
    ResponseValidationFailed(String),
    /// API endpoint not available
    #[allow(dead_code)]
    EndpointUnavailable(String),
}

/// Event processing errors
#[derive(Debug)]
pub enum EventError {
    /// Invalid chain specified
    InvalidChain(String),
    /// Contract address invalid
    InvalidAddress(String),
    /// Event parsing failed
    #[allow(dead_code)]
    ParseError(String),
    /// RPC connection failed
    RpcConnectionFailed(String),
}

impl fmt::Display for AggSandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggSandboxError::Config(e) => write!(f, "Configuration error: {e}"),
            AggSandboxError::Docker(e) => write!(f, "Docker error: {e}"),
            AggSandboxError::Api(e) => write!(f, "API error: {e}"),
            AggSandboxError::Events(e) => write!(f, "Event processing error: {e}"),
            AggSandboxError::Io(e) => write!(f, "I/O error: {e}"),
            AggSandboxError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::EnvVarNotFound(var) => {
                write!(f, "Environment variable '{var}' not found")
            }
            ConfigError::InvalidValue { key, value, reason } => {
                write!(f, "Invalid value '{value}' for '{key}': {reason}")
            }
            ConfigError::MissingRequired(key) => {
                write!(f, "Required configuration '{key}' is missing")
            }
            ConfigError::ValidationFailed(msg) => {
                write!(f, "Configuration validation failed: {msg}")
            }
        }
    }
}

impl fmt::Display for DockerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DockerError::ComposeFileNotFound(file) => {
                write!(f, "Docker compose file not found: {file}")
            }
            DockerError::CommandFailed { command, stderr } => {
                write!(f, "Docker command '{command}' failed: {stderr}")
            }
            DockerError::ServiceNotRunning(service) => {
                write!(f, "Docker service '{service}' is not running")
            }
            DockerError::ComposeValidationFailed(msg) => {
                write!(f, "Docker compose validation failed: {msg}")
            }
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::RequestFailed {
                url,
                status,
                message,
            } => {
                write!(
                    f,
                    "HTTP request to '{url}' failed with status {status}: {message}"
                )
            }
            ApiError::NetworkError(msg) => {
                write!(f, "Network connection failed: {msg}")
            }
            ApiError::JsonParseError(msg) => {
                write!(f, "Failed to parse JSON response: {msg}")
            }
            ApiError::ResponseValidationFailed(msg) => {
                write!(f, "API response validation failed: {msg}")
            }
            ApiError::EndpointUnavailable(endpoint) => {
                write!(f, "API endpoint '{endpoint}' is not available")
            }
        }
    }
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::InvalidChain(chain) => {
                write!(f, "Invalid chain '{chain}' specified")
            }
            EventError::InvalidAddress(addr) => {
                write!(f, "Invalid contract address '{addr}'")
            }
            EventError::ParseError(msg) => {
                write!(f, "Failed to parse event data: {msg}")
            }
            EventError::RpcConnectionFailed(msg) => {
                write!(f, "RPC connection failed: {msg}")
            }
        }
    }
}

impl std::error::Error for AggSandboxError {}
impl std::error::Error for ConfigError {}
impl std::error::Error for DockerError {}
impl std::error::Error for ApiError {}
impl std::error::Error for EventError {}

// Conversion from standard library errors
impl From<std::io::Error> for AggSandboxError {
    fn from(err: std::io::Error) -> Self {
        AggSandboxError::Io(err)
    }
}

impl From<ConfigError> for AggSandboxError {
    fn from(err: ConfigError) -> Self {
        AggSandboxError::Config(err)
    }
}

impl From<DockerError> for AggSandboxError {
    fn from(err: DockerError) -> Self {
        AggSandboxError::Docker(err)
    }
}

impl From<ApiError> for AggSandboxError {
    fn from(err: ApiError) -> Self {
        AggSandboxError::Api(err)
    }
}

impl From<EventError> for AggSandboxError {
    fn from(err: EventError) -> Self {
        AggSandboxError::Events(err)
    }
}

// Conversion from external library errors
impl From<reqwest::Error> for AggSandboxError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AggSandboxError::Api(ApiError::NetworkError("Request timeout".to_string()))
        } else if err.is_connect() {
            AggSandboxError::Api(ApiError::NetworkError("Connection failed".to_string()))
        } else if let Some(status) = err.status() {
            AggSandboxError::Api(ApiError::RequestFailed {
                url: err
                    .url()
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                status: status.as_u16(),
                message: err.to_string(),
            })
        } else {
            AggSandboxError::Api(ApiError::NetworkError(err.to_string()))
        }
    }
}

impl From<serde_json::Error> for AggSandboxError {
    fn from(err: serde_json::Error) -> Self {
        AggSandboxError::Api(ApiError::JsonParseError(err.to_string()))
    }
}

// Conversion from anyhow::Error for compatibility
impl From<anyhow::Error> for AggSandboxError {
    fn from(err: anyhow::Error) -> Self {
        AggSandboxError::Other(err.to_string())
    }
}

/// Result type alias for AggSandbox operations
pub type Result<T> = std::result::Result<T, AggSandboxError>;

/// Helper trait for adding context to errors
pub trait ErrorContext<T> {
    #[allow(dead_code)]
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<AggSandboxError>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let context = f();
            let base_error = e.into();
            AggSandboxError::Other(format!("{context}: {base_error}"))
        })
    }
}

/// Helper functions for creating specific error types
impl ConfigError {
    pub fn env_var_not_found(var: &str) -> Self {
        ConfigError::EnvVarNotFound(var.to_string())
    }

    pub fn invalid_value(key: &str, value: &str, reason: &str) -> Self {
        ConfigError::InvalidValue {
            key: key.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn missing_required(key: &str) -> Self {
        ConfigError::MissingRequired(key.to_string())
    }

    pub fn validation_failed(msg: &str) -> Self {
        ConfigError::ValidationFailed(msg.to_string())
    }
}

impl DockerError {
    #[allow(dead_code)]
    pub fn compose_file_not_found(file: &str) -> Self {
        DockerError::ComposeFileNotFound(file.to_string())
    }

    pub fn command_failed(command: &str, stderr: &str) -> Self {
        DockerError::CommandFailed {
            command: command.to_string(),
            stderr: stderr.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn service_not_running(service: &str) -> Self {
        DockerError::ServiceNotRunning(service.to_string())
    }

    pub fn compose_validation_failed(msg: &str) -> Self {
        DockerError::ComposeValidationFailed(msg.to_string())
    }
}

impl ApiError {
    pub fn request_failed(url: &str, status: u16, message: &str) -> Self {
        ApiError::RequestFailed {
            url: url.to_string(),
            status,
            message: message.to_string(),
        }
    }

    pub fn network_error(msg: &str) -> Self {
        ApiError::NetworkError(msg.to_string())
    }

    pub fn json_parse_error(msg: &str) -> Self {
        ApiError::JsonParseError(msg.to_string())
    }

    #[allow(dead_code)]
    pub fn response_validation_failed(msg: &str) -> Self {
        ApiError::ResponseValidationFailed(msg.to_string())
    }

    #[allow(dead_code)]
    pub fn endpoint_unavailable(endpoint: &str) -> Self {
        ApiError::EndpointUnavailable(endpoint.to_string())
    }
}

impl EventError {
    pub fn invalid_chain(chain: &str) -> Self {
        EventError::InvalidChain(chain.to_string())
    }

    pub fn invalid_address(addr: &str) -> Self {
        EventError::InvalidAddress(addr.to_string())
    }

    #[allow(dead_code)]
    pub fn parse_error(msg: &str) -> Self {
        EventError::ParseError(msg.to_string())
    }

    pub fn rpc_connection_failed(msg: &str) -> Self {
        EventError::RpcConnectionFailed(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::env_var_not_found("TEST_VAR");
        assert_eq!(err.to_string(), "Environment variable 'TEST_VAR' not found");
    }

    #[test]
    fn test_docker_error_display() {
        let err = DockerError::command_failed("docker-compose up", "service failed");
        assert_eq!(
            err.to_string(),
            "Docker command 'docker-compose up' failed: service failed"
        );
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::request_failed("http://localhost:5577", 404, "Not Found");
        assert_eq!(
            err.to_string(),
            "HTTP request to 'http://localhost:5577' failed with status 404: Not Found"
        );
    }

    #[test]
    fn test_event_error_display() {
        let err = EventError::invalid_chain("invalid-chain");
        assert_eq!(err.to_string(), "Invalid chain 'invalid-chain' specified");
    }

    #[test]
    fn test_error_conversion() {
        let config_err = ConfigError::env_var_not_found("TEST");
        let agg_err: AggSandboxError = config_err.into();

        match agg_err {
            AggSandboxError::Config(_) => (),
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_error_context() {
        let result: std::result::Result<i32, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));

        let with_context = result.with_context(|| "While reading config file".to_string());
        assert!(with_context.is_err());
        assert!(with_context
            .unwrap_err()
            .to_string()
            .contains("While reading config file"));
    }
}
