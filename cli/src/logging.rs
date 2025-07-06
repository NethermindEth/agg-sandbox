/// Structured logging configuration and utilities
///
/// This module provides centralized logging setup using the tracing crate,
/// supporting multiple output formats and verbosity levels for better debugging
/// and production monitoring.
use std::env;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Logging output format options
#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    /// Human-readable format with colors (default for interactive use)
    Pretty,
    /// Compact single-line format
    Compact,
    /// Structured JSON format (for production/parsing)
    Json,
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Verbosity level
    pub level: Level,
    /// Output format
    pub format: LogFormat,
    /// Whether to include file/line information
    pub include_location: bool,
    /// Whether to include target module information
    pub include_target: bool,
    /// Whether to include span information
    pub include_spans: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            format: LogFormat::Pretty,
            include_location: false,
            include_target: false,
            include_spans: false,
        }
    }
}

impl LogConfig {
    /// Create a verbose logging configuration
    #[allow(dead_code)]
    pub fn verbose() -> Self {
        Self {
            level: Level::DEBUG,
            format: LogFormat::Pretty,
            include_location: true,
            include_target: true,
            include_spans: true,
        }
    }

    /// Create a quiet logging configuration
    #[allow(dead_code)]
    pub fn quiet() -> Self {
        Self {
            level: Level::ERROR,
            format: LogFormat::Compact,
            include_location: false,
            include_target: false,
            include_spans: false,
        }
    }

    /// Create a production logging configuration
    #[allow(dead_code)]
    pub fn production() -> Self {
        Self {
            level: Level::INFO,
            format: LogFormat::Json,
            include_location: true,
            include_target: true,
            include_spans: true,
        }
    }
}

/// Initialize logging with the given configuration
pub fn init_logging(config: &LogConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create base filter from level
    let filter = EnvFilter::builder()
        .with_default_directive(config.level.into())
        .from_env_lossy()
        // Allow overriding specific modules
        .add_directive("aggsandbox=debug".parse()?)
        .add_directive("hyper=warn".parse()?)
        .add_directive("reqwest=warn".parse()?)
        .add_directive("wiremock=warn".parse()?);

    // Configure span events
    let span_events = if config.include_spans {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    match config.format {
        LogFormat::Pretty => {
            let fmt_layer = fmt::layer()
                .with_ansi(atty::is(atty::Stream::Stderr))
                .with_target(config.include_target)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .with_span_events(span_events)
                .pretty();

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Compact => {
            let fmt_layer = fmt::layer()
                .with_ansi(atty::is(atty::Stream::Stderr))
                .with_target(config.include_target)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .with_span_events(span_events)
                .compact();

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Json => {
            let fmt_layer = fmt::layer()
                .with_target(config.include_target)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .with_span_events(span_events)
                .json();

            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
        }
    }

    Ok(())
}

/// Initialize logging with automatic configuration based on environment
#[allow(dead_code)]
pub fn init_auto_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = if env::var("AGGSANDBOX_LOG_FORMAT").as_deref() == Ok("json") {
        LogConfig::production()
    } else if env::var("AGGSANDBOX_VERBOSE").is_ok() || env::var("RUST_LOG").is_ok() {
        LogConfig::verbose()
    } else if env::var("AGGSANDBOX_QUIET").is_ok() {
        LogConfig::quiet()
    } else {
        LogConfig::default()
    };

    init_logging(&config)
}

/// Get appropriate logging level from CLI verbosity flags
pub fn level_from_verbosity(verbose: u8, quiet: bool) -> Level {
    if quiet {
        Level::ERROR
    } else {
        match verbose {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        }
    }
}

/// Get logging format from string
pub fn format_from_str(format: &str) -> Result<LogFormat, String> {
    match format.to_lowercase().as_str() {
        "pretty" => Ok(LogFormat::Pretty),
        "compact" => Ok(LogFormat::Compact),
        "json" => Ok(LogFormat::Json),
        _ => Err(format!(
            "Unknown log format: {format}. Supported: pretty, compact, json"
        )),
    }
}

/// Structured logging macros for common operations
#[macro_export]
macro_rules! log_operation {
    ($operation:expr, $($field:tt)*) => {
        tracing::info!(
            operation = $operation,
            $($field)*
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $context:expr, $($field:tt)*) => {
        tracing::error!(
            error = %$error,
            context = $context,
            $($field)*
        );
    };
}

#[macro_export]
macro_rules! log_timing {
    ($operation:expr, $duration:expr, $($field:tt)*) => {
        tracing::info!(
            operation = $operation,
            duration_ms = $duration.as_millis(),
            $($field)*
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_defaults() {
        let config = LogConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert!(matches!(config.format, LogFormat::Pretty));
        assert!(!config.include_location);
        assert!(!config.include_target);
        assert!(!config.include_spans);
    }

    #[test]
    fn test_verbose_config() {
        let config = LogConfig::verbose();
        assert_eq!(config.level, Level::DEBUG);
        assert!(config.include_location);
        assert!(config.include_target);
        assert!(config.include_spans);
    }

    #[test]
    fn test_quiet_config() {
        let config = LogConfig::quiet();
        assert_eq!(config.level, Level::ERROR);
        assert!(matches!(config.format, LogFormat::Compact));
        assert!(!config.include_location);
    }

    #[test]
    fn test_production_config() {
        let config = LogConfig::production();
        assert_eq!(config.level, Level::INFO);
        assert!(matches!(config.format, LogFormat::Json));
        assert!(config.include_location);
        assert!(config.include_target);
    }

    #[test]
    fn test_level_from_verbosity() {
        assert_eq!(level_from_verbosity(0, false), Level::INFO);
        assert_eq!(level_from_verbosity(1, false), Level::DEBUG);
        assert_eq!(level_from_verbosity(2, false), Level::TRACE);
        assert_eq!(level_from_verbosity(0, true), Level::ERROR);
        assert_eq!(level_from_verbosity(1, true), Level::ERROR);
    }

    #[test]
    fn test_format_from_str() {
        assert!(matches!(format_from_str("pretty"), Ok(LogFormat::Pretty)));
        assert!(matches!(format_from_str("compact"), Ok(LogFormat::Compact)));
        assert!(matches!(format_from_str("json"), Ok(LogFormat::Json)));
        assert!(matches!(format_from_str("PRETTY"), Ok(LogFormat::Pretty)));
        assert!(format_from_str("invalid").is_err());
    }
}
