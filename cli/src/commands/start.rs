use crate::config::Config;
use crate::error::Result;
use crate::logs;
use colored::*;

/// Handle the start command
pub fn handle_start(detach: bool, build: bool, fork: bool, multi_l2: bool) -> Result<()> {
    use crate::docker::{execute_docker_command, SandboxConfig};

    // Create sandbox configuration
    let config = SandboxConfig::new(fork, multi_l2);

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
        eprintln!("{}", format!("❌ {e}").red());
        eprintln!("{}", "Please set the fork URLs in your .env file".yellow());
        std::process::exit(1);
    }

    // Display fork URLs if in fork mode
    if fork {
        display_fork_urls(multi_l2);
    }

    // Create Docker builder with proper configuration
    let docker_builder = config.create_docker_builder()?;

    // Build and execute Docker command
    let cmd = docker_builder.build_up_command(detach, build);

    if detach {
        // Execute in detached mode
        if execute_docker_command(cmd, true).is_err() {
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
        println!("{}", success_msg.green());

        // Load config and print appropriate info
        if let Ok(config) = Config::load() {
            match (fork, multi_l2) {
                (_, true) => logs::print_multi_l2_info(&config, fork),
                (true, false) => logs::print_sandbox_fork_info(&config),
                (false, false) => logs::print_sandbox_info(&config),
            }
        }
    } else {
        // Run in foreground mode
        println!("{}", "Starting services in foreground mode...".cyan());
        println!("{}", "Press Ctrl+C to stop the sandbox".yellow());

        if execute_docker_command(cmd, false).is_err() {
            eprintln!("{}", "❌ Failed to start sandbox".red());
            std::process::exit(1);
        } else {
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
        println!("{}", "Fork URLs detected:".cyan());
        println!("  Mainnet: {}", fork_mainnet.yellow());
        println!("  AggLayer 1: {}", fork_agglayer_1.yellow());
        println!("  AggLayer 2: {}", fork_agglayer_2.yellow());
    } else {
        println!("{}", "Fork URLs detected:".cyan());
        println!("  Mainnet: {}", fork_mainnet.yellow());
        println!("  AggLayer 1: {}", fork_agglayer_1.yellow());
    }
}
