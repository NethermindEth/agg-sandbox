use crate::ui;

/// Handle the status command
#[allow(clippy::disallowed_methods)] // Allow std::process::exit for command handler
pub fn handle_status() {
    use crate::docker::{create_auto_docker_builder, execute_docker_command_with_output};

    ui::ui().info("📊 Sandbox service status:");

    // Create Docker builder that auto-detects configuration
    let docker_builder = create_auto_docker_builder();
    let cmd = docker_builder.build_ps_command();

    // Execute the status command and display output
    if let Ok(output) = execute_docker_command_with_output(cmd) {
        print!("{output}");
    } else {
        ui::ui().error("Failed to get service status");
        std::process::exit(1);
    }
}
