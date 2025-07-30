/// Command handlers for the Agglayer sandbox CLI
///
/// This module contains all command handlers, extracted from main.rs
/// for better code organization and maintainability.
pub mod events;
pub mod info;
pub mod logs;
pub mod restart;
pub mod show;
pub mod sponsor_claim;
pub mod start;
pub mod status;
pub mod stop;

#[cfg(test)]
mod tests;

// Re-export command handlers for easier access
pub use events::handle_events;
pub use info::handle_info;
pub use logs::handle_logs;
pub use restart::handle_restart;
pub use show::{handle_show, ShowCommands};
pub use sponsor_claim::{handle_sponsor_claim, handle_claim_status};
pub use start::handle_start;
pub use status::handle_status;
pub use stop::handle_stop;
