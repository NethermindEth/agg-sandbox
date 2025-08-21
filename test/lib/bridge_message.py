#!/usr/bin/env python3
"""
Bridge Message Module - Python Implementation
Functions for bridging messages using aggsandbox CLI
"""

import subprocess
import os
from typing import Optional
from bridge_lib import BridgeLogger, BridgeUtils
from aggsandbox_api import AggsandboxAPI

class BridgeMessage:
    """Message bridging operations"""
    
    @staticmethod
    def bridge_message(source_network: int, dest_network: int, to_address: str,
                      message_data: str, private_key: str, 
                      force_update_ger: bool = True) -> Optional[str]:
        """Bridge a message using aggsandbox CLI"""
        BridgeLogger.step(f"Bridging message from network {source_network} to network {dest_network}")
        BridgeLogger.info(f"To address: {to_address}")
        BridgeLogger.info(f"Message data: {message_data[:66]}...")
        
        cmd = [
            "aggsandbox", "bridge", "message",
            "--network", str(source_network),
            "--destination-network", str(dest_network),
            "--to-address", to_address,
            "--message-data", message_data,
            "--private-key", private_key
        ]
        
        if force_update_ger:
            cmd.append("--force-update-global-exit-root")
        
        if os.environ.get('DEBUG') == '1':
            cmd.append("--verbose")
        
        BridgeLogger.debug(f"Executing: {' '.join(cmd)}")
        
        success, output = AggsandboxAPI.bridge_message(
            network=source_network, 
            destination_network=dest_network, 
            target=to_address, 
            data=message_data, 
            private_key=private_key
        )
        if not success:
            BridgeLogger.error(f"Bridge message transaction failed: {output}")
            return None
        
        tx_hash = BridgeUtils.extract_tx_hash(output)
        if tx_hash:
            BridgeLogger.success(f"Bridge message transaction initiated: {tx_hash}")
            return tx_hash
        else:
            BridgeLogger.warning("Could not extract transaction hash")
            return None
    
    @staticmethod
    def bridge_text_message(source_network: int, dest_network: int, to_address: str,
                           text_message: str, private_key: str) -> Optional[str]:
        """Bridge a simple text message"""
        BridgeLogger.step(f"Bridging text message: '{text_message}'")
        
        # Encode the text message as bytes
        try:
            cmd = ["cast", "abi-encode", "f(string)", text_message]
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            message_data = result.stdout.strip()
            
            BridgeLogger.debug(f"Encoded message data: {message_data}")
            
            return BridgeMessage.bridge_message(source_network, dest_network, 
                                              to_address, message_data, private_key)
        except subprocess.CalledProcessError as e:
            BridgeLogger.error(f"Failed to encode text message: {e}")
            return None
    
    @staticmethod
    def bridge_function_call_message(source_network: int, dest_network: int, 
                                   to_address: str, function_signature: str,
                                   private_key: str, *args) -> Optional[str]:
        """Bridge a function call message"""
        BridgeLogger.step("Bridging function call message")
        BridgeLogger.info(f"Function: {function_signature}")
        BridgeLogger.info(f"Parameters: {args}")
        
        # Encode the function call
        try:
            cmd = ["cast", "abi-encode", function_signature] + list(args)
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            message_data = result.stdout.strip()
            
            BridgeLogger.debug(f"Encoded function call: {message_data}")
            
            return BridgeMessage.bridge_message(source_network, dest_network,
                                              to_address, message_data, private_key)
        except subprocess.CalledProcessError as e:
            BridgeLogger.error(f"Failed to encode function call: {e}")
            return None
