#!/usr/bin/env python3
"""
Bridge Asset Module - Python Implementation
Functions for bridging assets using aggsandbox CLI
"""

import subprocess
import time
import os
from typing import Optional, Tuple
from bridge_lib import BridgeLogger, BridgeUtils, BRIDGE_CONFIG
from aggsandbox_api import AggsandboxAPI, BridgeAssetArgs

class BridgeAsset:
    """Asset bridging operations"""
    
    @staticmethod
    def bridge_asset(source_network: int, dest_network: int, amount: int,
                    token_address: str, to_address: str, private_key: str) -> Optional[str]:
        """Bridge assets using aggsandbox CLI"""
        BridgeLogger.step(f"Bridging {amount} tokens from network {source_network} to network {dest_network}")
        BridgeLogger.info(f"Token: {token_address}")
        BridgeLogger.info(f"To address: {to_address}")
        
        # Use the new aggsandbox API
        args = BridgeAssetArgs(
            network=source_network,
            destination_network=dest_network,
            amount=str(amount),
            token_address=token_address,
            to_address=to_address,
            private_key=private_key
        )
        
        success, output = AggsandboxAPI.bridge_asset(args)
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
    def find_bridge_by_tx_hash(tx_hash: str, source_network: int, max_attempts: int = 6) -> Optional[dict]:
        """Find bridge transaction in bridge events using aggsandbox show bridges --network-id --json"""
        BridgeLogger.step(f"Finding bridge in network {source_network} bridge events")
        BridgeLogger.info(f"Looking for bridge TX: {tx_hash}")
        
        for attempt in range(max_attempts):
            BridgeLogger.info(f"Checking network {source_network} bridge events (attempt {attempt + 1}/{max_attempts})")
            time.sleep(3)  # Wait between attempts
            
            # Get bridges from source network where bridge events are stored
            bridge_data = AggsandboxAPI.get_bridges(source_network)
            if bridge_data and bridge_data.get('bridges'):
                BridgeLogger.debug(f"Found {len(bridge_data['bridges'])} total bridges on network {source_network}")
                
                # Look for our specific bridge transaction
                for bridge in bridge_data['bridges']:
                    if bridge.get('tx_hash') == tx_hash:
                        BridgeLogger.success(f"✅ Found our bridge on network {source_network} (attempt {attempt + 1})!")
                        return bridge
                
                BridgeLogger.debug(f"Our TX {tx_hash} not found yet in network {source_network} bridges")
        
        BridgeLogger.warning(f"Bridge TX {tx_hash} not found after {max_attempts} attempts")
        return None
    
    @staticmethod
    def get_most_recent_bridge(source_network: int) -> Optional[dict]:
        """Get the most recent bridge from specified network bridge events"""
        bridge_data = AggsandboxAPI.get_bridges(source_network)
        if bridge_data and bridge_data.get('bridges'):
            return bridge_data['bridges'][0]  # Most recent
        return None