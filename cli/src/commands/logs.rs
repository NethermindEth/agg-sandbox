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
        execute_docker_command(cmd, false).inspect_err(|_e| {
            eprintln!("{}", "❌ Failed to show logs".red());
        })?
    } else {
        // For non-follow mode, capture and display output
        let output = execute_docker_command_with_output(cmd).inspect_err(|_e| {
            eprintln!("{}", "❌ Failed to show logs".red());
        })?;
        print!("{output}");
    }

    Ok(())
}
