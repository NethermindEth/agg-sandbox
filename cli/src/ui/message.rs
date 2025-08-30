//! Message trait and basic message implementations
//!
//! This module defines the core Message trait that allows different types
//! of data to be formatted for both human-readable and JSON output.

use serde::Serialize;
use serde_json::{json, Value};

/// A trait for objects that can be formatted as both human-readable text and JSON
pub trait Message {
    /// Return the human-readable text representation
    fn text(&self) -> String;

    /// Return the JSON representation
    fn json(&self) -> Value;
}

/// Basic implementation for types that can be converted to string and serialized
impl<T> Message for T
where
    T: ToString + Serialize,
{
    fn text(&self) -> String {
        self.to_string()
    }

    fn json(&self) -> Value {
        // Try to serialize the object directly first
        serde_json::to_value(self).unwrap_or_else(|_| json!({ "message": self.to_string() }))
    }
}

/// A simple text message
#[derive(Debug, Serialize)]
pub struct TextMessage {
    pub content: String,
}

impl TextMessage {
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Message for TextMessage {
    fn text(&self) -> String {
        self.content.clone()
    }

    fn json(&self) -> Value {
        json!({
            "message_type": "text",
            "content": self.content
        })
    }
}

/// A message with structured data
#[derive(Debug, Serialize)]
pub struct DataMessage {
    pub title: String,
    pub data: Value,
}

impl DataMessage {
    #[must_use]
    pub fn new<T: Serialize>(title: impl Into<String>, data: &T) -> Self {
        Self {
            title: title.into(),
            data: json!(data),
        }
    }
}

impl Message for DataMessage {
    fn text(&self) -> String {
        // For human output, we'll format this nicely
        format!(
            "{}\n{}",
            self.title,
            serde_json::to_string_pretty(&self.data).unwrap_or_default()
        )
    }

    fn json(&self) -> Value {
        json!({
            "message_type": "data",
            "title": self.title,
            "data": self.data
        })
    }
}

/// A table-like message with title and key-value rows
#[derive(Debug, Serialize)]
pub struct TableMessage {
    pub title: String,
    pub rows: Vec<(String, String)>,
}

impl TableMessage {
    #[must_use]
    pub fn new(title: impl Into<String>, rows: &[(&str, &str)]) -> Self {
        Self {
            title: title.into(),
            rows: rows
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

impl Message for TableMessage {
    fn text(&self) -> String {
        // This will be handled by TableFormatter for human output
        format!("{}: {} rows", self.title, self.rows.len())
    }

    fn json(&self) -> Value {
        let rows_object: serde_json::Map<String, Value> = self
            .rows
            .iter()
            .map(|(k, v)| (k.clone(), json!(v)))
            .collect();

        json!({
            "message_type": "table",
            "title": self.title,
            "data": rows_object
        })
    }
}
