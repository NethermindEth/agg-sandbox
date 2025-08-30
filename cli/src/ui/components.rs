//! UI components for different types of messages
//!
//! This module provides specialized message components like errors, warnings,
//! success messages, etc. Each component knows how to format itself for both
//! human and JSON output.

use crate::ui::message::Message;
use colored::*;
use serde::Serialize;
use serde_json::{json, Value};

/// Success message with green checkmark
#[derive(Debug, Serialize)]
pub struct SuccessMessage {
    pub content: String,
}

impl SuccessMessage {
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Message for SuccessMessage {
    fn text(&self) -> String {
        format!("{} {}", "✓".green().bold(), self.content)
    }

    fn json(&self) -> Value {
        json!({
            "message_type": "success",
            "content": self.content
        })
    }
}

/// Error message with red X mark
#[derive(Debug, Serialize)]
pub struct ErrorMessage {
    pub content: String,
}

impl ErrorMessage {
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Message for ErrorMessage {
    fn text(&self) -> String {
        format!("{} {}", "✗".red().bold(), self.content)
    }

    fn json(&self) -> Value {
        json!({
            "message_type": "error",
            "content": self.content
        })
    }
}

/// Warning message with yellow warning sign
#[derive(Debug, Serialize)]
pub struct WarningMessage {
    pub content: String,
}

impl WarningMessage {
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Message for WarningMessage {
    fn text(&self) -> String {
        format!("{} {}", "⚠".yellow().bold(), self.content)
    }

    fn json(&self) -> Value {
        json!({
            "message_type": "warning",
            "content": self.content
        })
    }
}

/// Info message with blue info icon
#[derive(Debug, Serialize)]
pub struct InfoMessage {
    pub content: String,
}

impl InfoMessage {
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Message for InfoMessage {
    fn text(&self) -> String {
        format!("{} {}", "ℹ".blue().bold(), self.content)
    }

    fn json(&self) -> Value {
        json!({
            "message_type": "info",
            "content": self.content
        })
    }
}

/// Tip message with light bulb
#[derive(Debug, Serialize)]
pub struct TipMessage {
    pub content: String,
}

impl TipMessage {
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Message for TipMessage {
    fn text(&self) -> String {
        format!("{} Tip: {}", "💡".bright_yellow().bold(), self.content)
    }

    fn json(&self) -> Value {
        json!({
            "message_type": "tip",
            "content": self.content
        })
    }
}
