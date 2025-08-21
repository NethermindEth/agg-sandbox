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
    agg_erc20_l2: Optional[str] = None

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
        """Load environment configuration from aggsandbox info"""
        BridgeLogger.step("Loading environment from aggsandbox info")
        
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
            network_id_mainnet=0,                          # L1 network ID
            network_id_agglayer_1=1,                       # L2 network ID
            chain_id_agglayer_1=config_data['l2_chain_id'],# L2 chain ID
            agg_erc20_l1=config_data['agg_erc20_l1'],     # L1 AggERC20 contract
            agg_erc20_l2=config_data['agg_erc20_l2']      # L2 AggERC20 contract
        )
        
        BridgeLogger.success("Environment configuration loaded from aggsandbox")
        BridgeLogger.info(f"L1 RPC: {config.rpc_1}")
        BridgeLogger.info(f"L2 RPC: {config.rpc_2}")
        BridgeLogger.info(f"Account 1: {config.account_address_1}")
        BridgeLogger.info(f"Account 2: {config.account_address_2}")
        
        return config
    
    @staticmethod
    def _parse_sandbox_info(info_output: str) -> Dict[str, Any]:
        """Parse aggsandbox info output to extract configuration"""
        lines = info_output.split('\n')
        
        accounts = []
        private_keys = []
        l1_rpc = None
        l2_rpc = None
        l2_chain_id = None
        agg_erc20_l1 = None
        agg_erc20_l2 = None
        
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
            
            # Parse AggERC20 contracts
            elif 'AggERC20:' in line:
                contract_addr = line.split('AggERC20: ')[1].strip()
                if agg_erc20_l1 is None:  # First occurrence is L1
                    agg_erc20_l1 = contract_addr
                else:  # Second occurrence is L2
                    agg_erc20_l2 = contract_addr
            
            i += 1
        
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
            'l2_chain_id': l2_chain_id,
            'agg_erc20_l1': agg_erc20_l1,
            'agg_erc20_l2': agg_erc20_l2
        }
    
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
