use clap::{Parser, Subcommand};
use colored::*;
use std::process::Command;
use std::path::Path;
use anyhow::{Result, Context};

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure we're in the right directory (where docker-compose.yml exists)
    if !Path::new("docker-compose.yml").exists() {
        eprintln!("{}", "Error: docker-compose.yml not found in current directory".red());
        eprintln!("{}", "Please run this command from the project root directory".yellow());
        std::process::exit(1);
    }

    match cli.command {
        Commands::Start { detach, build } => start_sandbox(detach, build),
        Commands::Stop { volumes } => stop_sandbox(volumes),
        Commands::Status => show_status(),
        Commands::Logs { follow, service } => show_logs(follow, service),
        Commands::Restart => restart_sandbox(),
    }
}

fn start_sandbox(detach: bool, build: bool) -> Result<()> {
    println!("{}", "🚀 Starting AggLayer sandbox environment...".green().bold());
    
    let mut cmd = Command::new("docker-compose");
    cmd.arg("up");
    
    if detach {
        cmd.arg("-d");
    }
    
    if build {
        cmd.arg("--build");
    }
    
    let status = cmd
        .status()
        .context("Failed to execute docker-compose up")?;
    
    if status.success() {
        if detach {
            println!("{}", "✅ Sandbox started in detached mode".green());
        } else {
            println!("{}", "✅ Sandbox stopped".green());
        }
    } else {
        eprintln!("{}", "❌ Failed to start sandbox".red());
        std::process::exit(1);
    }
    
    Ok(())
}

fn stop_sandbox(volumes: bool) -> Result<()> {
    println!("{}", "🛑 Stopping AggLayer sandbox environment...".yellow().bold());
    
    let mut cmd = Command::new("docker-compose");
    cmd.arg("down");
    
    if volumes {
        cmd.arg("-v");
    }
    
    let status = cmd
        .status()
        .context("Failed to execute docker-compose down")?;
    
    if status.success() {
        println!("{}", "✅ Sandbox stopped successfully".green());
    } else {
        eprintln!("{}", "❌ Failed to stop sandbox".red());
        std::process::exit(1);
    }
    
    Ok(())
}

fn show_status() -> Result<()> {
    println!("{}", "📊 Sandbox service status:".blue().bold());
    
    let status = Command::new("docker-compose")
        .arg("ps")
        .status()
        .context("Failed to execute docker-compose ps")?;
    
    if !status.success() {
        eprintln!("{}", "❌ Failed to get service status".red());
        std::process::exit(1);
    }
    
    Ok(())
}

fn show_logs(follow: bool, service: Option<String>) -> Result<()> {
    let service_name = service.as_deref().unwrap_or("all services");
    println!("{} {}", "📋 Showing logs for:".blue().bold(), service_name.cyan());
    
    let mut cmd = Command::new("docker-compose");
    cmd.arg("logs");
    
    if follow {
        cmd.arg("-f");
    }
    
    if let Some(svc) = service {
        cmd.arg(svc);
    }
    
    let status = cmd
        .status()
        .context("Failed to execute docker-compose logs")?;
    
    if !status.success() {
        eprintln!("{}", "❌ Failed to show logs".red());
        std::process::exit(1);
    }
    
    Ok(())
}

fn restart_sandbox() -> Result<()> {
    println!("{}", "🔄 Restarting AggLayer sandbox environment...".yellow().bold());
    
    // First stop
    stop_sandbox(false)?;
    
    // Then start
    start_sandbox(true, false)?;
    
    println!("{}", "✅ Sandbox restarted successfully".green());
    
    Ok(())
} 