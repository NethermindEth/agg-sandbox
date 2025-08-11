use crate::config::Config;
use crate::logs;
use crate::progress::{MultiStepProgress, ProgressBar, StatusReporter};
use colored::*;
use tracing::{error, info};

/// Handle the start command
#[allow(clippy::disallowed_methods)] // Allow std::process::exit and tracing macros
pub async fn handle_start(detach: bool, build: bool, fork: bool, multi_l2: bool, claim_all: bool) {
    handle_start_async(detach, build, fork, multi_l2, claim_all).await;
}

/// Async implementation of start command with progress tracking
#[allow(clippy::disallowed_methods)] // Allow std::process::exit and tracing macros
async fn handle_start_async(detach: bool, build: bool, fork: bool, multi_l2: bool, claim_all: bool) {
    use crate::docker::{execute_docker_command, SandboxConfig};

    let reporter = StatusReporter::new();

    // Setup progress tracking
    let steps = vec![
        "Validating configuration".to_string(),
        "Setting up environment".to_string(),
        if build {
            "Building Docker images".to_string()
        } else {
            "Preparing Docker services".to_string()
        },
        "Starting services".to_string(),
        "Verifying startup".to_string(),
    ];

    let mut progress = MultiStepProgress::new(steps);

    // Create sandbox configuration
    let config = SandboxConfig::new(fork, multi_l2, claim_all);

    info!(
        mode = %config.mode_description(),
        detach = detach,
        build = build,
        fork = fork,
        multi_l2 = multi_l2,
        claim_all = claim_all,
        "Starting Agglayer sandbox environment"
    );

    println!(
        "{}",
        format!(
            "🚀 Starting Agglayer sandbox environment in {}",
            config.mode_description()
        )
        .green()
        .bold()
    );

    // Step 1: Validate configuration
    if let Some(handle) = progress.start_step("Validating configuration") {
        // Validate fork mode configuration
        if let Err(e) = config.validate_fork_config() {
            progress.fail_step(handle, &e.to_string());
            error!(error = %e, "Fork mode configuration validation failed");
            reporter
                .error(&format!("Configuration validation failed: {e}"))
                .await;
            reporter
                .tip("Please set the required fork URLs in your .env file")
                .await;
            std::process::exit(1);
        }
        progress.complete_step(handle);
    }

    // Step 2: Setup environment
    if let Some(handle) = progress.start_step("Setting up environment") {
        // Display fork URLs if in fork mode
        if fork {
            info!("Displaying fork URL configuration");
            display_fork_urls(multi_l2);
        }

        // Create Docker builder with proper configuration
        info!("Creating Docker configuration");
        let docker_builder = config.create_docker_builder();

        progress.complete_step(handle);

        // Build and execute Docker command
        info!(detach = detach, build = build, "Building Docker command");
        let cmd = docker_builder.build_up_command(detach, build);

        // Step 3: Build/Prepare Docker images
        if let Some(handle) = progress.start_step(if build {
            "Building Docker images"
        } else {
            "Preparing Docker services"
        }) {
            // This step is handled by Docker, so we complete it immediately
            progress.complete_step(handle);
        }

        // Step 4: Start services
        if let Some(handle) = progress.start_step("Starting services") {
            if detach {
                // Execute in detached mode with progress
                info!("Executing Docker command in detached mode");
                let mut progress_bar = ProgressBar::new("Starting sandbox services...".to_string());
                let progress_handle = progress_bar.start().await;

                if execute_docker_command(cmd, true).is_err() {
                    progress_handle
                        .finish_with_error("Failed to start sandbox services")
                        .await;
                    progress.fail_step(handle, "Docker command execution failed");
                    error!("Failed to start sandbox in detached mode");
                    reporter
                        .error("Failed to start sandbox in detached mode")
                        .await;
                    std::process::exit(1);
                }

                progress_handle
                    .finish_with_message("Services started successfully")
                    .await;
                progress.complete_step(handle);

                // Step 5: Verify startup
                if let Some(verify_handle) = progress.start_step("Verifying startup") {
                    // Display success message
                    let success_msg = match (fork, multi_l2) {
                        (true, true) => "Multi-L2 sandbox started in fork mode (detached)",
                        (true, false) => "Sandbox started in fork mode (detached)",
                        (false, true) => "Multi-L2 sandbox started (detached)",
                        (false, false) => "Sandbox started in detached mode",
                    };
                    info!(
                        fork = fork,
                        multi_l2 = multi_l2,
                        "Sandbox started successfully in detached mode"
                    );

                    progress.complete_step(verify_handle);

                    // Print success message after progress is complete
                    println!("\n{} {}", "✅".green().bold(), success_msg.green());
                }

                // Load config and print appropriate info
                info!("Loading configuration for info display");
                if let Ok(config) = Config::load() {
                    match (fork, multi_l2) {
                        (_, true) => {
                            info!("Displaying multi-L2 configuration info");
                            logs::print_multi_l2_info(&config, fork);
                        }
                        (true, false) => {
                            info!("Displaying fork mode configuration info");
                            logs::print_sandbox_fork_info(&config);
                        }
                        (false, false) => {
                            info!("Displaying standard configuration info");
                            logs::print_sandbox_info(&config);
                        }
                    }
                }
            } else {
                // Run in foreground mode
                info!("Starting sandbox in foreground mode");
                reporter
                    .info("Starting services in foreground mode...")
                    .await;
                reporter.warning("Press Ctrl+C to stop the sandbox").await;

                if execute_docker_command(cmd, false).is_err() {
                    progress.fail_step(handle, "Docker command execution failed");
                    error!("Failed to start sandbox in foreground mode");
                    reporter
                        .error("Failed to start sandbox in foreground mode")
                        .await;
                    std::process::exit(1);
                } else {
                    progress.complete_step(handle);
                    info!("Sandbox stopped gracefully");
                    reporter.success("Sandbox stopped gracefully").await;
                }
            }
        }
    }
}

#[allow(clippy::disallowed_methods)] // Allow tracing macros
fn display_fork_urls(multi_l2: bool) {
    let fork_mainnet = std::env::var("FORK_URL_MAINNET").unwrap_or_default();
    let fork_agglayer_1 = std::env::var("FORK_URL_AGGLAYER_1").unwrap_or_default();

    if multi_l2 {
        let fork_agglayer_2 = std::env::var("FORK_URL_AGGLAYER_2").unwrap_or_default();
        info!(
            mainnet = %fork_mainnet,
            agglayer_1 = %fork_agglayer_1,
            agglayer_2 = %fork_agglayer_2,
            multi_l2 = true,
            "Displaying fork URLs for multi-L2 configuration"
        );
        println!("{}", "Fork URLs detected:".cyan());
        println!("  Mainnet: {}", fork_mainnet.yellow());
        println!("  Agglayer 1: {}", fork_agglayer_1.yellow());
        println!("  Agglayer 2: {}", fork_agglayer_2.yellow());
    } else {
        info!(
            mainnet = %fork_mainnet,
            agglayer_1 = %fork_agglayer_1,
            multi_l2 = false,
            "Displaying fork URLs for single L2 configuration"
        );
        println!("{}", "Fork URLs detected:".cyan());
        println!("  Mainnet: {}", fork_mainnet.yellow());
        println!("  Agglayer 1: {}", fork_agglayer_1.yellow());
    }
}
