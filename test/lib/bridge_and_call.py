#!/usr/bin/env python3
"""
Bridge and Call Module - Python Implementation
Functions for bridging assets and executing calls using aggsandbox CLI
"""

import subprocess
import json
import os
from typing import Optional, Tuple
from bridge_lib import BridgeLogger, AggsandboxAPI, BridgeUtils, BRIDGE_CONFIG

class BridgeAndCall:
    """Bridge and call operations"""
    
    @staticmethod
    def bridge_and_call(source_network: int, dest_network: int, amount: int,
                       token_address: str, call_address: str, call_data: str,
                       private_key: str, fallback_address: Optional[str] = None) -> Optional[str]:
        """Bridge and call using aggsandbox CLI"""
        BridgeLogger.step(f"Executing bridge and call from network {source_network} to network {dest_network}")
        BridgeLogger.info(f"Amount: {amount} tokens")
        BridgeLogger.info(f"Token: {token_address}")
        BridgeLogger.info(f"Call address: {call_address}")
        BridgeLogger.info(f"Call data: {call_data[:66]}...")
        BridgeLogger.info(f"Fallback address: {fallback_address or 'auto-detected'}")
        
        cmd = [
            "aggsandbox", "bridge", "call",
            "--network-id", str(source_network),
            "--destination-network", str(dest_network),
            "--amount", str(amount),
            "--token-address", token_address,
            "--call-address", call_address,
            "--call-data", call_data,
            "--private-key", private_key
        ]
        
        if fallback_address:
            cmd.extend(["--fallback-address", fallback_address])
        
        if os.environ.get('DEBUG') == '1':
            cmd.append("--verbose")
        
        BridgeLogger.debug(f"Executing: {' '.join(cmd)}")
        
        success, output = AggsandboxAPI.run_command(cmd)
        if not success:
            BridgeLogger.error(f"Bridge and call transaction failed: {output}")
            return None
        
        tx_hash = BridgeUtils.extract_tx_hash(output)
        if tx_hash:
            BridgeLogger.success(f"Bridge and call transaction initiated: {tx_hash}")
            return tx_hash
        else:
            BridgeLogger.warning("Could not extract transaction hash")
            return None
    
    @staticmethod
    def bridge_and_call_function(source_network: int, dest_network: int, amount: int,
                                token_address: str, call_address: str, 
                                function_signature: str, private_key: str,
                                fallback_address: Optional[str] = None, *args) -> Optional[str]:
        """Bridge and call with function signature encoding"""
        BridgeLogger.step("Executing bridge and call with function encoding")
        BridgeLogger.info(f"Function: {function_signature}")
        BridgeLogger.info(f"Parameters: {args}")
        
        # Encode the function call
        try:
            cmd = ["cast", "abi-encode", function_signature] + list(args)
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            call_data = result.stdout.strip()
            
            BridgeLogger.debug(f"Encoded call data: {call_data}")
            
            return BridgeAndCall.bridge_and_call(source_network, dest_network, amount,
                                               token_address, call_address, call_data,
                                               private_key, fallback_address)
        except subprocess.CalledProcessError as e:
            BridgeLogger.error(f"Failed to encode function call: {e}")
            return None
    
    @staticmethod
    def deploy_bridge_call_receiver(network_id: int, private_key: str,
                                   contract_name: str = "SimpleBridgeAndCallReceiver") -> Optional[str]:
        """Deploy a receiver contract for testing bridge and call"""
        if not BRIDGE_CONFIG:
            BridgeLogger.error("Bridge configuration not initialized")
            return None
        
        BridgeLogger.step(f"Deploying bridge and call receiver contract on network {network_id}")
        BridgeLogger.info(f"Contract: {contract_name}")
        
        rpc_url = BridgeUtils.get_rpc_url(network_id, BRIDGE_CONFIG)
        
        cmd = [
            "forge", "create", f"test/contracts/{contract_name}.sol:{contract_name}",
            "--rpc-url", rpc_url,
            "--private-key", private_key,
            "--json"
        ]
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            deploy_data = json.loads(result.stdout)
            
            contract_address = deploy_data.get('deployedTo')
            if contract_address:
                BridgeLogger.success(f"Contract deployed at: {contract_address}")
                return contract_address
            else:
                BridgeLogger.error("Could not extract contract address from deployment output")
                return None
                
        except (subprocess.CalledProcessError, json.JSONDecodeError) as e:
            BridgeLogger.error(f"Contract deployment failed: {e}")
            return None
    
    @staticmethod
    def verify_bridge_and_call_execution(receiver_contract: str, network_id: int,
                                        expected_message: Optional[str] = None,
                                        expected_amount: Optional[int] = None) -> bool:
        """Verify bridge and call execution by checking receiver contract state"""
        if not BRIDGE_CONFIG:
            return False
        
        BridgeLogger.step("Verifying bridge and call execution")
        BridgeLogger.info(f"Receiver contract: {receiver_contract}")
        BridgeLogger.info(f"Network: {network_id}")
        
        rpc_url = BridgeUtils.get_rpc_url(network_id, BRIDGE_CONFIG)
        
        try:
            # Check call count
            cmd = ["cast", "call", receiver_contract, "getCallCount()", "--rpc-url", rpc_url]
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            call_count_hex = result.stdout.strip()
            call_count = int(call_count_hex, 16) if call_count_hex else 0
            
            BridgeLogger.info(f"Call count: {call_count}")
            
            if call_count > 0:
                BridgeLogger.success(f"✅ Bridge and call was executed ({call_count} calls recorded)")
                
                # Get last message if expected message is provided
                if expected_message:
                    cmd = ["cast", "call", receiver_contract, "getLastMessage()", "--rpc-url", rpc_url]
                    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
                    last_message_hex = result.stdout.strip()
                    
                    if last_message_hex:
                        # Decode the string from hex
                        cmd = ["cast", "abi-decode", "f()(string)", last_message_hex]
                        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
                        decoded_message = result.stdout.strip()
                        
                        BridgeLogger.info(f"Last message received: '{decoded_message}'")
                        
                        if decoded_message == expected_message:
                            BridgeLogger.success("✅ Message matches expected value")
                        else:
                            BridgeLogger.warning("⚠️ Message doesn't match expected value")
                            BridgeLogger.info(f"Expected: '{expected_message}'")
                            BridgeLogger.info(f"Received: '{decoded_message}'")
                
                return True
            else:
                BridgeLogger.error("❌ No calls recorded in receiver contract")
                return False
                
        except (subprocess.CalledProcessError, ValueError) as e:
            BridgeLogger.error(f"Could not verify bridge and call execution: {e}")
            return False
