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
from bridge_lib import BridgeLogger, BridgeUtils, BRIDGE_CONFIG
from aggsandbox_api import AggsandboxAPI, BridgeClaimArgs

class ClaimAsset:
    """Asset claiming operations"""
    
    @staticmethod
    def claim_asset(dest_network: int, tx_hash: str, source_network: int,
                   private_key: str, deposit_count: Optional[int] = None) -> Optional[str]:
        """Claim bridged assets using aggsandbox CLI"""
        BridgeLogger.step(f"Claiming bridged assets on network {dest_network}")
        BridgeLogger.info(f"Source transaction: {tx_hash}")
        BridgeLogger.info(f"Source network: {source_network}")
        
        # Use the new aggsandbox API
        args = BridgeClaimArgs(
            network=dest_network,
            tx_hash=tx_hash,
            source_network=source_network,
            private_key=private_key,
            deposit_count=deposit_count
        )
        
        success, output = AggsandboxAPI.bridge_claim(args)
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
    def verify_claim_status(network_id: int, bridge_tx_hash: str, deposit_count: int) -> Optional[str]:
        """Verify claim status using aggsandbox show claims --network-id --json"""
        BridgeLogger.step(f"Verifying claim status on network {network_id}")
        BridgeLogger.info(f"Looking for claim of bridge TX: {bridge_tx_hash}")
        BridgeLogger.info(f"Deposit count: {deposit_count}")
        
        # Get claims data using aggsandbox API
        claims_data = AggsandboxAPI.get_claims(network_id)
        if not claims_data:
            BridgeLogger.error("Could not get claims data")
            return None
        
        claims = claims_data.get('claims', [])
        BridgeLogger.info(f"Found {len(claims)} total claims on network {network_id}")
        
        # Look for our specific claim by deposit count or other identifiers
        our_claim = None
        for claim in claims:
            # Match by deposit count (most reliable identifier)
            if claim.get('deposit_count') == deposit_count:
                our_claim = claim
                break
            # Alternative: match by bridge transaction hash if available
            elif claim.get('bridge_tx_hash') == bridge_tx_hash:
                our_claim = claim
                break
        
        if our_claim:
            claim_status = our_claim.get('status', 'unknown')
            claim_amount = our_claim.get('amount', '0')
            claim_block = our_claim.get('block_num', 'unknown')
            
            BridgeLogger.success(f"Found our claim:")
            BridgeLogger.info(f"  Status: {claim_status}")
            BridgeLogger.info(f"  Amount: {claim_amount} tokens")
            BridgeLogger.info(f"  Block: {claim_block}")
            BridgeLogger.info(f"  Deposit Count: {deposit_count}")
            
            if claim_status == 'complete' or claim_status == 'completed':
                BridgeLogger.success("✅ Claim is complete!")
                return "complete"
            elif claim_status == 'pending':
                BridgeLogger.warning("⚠️ Claim is still pending")
                return "pending"
            else:
                BridgeLogger.info(f"Claim status: {claim_status}")
                return claim_status
        else:
            BridgeLogger.warning(f"Our claim not found (deposit_count: {deposit_count})")
            BridgeLogger.info("This might mean the claim hasn't been processed yet")
            return "not_found"
