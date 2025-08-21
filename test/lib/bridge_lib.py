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
        """Load environment variables from .env file"""
        env_file = ".env"
        if os.path.exists(env_file):
            with open(env_file) as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#') and '=' in line:
                        key, value = line.split('=', 1)
                        os.environ[key] = value
            BridgeLogger.info("Loaded environment variables from .env")
        else:
            raise FileNotFoundError(".env file not found")
        
        # Extract required variables
        try:
            config = BridgeConfig(
                private_key_1=os.environ['PRIVATE_KEY_1'],
                private_key_2=os.environ['PRIVATE_KEY_2'],
                account_address_1=os.environ['ACCOUNT_ADDRESS_1'],
                account_address_2=os.environ['ACCOUNT_ADDRESS_2'],
                rpc_1=os.environ['RPC_1'],
                rpc_2=os.environ['RPC_2'],
                network_id_mainnet=int(os.environ.get('NETWORK_ID_MAINNET', '0')),
                network_id_agglayer_1=int(os.environ.get('NETWORK_ID_AGGLAYER_1', '1')),
                chain_id_agglayer_1=int(os.environ['CHAIN_ID_AGGLAYER_1']),
                agg_erc20_l1=os.environ['AGG_ERC20_L1'],
                agg_erc20_l2=os.environ.get('AGG_ERC20_L2')
            )
            BridgeLogger.success("All required environment variables loaded")
            return config
        except KeyError as e:
            raise ValueError(f"Missing required environment variable: {e}")
    
    @staticmethod
    def validate_sandbox_status() -> bool:
        """Validate that aggsandbox is running"""
        BridgeLogger.step("Validating sandbox status")
        
        # Check if aggsandbox command exists
        try:
            subprocess.run(['aggsandbox', '--version'], 
                         capture_output=True, check=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            BridgeLogger.error("aggsandbox CLI not found")
            return False
        
        # Check if sandbox is running
        try:
            subprocess.run(['aggsandbox', 'status', '--quiet'], 
                         capture_output=True, check=True)
            BridgeLogger.success("Sandbox is running and accessible")
            return True
        except subprocess.CalledProcessError:
            BridgeLogger.error("Sandbox is not running. Start with: aggsandbox start --detach")
            return False

class AggsandboxAPI:
    """Interface to aggsandbox CLI commands"""
    
    @staticmethod
    def run_command(cmd: List[str]) -> Tuple[bool, str]:
        """Run a command and return (success, output)"""
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            return True, result.stdout.strip()
        except subprocess.CalledProcessError as e:
            return False, e.stderr.strip()
    
    @staticmethod
    def get_bridges(network_id: int) -> Optional[Dict[str, Any]]:
        """Get bridge information for a network"""
        cmd = ["aggsandbox", "show", "bridges", "--network-id", str(network_id), "--json"]
        success, output = AggsandboxAPI.run_command(cmd)
        
        if not success:
            BridgeLogger.error(f"Could not get bridges: {output}")
            return None
        
        try:
            return json.loads(output)
        except json.JSONDecodeError as e:
            BridgeLogger.error(f"Could not parse bridge JSON: {e}")
            return None
    
    @staticmethod
    def get_claims(network_id: int) -> Optional[Dict[str, Any]]:
        """Get claims information for a network"""
        cmd = ["aggsandbox", "show", "claims", "--network-id", str(network_id), "--json"]
        success, output = AggsandboxAPI.run_command(cmd)
        
        if not success:
            BridgeLogger.error(f"Could not get claims: {output}")
            return None
        
        try:
            return json.loads(output)
        except json.JSONDecodeError as e:
            BridgeLogger.error(f"Could not parse claims JSON: {e}")
            return None

class BridgeUtils:
    """Utility functions for bridge operations"""
    
    @staticmethod
    def extract_tx_hash(output: str) -> Optional[str]:
        """Extract transaction hash from aggsandbox output"""
        lines = output.split('\n')
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
        """Get token balance using cast"""
        rpc_url = BridgeUtils.get_rpc_url(network_id, config)
        
        if token_address == "0x0000000000000000000000000000000000000000":
            # ETH balance
            cmd = ["cast", "balance", account_address, "--rpc-url", rpc_url]
        else:
            # ERC20 balance
            cmd = ["cast", "call", token_address, "balanceOf(address)", 
                   account_address, "--rpc-url", rpc_url]
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            hex_balance = result.stdout.strip()
            
            # Convert to decimal
            if hex_balance and hex_balance != "0x":
                return int(hex_balance, 16)
            return 0
        except subprocess.CalledProcessError:
            return 0

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
    'AggsandboxAPI', 'BridgeUtils', 'BRIDGE_CONFIG'
]
