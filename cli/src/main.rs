use clap::{Parser, Subcommand};
use colored::*;
use std::path::Path;

mod api;
mod api_client;
mod commands;
mod config;
mod docker;
mod error;
mod events;
mod logging;
mod logs;
mod progress;
mod types;
mod utils;
mod validation;

use commands::{BridgeCommands, ShowCommands};
use error::Result;
use logging::LogConfig;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "aggsandbox")]
#[command(about = "🚀 CLI for managing Agglayer sandbox environment")]
#[command(
    long_about = "AggSandbox CLI provides comprehensive tools for managing your Agglayer sandbox environment.\n\nThis tool helps you start, stop, monitor, and interact with your sandbox infrastructure\nincluding L1/L2 chains, bridge services, and blockchain events.\n\nExamples:\n  aggsandbox start --detach             # Start sandbox in background\n  aggsandbox start --fork --multi-l2    # Start with real data and multiple L2s\n  aggsandbox logs -f aggkit             # Follow aggkit logs\n  aggsandbox show bridges --network-id 0 # Show L1 bridge information\n  aggsandbox events --chain anvil-l1    # Show recent blockchain events"
)]
#[command(version = "0.1.0")]
#[command(author = "Agglayer Team")]
#[command(
    help_template = "{before-help}{name} {version}\n{about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Enable verbose output (-v for debug, -vv for trace)
    #[arg(short, long, global = true, action = clap::ArgAction::Count, help = "Enable verbose output (-v debug, -vv trace)")]
    verbose: u8,
    /// Enable quiet mode (only errors and warnings)
    #[arg(
        short,
        long,
        global = true,
        help = "Suppress all output except errors and warnings"
    )]
    quiet: bool,
    /// Set log format style
    #[arg(long, global = true, default_value = "pretty", value_parser = ["pretty", "compact", "json"], help = "Set log output format")]
    log_format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// 🚀 Start the sandbox environment
    #[command(
        long_about = "Start the Agglayer sandbox environment with Docker Compose.\n\nThis command initializes and starts all required services including:\n- L1 Ethereum node (Anvil)\n- L2 Polygon zkEVM node (Anvil)\n- Bridge service\n- Agglayer service\n\nExamples:\n  aggsandbox start                     # Start with default settings\n  aggsandbox start --detach            # Start in background\n  aggsandbox start --build             # Rebuild images before starting\n  aggsandbox start --fork              # Use real blockchain data\n  aggsandbox start --fork --multi-l2   # Fork mode with multiple L2 chains"
    )]
    Start {
        /// Run services in detached mode (background)
        #[arg(short, long, help = "Start services in background (detached mode)")]
        detach: bool,
        /// Build Docker images before starting services
        #[arg(short, long, help = "Rebuild Docker images before starting")]
        build: bool,
        /// Enable fork mode using real blockchain data
        #[arg(
            short,
            long,
            help = "Use real blockchain data from FORK_URL environment variables"
        )]
        fork: bool,
        /// Enable multi-L2 configuration with additional chains
        #[arg(
            short,
            long,
            help = "Start with a second L2 chain for multi-chain testing"
        )]
        multi_l2: bool,
        // Enable automatic claim sponsoring
        #[arg(
            short,
            long,
            help = "Claimsponsor will sponsor all claims automatically"
        )]
        claim_all: bool,
    },
    /// 🛑 Stop the sandbox environment
    #[command(
        long_about = "Stop all sandbox services using docker-compose down.\n\nThis command gracefully shuts down all running services and containers.\nOptionally, you can also remove associated Docker volumes.\n\nExamples:\n  aggsandbox stop          # Stop services, keep data\n  aggsandbox stop -v       # Stop services and remove volumes"
    )]
    Stop {
        /// Remove Docker volumes when stopping (⚠️  deletes all data)
        #[arg(
            short,
            long,
            help = "Remove Docker volumes and all persistent data (⚠️  destructive)"
        )]
        volumes: bool,
    },
    /// 📊 Show status of all services
    #[command(
        long_about = "Display the current status of all sandbox services.\n\nShows which containers are running, stopped, or have errors.\nIncludes health checks and port information for active services.\n\nExample:\n  aggsandbox status"
    )]
    Status,
    /// 📋 Show logs from services
    #[command(
        long_about = "Display logs from sandbox services.\n\nView logs from all services or filter by specific service name.\nUse --follow to stream logs in real-time.\n\nExamples:\n  aggsandbox logs                    # Show all logs\n  aggsandbox logs aggkit             # Show aggkit logs (bridge, oracle, etc.)\n  aggsandbox logs -f                 # Follow all logs\n  aggsandbox logs -f anvil-l1        # Follow L1 node logs\n  aggsandbox logs -f aggkit          # Follow aggkit logs in real-time"
    )]
    Logs {
        /// Follow log output in real-time
        #[arg(short, long, help = "Stream logs continuously (like 'tail -f')")]
        follow: bool,
        /// Specific service name to show logs for
        #[arg(help = "Service name (e.g., aggkit, anvil-l1, anvil-l2, contract-deployer)")]
        service: Option<String>,
    },
    /// 🔄 Restart the sandbox environment
    #[command(
        long_about = "Restart all sandbox services.\n\nThis performs a stop followed by start operation,\npreserving volumes and configuration.\n\nExample:\n  aggsandbox restart"
    )]
    Restart,
    /// ℹ️  Show sandbox configuration and accounts
    #[command(
        long_about = "Display comprehensive sandbox configuration information.\n\nShows:\n- Network configuration (L1/L2 RPC URLs, Chain IDs)\n- Account addresses and balances\n- Contract deployment addresses\n- Bridge service endpoints\n\nExample:\n  aggsandbox info"
    )]
    Info,
    /// 🌉 Show bridge and blockchain information
    #[command(
        long_about = "Access bridge data and blockchain information.\n\nQuery bridges, claims, proofs, and other bridge-related data\nfrom the Agglayer bridge service API.\n\nExamples:\n  aggsandbox show bridges --network-id 0     # List bridges for L1\n  aggsandbox show claims --network-id 1      # Show claims for first L2\n  aggsandbox show proof --network-id 0 --leaf-index 0 --deposit-count 1"
    )]
    Show {
        #[command(subcommand)]
        subcommand: ShowCommands,
    },
    /// 🌉 Execute bridge operations (asset transfers, claims, messages)
    #[command(
        long_about = "Execute bridge operations using direct smart contract interactions.\n\nPerform cross-chain asset transfers, claim bridged assets, and send messages\nbetween L1 and L2 networks with user-friendly commands.\n\nExamples:\n  aggsandbox bridge asset --network 0 --destination-network 1 --amount 0.1 --token-address 0x0000...\n  aggsandbox bridge claim --network 1 --tx-hash 0xabc... --source-network 0\n  aggsandbox bridge message --network 0 --destination-network 1 --target 0x123... --data 0xabc..."
    )]
    Bridge {
        #[command(subcommand)]
        subcommand: BridgeCommands,
    },
    /// 📡 Fetch and display blockchain events
    #[command(
        long_about = "Monitor blockchain events from L1 and L2 chains.\n\nFetch and display recent events from specified blockchain,\nwith options to filter by contract address and block range.\n\nExamples:\n  aggsandbox events --network-id 0                # Recent L1 events\n  aggsandbox events --network-id 1 --blocks 20    # Last 20 blocks from first L2\n  aggsandbox events --network-id 0 --address 0x123 # Events from specific contract\n\nLegacy (deprecated) examples:\n  aggsandbox events --chain anvil-l1              # Use --network-id 0 instead"
    )]
    Events {
        /// Network ID to fetch events from (preferred over --chain)
        #[arg(short = 'n', long, help = "Network ID to query (0=L1, 1=L2, 2=L3)")]
        network_id: Option<u64>,
        /// Blockchain to fetch events from (deprecated, use --network-id instead)
        #[arg(short, long, value_parser = ["anvil-l1", "anvil-l2", "anvil-l3"], help = "Chain to query (anvil-l1, anvil-l2, or anvil-l3) - DEPRECATED: use --network-id")]
        chain: Option<String>,
        /// Number of recent blocks to scan for events
        #[arg(
            short,
            long,
            default_value = "10",
            help = "Number of recent blocks to scan (default: 10)"
        )]
        blocks: u64,
        /// Filter events by contract address
        #[arg(short = 'a', long, help = "Contract address to filter events (0x...)")]
        address: Option<String>,
    },
    /// 💸  Sponsor a bridge claim
    #[command(long_about = "Submit a bridge claim to the Claim-Sponsor bot.\n\
        \n\
        This command performs all steps automatically:\n\
        1. Computes the global index.\n\
        2. Calls the AggKit REST API to fetch Merkle proofs.\n\
        3. Assembles the JSON body required by `/bridge/v1/sponsor-claim`.\n\
        4. Posts the claim.\n")]
    SponsorClaim {
        /// Deposit counter on the *origin* chain (starts at 0)
        #[arg(short = 'd', long)]
        deposit: u32,

        /// Network ID the deposit originated on (omit or 0 for L1)
        #[arg(long, default_value_t = 0)]
        origin_network: u64,

        /// ID of the destination network (omit or 1 for L2)
        #[arg(long, default_value_t = 1)]
        destination_network: u64,
    },
    /// 🔎 Query the status of a sponsored claim by global index
    ClaimStatus {
        /// Global index of the claim you want to check
        #[arg(short = 'g', long = "global-index")]
        global_index: u64,
        /// Network ID to check claims from
        #[arg(long = "network-id")]
        network_id: u64,
    },
}

