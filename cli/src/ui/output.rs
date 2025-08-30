//! Output utilities and compatibility layer
//!
//! This module provides utilities for output management and backward compatibility
//! with existing printing patterns in the CLI.

use crate::ui::{formatting::*, UI};
use serde_json::Value;

/// Backward compatibility functions for existing code
pub mod compat {
    use super::*;
    use crate::ui;

    /// Print JSON response with title (backward compatibility)
    pub fn print_json_response(title: &str, data: &Value) {
        // Create a UI instance for human output (preserving old behavior)
        let ui_instance = crate::ui::UI::new(crate::ui::OutputFormat::Human);
        let formatted = JsonFormatter::new(data)
            .title(&format!("📋 {}", title))
            .build();
        ui_instance.println(&formatted);
    }

    /// Print raw JSON (backward compatibility)
    pub fn print_raw_json(data: &Value) {
        // Raw JSON output - just print the JSON directly
        if let Ok(json_str) = serde_json::to_string(data) {
            println!("{}", json_str);
        } else {
            println!("{:?}", data);
        }
    }

    /// Print table (backward compatibility)
    pub fn print_table(title: &str, rows: &[(&str, &str)]) {
        let ui_instance = crate::ui::UI::new(crate::ui::OutputFormat::Human);
        ui_instance.table(title, rows);
    }
}

/// Output builder for complex formatting
pub struct OutputBuilder {
    ui: UI,
    elements: Vec<OutputElement>,
}

#[derive(Debug)]
enum OutputElement {
    Success(String),
    Error(String),
    Warning(String),
    Info(String),
    Tip(String),
    Table {
        title: String,
        rows: Vec<(String, String)>,
    },
    Json {
        title: Option<String>,
        data: Value,
    },
    BlankLine,
}

impl OutputBuilder {
    #[must_use]
    pub fn new(ui: UI) -> Self {
        Self {
            ui,
            elements: Vec::new(),
        }
    }

    #[must_use]
    pub fn success(mut self, message: impl Into<String>) -> Self {
        self.elements.push(OutputElement::Success(message.into()));
        self
    }

    #[must_use]
    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.elements.push(OutputElement::Error(message.into()));
        self
    }

    #[must_use]
    pub fn warning(mut self, message: impl Into<String>) -> Self {
        self.elements.push(OutputElement::Warning(message.into()));
        self
    }

    #[must_use]
    pub fn info(mut self, message: impl Into<String>) -> Self {
        self.elements.push(OutputElement::Info(message.into()));
        self
    }

    #[must_use]
    pub fn tip(mut self, message: impl Into<String>) -> Self {
        self.elements.push(OutputElement::Tip(message.into()));
        self
    }

    #[must_use]
    pub fn table(mut self, title: impl Into<String>, rows: &[(&str, &str)]) -> Self {
        self.elements.push(OutputElement::Table {
            title: title.into(),
            rows: rows
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        });
        self
    }

    #[must_use]
    pub fn json(mut self, title: Option<impl Into<String>>, data: &Value) -> Self {
        self.elements.push(OutputElement::Json {
            title: title.map(Into::into),
            data: data.clone(),
        });
        self
    }

    #[must_use]
    pub fn blank_line(mut self) -> Self {
        self.elements.push(OutputElement::BlankLine);
        self
    }

    /// Build and output all elements
    pub fn build(self) {
        for element in self.elements {
            match element {
                OutputElement::Success(msg) => self.ui.success(&msg),
                OutputElement::Error(msg) => self.ui.error(&msg),
                OutputElement::Warning(msg) => self.ui.warning(&msg),
                OutputElement::Info(msg) => self.ui.info(&msg),
                OutputElement::Tip(msg) => self.ui.tip(&msg),
                OutputElement::Table { title, rows } => {
                    let rows_ref: Vec<(&str, &str)> =
                        rows.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                    self.ui.table(&title, &rows_ref);
                }
                OutputElement::Json { title, data } => {
                    if let Some(title) = title {
                        let formatted = JsonFormatter::new(&data).title(title).build();
                        self.ui.println(&formatted);
                    } else {
                        self.ui.json(&data);
                    }
                }
                OutputElement::BlankLine => self.ui.blank_line(),
            }
        }
    }
}

/// Helper for migrating from println! patterns
pub fn migrate_println_usage() {
    eprintln!("⚠️  Consider migrating from direct println! usage to the UI module");
    eprintln!("   Replace: println!(\"message\")");
    eprintln!("   With: ui::ui().info(\"message\")");
    eprintln!("   Or use: ui_info!(\"message\") macro");
}
