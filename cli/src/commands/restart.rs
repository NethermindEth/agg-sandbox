use crate::error::Result;
use colored::*;

/// Handle the restart command
pub fn handle_restart() -> Result<()> {
    println!(
        "{}",
        "🔄 Restarting AggLayer sandbox environment..."
            .yellow()
            .bold()
    );

    // First stop
    super::stop::handle_stop(false)?;

    // Then start in basic local mode
    super::start::handle_start(true, false, false, false)?;

    println!("{}", "✅ Sandbox restarted successfully".green());

    Ok(())
}
