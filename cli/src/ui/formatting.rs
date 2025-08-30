//! Formatting utilities for structured output
//!
//! This module provides formatters for different types of structured data
//! like tables, JSON, and other complex layouts.

use crate::ui::message::Message;
use colored::*;
use serde::Serialize;
use serde_json::Value;

/// Table formatter for creating nicely formatted tables
pub struct TableFormatter {
    title: Option<String>,
    rows: Vec<(String, String)>,
    emoji: Option<String>,
}

impl Default for TableFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TableFormatter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: None,
            rows: Vec::new(),
            emoji: None,
        }
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn rows(mut self, rows: &[(&str, &str)]) -> Self {
        self.rows = rows
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self
    }

    #[must_use]
    pub fn build(self) -> FormattedTable {
        FormattedTable {
            title: self.title,
            rows: self.rows,
            emoji: self.emoji,
        }
    }
}

/// A formatted table ready for display
#[derive(Debug, Serialize)]
pub struct FormattedTable {
    title: Option<String>,
    rows: Vec<(String, String)>,
    emoji: Option<String>,
}

impl Message for FormattedTable {
    fn text(&self) -> String {
        let mut output = String::new();

        // Add title with optional emoji
        if let Some(title) = &self.title {
            let title_line = if let Some(emoji) = &self.emoji {
                format!("{} {}", emoji, title)
            } else {
                title.clone()
            };
            output.push_str(&format!("{}\n", title_line.bold()));
            output.push_str(
                "┌────────────────────────┬─────────────────────────────────────────────┐\n",
            );
        }

        // Add rows
        for (key, value) in &self.rows {
            output.push_str(&format!(
                "│ {:<22} │ {:<43} │\n",
                key.bright_white(),
                value.yellow()
            ));
        }

        // Add footer
        if self.title.is_some() {
            output.push_str(
                "└────────────────────────┴─────────────────────────────────────────────┘",
            );
        }

        output
    }

    fn json(&self) -> Value {
        let rows_object: serde_json::Map<String, serde_json::Value> = self
            .rows
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();

        serde_json::json!({
            "message_type": "table",
            "title": self.title,
            "emoji": self.emoji,
            "data": rows_object
        })
    }
}

/// JSON formatter for pretty-printing JSON data
pub struct JsonFormatter<'a> {
    data: &'a Value,
    title: Option<String>,
}

impl<'a> JsonFormatter<'a> {
    #[must_use]
    pub fn new(data: &'a Value) -> Self {
        Self { data, title: None }
    }

    #[must_use]
    pub fn build(self) -> FormattedJson<'a> {
        FormattedJson {
            data: self.data,
            title: self.title,
        }
    }
}

/// A formatted JSON object ready for display
#[derive(Debug)]
pub struct FormattedJson<'a> {
    data: &'a Value,
    title: Option<String>,
}

impl<'a> Message for FormattedJson<'a> {
    fn text(&self) -> String {
        let mut output = String::new();

        if let Some(title) = &self.title {
            output.push_str(&format!("{}\n", title.bold()));
            output.push_str(&"─".repeat(50));
            output.push('\n');
        }

        output.push_str(&serde_json::to_string_pretty(self.data).unwrap_or_default());
        output
    }

    fn json(&self) -> Value {
        if let Some(title) = &self.title {
            serde_json::json!({
                "message_type": "formatted_json",
                "title": title,
                "data": self.data
            })
        } else {
            self.data.clone()
        }
    }
}

impl<'a> Serialize for FormattedJson<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // For serialization, we just serialize the underlying data
        self.data.serialize(serializer)
    }
}
