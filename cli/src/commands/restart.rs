use colored::*;

/// Handle the restart command
pub async fn handle_restart() {
    println!(
        "{}",
        "🔄 Restarting Agglayer sandbox environment..."
            .yellow()
            .bold()
    );

    // First stop
    super::stop::handle_stop(false);

    // Then start in basic local mode
    super::start::handle_start(true, false, false, false, false).await;

    println!("{}", "✅ Sandbox restarted successfully".green());
}