#[tokio::main]
#[allow(clippy::disallowed_methods)] // Allow std::process::exit in main and tracing macros
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

#[allow(clippy::disallowed_methods)] // Allow tracing macros
async fn run(cli: Cli) -> Result<()> {
    info!("Starting AggSandbox CLI v0.1.0");

    // Ensure we're in the right directory (check for appropriate compose file based on command)
    let needs_multi_l2 = match &cli.command {
        Commands::Start { multi_l2, .. } => *multi_l2,
        _ => false,
    };

    let compose_file = if needs_multi_l2 {
        "docker-compose.multi-l2.yml"
    } else {
        "docker-compose.yml"
    };

    if !Path::new(compose_file).exists() {
        error!("{} not found in current directory", compose_file);
        warn!("Please run this command from the project root directory");
        return Err(error::AggSandboxError::Config(
            error::ConfigError::missing_required(&format!(
                "{compose_file} file in working directory"
            )),
        ));
    }

    info!("Found {} in current directory", compose_file);

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
            claim_all,
        } => {
            info!(
                detach = detach,
                build = build,
                fork = fork,
                multi_l2 = multi_l2,
                claim_all = claim_all,
                "Executing start command"
            );
            commands::handle_start(detach, build, fork, multi_l2, claim_all).await;
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
            commands::handle_restart().await;
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
        Commands::Bridge { subcommand } => {
            info!(subcommand = ?subcommand, "Executing bridge command");
            commands::handle_bridge(subcommand).await
        }
        Commands::Events {
            network_id,
            chain,
            blocks,
            address,
        } => {
            info!(network_id = ?network_id, chain = ?chain, blocks = blocks, address = ?address, "Executing events command");
            commands::handle_events(network_id, chain, blocks, address).await
        }
        Commands::SponsorClaim {
            deposit,
            origin_network,
            destination_network,
        } => {
            info!(deposit, origin_network, "Executing sponsor-claim command");
            commands::handle_sponsor_claim(deposit, origin_network, destination_network).await?;
            Ok(())
        }
        Commands::ClaimStatus {
            global_index,
            network_id,
        } => {
            info!(global_index, "Executing claim-status command");
            commands::handle_claim_status(global_index, network_id).await?;
            Ok(())
        }
    };

    match &result {
        Ok(_) => info!("Command completed successfully"),
        Err(e) => error!(error = %e, "Command failed"),
    }

    result
}

