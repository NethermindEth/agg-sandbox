use thiserror::Error;

/// Main error type for the AggSandbox CLI
#[derive(Error, Debug)]
pub enum AggSandboxError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    /// Docker-related errors
    #[error("Docker error: {0}")]
    Docker(#[from] DockerError),
    /// API-related errors
    #[error("API error: {0}")]
    Api(#[from] ApiError),
    /// Event processing errors
    #[error("Event processing error: {0}")]
    Events(#[from] EventError),
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Generic errors with context
    #[error("{0}")]
    Other(String),
}

/// Configuration-specific errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Environment variable not found
    #[error("Environment variable '{0}' not found")]
    EnvVarNotFound(String),
    /// Invalid configuration value
    #[error("Invalid value '{value}' for '{key}': {reason}")]
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },
    /// Missing required configuration
    #[error("Required configuration '{0}' is missing")]
    MissingRequired(String),
    /// Configuration validation failed
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
}

/// Docker-related errors
#[derive(Error, Debug)]
pub enum DockerError {
    /// Docker compose file not found
    #[error("Docker compose file not found: {0}")]
    #[allow(dead_code)]
    ComposeFileNotFound(String),
    /// Docker command execution failed
    #[error("Docker command '{command}' failed: {stderr}")]
    CommandFailed { command: String, stderr: String },
    /// Docker service not running
    #[error("Docker service '{0}' is not running")]
    #[allow(dead_code)]
    ServiceNotRunning(String),
    /// Docker compose validation failed
    #[error("Docker compose validation failed: {0}")]
    ComposeValidationFailed(String),
}

/// API-related errors
#[derive(Error, Debug)]
pub enum ApiError {
    /// HTTP request failed
    #[error("HTTP request to '{url}' failed with status {status}: {message}")]
    RequestFailed {
        url: String,
        status: u16,
        message: String,
    },
    /// Network connection failed
    #[error("Network connection failed: {0}")]
    NetworkError(String),
    /// JSON parsing failed
    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(String),
    /// API response validation failed
    #[error("API response validation failed: {0}")]
    #[allow(dead_code)]
    ResponseValidationFailed(String),
    /// API endpoint not available
    #[error("API endpoint '{0}' is not available")]
    #[allow(dead_code)]
    EndpointUnavailable(String),
}

/// Event processing errors
#[derive(Error, Debug)]
pub enum EventError {
    /// Invalid chain specified
    #[error("Invalid chain '{0}' specified")]
    InvalidChain(String),
    /// Contract address invalid
    #[error("Invalid contract address '{0}'")]
    InvalidAddress(String),
    /// Event parsing failed
    #[error("Failed to parse event data: {0}")]
    #[allow(dead_code)]
    ParseError(String),
    /// RPC connection failed
    #[error("RPC connection failed: {0}")]
    RpcConnectionFailed(String),
}

// Note: Basic From implementations are handled automatically by thiserror's #[from] attribute

// Conversion from external library errors
impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::NetworkError("Request timeout".to_string())
        } else if err.is_connect() {
            ApiError::NetworkError("Connection failed".to_string())
        } else if let Some(status) = err.status() {
            ApiError::RequestFailed {
                url: err
                    .url()
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                status: status.as_u16(),
                message: err.to_string(),
            }
        } else {
            ApiError::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::JsonParseError(err.to_string())
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
