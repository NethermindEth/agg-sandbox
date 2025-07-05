use crate::error::Result;
use crate::events;

/// Handle the events command
pub async fn handle_events(chain: String, blocks: u64, address: Option<String>) -> Result<()> {
    events::fetch_and_display_events(&chain, blocks, address).await
}
