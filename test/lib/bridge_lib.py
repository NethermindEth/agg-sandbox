#!/usr/bin/env python3
"""
Bridge Test Library - Python Implementation
Modular bridge testing library for Agglayer Nether Sandbox
Version: 1.0 - Python implementation with clean architecture
"""

import subprocess
import json
import time
import os
import sys
from typing import Optional, Dict, Any, List, Tuple
from dataclasses import dataclass
from enum import Enum

# Import AggsandboxAPI
try:
    from aggsandbox_api import AggsandboxAPI
except ImportError:
    # If running as a script, add current directory to path
    sys.path.append(os.path.dirname(os.path.abspath(__file__)))
    from aggsandbox_api import AggsandboxAPI

class NetworkID(Enum):
    """Network identifiers"""
    MAINNET = 0
    AGGLAYER_1 = 1
    AGGLAYER_2 = 2

@dataclass
class BridgeConfig:
    """Bridge configuration from environment"""
    private_key_1: str
    private_key_2: str
    account_address_1: str
    account_address_2: str
    rpc_1: str
    rpc_2: str
    network_id_mainnet: int
    network_id_agglayer_1: int
    chain_id_agglayer_1: int
    agg_erc20_l1: str
    # Optional fields (must come after required fields)
    rpc_3: Optional[str] = None  # L3 RPC (L2-2 in multi-L2 mode)
    network_id_agglayer_2: Optional[int] = None  # L3 network ID
    chain_id_agglayer_2: Optional[int] = None  # L3 chain ID
    agg_erc20_l2: Optional[str] = None
    agg_erc20_l3: Optional[str] = None  # L3 AggERC20 contract
    asset_and_call_receiver_l2: Optional[str] = None  # Bridge-and-call receiver contract

class BridgeLogger:
    """Colored logging for bridge operations"""
    
    # ANSI color codes
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    RED = '\033[0;31m'
    BLUE = '\033[0;34m'
    CYAN = '\033[0;36m'
    NC = '\033[0m'  # No Color
    
    @classmethod
    def step(cls, msg: str):
        print(f"{cls.GREEN}[STEP]{cls.NC} {msg}")
    
    @classmethod
    def info(cls, msg: str):
        print(f"{cls.YELLOW}[INFO]{cls.NC} {msg}")
    
    @classmethod
    def success(cls, msg: str):
        print(f"{cls.GREEN}[SUCCESS]{cls.NC} {msg}")
    
    @classmethod
    def error(cls, msg: str):
        print(f"{cls.RED}[ERROR]{cls.NC} {msg}")
    
    @classmethod
    def warning(cls, msg: str):
        print(f"{cls.CYAN}[WARNING]{cls.NC} {msg}")
    
    @classmethod
    def debug(cls, msg: str):
        if os.environ.get('DEBUG') == '1':
            print(f"{cls.BLUE}[DEBUG]{cls.NC} {msg}")

