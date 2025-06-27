use colored::*;
use std::env;

/// Helper function to get environment variable with fallback, reloading .env file
fn get_env_var(key: &str, fallback: &str) -> String {
    // Reload .env file to get the most up-to-date values
    dotenv::dotenv().ok();
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}

/// Print accounts and private keys section
fn print_accounts_and_keys() {
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
}

/// Print contract addresses section
fn print_contract_addresses() {
    println!();
    println!("{}", "Base Protocol Contracts:".cyan().bold());
    println!("{}", "L1 Contracts:".green());
    println!(
        "  FflonkVerifier: {}",
        get_env_var("FFLONK_VERIFIER_L1", "Not deployed").white()
    );
    println!(
        "  PolygonZkEVM: {}",
        get_env_var("POLYGON_ZKEVM_L1", "Not deployed").white()
    );
    println!(
        "  PolygonZkEVMBridge: {}",
        get_env_var("POLYGON_ZKEVM_BRIDGE_L1", "Not deployed").white()
    );
    println!(
        "  PolygonZkEVMTimelock: {}",
        get_env_var("POLYGON_ZKEVM_TIMELOCK_L1", "Not deployed").white()
    );
    println!(
        "  PolygonZkEVMGlobalExitRoot: {}",
        get_env_var("POLYGON_ZKEVM_GLOBAL_EXIT_ROOT_L1", "Not deployed").white()
    );
    println!(
        "  PolygonRollupManager: {}",
        get_env_var("POLYGON_ROLLUP_MANAGER_L1", "Not deployed").white()
    );
    println!(
        "  ERC20: {}",
        get_env_var("ERC20", "Not deployed").white()
    );

    println!("{}", "L2 Contracts:".green());
    println!(
        "  PolygonZkEVMBridge: {}",
        get_env_var("POLYGON_ZKEVM_BRIDGE_L2", "Not deployed").white()
    );
    println!(
        "  PolygonZkEVMTimelock: {}",
        get_env_var("POLYGON_ZKEVM_TIMELOCK_L2", "Not deployed").white()
    );
}

/// Print sandbox information for local mode
pub fn print_sandbox_info() {
    print_accounts_and_keys();

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

    // Add contract addresses section
    print_contract_addresses();

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
    println!("• Sandbox info: {}", "agg-sandbox info".yellow());
    println!();
}

/// Print sandbox information for fork mode
pub fn print_sandbox_fork_info() {
    print_accounts_and_keys();

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

    // Add contract addresses section
    print_contract_addresses();

    println!();
    println!("{}", "🔧 Next steps:".blue().bold());
    println!("• Check status: {}", "agg-sandbox status".yellow());
    println!("• View logs: {}", "agg-sandbox logs --follow".yellow());
    println!(
        "• Start multi-L2 with fork: {}",
        "agg-sandbox start --multi-l2 --fork --detach".yellow()
    );
    println!("• Stop sandbox: {}", "agg-sandbox stop".yellow());
    println!("• Sandbox info: {}", "agg-sandbox info".yellow());
    println!();
    println!(
        "{}",
        "⚠️  Note: Fork mode uses live blockchain data from the configured fork URLs".yellow()
    );
    println!();
}

/// Print sandbox information for multi-L2 mode
pub fn print_multi_l2_info(fork: bool) {
    print_accounts_and_keys();

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

    // Add contract addresses section
    print_contract_addresses();

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
