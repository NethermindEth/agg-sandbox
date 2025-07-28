use super::config::Config;
use colored::*;

/// Print accounts and private keys section
fn print_accounts_and_keys(config: &Config) {
    println!();
    println!("{}", "Available Accounts".cyan().bold());
    println!("{}", "-----------------------".cyan());

    for (i, account) in config.accounts.accounts.iter().enumerate() {
        println!("({i}): {}", account.as_ref().yellow());
    }

    println!();
    println!("{}", "Private Keys".cyan().bold());
    println!("{}", "-----------------------".cyan());

    for (i, key) in config.accounts.private_keys.iter().enumerate() {
        println!("({i}): {}", key.yellow());
    }
}

/// Print contract addresses section
fn print_contract_addresses(config: &Config) {
    println!();
    println!("{}", "Base Protocol Contracts:".cyan().bold());
    println!("{}", "L1 Contracts:".green());
    println!(
        "  FflonkVerifier: {}",
        config
            .contracts
            .get_contract("l1", "FflonkVerifier")
            .white()
    );
    println!(
        "  PolygonZkEVM: {}",
        config.contracts.get_contract("l1", "PolygonZkEVM").white()
    );
    println!(
        "  PolygonZkEVMBridge: {}",
        config
            .contracts
            .get_contract("l1", "PolygonZkEVMBridge")
            .white()
    );
    println!(
        "  PolygonZkEVMTimelock: {}",
        config
            .contracts
            .get_contract("l1", "PolygonZkEVMTimelock")
            .white()
    );
    println!(
        "  PolygonZkEVMGlobalExitRoot: {}",
        config
            .contracts
            .get_contract("l1", "PolygonZkEVMGlobalExitRoot")
            .white()
    );
    println!(
        "  PolygonRollupManager: {}",
        config
            .contracts
            .get_contract("l1", "PolygonRollupManager")
            .white()
    );
    println!(
        "  AggERC20: {}",
        config.contracts.get_contract("l1", "AggERC20").white()
    );
    println!(
        "  BridgeExtension: {}",
        config
            .contracts
            .get_contract("l1", "BridgeExtension")
            .white()
    );
    println!(
        "  GlobalExitRootManager: {}",
        config
            .contracts
            .get_contract("l1", "GlobalExitRootManager")
            .white()
    );

    println!("{}", "L2 Contracts:".green());
    println!(
        "  PolygonZkEVMBridge: {}",
        config
            .contracts
            .get_contract("l2", "PolygonZkEVMBridge")
            .white()
    );
    println!(
        "  PolygonZkEVMTimelock: {}",
        config
            .contracts
            .get_contract("l2", "PolygonZkEVMTimelock")
            .white()
    );
    println!(
        "  AggERC20: {}",
        config.contracts.get_contract("l2", "AggERC20").white()
    );
    println!(
        "  BridgeExtension: {}",
        config
            .contracts
            .get_contract("l2", "BridgeExtension")
            .white()
    );
    println!(
        "  GlobalExitRootManager: {}",
        config
            .contracts
            .get_contract("l2", "GlobalExitRootManager")
            .white()
    );
}

/// Print contract addresses section for multi-L2 mode (includes L3)
fn print_multi_l2_contract_addresses(config: &Config) {
    println!();
    println!("{}", "Base Protocol Contracts:".cyan().bold());
    println!("{}", "L1 Contracts:".green());
    println!(
        "  FflonkVerifier: {}",
        config
            .contracts
            .get_contract("l1", "FflonkVerifier")
            .white()
    );
    println!(
        "  PolygonZkEVM: {}",
        config.contracts.get_contract("l1", "PolygonZkEVM").white()
    );
    println!(
        "  PolygonZkEVMBridge: {}",
        config
            .contracts
            .get_contract("l1", "PolygonZkEVMBridge")
            .white()
    );
    println!(
        "  PolygonZkEVMTimelock: {}",
        config
            .contracts
            .get_contract("l1", "PolygonZkEVMTimelock")
            .white()
    );
    println!(
        "  PolygonZkEVMGlobalExitRoot: {}",
        config
            .contracts
            .get_contract("l1", "PolygonZkEVMGlobalExitRoot")
            .white()
    );
    println!(
        "  PolygonRollupManager: {}",
        config
            .contracts
            .get_contract("l1", "PolygonRollupManager")
            .white()
    );
    println!(
        "  AggERC20: {}",
        config.contracts.get_contract("l1", "AggERC20").white()
    );
    println!(
        "  BridgeExtension: {}",
        config
            .contracts
            .get_contract("l1", "BridgeExtension")
            .white()
    );
    println!(
        "  GlobalExitRootManager: {}",
        config
            .contracts
            .get_contract("l1", "GlobalExitRootManager")
            .white()
    );

    println!("{}", "L2-1 Contracts:".green());
    println!(
        "  PolygonZkEVMBridge: {}",
        config
            .contracts
            .get_contract("l2", "PolygonZkEVMBridge")
            .white()
    );
    println!(
        "  PolygonZkEVMTimelock: {}",
        config
            .contracts
            .get_contract("l2", "PolygonZkEVMTimelock")
            .white()
    );
    println!(
        "  AggERC20: {}",
        config.contracts.get_contract("l2", "AggERC20").white()
    );
    println!(
        "  BridgeExtension: {}",
        config
            .contracts
            .get_contract("l2", "BridgeExtension")
            .white()
    );
    println!(
        "  GlobalExitRootManager: {}",
        config
            .contracts
            .get_contract("l2", "GlobalExitRootManager")
            .white()
    );

    println!("{}", "L2-2 Contracts:".green());
    println!(
        "  PolygonZkEVMBridge: {}",
        config
            .contracts
            .get_contract("l3", "PolygonZkEVMBridge")
            .white()
    );
    println!(
        "  PolygonZkEVMTimelock: {}",
        config
            .contracts
            .get_contract("l3", "PolygonZkEVMTimelock")
            .white()
    );
    println!(
        "  AggERC20: {}",
        config.contracts.get_contract("l3", "AggERC20").white()
    );
    println!(
        "  BridgeExtension: {}",
        config
            .contracts
            .get_contract("l3", "BridgeExtension")
            .white()
    );
    println!(
        "  GlobalExitRootManager: {}",
        config
            .contracts
            .get_contract("l3", "GlobalExitRootManager")
            .white()
    );
}

