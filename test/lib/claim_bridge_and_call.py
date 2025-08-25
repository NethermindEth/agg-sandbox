#!/usr/bin/env python3
"""
Claim Bridge and Call Module - Python Implementation
Functions for claiming bridge and call transactions using aggsandbox CLI
"""

import time
import json
import subprocess
import os
from typing import Optional, Dict, Any
from bridge_lib import BridgeLogger, BridgeUtils, BRIDGE_CONFIG
from aggsandbox_api import AggsandboxAPI

class ClaimBridgeAndCall:
    """Bridge and call claiming operations"""
    
    @staticmethod
    def claim_bridge_and_call(dest_network: int, tx_hash: str, source_network: int,
                             private_key: str, deposit_count: Optional[int] = None) -> Optional[str]:
        """Claim bridge and call - first claim asset, then claim message (which triggers call)"""
        BridgeLogger.step(f"Claiming bridge and call on network {dest_network}")
        BridgeLogger.info(f"Source transaction: {tx_hash}")
        BridgeLogger.info(f"Source network: {source_network}")
        BridgeLogger.info("This will claim asset first, then message (which triggers the call)")
        
        # Import here to avoid circular imports
        from claim_asset import ClaimAsset
        from claim_message import ClaimMessage
        
        # Step 1: Claim the asset first
        BridgeLogger.step("Step 1: Claiming asset")
        asset_claim_tx = ClaimAsset.claim_asset(dest_network, tx_hash, source_network)
        
        if not asset_claim_tx:
            BridgeLogger.error("Failed to claim asset")
            return None
        elif asset_claim_tx == "already_claimed":
            BridgeLogger.info("Asset was already claimed, proceeding to message claim")
        else:
            BridgeLogger.success(f"Asset claimed: {asset_claim_tx}")
        
        # Step 2: Claim the message (this triggers the call execution)
        BridgeLogger.step("Step 2: Claiming message (triggers call execution)")
        message_claim_tx = ClaimMessage.claim_message(dest_network, tx_hash, source_network, private_key, deposit_count)
        
        if not message_claim_tx:
            BridgeLogger.error("Failed to claim message")
            return None
        elif message_claim_tx == "already_claimed":
            BridgeLogger.info("Message was already claimed")
            return "already_claimed"
        else:
            BridgeLogger.success(f"Message claimed and call executed: {message_claim_tx}")
            return message_claim_tx
    
    @staticmethod
    def get_bridge_and_call_info(network_id: int, tx_hash: str) -> Optional[Dict[str, Any]]:
        """Get bridge and call information from the bridge service"""
        BridgeLogger.step("Getting bridge and call information")
        BridgeLogger.info(f"Network: {network_id}")
        BridgeLogger.info(f"Transaction: {tx_hash}")
        
        bridge_data = AggsandboxAPI.get_bridges(network_id)
        if not bridge_data:
            return None
        
        bridges = bridge_data.get('bridges', [])
        
        # Find bridges with our transaction hash
        matching_bridges = [b for b in bridges if b.get('bridge_tx_hash') == tx_hash]
        
        if not matching_bridges:
            BridgeLogger.warning(f"No bridges found for transaction {tx_hash}")
            return None
        
        # Analyze asset and message deposits
        asset_deposits = [b for b in matching_bridges if b.get('leaf_type') == 0]
        message_deposits = [b for b in matching_bridges if b.get('leaf_type') == 1]
        
        BridgeLogger.info(f"Asset deposits found: {len(asset_deposits)}")
        BridgeLogger.info(f"Message deposits found: {len(message_deposits)}")
        
        if asset_deposits and message_deposits:
            BridgeLogger.success("✅ Bridge and call transaction found (has both asset and message)")
        elif asset_deposits:
            BridgeLogger.warning("⚠️ Only asset deposit found (missing message)")
        elif message_deposits:
            BridgeLogger.warning("⚠️ Only message deposit found (missing asset)")
        else:
            BridgeLogger.warning("⚠️ No deposits found for this transaction")
        
        return {
            'asset_deposits': asset_deposits,
            'message_deposits': message_deposits,
            'total_bridges': len(matching_bridges)
        }
    
    @staticmethod
    def verify_bridge_and_call_claim(claim_tx_hash: str, network_id: int,
                                    expected_call_target: Optional[str] = None) -> bool:
        """Verify bridge and call claim was successful and call was executed"""
        if not BRIDGE_CONFIG:
            return False
        
        BridgeLogger.step("Verifying bridge and call claim execution")
        BridgeLogger.info(f"Claim transaction: {claim_tx_hash}")
        BridgeLogger.info(f"Network: {network_id}")
        
        rpc_url = BridgeUtils.get_rpc_url(network_id, BRIDGE_CONFIG)
        
        try:
            # Get transaction receipt
            cmd = ["cast", "receipt", claim_tx_hash, "--rpc-url", rpc_url, "--json"]
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            receipt = json.loads(result.stdout)
            
            status = receipt.get('status')
            if status == '0x1':
                BridgeLogger.success("Bridge and call claim transaction was successful")
                
                logs = receipt.get('logs', [])
                BridgeLogger.info(f"Total log entries: {len(logs)}")
                
                # Look for ClaimEvent logs
                claim_events = [log for log in logs 
                              if log.get('topics', [{}])[0] == "0x25308c93ceeed775b33ab0a7fa6302fc6f1e36a6c5a8b3ad44b22e2d960529b6"]
                BridgeLogger.info(f"Claim events found: {len(claim_events)}")
                
                # Check if expected call target was invoked
                if expected_call_target:
                    target_logs = [log for log in logs if log.get('address') == expected_call_target]
                    
                    if target_logs:
                        BridgeLogger.success(f"✅ Call target {expected_call_target} was invoked ({len(target_logs)} events)")
                    else:
                        BridgeLogger.warning(f"⚠️ Call target {expected_call_target} was not invoked")
                
                # Check for multiple contract interactions
                unique_addresses = len(set(log.get('address') for log in logs))
                BridgeLogger.info(f"Unique contract addresses in logs: {unique_addresses}")
                
                if unique_addresses > 1:
                    BridgeLogger.success("✅ Multiple contracts involved - likely indicates call execution")
                
                return True
            else:
                BridgeLogger.error(f"Bridge and call claim transaction failed (status: {status})")
                
                # Try to get revert reason
                revert_reason = receipt.get('revertReason')
                if revert_reason:
                    BridgeLogger.error(f"Revert reason: {revert_reason}")
                
                return False
                
        except (subprocess.CalledProcessError, json.JSONDecodeError) as e:
            BridgeLogger.error(f"Could not verify claim transaction: {e}")
            return False
