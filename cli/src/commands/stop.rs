use colored::*;

/// Handle the stop command
pub fn handle_stop(volumes: bool) {
    use crate::docker::{create_auto_docker_builder, execute_docker_command};

    println!(
        "{}",
        "🛑 Stopping AggLayer sandbox environment..."
            .yellow()
            .bold()
    );

    // Create Docker builder that auto-detects configuration
    let docker_builder = create_auto_docker_builder();
    let cmd = docker_builder.build_down_command(volumes);

    // Execute the stop command
    if execute_docker_command(cmd, true).is_err() {
        eprintln!("{}", "❌ Failed to stop sandbox".red());
        std::process::exit(1);
    } else {
        println!("{}", "✅ Sandbox stopped successfully".green());
    }
}
