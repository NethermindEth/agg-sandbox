use crate::config::Config;
use crate::error::Result;
use crate::logs;
use colored::*;

/// Handle the info command
pub async fn handle_info() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "📋 Agglayer Sandbox Information".blue().bold());
    
    // Check if we're in multi-L2 mode by looking for L3 network configuration
    if config.networks.l3.is_some() {
        // Multi-L2 mode - check if we're also in fork mode
        // We can detect fork mode by checking if the L1 RPC URL contains a fork URL pattern
        // or if we have specific fork indicators in the config
        let is_fork_mode = config.networks.l1.rpc_url.as_str().contains("alchemy.com") 
            || config.networks.l1.rpc_url.as_str().contains("infura.io")
            || config.networks.l1.rpc_url.as_str().contains("mainnet")
            || config.networks.l2.rpc_url.as_str().contains("alchemy.com")
            || config.networks.l2.rpc_url.as_str().contains("infura.io")
            || config.networks.l2.rpc_url.as_str().contains("polygon-mainnet");
            
        logs::print_multi_l2_info(&config, is_fork_mode);
    } else {
        // Regular mode or fork mode (but not multi-L2)
        let is_fork_mode = config.networks.l1.rpc_url.as_str().contains("alchemy.com") 
            || config.networks.l1.rpc_url.as_str().contains("infura.io")
            || config.networks.l1.rpc_url.as_str().contains("mainnet")
            || config.networks.l2.rpc_url.as_str().contains("alchemy.com")
            || config.networks.l2.rpc_url.as_str().contains("infura.io")
            || config.networks.l2.rpc_url.as_str().contains("polygon-mainnet");
            
        if is_fork_mode {
            logs::print_sandbox_fork_info(&config);
        } else {
            logs::print_sandbox_info(&config);
        }
    }

    Ok(())
}