class BridgeEnvironment:
    """Environment management for bridge testing"""
    
    @staticmethod
    def load_environment() -> BridgeConfig:
        """Load environment configuration from aggsandbox info and .env file"""
        BridgeLogger.step("Loading environment from aggsandbox info")
        
        # Load .env file if it exists
        BridgeEnvironment._load_env_file()
        
        # Get sandbox info
        success, info_output = AggsandboxAPI.info()
        if not success:
            raise RuntimeError(f"Failed to get sandbox info: {info_output}")
        
        # Parse the info output to extract configuration
        config_data = BridgeEnvironment._parse_sandbox_info(info_output)
        
        config = BridgeConfig(
            private_key_1=config_data['private_keys'][0],  # Account (0)
            private_key_2=config_data['private_keys'][1],  # Account (1) 
            account_address_1=config_data['accounts'][0],  # Account (0)
            account_address_2=config_data['accounts'][1],  # Account (1)
            rpc_1=config_data['l1_rpc'],                   # L1 RPC
            rpc_2=config_data['l2_rpc'],                   # L2 RPC
            rpc_3=config_data.get('l3_rpc'),               # L3 RPC (if multi-L2)
            network_id_mainnet=0,                          # L1 network ID
            network_id_agglayer_1=1,                       # L2 network ID
            network_id_agglayer_2=config_data.get('network_id_agglayer_2'), # L3 network ID
            chain_id_agglayer_1=config_data['l2_chain_id'],# L2 chain ID
            chain_id_agglayer_2=config_data.get('l3_chain_id'), # L3 chain ID
            agg_erc20_l1=config_data['agg_erc20_l1'],     # L1 AggERC20 contract
            agg_erc20_l2=config_data['agg_erc20_l2'],     # L2 AggERC20 contract
            agg_erc20_l3=config_data.get('agg_erc20_l3'), # L3 AggERC20 contract
            asset_and_call_receiver_l2=os.environ.get('ASSET_AND_CALL_RECEIVER_L2')  # Bridge-and-call receiver
        )
        
        BridgeLogger.success("Environment configuration loaded from aggsandbox")
        BridgeLogger.info(f"L1 RPC: {config.rpc_1}")
        BridgeLogger.info(f"L2 RPC: {config.rpc_2}")
        if config.rpc_3:
            BridgeLogger.info(f"L3 RPC: {config.rpc_3}")
        BridgeLogger.info(f"Account 1: {config.account_address_1}")
        BridgeLogger.info(f"Account 2: {config.account_address_2}")
        if config.network_id_agglayer_2:
            BridgeLogger.debug(f"Multi-L2 mode detected: L3 Network ID {config.network_id_agglayer_2}")
        
        return config
    
    @staticmethod
    def _parse_sandbox_info(info_output: str) -> Dict[str, Any]:
        """Parse aggsandbox info output to extract configuration"""
        lines = info_output.split('\n')
        
        accounts = []
        private_keys = []
        l1_rpc = None
        l2_rpc = None
        l3_rpc = None
        l2_chain_id = None
        l3_chain_id = None
        agg_erc20_l1 = None
        agg_erc20_l2 = None
        agg_erc20_l3 = None
        is_multi_l2 = False
        
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            
            # Parse Available Accounts section
            if line == "Available Accounts":
                i += 2  # Skip separator line
                while i < len(lines) and lines[i].strip().startswith('('):
                    account_line = lines[i].strip()
                    if ': 0x' in account_line:
                        account_addr = account_line.split(': ')[1]
                        accounts.append(account_addr)
                    i += 1
                continue
            
            # Parse Private Keys section
            elif line == "Private Keys":
                i += 2  # Skip separator line
                while i < len(lines) and lines[i].strip().startswith('('):
                    key_line = lines[i].strip()
                    if ': 0x' in key_line:
                        private_key = key_line.split(': ')[1]
                        private_keys.append(private_key)
                    i += 1
                continue
            
            # Parse L1 RPC (looking for Chain ID: 1)
            elif 'Chain ID: 1    RPC:' in line:
                rpc_part = line.split('RPC: ')[1]
                l1_rpc = rpc_part.strip()
            
            # Parse L2 RPC and Chain ID (looking for Chain ID: 1101)
            elif 'Chain ID: 1101    RPC:' in line:
                chain_id_part = line.split('Chain ID: ')[1].split('RPC:')[0].strip()
                l2_chain_id = int(chain_id_part)
                rpc_part = line.split('RPC: ')[1]
                l2_rpc = rpc_part.strip()
            
            # Parse L3 RPC and Chain ID (looking for Chain ID: 137 - L2-2)
            elif 'Chain ID: 137    RPC:' in line:
                chain_id_part = line.split('Chain ID: ')[1].split('RPC:')[0].strip()
                l3_chain_id = int(chain_id_part)
                rpc_part = line.split('RPC: ')[1]
                l3_rpc = rpc_part.strip()
                is_multi_l2 = True
            
            # Detect multi-L2 mode
            elif 'Multi-L2 Polygon Sandbox Config:' in line or 'L2-2 (' in line:
                is_multi_l2 = True
            
            # Parse AggERC20 contracts
            elif 'AggERC20:' in line:
                contract_addr = line.split('AggERC20: ')[1].strip()
                if agg_erc20_l1 is None:  # First occurrence is L1
                    agg_erc20_l1 = contract_addr
                elif agg_erc20_l2 is None:  # Second occurrence is L2
                    agg_erc20_l2 = contract_addr
                elif agg_erc20_l3 is None and is_multi_l2:  # Third occurrence is L3 (if multi-L2)
                    agg_erc20_l3 = contract_addr
            
            i += 1
        
        # Add manual L3 configuration if multi-L2 mode detected but L3 info missing
        if is_multi_l2:
            if not l3_rpc:
                l3_rpc = "http://localhost:8547"  # Standard L3 RPC port
                BridgeLogger.debug("Added manual L3 RPC configuration")
            if not l3_chain_id:
                l3_chain_id = 137  # Standard L3 chain ID for Agglayer-2
                BridgeLogger.debug("Added manual L3 chain ID configuration")
            if not agg_erc20_l3:
                agg_erc20_l3 = agg_erc20_l2  # L3 uses same AggERC20 address as L2
                BridgeLogger.debug("Added manual L3 AggERC20 configuration")
        
        # Validate we got all required data
        if not accounts or len(accounts) < 2:
            raise ValueError("Could not parse accounts from sandbox info")
        if not private_keys or len(private_keys) < 2:
            raise ValueError("Could not parse private keys from sandbox info")
        if not l1_rpc or not l2_rpc:
            raise ValueError("Could not parse RPC URLs from sandbox info")
        if not l2_chain_id:
            raise ValueError("Could not parse L2 chain ID from sandbox info")
        if not agg_erc20_l1 or not agg_erc20_l2:
            raise ValueError("Could not parse AggERC20 contract addresses from sandbox info")
        
        return {
            'accounts': accounts,
            'private_keys': private_keys,
            'l1_rpc': l1_rpc,
            'l2_rpc': l2_rpc,
            'l3_rpc': l3_rpc,
            'l2_chain_id': l2_chain_id,
            'l3_chain_id': l3_chain_id,
            'network_id_agglayer_2': 2 if is_multi_l2 else None,
            'agg_erc20_l1': agg_erc20_l1,
            'agg_erc20_l2': agg_erc20_l2,
            'agg_erc20_l3': agg_erc20_l3
        }
    
    @staticmethod
    def _load_env_file():
        """Load .env file variables into environment"""
        env_path = os.path.join(os.path.dirname(__file__), '..', '..', '.env')
        if os.path.exists(env_path):
            BridgeLogger.debug(f"Loading .env file from {env_path}")
            with open(env_path, 'r') as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#') and '=' in line:
                        key, value = line.split('=', 1)
                        os.environ[key.strip()] = value.strip()
            BridgeLogger.debug("✅ .env file loaded successfully")
        else:
            BridgeLogger.debug("No .env file found")
    
    @staticmethod
    def validate_sandbox_status() -> bool:
        """Validate that aggsandbox is running"""
        BridgeLogger.step("Validating sandbox status")
        
        # Check if sandbox is running using AggsandboxAPI
        success, output = AggsandboxAPI.status(quiet=True)
        if success:
            BridgeLogger.success("Sandbox is running and accessible")
            return True
        else:
            BridgeLogger.error("Sandbox is not running. Start with: aggsandbox start --detach")
            BridgeLogger.error(f"Status check failed: {output}")
            return False

