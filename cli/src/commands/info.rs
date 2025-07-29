use crate::config::Config;
use crate::error::Result;
use crate::logs;
use colored::*;

/// Handle the info command
pub async fn handle_info() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "📋 Agglayer Sandbox Information".blue().bold());
    logs::print_sandbox_info(&config);

    Ok(())
}
