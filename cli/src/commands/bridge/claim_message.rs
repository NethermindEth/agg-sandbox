use crate::error::Result;
use ethers::prelude::*;
use ethers::providers::Http;
use ethers::signers::LocalWallet;
use std::sync::Arc;

use super::{BridgeContract, GasOptions};

/// Execute claimMessage contract call
pub async fn execute_claim_message(
    bridge: &BridgeContract<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>,
    deposit_count: u64,
    mainnet_root: H256,
    rollup_root: H256,
    origin_network: u32,
    origin_addr: Address,
    destination_network_id: u32,
    dest_addr: Address,
    amount_wei: U256,
    metadata_bytes: Vec<u8>,
    gas_options: &GasOptions,
) -> Result<H256> {
    let mut call = bridge.claim_message(
        deposit_count.into(), // globalIndex
        mainnet_root.into(),  // mainnetExitRoot
        rollup_root.into(),   // rollupExitRoot
        origin_network,
        origin_addr, // originAddress for message
        destination_network_id,
        dest_addr,
        amount_wei,
        ethers::types::Bytes::from(metadata_bytes), // message data
    );

    if gas_options.gas_limit.is_none() {
        call = call.gas(3_000_000u64); // Default high gas limit for claims
    }

    let call = gas_options.apply_to_call_with_return(call);
    let tx = call.send().await.map_err(|e| {
        crate::error::AggSandboxError::Config(crate::error::ConfigError::validation_failed(
            &format!("Failed to send claim message transaction: {e}"),
        ))
    })?;
    Ok(tx.tx_hash())
}
