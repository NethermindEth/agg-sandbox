use crate::error::Result;
use crate::validation::Validator;
use colored::*;

/// Handle the logs command
pub fn handle_logs(follow: bool, service: Option<String>) -> Result<()> {
    use crate::docker::{
        create_auto_docker_builder, execute_docker_command, execute_docker_command_with_output,
    };

    // Validate service name if provided
    let validated_service = if let Some(svc) = service {
        Some(Validator::validate_service_name(&svc)?)
    } else {
        None
    };

    let service_name = validated_service.as_deref().unwrap_or("all services");
    println!(
        "{} {}",
        "📋 Showing logs for:".blue().bold(),
        service_name.cyan()
    );

    // Create Docker builder that auto-detects configuration
    let mut docker_builder = create_auto_docker_builder();

    // Add service if specified
    if let Some(svc) = validated_service {
        docker_builder.add_service(svc);
    }

    let cmd = docker_builder.build_logs_command(follow);

    // Handle follow vs non-follow modes differently
    if follow {
        // For follow mode, we need real-time output
        if execute_docker_command(cmd, false).is_err() {
            eprintln!("{}", "❌ Failed to show logs".red());
            std::process::exit(1);
        }
    } else {
        // For non-follow mode, capture and display output
        match execute_docker_command_with_output(cmd) {
            Ok(output) => {
                print!("{output}");
            }
            Err(_) => {
                eprintln!("{}", "❌ Failed to show logs".red());
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
