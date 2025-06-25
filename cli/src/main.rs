use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::Path;
use std::process::Command;

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

    // Capture output to ensure consistent behavior across platforms
    let output = cmd
        .output()
        .context("Failed to execute docker-compose up")?;

    // In detached mode, only show output if there's an error
    // In non-detached mode, always show output
    if !detach || !output.status.success() {
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    if output.status.success() {
        if detach {
            let success_msg = match (fork, multi_l2) {
                (true, true) => "✅ Multi-L2 sandbox started in fork mode (detached)",
                (true, false) => "✅ Sandbox started in fork mode (detached)",
                (false, true) => "✅ Multi-L2 sandbox started (detached)",
                (false, false) => "✅ Sandbox started in detached mode",
            };
            println!("{}", success_msg.green());

            // Print appropriate info
            match (fork, multi_l2) {
                (_, true) => print_multi_l2_info(fork),
                (true, false) => print_sandbox_fork_info(),
                (false, false) => print_sandbox_info(),
            }
        } else {
            println!("{}", "✅ Sandbox stopped".green());
        }
    } else {
        eprintln!("{}", "❌ Failed to start sandbox".red());
        std::process::exit(1);
    }

    Ok(())
}

fn print_sandbox_info() {
    println!();
    println!("{}", "Available Accounts".cyan().bold());
    println!("{}", "-----------------------".cyan());
    let accounts = [
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
        "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
        "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
        "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65",
        "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc",
        "0x976EA74026E726554dB657fA54763abd0C3a0aa9",
        "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
        "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f",
        "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
    ];

    for (i, account) in accounts.iter().enumerate() {
        println!("({}): {}", i, account.yellow());
    }

    println!();
    println!("{}", "Private Keys".cyan().bold());
    println!("{}", "-----------------------".cyan());
    let private_keys = [
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
        "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
        "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
        "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a",
        "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba",
        "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e",
        "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356",
        "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97",
        "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
    ];

    for (i, key) in private_keys.iter().enumerate() {
        println!("({}): {}", i, key.yellow());
    }

    println!();
    println!("{}", "Polygon Sandbox Config:".cyan().bold());
    println!("{}", "L1 (Ethereum Mainnet Simulation):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        "Ethereum-L1".white(),
        "1".white(),
        "http://127.0.0.1:8545".white()
    );
    println!("{}", "L2 (Polygon zkEVM Simulation):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        "Polygon-zkEVM".white(),
        "1101".white(),
        "http://127.0.0.1:8546".white()
    );

    println!();
    println!("{}", "🔧 Next steps:".blue().bold());
    println!("• Check status: {}", "agg-sandbox status".yellow());
    println!("• View logs: {}", "agg-sandbox logs --follow".yellow());
    println!(
        "• Start in fork mode: {}",
        "agg-sandbox start --fork --detach".yellow()
    );
    println!(
        "• Start with second L2: {}",
        "agg-sandbox start --multi-l2 --detach".yellow()
    );
    println!("• Stop sandbox: {}", "agg-sandbox stop".yellow());
    println!();
}

fn print_sandbox_fork_info() {
    println!();
    println!("{}", "Available Accounts".cyan().bold());
    println!("{}", "-----------------------".cyan());
    let accounts = [
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
        "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
        "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
        "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65",
        "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc",
        "0x976EA74026E726554dB657fA54763abd0C3a0aa9",
        "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
        "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f",
        "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
    ];

    for (i, account) in accounts.iter().enumerate() {
        println!("({}): {}", i, account.yellow());
    }

    println!();
    println!("{}", "Private Keys".cyan().bold());
    println!("{}", "-----------------------".cyan());
    let private_keys = [
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
        "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
        "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
        "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a",
        "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba",
        "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e",
        "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356",
        "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97",
        "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
    ];

    for (i, key) in private_keys.iter().enumerate() {
        println!("({}): {}", i, key.yellow());
    }

    println!();
    println!("{}", "Polygon Sandbox Config (FORK MODE):".cyan().bold());
    println!("{}", "L1 (Ethereum Mainnet Fork):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        "Ethereum-L1-Fork".white(),
        "1".white(),
        "http://127.0.0.1:8545".white()
    );
    println!("{}", "L2 (Polygon zkEVM Fork):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        "Polygon-zkEVM-Fork".white(),
        "1101".white(),
        "http://127.0.0.1:8546".white()
    );

    println!();
    println!("{}", "🔧 Next steps:".blue().bold());
    println!("• Check status: {}", "agg-sandbox status".yellow());
    println!("• View logs: {}", "agg-sandbox logs --follow".yellow());
    println!(
        "• Start multi-L2 with fork: {}",
        "agg-sandbox start --multi-l2 --fork --detach".yellow()
    );
    println!("• Stop sandbox: {}", "agg-sandbox stop".yellow());
    println!();
    println!(
        "{}",
        "⚠️  Note: Fork mode uses live blockchain data from the configured fork URLs".yellow()
    );
    println!();
}

fn print_multi_l2_info(fork: bool) {
    println!();
    println!("{}", "Available Accounts".cyan().bold());
    println!("{}", "-----------------------".cyan());
    let accounts = [
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
        "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
        "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
        "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65",
        "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc",
        "0x976EA74026E726554dB657fA54763abd0C3a0aa9",
        "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
        "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f",
        "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
    ];

    for (i, account) in accounts.iter().enumerate() {
        println!("({}): {}", i, account.yellow());
    }

    println!();
    println!("{}", "Private Keys".cyan().bold());
    println!("{}", "-----------------------".cyan());
    let private_keys = [
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
        "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
        "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
        "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a",
        "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba",
        "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e",
        "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356",
        "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97",
        "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
    ];

    for (i, key) in private_keys.iter().enumerate() {
        println!("({}): {}", i, key.yellow());
    }

    println!();
    let config_title = if fork {
        "Multi-L2 Polygon Sandbox Config (FORK MODE):"
    } else {
        "Multi-L2 Polygon Sandbox Config:"
    };
    println!("{}", config_title.cyan().bold());

    let l1_name = if fork {
        "Ethereum-L1-Fork"
    } else {
        "Ethereum-L1"
    };
    let l2_1_name = if fork {
        "Polygon-zkEVM-Fork"
    } else {
        "Polygon-zkEVM"
    };
    let l2_2_name = if fork {
        "AggLayer-2-Fork"
    } else {
        "AggLayer-2"
    };

    println!("{}", "L1 (Ethereum Mainnet):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        l1_name.white(),
        "1".white(),
        "http://127.0.0.1:8545".white()
    );
    println!("{}", "L2-1 (Polygon zkEVM):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        l2_1_name.white(),
        "1101".white(),
        "http://127.0.0.1:8546".white()
    );
    println!("{}", "L2-2 (Second AggLayer Chain):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        l2_2_name.white(),
        "1102".white(),
        "http://127.0.0.1:8547".white()
    );

    println!();
    println!("{}", "🔧 Next steps:".blue().bold());
    println!("• Check status: {}", "agg-sandbox status".yellow());
    println!("• View logs: {}", "agg-sandbox logs --follow".yellow());
    println!("• Stop sandbox: {}", "agg-sandbox stop".yellow());

    if fork {
        println!();
        println!(
            "{}",
            "⚠️  Note: Fork mode uses live blockchain data from the configured fork URLs".yellow()
        );
    }
    println!();
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
    print_sandbox_info();
    Ok(())
}
