//! Unified UI module for consistent user interface across the CLI
//!
//! This module provides a centralized way to handle all user-facing output,
//! ensuring consistent formatting, styling, and support for both human-readable
//! and JSON output formats.

use serde::Serialize;

pub mod components;
pub mod formatting;
pub mod message;

pub use components::*;
pub use formatting::*;
pub use message::*;

/// Output format for the CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Human-readable output with colors and formatting
    #[default]
    Human,
    /// JSON output for scripting and automation
    Json,
}

/// Central UI manager for all CLI output
///
/// This struct manages output formatting, colors, and provides consistent
/// methods for displaying various types of information to users.
#[derive(Debug, Default)]
pub struct UI {
    output_format: OutputFormat,
    quiet: bool,
}

impl UI {
    /// Create a new UI instance with the specified output format
    #[must_use]
    pub fn new(output_format: OutputFormat) -> Self {
        Self {
            output_format,
            quiet: false,
        }
    }

    /// Create a UI instance with quiet mode enabled
    #[must_use]
    pub fn quiet(output_format: OutputFormat) -> Self {
        Self {
            output_format,
            quiet: true,
        }
    }

    /// Check if the output format is JSON
    #[must_use]
    pub fn is_json(&self) -> bool {
        matches!(self.output_format, OutputFormat::Json)
    }

    /// Format a message according to the current output format
    fn format_message(&self, message: &impl Message) -> String {
        match self.output_format {
            OutputFormat::Human => message.text(),
            OutputFormat::Json => message.json().to_string(),
        }
    }

    /// Print a message to stdout
    pub fn println(&self, message: &impl Message) {
        if !self.quiet {
            println!("{}", self.format_message(message));
        }
    }

    /// Print a message to stderr
    pub fn eprintln(&self, message: &impl Message) {
        eprintln!("{}", self.format_message(message));
    }

    /// Print a blank line
    pub fn blank_line(&self) {
        if !self.quiet && matches!(self.output_format, OutputFormat::Human) {
            println!();
        }
    }

    /// Print success message
    pub fn success(&self, message: &str) {
        self.println(&SuccessMessage::new(message));
    }

    /// Print error message
    pub fn error(&self, message: &str) {
        self.eprintln(&ErrorMessage::new(message));
    }

    /// Print warning message
    pub fn warning(&self, message: &str) {
        if !self.quiet {
            self.println(&WarningMessage::new(message));
        }
    }

    /// Print info message
    pub fn info(&self, message: &str) {
        if !self.quiet {
            self.println(&InfoMessage::new(message));
        }
    }

    /// Print tip message
    pub fn tip(&self, message: &str) {
        if !self.quiet {
            self.println(&TipMessage::new(message));
        }
    }

    /// Print a table with title and rows
    pub fn table(&self, title: &str, rows: &[(&str, &str)]) {
        let table_output = TableFormatter::new().title(title).rows(rows).build();
        self.println(&table_output);
    }

    /// Print structured data as JSON or formatted output
    pub fn data<T: Serialize>(&self, _title: &str, data: &T) {
        if self.is_json() {
            if let Ok(json_val) = serde_json::to_value(data) {
                let formatted = JsonFormatter::new(&json_val).build();
                self.println(&formatted);
            }
        } else {
            if let Ok(json_val) = serde_json::to_value(data) {
                let formatted = JsonFormatter::new(&json_val).build();
                self.println(&formatted);
            }
        }
    }

    /// Print raw JSON (only for JSON mode, otherwise formats nicely)
    pub fn json(&self, data: &serde_json::Value) {
        if self.is_json() {
            println!("{}", data);
        } else {
            let formatted = JsonFormatter::new(data).build();
            self.println(&formatted);
        }
    }
}

/// Global UI instance for backward compatibility during migration
static mut GLOBAL_UI: Option<UI> = None;
static UI_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global UI instance
pub fn init_ui(output_format: OutputFormat, quiet: bool) {
    UI_INIT.call_once(|| unsafe {
        GLOBAL_UI = Some(if quiet {
            UI::quiet(output_format)
        } else {
            UI::new(output_format)
        });
    });
}

/// Get the global UI instance
///
/// # Panics
/// Panics if UI has not been initialized with `init_ui()`
#[must_use]
pub fn ui() -> &'static UI {
    unsafe {
        GLOBAL_UI
            .as_ref()
            .expect("UI not initialized. Call init_ui() first.")
    }
}

/// Convenience macros for common UI operations
#[macro_export]
macro_rules! ui_success {
    ($msg:expr) => {
        $crate::ui::ui().success($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ui::ui().success(&format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! ui_error {
    ($msg:expr) => {
        $crate::ui::ui().error($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ui::ui().error(&format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! ui_warning {
    ($msg:expr) => {
        $crate::ui::ui().warning($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ui::ui().warning(&format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! ui_info {
    ($msg:expr) => {
        $crate::ui::ui().info($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ui::ui().info(&format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! ui_tip {
    ($msg:expr) => {
        $crate::ui::ui().tip($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ui::ui().tip(&format!($fmt, $($arg)*))
    };
}
