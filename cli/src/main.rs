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
    /// Start the sandbox environment (docker-compose up)
    Start {
        /// Run in detached mode
        #[arg(short, long)]
        detach: bool,
        /// Build images before starting
        #[arg(short, long)]
        build: bool,
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

    match cli.command {
        Commands::Start { detach, build } => start_sandbox(detach, build),
        Commands::Stop { volumes } => stop_sandbox(volumes),
        Commands::Status => show_status(),
        Commands::Logs { follow, service } => show_logs(follow, service),
        Commands::Restart => restart_sandbox(),
        Commands::Info => show_info(),
    }
}

fn start_sandbox(detach: bool, build: bool) -> Result<()> {
    println!(
        "{}",
        "🚀 Starting AggLayer sandbox environment...".green().bold()
    );

    let mut cmd = Command::new("docker-compose");
    cmd.arg("up");

    if detach {
        cmd.arg("-d");
    }

    if build {
        cmd.arg("--build");
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
            println!("{}", "✅ Sandbox started in detached mode".green());
            print_sandbox_info();
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
    println!("• Stop sandbox: {}", "agg-sandbox stop".yellow());
    println!();
}

fn stop_sandbox(volumes: bool) -> Result<()> {
    println!(
        "{}",
        "🛑 Stopping AggLayer sandbox environment..."
            .yellow()
            .bold()
    );

    let mut cmd = Command::new("docker-compose");
    cmd.arg("down");

    if volumes {
        cmd.arg("-v");
    }

    // Capture output to ensure consistent behavior across platforms
    let output = cmd
        .output()
        .context("Failed to execute docker-compose down")?;

    // Print stdout and stderr to ensure Docker output is shown
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if output.status.success() {
        println!("{}", "✅ Sandbox stopped successfully".green());
    } else {
        eprintln!("{}", "❌ Failed to stop sandbox".red());
        std::process::exit(1);
    }

    Ok(())
}

fn show_status() -> Result<()> {
    println!("{}", "📊 Sandbox service status:".blue().bold());

    let output = Command::new("docker-compose")
        .arg("ps")
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

    // Then start
    start_sandbox(true, false)?;

    println!("{}", "✅ Sandbox restarted successfully".green());

    Ok(())
}

fn show_info() -> Result<()> {
    println!("{}", "📋 AggLayer Sandbox Information".blue().bold());
    print_sandbox_info();
    Ok(())
}
