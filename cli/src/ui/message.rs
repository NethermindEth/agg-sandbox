//! Message trait and basic message implementations
//!
//! This module defines the core Message trait that allows different types
//! of data to be formatted for both human-readable and JSON output.

use serde_json::Value;

/// A trait for objects that can be formatted as both human-readable text and JSON
pub trait Message {
    /// Return the human-readable text representation
    fn text(&self) -> String;

    /// Return the JSON representation
    fn json(&self) -> Value;
}
