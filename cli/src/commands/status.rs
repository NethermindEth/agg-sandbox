use colored::*;

/// Handle the status command
pub fn handle_status() {
    use crate::docker::{create_auto_docker_builder, execute_docker_command_with_output};

    println!("{}", "📊 Sandbox service status:".blue().bold());

    // Create Docker builder that auto-detects configuration
    let docker_builder = create_auto_docker_builder();
    let cmd = docker_builder.build_ps_command();

    // Execute the status command and display output
    if let Ok(output) = execute_docker_command_with_output(cmd) {
        print!("{output}");
    } else {
        eprintln!("{}", "❌ Failed to get service status".red());
        std::process::exit(1);
    }
}
