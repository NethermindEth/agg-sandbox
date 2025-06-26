use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::Path;
use std::process::Command;

mod logs;

#[derive(Parser)]
#[command(name = "agg-sandbox")]
#[command(about = "CLI for managing AggLayer sandbox environment")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the sandbox environment
    Start {
        /// Run in detached mode
        #[arg(short, long)]
        detach: bool,
        /// Build images before starting
        #[arg(short, long)]
        build: bool,
        /// Enable fork mode (uses real blockchain data from FORK_URL environment variables)
        #[arg(short, long)]
        fork: bool,
        /// Enable multi-L2 mode (runs with a second L2 chain)
        #[arg(short, long)]
        multi_l2: bool,
    },
    /// Stop the sandbox environment (docker-compose down)
    Stop {
        /// Remove volumes when stopping
        #[arg(short, long)]
        volumes: bool,
    },
    /// Show status of services
    Status,
    /// Show logs from services
    Logs {
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        /// Service name to show logs for (optional)
        service: Option<String>,
    },
    /// Restart the sandbox environment
    Restart,
    /// Show sandbox configuration and accounts
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure we're in the right directory (where docker-compose.yml exists)
    if !Path::new("docker-compose.yml").exists() {
        eprintln!(
            "{}",
            "Error: docker-compose.yml not found in current directory".red()
        );
        eprintln!(
            "{}",
            "Please run this command from the project root directory".yellow()
        );
        std::process::exit(1);
    }

    // Load environment variables from .env file if it exists
    if Path::new(".env").exists() {
        dotenv::dotenv().ok();
    }

    match cli.command {
        Commands::Start {
            detach,
            build,
            fork,
            multi_l2,
        } => start_sandbox(detach, build, fork, multi_l2),
        Commands::Stop { volumes } => stop_sandbox(volumes),
        Commands::Status => show_status(),
        Commands::Logs { follow, service } => show_logs(follow, service),
        Commands::Restart => restart_sandbox(),
        Commands::Info => show_info(),
    }
}

