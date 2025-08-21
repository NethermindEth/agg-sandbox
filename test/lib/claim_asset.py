#!/usr/bin/env python3
"""
Claim Asset Module - Python Implementation
Functions for claiming bridged assets using aggsandbox CLI
"""

import time
import os
import json
import subprocess
from typing import Optional
from bridge_lib import BridgeLogger, AggsandboxAPI, BridgeUtils, BRIDGE_CONFIG

class ClaimAsset:
    """Asset claiming operations"""
    
    @staticmethod
    def claim_asset(dest_network: int, tx_hash: str, source_network: int,
                   private_key: str, deposit_count: Optional[int] = None) -> Optional[str]:
        """Claim bridged assets using aggsandbox CLI"""
        BridgeLogger.step(f"Claiming bridged assets on network {dest_network}")
        BridgeLogger.info(f"Source transaction: {tx_hash}")
        BridgeLogger.info(f"Source network: {source_network}")
        
        cmd = [
            "aggsandbox", "bridge", "claim",
            "--network", str(dest_network),
            "--tx-hash", tx_hash,
            "--source-network", str(source_network),
            "--private-key", private_key
        ]
        
        if deposit_count is not None:
            cmd.extend(["--deposit-count", str(deposit_count)])
        
        if os.environ.get('DEBUG') == '1':
            cmd.append("--verbose")
        
        BridgeLogger.debug(f"Executing: {' '.join(cmd)}")
        
        success, output = AggsandboxAPI.run_command(cmd)
        if not success:
            BridgeLogger.error(f"Claim transaction failed: {output}")
            
            # Check for common error patterns
            if "AlreadyClaimed" in output:
                BridgeLogger.warning("Asset was already claimed")
                return "already_claimed"
            elif "GlobalExitRootInvalid" in output:
                BridgeLogger.warning("Global exit root invalid - may need to wait longer")
                return None
            
            return None
        
        tx_hash = BridgeUtils.extract_tx_hash(output)
        if tx_hash:
            BridgeLogger.success(f"Claim transaction completed: {tx_hash}")
            return tx_hash
        else:
            BridgeLogger.success("Claim transaction completed successfully")
            return "completed"
    
    @staticmethod
    def claim_asset_with_retry(dest_network: int, tx_hash: str, source_network: int,
                              private_key: str, deposit_count: Optional[int] = None,
                              max_retries: int = 3, retry_delay: int = 10) -> Optional[str]:
        """Claim asset with retry logic"""
        BridgeLogger.step(f"Claiming asset with retry logic (max {max_retries} attempts)")
        
        for attempt in range(1, max_retries + 1):
            BridgeLogger.info(f"Claim attempt {attempt}/{max_retries}")
            
            result = ClaimAsset.claim_asset(dest_network, tx_hash, source_network, 
                                          private_key, deposit_count)
            
            if result == "already_claimed":
                BridgeLogger.info("Asset was already claimed")
                return result
            elif result:
                BridgeLogger.success(f"Asset claimed successfully on attempt {attempt}")
                return result
            else:
                if attempt < max_retries:
                    BridgeLogger.warning(f"Claim failed, retrying in {retry_delay}s...")
                    time.sleep(retry_delay)
                else:
                    BridgeLogger.error("Max retries reached, claim failed")
        
        return None
    
    @staticmethod
    def verify_claim_transaction(tx_hash: str, network_id: int) -> bool:
        """Verify claim transaction was successful"""
        if not BRIDGE_CONFIG:
            return False
        
        BridgeLogger.step(f"Verifying claim transaction: {tx_hash}")
        
        rpc_url = BridgeUtils.get_rpc_url(network_id, BRIDGE_CONFIG)
        
        cmd = ["cast", "receipt", tx_hash, "--rpc-url", rpc_url, "--json"]
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            receipt = json.loads(result.stdout)
            
            status = receipt.get('status')
            if status == '0x1':
                BridgeLogger.success("Claim transaction was successful")
                
                if os.environ.get('DEBUG') == '1':
                    logs = receipt.get('logs', [])
                    BridgeLogger.debug(f"Transaction logs: {len(logs)} entries")
                
                return True
            else:
                BridgeLogger.error(f"Claim transaction failed (status: {status})")
                return False
                
        except (subprocess.CalledProcessError, json.JSONDecodeError) as e:
            BridgeLogger.error(f"Could not verify transaction: {e}")
            return False
