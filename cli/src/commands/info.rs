use crate::config::Config;
use crate::docker::{create_auto_docker_builder, execute_docker_command_with_output};
use crate::error::Result;
use crate::logs;
use colored::*;

/// Detect the actual running mode by checking which services are running
fn detect_running_mode() -> (bool, bool) {
    // Returns (is_multi_l2, is_fork)
    let docker_builder = create_auto_docker_builder();
    let cmd = docker_builder.build_ps_command();
    
    if let Ok(output) = execute_docker_command_with_output(cmd) {
        // Check if L3 services are running to determine multi-L2 mode
        let has_l3_services = output.contains("anvil-l3") || output.contains("aggkit-l3");
        
        // For fork mode detection, we'll still rely on URL patterns in the config
        // since the running services don't clearly indicate fork vs non-fork mode
        (has_l3_services, false) // We'll handle fork detection separately
    } else {
        // If we can't get status, fall back to config-based detection
        (false, false)
    }
}

/// Handle the info command
pub async fn handle_info() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "📋 Agglayer Sandbox Information".blue().bold());

    // Detect the actual running mode by checking which services are running
    let (is_multi_l2_running, _) = detect_running_mode();
    
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
