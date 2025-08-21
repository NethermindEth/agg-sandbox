#!/usr/bin/env python3
"""
Claim Message Module - Python Implementation
Functions for claiming bridged messages using aggsandbox CLI
"""

import time
import os
import subprocess
from typing import Optional
from bridge_lib import BridgeLogger, BridgeUtils
from aggsandbox_api import AggsandboxAPI, BridgeClaimArgs

class ClaimMessage:
    """Message claiming operations"""
    
    @staticmethod
    def claim_message(dest_network: int, tx_hash: str, source_network: int,
                     private_key: str, deposit_count: Optional[int] = None) -> Optional[str]:
        """Claim bridged messages using aggsandbox CLI"""
        BridgeLogger.step(f"Claiming bridged message on network {dest_network}")
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
        
        # Create claim args
        claim_args = BridgeClaimArgs(
            network=dest_network,
            tx_hash=tx_hash,
            source_network=source_network,
            private_key=private_key,
            deposit_count=deposit_count
        )
        
        success, output = AggsandboxAPI.bridge_claim(claim_args)
        if not success:
            BridgeLogger.error(f"Claim message transaction failed: {output}")
            
            # Check for common error patterns
            if "AlreadyClaimed" in output:
                BridgeLogger.warning("Message was already claimed")
                return "already_claimed"
            elif "GlobalExitRootInvalid" in output:
                BridgeLogger.warning("Global exit root invalid - may need to wait longer")
                return None
            
            return None
        
        tx_hash = BridgeUtils.extract_tx_hash(output)
        if tx_hash:
            BridgeLogger.success(f"Claim message transaction completed: {tx_hash}")
            return tx_hash
        else:
            BridgeLogger.success("Claim message transaction completed successfully")
            return "completed"
    
    @staticmethod
    def claim_message_with_retry(dest_network: int, tx_hash: str, source_network: int,
                                private_key: str, deposit_count: Optional[int] = None,
                                max_retries: int = 3, retry_delay: int = 10) -> Optional[str]:
        """Claim message with retry logic"""
        BridgeLogger.step(f"Claiming message with retry logic (max {max_retries} attempts)")
        
        for attempt in range(1, max_retries + 1):
            BridgeLogger.info(f"Claim attempt {attempt}/{max_retries}")
            
            result = ClaimMessage.claim_message(dest_network, tx_hash, source_network,
                                              private_key, deposit_count)
            
            if result == "already_claimed":
                BridgeLogger.info("Message was already claimed")
                return result
            elif result:
                BridgeLogger.success(f"Message claimed successfully on attempt {attempt}")
                return result
            else:
                if attempt < max_retries:
                    BridgeLogger.warning(f"Claim failed, retrying in {retry_delay}s...")
                    time.sleep(retry_delay)
                else:
                    BridgeLogger.error("Max retries reached, claim failed")
        
        return None
    
    @staticmethod
    def decode_message_data(message_data: str, function_signature: Optional[str] = None) -> Optional[str]:
        """Decode message data for debugging"""
        BridgeLogger.step("Decoding message data")
        BridgeLogger.info(f"Raw data: {message_data}")
        
        if function_signature:
            # Try to decode with provided function signature
            try:
                cmd = ["cast", "abi-decode", function_signature, message_data]
                result = subprocess.run(cmd, capture_output=True, text=True, check=True)
                decoded = result.stdout.strip()
                BridgeLogger.success(f"Decoded with signature '{function_signature}': {decoded}")
                return decoded
            except subprocess.CalledProcessError:
                BridgeLogger.warning("Could not decode with provided signature")
        
        # Try common decodings
        decodings = [
            ("string", "f(string)"),
            ("bytes", "f(bytes)")
        ]
        
        for desc, sig in decodings:
            try:
                cmd = ["cast", "abi-decode", sig, message_data]
                result = subprocess.run(cmd, capture_output=True, text=True, check=True)
                decoded = result.stdout.strip()
                BridgeLogger.info(f"As {desc}: {decoded}")
            except subprocess.CalledProcessError:
                pass
        
        BridgeLogger.info(f"Hex data: {message_data}")
        return None