fn start_sandbox(detach: bool, build: bool, fork: bool, multi_l2: bool) -> Result<()> {
    // Determine mode for messaging
    let mode_str = match (fork, multi_l2) {
        (true, true) => "multi-L2 fork mode",
        (true, false) => "fork mode",
        (false, true) => "multi-L2 mode",
        (false, false) => "local mode",
    };

    println!(
        "{}",
        format!(
            "🚀 Starting AggLayer sandbox environment in {}...",
            mode_str
        )
        .green()
        .bold()
    );

    // Fork mode validation
    if fork {
        let fork_mainnet = std::env::var("FORK_URL_MAINNET").unwrap_or_default();
        let fork_agglayer_1 = std::env::var("FORK_URL_AGGLAYER_1").unwrap_or_default();

        if fork_mainnet.is_empty() {
            eprintln!(
                "{}",
                "❌ FORK_URL_MAINNET environment variable is not set".red()
            );
            eprintln!("{}", "Please set the fork URLs in your .env file".yellow());
            std::process::exit(1);
        }

        if fork_agglayer_1.is_empty() {
            eprintln!(
                "{}",
                "❌ FORK_URL_AGGLAYER_1 environment variable is not set".red()
            );
            eprintln!("{}", "Please set the fork URLs in your .env file".yellow());
            std::process::exit(1);
        }

        // Additional validation for multi-L2 fork mode
        if multi_l2 {
            let fork_agglayer_2 = std::env::var("FORK_URL_AGGLAYER_2").unwrap_or_default();
            if fork_agglayer_2.is_empty() {
                eprintln!(
                    "{}",
                    "❌ FORK_URL_AGGLAYER_2 environment variable is not set".red()
                );
                eprintln!(
                    "{}",
                    "Please set the fork URLs in your .env file for the second L2 chain".yellow()
                );
                std::process::exit(1);
            }
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

    let mut cmd = Command::new("docker-compose");
    
    // Add compose files based on mode
    if multi_l2 {
        cmd.arg("-f").arg("docker-compose.yml");
        cmd.arg("-f").arg("docker-compose.multi-l2.yml");
    }

    cmd.arg("up");

    if detach {
        cmd.arg("-d");
    }

    if build {
        cmd.arg("--build");
    }

    // Set environment variables based on mode
    if fork {
        cmd.env("ENABLE_FORK_MODE", "true");
        cmd.env(
            "FORK_URL_MAINNET",
            std::env::var("FORK_URL_MAINNET").unwrap_or_default(),
        );
        cmd.env(
            "FORK_URL_AGGLAYER_1",
            std::env::var("FORK_URL_AGGLAYER_1").unwrap_or_default(),
        );
        if multi_l2 {
            cmd.env(
                "FORK_URL_AGGLAYER_2",
                std::env::var("FORK_URL_AGGLAYER_2").unwrap_or_default(),
            );
        }
    } else {
        cmd.env("ENABLE_FORK_MODE", "false");
    }

    // Set chain IDs
    cmd.env(
        "CHAIN_ID_MAINNET",
        std::env::var("CHAIN_ID_MAINNET").unwrap_or_else(|_| "1".to_string()),
    );
    cmd.env(
        "CHAIN_ID_AGGLAYER_1",
        std::env::var("CHAIN_ID_AGGLAYER_1").unwrap_or_else(|_| "1101".to_string()),
    );
    if multi_l2 {
        cmd.env(
            "CHAIN_ID_AGGLAYER_2",
            std::env::var("CHAIN_ID_AGGLAYER_2").unwrap_or_else(|_| "1102".to_string()),
        );
    }

    // Handle detached vs non-detached mode differently
    if detach {
        // In detached mode, capture output to ensure consistent behavior across platforms
        let output = cmd
            .output()
            .context("Failed to execute docker-compose up")?;

        // Only show output if there's an error
        if !output.status.success() {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
            eprintln!("{}", "❌ Failed to start sandbox".red());
            std::process::exit(1);
        }

        let success_msg = match (fork, multi_l2) {
            (true, true) => "✅ Multi-L2 sandbox started in fork mode (detached)",
            (true, false) => "✅ Sandbox started in fork mode (detached)",
            (false, true) => "✅ Multi-L2 sandbox started (detached)",
            (false, false) => "✅ Sandbox started in detached mode",
        };
        println!("{}", success_msg.green());

        // Print appropriate info
        match (fork, multi_l2) {
            (_, true) => logs::print_multi_l2_info(fork),
            (true, false) => logs::print_sandbox_fork_info(),
            (false, false) => logs::print_sandbox_info(),
        }
    } else {
        // In non-detached mode, run in foreground with real-time output
        println!("{}", "Starting services in foreground mode...".cyan());
        println!("{}", "Press Ctrl+C to stop the sandbox".yellow());

        let status = cmd
            .status()
            .context("Failed to execute docker-compose up")?;

        if status.success() {
            println!("{}", "✅ Sandbox stopped".green());
        } else {
            eprintln!("{}", "❌ Failed to start sandbox".red());
            std::process::exit(1);
        }
    }

    Ok(())
}

fn stop_sandbox(volumes: bool) -> Result<()> {
    println!(
        "{}",
        "🛑 Stopping AggLayer sandbox environment..."
            .yellow()
            .bold()
    );

    // Try to stop both regular and multi-L2 configurations
    let mut cmd = Command::new("docker-compose");
    cmd.arg("-f").arg("docker-compose.yml");

    // Check if multi-L2 compose file exists and add it
    if Path::new("docker-compose.multi-l2.yml").exists() {
        cmd.arg("-f").arg("docker-compose.multi-l2.yml");
    }

    cmd.arg("down");

    if volumes {
        cmd.arg("-v");
    }

    // Capture output to ensure consistent behavior across platforms
    let output = cmd
        .output()
        .context("Failed to execute docker-compose down")?;

    // Only show output if there's an error (suppress verbose success logs)
    if !output.status.success() {
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }
        eprintln!("{}", "❌ Failed to stop sandbox".red());
        std::process::exit(1);
    } else {
        println!("{}", "✅ Sandbox stopped successfully".green());
    }

    Ok(())
}

fn show_status() -> Result<()> {
    println!("{}", "📊 Sandbox service status:".blue().bold());

    let mut cmd = Command::new("docker-compose");
    cmd.arg("-f").arg("docker-compose.yml");

    // Check if multi-L2 compose file exists and add it
    if Path::new("docker-compose.multi-l2.yml").exists() {
        cmd.arg("-f").arg("docker-compose.multi-l2.yml");
    }

    cmd.arg("ps");

    let output = cmd
        .output()
        .context("Failed to execute docker-compose ps")?;

    // Print stdout and stderr to ensure Docker output is shown
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        eprintln!("{}", "❌ Failed to get service status".red());
        std::process::exit(1);
    }

    Ok(())
}

fn show_logs(follow: bool, service: Option<String>) -> Result<()> {
    let service_name = service.as_deref().unwrap_or("all services");
    println!(
        "{} {}",
        "📋 Showing logs for:".blue().bold(),
        service_name.cyan()
    );

    let mut cmd = Command::new("docker-compose");
    cmd.arg("-f").arg("docker-compose.yml");

    // Check if multi-L2 compose file exists and add it
    if Path::new("docker-compose.multi-l2.yml").exists() {
        cmd.arg("-f").arg("docker-compose.multi-l2.yml");
    }

    cmd.arg("logs");

    if follow {
        cmd.arg("-f");
    }

    if let Some(svc) = service {
        cmd.arg(svc);
    }

    // For logs command, especially with --follow, we need to inherit stdio for real-time output
    if follow {
        let status = cmd
            .status()
            .context("Failed to execute docker-compose logs")?;

        if !status.success() {
            eprintln!("{}", "❌ Failed to show logs".red());
            std::process::exit(1);
        }
    } else {
        // For non-follow logs, capture output for consistent behavior
        let output = cmd
            .output()
            .context("Failed to execute docker-compose logs")?;

        // Print stdout and stderr to ensure Docker output is shown
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            eprintln!("{}", "❌ Failed to show logs".red());
            std::process::exit(1);
        }
    }

    Ok(())
}

fn restart_sandbox() -> Result<()> {
    println!(
        "{}",
        "🔄 Restarting AggLayer sandbox environment..."
            .yellow()
            .bold()
    );

    // First stop
    stop_sandbox(false)?;

    // Then start in basic local mode
    start_sandbox(true, false, false, false)?;

    println!("{}", "✅ Sandbox restarted successfully".green());

    Ok(())
}

fn show_info() -> Result<()> {
    println!("{}", "📋 AggLayer Sandbox Information".blue().bold());
    logs::print_sandbox_info();
    Ok(())
}
