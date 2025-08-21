#!/usr/bin/env python3
"""
Bridge Asset Module - Python Implementation
Functions for bridging assets using aggsandbox CLI
"""

import subprocess
import time
import os
from typing import Optional, Tuple
from bridge_lib import BridgeLogger, AggsandboxAPI, BridgeUtils, BRIDGE_CONFIG

class BridgeAsset:
    """Asset bridging operations"""
    
    @staticmethod
    def bridge_asset(source_network: int, dest_network: int, amount: int,
                    token_address: str, to_address: str, private_key: str) -> Optional[str]:
        """Bridge assets using aggsandbox CLI"""
        BridgeLogger.step(f"Bridging {amount} tokens from network {source_network} to network {dest_network}")
        BridgeLogger.info(f"Token: {token_address}")
        BridgeLogger.info(f"To address: {to_address}")
        
        cmd = [
            "aggsandbox", "bridge", "asset",
            "--network", str(source_network),
            "--destination-network", str(dest_network),
            "--amount", str(amount),
            "--token-address", token_address,
            "--to-address", to_address,
            "--private-key", private_key
        ]
        
        if os.environ.get('DEBUG') == '1':
            cmd.append("--verbose")
        
        BridgeLogger.debug(f"Executing: {' '.join(cmd)}")
        
        success, output = AggsandboxAPI.run_command(cmd)
        if not success:
            BridgeLogger.error(f"Bridge transaction failed: {output}")
            return None
        
        tx_hash = BridgeUtils.extract_tx_hash(output)
        if tx_hash:
            BridgeLogger.success(f"Bridge transaction initiated: {tx_hash}")
            return tx_hash
        else:
            BridgeLogger.warning("Could not extract transaction hash")
            BridgeLogger.debug(f"Output: {output}")
            return None
    
    @staticmethod
    def wait_for_bridge_indexing(network_id: int, tx_hash: str, 
                               max_retries: int = 10, retry_delay: int = 2) -> bool:
        """Wait for bridge to be indexed"""
        BridgeLogger.step("Waiting for bridge indexing")
        BridgeLogger.info(f"Checking network {network_id} for bridge TX: {tx_hash}")
        
        # Initial wait
        time.sleep(5)
        
        for attempt in range(max_retries):
            BridgeLogger.debug(f"Checking bridge indexing (attempt {attempt + 1}/{max_retries})")
            
            bridge_data = AggsandboxAPI.get_bridges(network_id)
            if bridge_data and 'bridges' in bridge_data:
                # Look for our transaction hash
                for bridge in bridge_data['bridges']:
                    if bridge.get('tx_hash') == tx_hash:
                        BridgeLogger.success("Bridge found in indexing system")
                        return True
            
            if attempt < max_retries - 1:
                BridgeLogger.info(f"Bridge not indexed yet, waiting... (attempt {attempt + 1}/{max_retries})")
                time.sleep(retry_delay)
        
        BridgeLogger.error(f"Bridge with TX {tx_hash} not found after {max_retries} attempts")
        return False
    
    @staticmethod
    def execute_l1_to_l2_bridge(amount: int, token_address: str, 
                               source_account: str, dest_account: str,
                               source_private_key: str, dest_private_key: str) -> Optional[Tuple[str, str]]:
        """Execute complete L1 to L2 asset bridge flow"""
        if not BRIDGE_CONFIG:
            BridgeLogger.error("Bridge configuration not initialized")
            return None
        
        BridgeLogger.step("Executing complete L1 to L2 asset bridge flow")
        BridgeLogger.info(f"Amount: {amount} tokens")
        BridgeLogger.info(f"Token: {token_address}")
        BridgeLogger.info(f"From: {source_account} (L1)")
        BridgeLogger.info(f"To: {dest_account} (L2)")
        
        # Import claim module here to avoid circular imports
        from claim_asset import ClaimAsset
        
        # Step 1: Bridge assets
        bridge_tx_hash = BridgeAsset.bridge_asset(
            BRIDGE_CONFIG.network_id_mainnet,
            BRIDGE_CONFIG.network_id_agglayer_1,
            amount,
            token_address,
            dest_account,
            source_private_key
        )
        
        if not bridge_tx_hash:
            BridgeLogger.error("Failed to bridge assets from L1 to L2")
            return None
        
        BridgeLogger.success(f"Bridge transaction completed: {bridge_tx_hash}")
        
        # Step 2: Wait for indexing
        if not BridgeAsset.wait_for_bridge_indexing(BRIDGE_CONFIG.network_id_mainnet, bridge_tx_hash):
            BridgeLogger.error("Bridge indexing failed or timed out")
            return None
        
        # Step 3: Claim assets
        claim_tx_hash = ClaimAsset.claim_asset(
            BRIDGE_CONFIG.network_id_agglayer_1,
            bridge_tx_hash,
            BRIDGE_CONFIG.network_id_mainnet,
            dest_private_key
        )
        
        if not claim_tx_hash:
            BridgeLogger.error("Failed to claim assets on L2")
            return None
        
        BridgeLogger.success(f"Claim transaction completed: {claim_tx_hash}")
        return bridge_tx_hash, claim_tx_hash
    
    @staticmethod
    def execute_l2_to_l1_bridge(amount: int, token_address: str,
                               source_account: str, dest_account: str,
                               source_private_key: str, dest_private_key: str) -> Optional[Tuple[str, str]]:
        """Execute complete L2 to L1 asset bridge flow"""
        if not BRIDGE_CONFIG:
            BridgeLogger.error("Bridge configuration not initialized")
            return None
        
        BridgeLogger.step("Executing complete L2 to L1 asset bridge flow")
        BridgeLogger.info(f"Amount: {amount} tokens")
        BridgeLogger.info(f"Token: {token_address}")
        BridgeLogger.info(f"From: {source_account} (L2)")
        BridgeLogger.info(f"To: {dest_account} (L1)")
        
        # Import claim module here to avoid circular imports
        from claim_asset import ClaimAsset
        
        # Step 1: Bridge assets
        bridge_tx_hash = BridgeAsset.bridge_asset(
            BRIDGE_CONFIG.network_id_agglayer_1,
            BRIDGE_CONFIG.network_id_mainnet,
            amount,
            token_address,
            dest_account,
            source_private_key
        )
        
        if not bridge_tx_hash:
            BridgeLogger.error("Failed to bridge assets from L2 to L1")
            return None
        
        BridgeLogger.success(f"Bridge transaction completed: {bridge_tx_hash}")
        
        # Step 2: Wait for indexing (L1 network for L2->L1 bridges)
        if not BridgeAsset.wait_for_bridge_indexing(BRIDGE_CONFIG.network_id_mainnet, bridge_tx_hash):
            BridgeLogger.error("Bridge indexing failed or timed out")
            return None
        
        # Step 3: Claim assets
        claim_tx_hash = ClaimAsset.claim_asset(
            BRIDGE_CONFIG.network_id_mainnet,
            bridge_tx_hash,
            BRIDGE_CONFIG.network_id_agglayer_1,
            dest_private_key
        )
        
        if not claim_tx_hash:
            BridgeLogger.error("Failed to claim assets on L1")
            return None
        
        BridgeLogger.success(f"Claim transaction completed: {claim_tx_hash}")
        return bridge_tx_hash, claim_tx_hash