# AggsandboxAPI is now in aggsandbox_api.py - import from there

class BridgeUtils:
    """Utility functions for bridge operations"""
    
    @staticmethod
    def extract_tx_hash(output: str) -> Optional[str]:
        """Extract transaction hash from aggsandbox output"""
        lines = output.split('\n')
        
        # Look for claim transaction submitted (for claim operations)
        for line in lines:
            if '✅ claim transaction submitted:' in line.lower() and '0x' in line:
                words = line.split()
                for word in words:
                    if word.startswith('0x') and len(word) == 66:
                        return word
        
        # Look for bridge message transaction (for message bridges)
        for line in lines:
            line_clean = line.lower().strip()
            if 'bridge message transaction submitted' in line_clean and '0x' in line:
                words = line.split()
                for word in words:
                    if word.startswith('0x') and len(word) == 66:
                        return word
        
        # Look for bridge and call transaction (for bridge-and-call operations)
        for line in lines:
            if 'bridge and call transaction submitted' in line.lower() and '0x' in line:
                words = line.split()
                for word in words:
                    if word.startswith('0x') and len(word) == 66:
                        return word
        
        # Look for the bridge transaction specifically (not approval)
        for line in lines:
            if 'bridge transaction submitted' in line.lower() and '0x' in line:
                words = line.split()
                for word in words:
                    if word.startswith('0x') and len(word) == 66:
                        return word
        
        # Fallback: look for any transaction hash
        for line in lines:
            if 'transaction' in line.lower() and '0x' in line:
                words = line.split()
                for word in words:
                    if word.startswith('0x') and len(word) == 66:
                        return word
        
        return None
    
    @staticmethod
    def get_rpc_url(network_id: int, config: BridgeConfig) -> str:
        """Get RPC URL for a network"""
        if network_id == config.network_id_mainnet:
            return config.rpc_1
        elif network_id == config.network_id_agglayer_1:
            return config.rpc_2
        elif network_id == config.network_id_agglayer_2 and config.rpc_3:
            return config.rpc_3
        elif network_id == 2:  # Fallback for L2-2 (Agglayer-2) in multi-L2 mode
            return "http://localhost:8547"
        else:
            raise ValueError(f"Unknown network ID: {network_id}")
    
    @staticmethod
    def get_token_balance(token_address: str, account_address: str, 
                         network_id: int, config: BridgeConfig) -> int:
        """Get token balance using aggsandbox CLI (placeholder - would need aggsandbox balance command)"""
        # Note: aggsandbox doesn't have a direct balance command yet
        # This is a placeholder that would use aggsandbox when available
        # For now, we'll indicate this limitation
        BridgeLogger.debug(f"Balance check needed for {account_address} on network {network_id}")
        BridgeLogger.debug("Note: aggsandbox CLI doesn't have balance command yet")
        return 0  # Placeholder return
    
    @staticmethod
    def get_bridge_tx_hash(bridge: dict) -> str:
        """Get transaction hash from bridge object, handling both old and new field names"""
        return bridge.get('bridge_tx_hash') or bridge.get('bridge_tx_hash')
    
    @staticmethod
    def find_bridge_by_tx_hash(bridges: list, tx_hash: str) -> dict:
        """Find bridge in list by transaction hash, handling both old and new field names"""
        for bridge in bridges:
            if bridge.get('bridge_tx_hash') == tx_hash:
                return bridge
        return None

# Initialize global configuration
try:
    BRIDGE_CONFIG = BridgeEnvironment.load_environment()
    BridgeLogger.debug("Bridge library initialized successfully")
except Exception as e:
    BridgeLogger.error(f"Failed to initialize bridge library: {e}")
    BRIDGE_CONFIG = None

# Export main classes for use in other modules
__all__ = [
    'NetworkID', 'BridgeConfig', 'BridgeLogger', 'BridgeEnvironment',
    'BridgeUtils', 'BRIDGE_CONFIG'
]
