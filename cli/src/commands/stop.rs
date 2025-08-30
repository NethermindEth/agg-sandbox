use crate::ui;

/// Handle the stop command
#[allow(clippy::disallowed_methods)] // Allow std::process::exit for command handler
pub fn handle_stop(volumes: bool) {
    use crate::docker::{create_auto_docker_builder, execute_docker_command};

    ui::ui().warning("🛑 Stopping Agglayer sandbox environment...");

    // Create Docker builder that auto-detects configuration
    let docker_builder = create_auto_docker_builder();
    let cmd = docker_builder.build_down_command(volumes);

    // Execute the stop command
    if execute_docker_command(cmd, true).is_err() {
        ui::ui().error("Failed to stop sandbox");
        std::process::exit(1);
    } else {
        ui::ui().success("Sandbox stopped successfully");
    }
}
