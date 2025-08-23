use crate::config::Config;
use crate::docker::{create_auto_docker_builder, execute_docker_command_with_output};
use crate::error::Result;
use crate::logs;
use colored::*;

/// Detect the actual running mode by checking which services are running
fn detect_running_mode() -> (bool, bool, bool) {
    // Returns (is_multi_l2, is_fork, is_running)
    let docker_builder = create_auto_docker_builder();
    let cmd = docker_builder.build_ps_command();

    if let Ok(output) = execute_docker_command_with_output(cmd) {
        // Check if any aggsandbox services are running
        let has_sandbox_services =
            output.contains("anvil-l1") || output.contains("anvil-l2") || output.contains("aggkit");

        // Check if L3 services are running to determine multi-L2 mode
        let has_l3_services = output.contains("anvil-l3") || output.contains("aggkit-l3");

        // For fork mode detection, we'll still rely on URL patterns in the config
        // since the running services don't clearly indicate fork vs non-fork mode
        (has_l3_services, false, has_sandbox_services)
    } else {
        // If we can't get status, assume nothing is running
        (false, false, false)
    }
}

/// Handle the info command
pub async fn handle_info() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "📋 Agglayer Sandbox Information".blue().bold());

    // Detect the actual running mode by checking which services are running
    let (is_multi_l2_running, _, is_sandbox_running) = detect_running_mode();

    // Check if sandbox is actually running
    if !is_sandbox_running {
        println!();
        println!(
            "{}",
            "❌ No Agglayer sandbox is currently running".red().bold()
        );
        println!();
        println!("{}", "🚀 To start the sandbox:".yellow().bold());
        println!("• Standard mode: {}", "aggsandbox start --detach".green());
        println!(
            "• Fork mode: {}",
            "aggsandbox start --fork --detach".green()
        );
        println!(
            "• Multi-L2 mode: {}",
            "aggsandbox start --multi-l2 --detach".green()
        );
        println!();
        println!("{}", "📊 Check running services: aggsandbox status".cyan());
        return Ok(());
    }

    // Detect fork mode by checking URL patterns
    let is_fork_mode = config.networks.l1.rpc_url.as_str().contains("alchemy.com")
        || config.networks.l1.rpc_url.as_str().contains("infura.io")
        || config.networks.l1.rpc_url.as_str().contains("mainnet")
        || config.networks.l2.rpc_url.as_str().contains("alchemy.com")
        || config.networks.l2.rpc_url.as_str().contains("infura.io")
        || config
            .networks
            .l2
            .rpc_url
            .as_str()
            .contains("polygon-mainnet");

    // Choose the appropriate display function based on actual running mode
    if is_multi_l2_running {
        logs::print_multi_l2_info(&config, is_fork_mode);
    } else if is_fork_mode {
        logs::print_sandbox_fork_info(&config);
    } else {
        logs::print_sandbox_info(&config);
    }

    Ok(())
}