/// Print sandbox information for local mode
pub fn print_sandbox_info(config: &Config) {
    print_accounts_and_keys(config);

    println!();
    println!("{}", "Polygon Sandbox Config:".cyan().bold());
    println!("{}", "L1 (Ethereum Mainnet Simulation):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        config.networks.l1.name.white(),
        config.networks.l1.chain_id.as_ref().white(),
        config.networks.l1.rpc_url.as_ref().white()
    );
    println!("{}", "L2 (Polygon zkEVM Simulation):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        config.networks.l2.name.white(),
        config.networks.l2.chain_id.as_ref().white(),
        config.networks.l2.rpc_url.as_ref().white()
    );

    // Add contract addresses section
    print_contract_addresses(config);

    println!();
    println!("{}", "🔧 Next steps:".blue().bold());
    println!("• Check status: {}", "aggsandbox status".yellow());
    println!("• View logs: {}", "aggsandbox logs --follow".yellow());
    println!(
        "• Start in fork mode: {}",
        "aggsandbox start --fork --detach".yellow()
    );
    println!(
        "• Start with second L2: {}",
        "aggsandbox start --multi-l2 --detach".yellow()
    );
    println!("• Stop sandbox: {}", "aggsandbox stop".yellow());
    println!("• Sandbox info: {}", "aggsandbox info".yellow());
    println!();
}

/// Print sandbox information for fork mode
pub fn print_sandbox_fork_info(config: &Config) {
    print_accounts_and_keys(config);

    println!();
    println!("{}", "Polygon Sandbox Config (FORK MODE):".cyan().bold());
    println!("{}", "L1 (Ethereum Mainnet Fork):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        format!("{}-Fork", config.networks.l1.name).white(),
        config.networks.l1.chain_id.as_ref().white(),
        config.networks.l1.rpc_url.as_ref().white()
    );
    println!("{}", "L2 (Polygon zkEVM Fork):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        format!("{}-Fork", config.networks.l2.name).white(),
        config.networks.l2.chain_id.as_ref().white(),
        config.networks.l2.rpc_url.as_ref().white()
    );

    // Add contract addresses section
    print_contract_addresses(config);

    println!();
    println!("{}", "🔧 Next steps:".blue().bold());
    println!("• Check status: {}", "aggsandbox status".yellow());
    println!("• View logs: {}", "aggsandbox logs --follow".yellow());
    println!(
        "• Start multi-L2 with fork: {}",
        "aggsandbox start --multi-l2 --fork --detach".yellow()
    );
    println!("• Stop sandbox: {}", "aggsandbox stop".yellow());
    println!("• Sandbox info: {}", "aggsandbox info".yellow());
    println!();
    println!(
        "{}",
        "⚠️  Note: Fork mode uses live blockchain data from the configured fork URLs".yellow()
    );
    println!();
}

/// Print sandbox information for multi-L2 mode
pub fn print_multi_l2_info(config: &Config, fork: bool) {
    print_accounts_and_keys(config);

    println!();
    let config_title = if fork {
        "Multi-L2 Polygon Sandbox Config (FORK MODE):"
    } else {
        "Multi-L2 Polygon Sandbox Config:"
    };
    println!("{}", config_title.cyan().bold());

    let l1_name = if fork {
        format!("{}-Fork", config.networks.l1.name)
    } else {
        config.networks.l1.name.clone()
    };
    let l2_1_name = if fork {
        format!("{}-Fork", config.networks.l2.name)
    } else {
        config.networks.l2.name.clone()
    };
    let l2_2_name = if let Some(l3) = &config.networks.l3 {
        if fork {
            format!("{}-Fork", l3.name)
        } else {
            l3.name.clone()
        }
    } else {
        "Not configured".to_string()
    };

    println!("{}", "L1 (Ethereum Mainnet):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        l1_name.white(),
        config.networks.l1.chain_id.as_ref().white(),
        config.networks.l1.rpc_url.as_ref().white()
    );
    println!("{}", "L2-1 (Polygon zkEVM):".green());
    println!(
        "  Name: {}    Chain ID: {}    RPC: {}",
        l2_1_name.white(),
        config.networks.l2.chain_id.as_ref().white(),
        config.networks.l2.rpc_url.as_ref().white()
    );

    if let Some(l3) = &config.networks.l3 {
        println!("{}", "L2-2 (Second Agglayer Chain):".green());
        println!(
            "  Name: {}    Chain ID: {}    RPC: {}",
            l2_2_name.white(),
            l3.chain_id.as_ref().white(),
            l3.rpc_url.as_ref().white()
        );
    }

    // Add contract addresses section
    print_multi_l2_contract_addresses(config);

    println!();
    println!("{}", "🔧 Next steps:".blue().bold());
    println!("• Check status: {}", "aggsandbox status".yellow());
    println!("• View logs: {}", "aggsandbox logs --follow".yellow());
    println!("• Stop sandbox: {}", "aggsandbox stop".yellow());

    if fork {
        println!();
        println!(
            "{}",
            "⚠️  Note: Fork mode uses live blockchain data from the configured fork URLs".yellow()
        );
    }
    println!();
}
