use clap::{Parser, Subcommand};
use colored::*;
use std::path::Path;

mod api;
mod commands;
mod config;
mod docker;
mod error;
mod events;
mod logging;
mod logs;
mod validation;

use commands::ShowCommands;
use error::Result;
use logging::LogConfig;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "aggsandbox")]
#[command(about = "CLI for managing AggLayer sandbox environment")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Enable verbose output
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Enable quiet mode (only errors)
    #[arg(short, long, global = true)]
    quiet: bool,
    /// Log format (pretty, compact, json)
    #[arg(long, global = true, default_value = "pretty")]
    log_format: String,
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
    let cli = Cli::parse();

    // Initialize logging based on CLI flags
    if let Err(e) = initialize_logging(&cli) {
        eprintln!("Failed to initialize logging: {e}");
        std::process::exit(1);
    }

    if let Err(e) = run(cli).await {
        print_error(&e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    info!("Starting AggSandbox CLI v0.1.0");

    // Ensure we're in the right directory (where docker-compose.yml exists)
    if !Path::new("docker-compose.yml").exists() {
        error!("docker-compose.yml not found in current directory");
        warn!("Please run this command from the project root directory");
        return Err(error::AggSandboxError::Config(
            error::ConfigError::missing_required("docker-compose.yml file in working directory"),
        ));
    }

    info!("Found docker-compose.yml in current directory");

    // Load environment variables from .env file if it exists
    if Path::new(".env").exists() {
        info!("Loading environment variables from .env file");
        dotenv::dotenv().ok();
    } else {
        info!("No .env file found, using system environment variables");
    }

    let result = match cli.command {
        Commands::Start {
            detach,
            build,
            fork,
            multi_l2,
        } => {
            info!(
                detach = detach,
                build = build,
                fork = fork,
                multi_l2 = multi_l2,
                "Executing start command"
            );
            commands::handle_start(detach, build, fork, multi_l2);
            Ok(())
        }
        Commands::Stop { volumes } => {
            info!(remove_volumes = volumes, "Executing stop command");
            commands::handle_stop(volumes);
            Ok(())
        }
        Commands::Status => {
            info!("Executing status command");
            commands::handle_status();
            Ok(())
        }
        Commands::Logs { follow, service } => {
            info!(follow = follow, service = ?service, "Executing logs command");
            commands::handle_logs(follow, service)
        }
        Commands::Restart => {
            info!("Executing restart command");
            commands::handle_restart();
            Ok(())
        }
        Commands::Info => {
            info!("Executing info command");
            commands::handle_info().await
        }
        Commands::Show { subcommand } => {
            info!(subcommand = ?subcommand, "Executing show command");
            commands::handle_show(subcommand).await
        }
        Commands::Events {
            chain,
            blocks,
            address,
        } => {
            info!(chain = %chain, blocks = blocks, address = ?address, "Executing events command");
            commands::handle_events(chain, blocks, address).await
        }
    };

    match &result {
        Ok(_) => info!("Command completed successfully"),
        Err(e) => error!(error = %e, "Command failed"),
    }

    result
}

/// Initialize logging based on CLI configuration
fn initialize_logging(cli: &Cli) -> Result<()> {
    let level = logging::level_from_verbosity(cli.verbose, cli.quiet);
    let format = logging::format_from_str(&cli.log_format).map_err(|e| {
        error::AggSandboxError::Config(error::ConfigError::invalid_value(
            "log_format",
            &cli.log_format,
            &e,
        ))
    })?;

    let config = LogConfig {
        level,
        format,
        include_location: cli.verbose > 0,
        include_target: cli.verbose > 1,
        include_spans: cli.verbose > 1,
    };

    logging::init_logging(&config).map_err(|e| {
        error::AggSandboxError::Config(error::ConfigError::validation_failed(&format!(
            "logging initialization: {e}"
        )))
    })?;

    Ok(())
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
