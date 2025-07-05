use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::Path;

mod api;
mod config;
mod docker;
mod events;
mod logs;

use config::Config;

#[derive(Parser)]
#[command(name = "aggsandbox")]
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
    /// Show bridge information
    Show {
        #[command(subcommand)]
        subcommand: ShowCommands,
    },
    /// Fetch and display events from blockchain
    Events {
        /// Chain to fetch events from (anvil-l1 or anvil-l2)
        #[arg(short, long)]
        chain: String,
        /// Number of latest blocks to scan (default: 10)
        #[arg(short, long, default_value = "10")]
        blocks: u64,
        /// Contract address to filter events (optional)
        #[arg(short = 'a', long)]
        address: Option<String>,
    },
}

#[derive(Subcommand)]
enum ShowCommands {
    /// Show bridges for a network
    Bridges {
        /// Network ID to query bridges for
        #[arg(short, long, default_value = "1")]
        network_id: u64,
    },
    /// Show claims for a network
    Claims {
        /// Network ID to query claims for
        #[arg(short, long, default_value = "1101")]
        network_id: u64,
    },
    /// Show claim proof
    ClaimProof {
        /// Network ID
        #[arg(short, long, default_value = "1")]
        network_id: u64,
        /// Leaf index
        #[arg(short, long, default_value = "0")]
        leaf_index: u64,
        /// Deposit count
        #[arg(short, long, default_value = "1")]
        deposit_count: u64,
    },
    /// Show L1 info tree index
    L1InfoTreeIndex {
        /// Network ID
        #[arg(short, long, default_value = "1")]
        network_id: u64,
        /// Deposit count
        #[arg(short, long, default_value = "0")]
        deposit_count: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
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
        Commands::Info => show_info().await,
        Commands::Show { subcommand } => show_bridge_info(subcommand).await,
        Commands::Events {
            chain,
            blocks,
            address,
        } => events::fetch_and_display_events(&chain, blocks, address).await,
    }
}

fn start_sandbox(detach: bool, build: bool, fork: bool, multi_l2: bool) -> Result<()> {
    use docker::{execute_docker_command, SandboxConfig};

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
    let docker_builder = config
        .create_docker_builder()
        .context("Failed to create Docker configuration")?;

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

fn stop_sandbox(volumes: bool) -> Result<()> {
    use docker::{create_auto_docker_builder, execute_docker_command};

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

    Ok(())
}

fn show_status() -> Result<()> {
    use docker::{create_auto_docker_builder, execute_docker_command_with_output};

    println!("{}", "📊 Sandbox service status:".blue().bold());

    // Create Docker builder that auto-detects configuration
    let docker_builder = create_auto_docker_builder();
    let cmd = docker_builder.build_ps_command();

    // Execute the status command and display output
    match execute_docker_command_with_output(cmd) {
        Ok(output) => {
            print!("{output}");
        }
        Err(_) => {
            eprintln!("{}", "❌ Failed to get service status".red());
            std::process::exit(1);
        }
    }

    Ok(())
}

fn show_logs(follow: bool, service: Option<String>) -> Result<()> {
    use docker::{
        create_auto_docker_builder, execute_docker_command, execute_docker_command_with_output,
    };

    let service_name = service.as_deref().unwrap_or("all services");
    println!(
        "{} {}",
        "📋 Showing logs for:".blue().bold(),
        service_name.cyan()
    );

    // Create Docker builder that auto-detects configuration
    let mut docker_builder = create_auto_docker_builder();

    // Add service if specified
    if let Some(svc) = service {
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

async fn show_info() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "📋 AggLayer Sandbox Information".blue().bold());
    logs::print_sandbox_info(&config);

    Ok(())
}

async fn show_bridge_info(subcommand: ShowCommands) -> Result<()> {
    let config = Config::load()?;

    match subcommand {
        ShowCommands::Bridges { network_id } => {
            let response = api::get_bridges(&config, network_id).await?;
            api::print_json_response("Bridge Information", &response.data);
        }
        ShowCommands::Claims { network_id } => {
            let response = api::get_claims(&config, network_id).await?;
            api::print_json_response("Claims Information", &response.data);
        }
        ShowCommands::ClaimProof {
            network_id,
            leaf_index,
            deposit_count,
        } => {
            let response =
                api::get_claim_proof(&config, network_id, leaf_index, deposit_count).await?;
            api::print_json_response("Claim Proof Information", &response.data);
        }
        ShowCommands::L1InfoTreeIndex {
            network_id,
            deposit_count,
        } => {
            let response = api::get_l1_info_tree_index(&config, network_id, deposit_count).await?;
            api::print_json_response("L1 Info Tree Index", &response.data);
        }
    }
    Ok(())
}
