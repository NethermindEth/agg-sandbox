use crate::config::Config;
use crate::error::Result;
use crate::logs;
use colored::*;
use tracing::{error, info};

/// Handle the start command
pub fn handle_start(detach: bool, build: bool, fork: bool, multi_l2: bool) -> Result<()> {
    use crate::docker::{execute_docker_command, SandboxConfig};

    // Create sandbox configuration
    let config = SandboxConfig::new(fork, multi_l2);

    info!(
        mode = %config.mode_description(),
        detach = detach,
        build = build,
        fork = fork,
        multi_l2 = multi_l2,
        "Starting AggLayer sandbox environment"
    );

    println!(
        "{}",
        format!(
            "🚀 Starting AggLayer sandbox environment in {}...",
            config.mode_description()
        )
        .green()
        .bold()
    );

    // Validate fork mode configuration
    if let Err(e) = config.validate_fork_config() {
        error!(error = %e, "Fork mode configuration validation failed");
        eprintln!("{}", format!("❌ {e}").red());
        eprintln!("{}", "Please set the fork URLs in your .env file".yellow());
        std::process::exit(1);
    }

    // Display fork URLs if in fork mode
    if fork {
        info!("Displaying fork URL configuration");
        display_fork_urls(multi_l2);
    }

    // Create Docker builder with proper configuration
    info!("Creating Docker configuration");
    let docker_builder = config.create_docker_builder()?;

    // Build and execute Docker command
    info!(detach = detach, build = build, "Building Docker command");
    let cmd = docker_builder.build_up_command(detach, build);

    if detach {
        // Execute in detached mode
        info!("Executing Docker command in detached mode");
        if execute_docker_command(cmd, true).is_err() {
            error!("Failed to start sandbox in detached mode");
            eprintln!("{}", "❌ Failed to start sandbox".red());
            std::process::exit(1);
        }

        // Display success message
        let success_msg = match (fork, multi_l2) {
            (true, true) => "✅ Multi-L2 sandbox started in fork mode (detached)",
            (true, false) => "✅ Sandbox started in fork mode (detached)",
            (false, true) => "✅ Multi-L2 sandbox started (detached)",
            (false, false) => "✅ Sandbox started in detached mode",
        };
        info!(
            fork = fork,
            multi_l2 = multi_l2,
            "Sandbox started successfully in detached mode"
        );
        println!("{}", success_msg.green());

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
        println!("{}", "Starting services in foreground mode...".cyan());
        println!("{}", "Press Ctrl+C to stop the sandbox".yellow());

        if execute_docker_command(cmd, false).is_err() {
            error!("Failed to start sandbox in foreground mode");
            eprintln!("{}", "❌ Failed to start sandbox".red());
            std::process::exit(1);
        } else {
            info!("Sandbox stopped gracefully");
            println!("{}", "✅ Sandbox stopped".green());
        }
    }

    Ok(())
}

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
        println!("  AggLayer 1: {}", fork_agglayer_1.yellow());
        println!("  AggLayer 2: {}", fork_agglayer_2.yellow());
    } else {
        info!(
            mainnet = %fork_mainnet,
            agglayer_1 = %fork_agglayer_1,
            multi_l2 = false,
            "Displaying fork URLs for single L2 configuration"
        );
        println!("{}", "Fork URLs detected:".cyan());
        println!("  Mainnet: {}", fork_mainnet.yellow());
        println!("  AggLayer 1: {}", fork_agglayer_1.yellow());
    }
}
