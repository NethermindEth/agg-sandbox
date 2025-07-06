use clap::{Parser, Subcommand};
use colored::*;
use std::path::Path;

mod api;
mod commands;
mod config;
mod docker;
mod error;
mod events;
mod logs;
mod validation;

use commands::ShowCommands;
use error::Result;

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

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        print_error(&e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
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
        } => commands::handle_start(detach, build, fork, multi_l2),
        Commands::Stop { volumes } => commands::handle_stop(volumes),
        Commands::Status => commands::handle_status(),
        Commands::Logs { follow, service } => commands::handle_logs(follow, service),
        Commands::Restart => commands::handle_restart(),
        Commands::Info => commands::handle_info().await,
        Commands::Show { subcommand } => commands::handle_show(subcommand).await,
        Commands::Events {
            chain,
            blocks,
            address,
        } => commands::handle_events(chain, blocks, address).await,
    }
}

/// Print user-friendly error messages
fn print_error(error: &error::AggSandboxError) {
    eprintln!("{} {error}", "❌ Error:".red().bold());

    // Provide additional context and suggestions based on error type
    match error {
        error::AggSandboxError::Config(config_err) => match config_err {
            error::ConfigError::EnvVarNotFound(var) => {
                eprintln!("{}", "💡 Suggestion:".yellow().bold());
                eprintln!("   Set the environment variable in your .env file:");
                eprintln!("   echo '{}=your_value' >> .env", var.cyan());
            }
            error::ConfigError::InvalidValue { key, .. } => {
                eprintln!("{}", "💡 Suggestion:".yellow().bold());
                eprintln!(
                    "   Check the value for '{}' in your configuration",
                    key.cyan()
                );
            }
            _ => {}
        },
        error::AggSandboxError::Docker(docker_err) => match docker_err {
            error::DockerError::ComposeFileNotFound(_) => {
                eprintln!("{}", "💡 Suggestion:".yellow().bold());
                eprintln!("   Make sure you're in the project root directory");
                eprintln!("   Run: cd /path/to/agg-sandbox");
            }
            error::DockerError::CommandFailed { .. } => {
                eprintln!("{}", "💡 Suggestion:".yellow().bold());
                eprintln!("   • Check if Docker is running: docker --version");
                eprintln!("   • Verify Docker Compose is installed: docker-compose --version");
                eprintln!("   • Try stopping existing containers: aggsandbox stop");
            }
            _ => {}
        },
        error::AggSandboxError::Api(api_err) => match api_err {
            error::ApiError::NetworkError(_) => {
                eprintln!("{}", "💡 Suggestion:".yellow().bold());
                eprintln!("   • Check if the sandbox is running: aggsandbox status");
                eprintln!("   • Verify network connectivity");
                eprintln!("   • Try starting the sandbox: aggsandbox start --detach");
            }
            error::ApiError::EndpointUnavailable(_) => {
                eprintln!("{}", "💡 Suggestion:".yellow().bold());
                eprintln!("   The API service may not be ready yet. Wait a moment and try again.");
            }
            _ => {}
        },
        error::AggSandboxError::Events(event_err) => match event_err {
            error::EventError::InvalidChain(_chain) => {
                eprintln!("{}", "💡 Suggestion:".yellow().bold());
                eprintln!("   Valid chains: {}", "anvil-l1, anvil-l2, anvil-l3".cyan());
                eprintln!("   Try: aggsandbox events --chain anvil-l1 --blocks 5");
            }
            error::EventError::RpcConnectionFailed(_) => {
                eprintln!("{}", "💡 Suggestion:".yellow().bold());
                eprintln!("   • Make sure the sandbox is running: aggsandbox status");
                eprintln!("   • Check if the specified chain is available");
            }
            _ => {}
        },
        _ => {}
    }
}