/// Initialize logging based on CLI configuration
#[allow(clippy::disallowed_methods)] // Allow for error propagation and print functions
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

/// Print user-friendly error messages with enhanced suggestions
fn print_error(error: &error::AggSandboxError) {
    eprintln!("\n{} {error}", "❌ Error:".red().bold());

    // Provide additional context and suggestions based on error type
    match error {
        error::AggSandboxError::Config(config_err) => {
            eprintln!("\n{}", "🔧 Configuration Issue".yellow().bold());
            match config_err {
                error::ConfigError::EnvVarNotFound(var) => {
                    eprintln!("{}", "💡 Quick Fix:".blue().bold());
                    eprintln!("   1. Create or edit your .env file:");
                    eprintln!(
                        "      {}",
                        format!("echo '{var}=your_value' >> .env").cyan()
                    );
                    eprintln!("   2. Or set it temporarily:");
                    eprintln!("      {}", format!("export {var}=your_value").cyan());
                    eprintln!("\n{}", "📚 Learn more:".dimmed());
                    eprintln!("   Check the README for required environment variables");
                }
                error::ConfigError::InvalidValue { key, .. } => {
                    eprintln!("{}", "💡 Quick Fix:".blue().bold());
                    eprintln!("   1. Check the value for '{}'", key.cyan());
                    eprintln!("   2. Refer to configuration documentation");
                    eprintln!("   3. Use 'aggsandbox info' to see current config");
                }
                error::ConfigError::MissingRequired(item) => {
                    eprintln!("{}", "💡 Quick Fix:".blue().bold());
                    eprintln!("   The following is required: {}", item.cyan());
                    eprintln!("   Make sure you're in the correct directory and all files exist");
                }
                _ => {
                    eprintln!("{}", "💡 Suggestion:".blue().bold());
                    eprintln!("   Check your configuration files and environment variables");
                }
            }
        }
        error::AggSandboxError::Docker(docker_err) => {
            eprintln!("\n{}", "🐳 Docker Issue".yellow().bold());
            match docker_err {
                error::DockerError::ComposeFileNotFound(_) => {
                    eprintln!("{}", "💡 Quick Fix:".blue().bold());
                    eprintln!("   1. Navigate to the project root directory:");
                    eprintln!("      {}", "cd /path/to/agg-sandbox".cyan());
                    eprintln!("   2. Verify docker-compose.yml exists:");
                    eprintln!("      {}", "ls docker-compose.yml".cyan());
                    eprintln!("\n{}", "🎯 Current directory should contain:".dimmed());
                    eprintln!("   • docker-compose.yml");
                    eprintln!("   • .env (optional)");
                    eprintln!("   • contracts/ directory");
                }
                error::DockerError::CommandFailed { .. } => {
                    eprintln!("{}", "💡 Troubleshooting Steps:".blue().bold());
                    eprintln!("   1. Check Docker is running:");
                    eprintln!("      {}", "docker --version".cyan());
                    eprintln!("   2. Verify Docker Compose:");
                    eprintln!("      {}", "docker compose version".cyan());
                    eprintln!("   3. Stop any existing containers:");
                    eprintln!("      {}", "aggsandbox stop".cyan());
                    eprintln!("   4. Check for port conflicts:");
                    eprintln!("      {}", "aggsandbox status".cyan());
                    eprintln!("\n{}", "🔍 If ports 8545, 8546 are in use:".dimmed());
                    eprintln!(
                        "   Stop other blockchain nodes or change ports in docker-compose.yml"
                    );
                }
                _ => {
                    eprintln!("{}", "💡 General Docker Help:".blue().bold());
                    eprintln!("   • Ensure Docker Desktop is running");
                    eprintln!("   • Check Docker has sufficient resources");
                    eprintln!("   • Try: docker system prune (removes unused data)");
                }
            }
        }
        error::AggSandboxError::Api(api_err) => {
            eprintln!("\n{}", "🌐 API Connection Issue".yellow().bold());
            match api_err {
                error::ApiError::NetworkError(_) => {
                    eprintln!("{}", "💡 Troubleshooting Steps:".blue().bold());
                    eprintln!("   1. Check sandbox status:");
                    eprintln!("      {}", "aggsandbox status".cyan());
                    eprintln!("   2. Start if not running:");
                    eprintln!("      {}", "aggsandbox start --detach".cyan());
                    eprintln!("   3. Wait for services to be ready (30-60s)");
                    eprintln!("   4. Check logs for errors:");
                    eprintln!("      {}", "aggsandbox logs".cyan());
                    eprintln!("\n{}", "⏱️  Services need time to start up".dimmed());
                }
                error::ApiError::EndpointUnavailable(_) => {
                    eprintln!("{}", "💡 Wait and Retry:".blue().bold());
                    eprintln!("   • API service is starting up");
                    eprintln!("   • Wait 30-60 seconds and try again");
                    eprintln!("   • Check service health:");
                    eprintln!("     {}", "aggsandbox logs aggkit".cyan());
                }
                error::ApiError::RequestFailed { status, .. } => {
                    eprintln!("{}", "💡 HTTP Error Help:".blue().bold());
                    match *status {
                        404 => eprintln!("   • Endpoint not found - check API version"),
                        500 => eprintln!("   • Server error - check service logs"),
                        503 => eprintln!("   • Service unavailable - wait and retry"),
                        _ => eprintln!("   • Check service logs for details"),
                    }
                    eprintln!("     {}", "aggsandbox logs".cyan());
                }
                _ => {
                    eprintln!("{}", "💡 General API Help:".blue().bold());
                    eprintln!("   • Verify all services are running");
                    eprintln!("   • Check network connectivity");
                    eprintln!("   • Review service logs for errors");
                }
            }
        }
        error::AggSandboxError::Events(event_err) => {
            eprintln!("\n{}", "📡 Blockchain Events Issue".yellow().bold());
            match event_err {
                error::EventError::InvalidChain(_chain) => {
                    eprintln!("{}", "💡 Valid Chains:".blue().bold());
                    eprintln!("   • {} - Ethereum L1 chain", "anvil-l1".green());
                    eprintln!("   • {} - Polygon zkEVM L2 chain", "anvil-l2".green());
                    eprintln!(
                        "   • {} - Additional L2 chain (if enabled)",
                        "anvil-l3".green()
                    );
                    eprintln!("\n{}", "📝 Example usage:".dimmed());
                    eprintln!(
                        "   {}",
                        "aggsandbox events --chain anvil-l1 --blocks 5".cyan()
                    );
                    eprintln!(
                        "   {}",
                        "aggsandbox events --chain anvil-l2 --blocks 10".cyan()
                    );
                }
                error::EventError::RpcConnectionFailed(_) => {
                    eprintln!("{}", "💡 RPC Connection Fix:".blue().bold());
                    eprintln!("   1. Ensure sandbox is running:");
                    eprintln!("      {}", "aggsandbox status".cyan());
                    eprintln!("   2. Check if chain is available:");
                    eprintln!("      {}", "aggsandbox info".cyan());
                    eprintln!("   3. Verify RPC endpoints are responding");
                    eprintln!("\n{}", "🔍 Multi-L2 mode:".dimmed());
                    eprintln!("   anvil-l3 is only available with --multi-l2 flag");
                }
                _ => {
                    eprintln!("{}", "💡 Events Troubleshooting:".blue().bold());
                    eprintln!("   • Check if the specified chain is running");
                    eprintln!("   • Verify block range is valid");
                    eprintln!("   • Ensure contract address format is correct");
                }
            }
        }
        _ => {
            eprintln!("\n{}", "🆘 General Help".yellow().bold());
            eprintln!("{}", "💡 Common Solutions:".blue().bold());
            eprintln!("   • Check if you're in the project root directory");
            eprintln!("   • Ensure Docker Desktop is running");
            eprintln!("   • Try restarting the sandbox:");
            eprintln!("     {}", "aggsandbox restart".cyan());
            eprintln!("   • Check the documentation or README");
        }
    }

    eprintln!("\n{}", "🔗 Need more help?".bright_blue().bold());
    eprintln!(
        "   • Run {} for detailed information",
        "aggsandbox --help".cyan()
    );
    eprintln!(
        "   • Use {} for command-specific help",
        "aggsandbox <command> --help".cyan()
    );
    eprintln!("   • Check logs with {}", "aggsandbox logs".cyan());
    eprintln!();
}
