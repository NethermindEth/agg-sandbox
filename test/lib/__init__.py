#!/usr/bin/env python3
"""
Bridge Test Library - Python Package
Modular bridge testing library for Agglayer Nether Sandbox
"""

from .bridge_lib import (
    NetworkID, BridgeConfig, BridgeLogger, BridgeEnvironment,
    AggsandboxAPI, BridgeUtils, BRIDGE_CONFIG
)
from .bridge_asset import BridgeAsset
from .bridge_message import BridgeMessage
from .claim_asset import ClaimAsset
from .claim_message import ClaimMessage
from .bridge_and_call import BridgeAndCall
from .claim_bridge_and_call import ClaimBridgeAndCall

# Initialize environment on import
def init_bridge_environment():
    """Initialize bridge test environment"""
    BridgeLogger.step("Initializing bridge test environment")
    
    try:
        # Load environment variables
        config = BridgeEnvironment.load_environment()
        
        # Validate sandbox status
        if not BridgeEnvironment.validate_sandbox_status():
            return False
        
        # Print configuration if verbose/debug
        if os.environ.get('VERBOSE') == '1' or os.environ.get('DEBUG') == '1':
            print_test_config(config)
        
        BridgeLogger.success("Bridge test environment initialized successfully")
        return True
        
    except Exception as e:
        BridgeLogger.error(f"Failed to initialize environment: {e}")
        return False

def print_test_config(config: BridgeConfig):
    """Print test configuration"""
    print("")
    BridgeLogger.info("========== TEST CONFIGURATION ==========")
    BridgeLogger.info("Networks:")
    BridgeLogger.info(f"  • L1 (Network {config.network_id_mainnet}): {config.rpc_1}")
    BridgeLogger.info(f"  • L2 (Network {config.network_id_agglayer_1}): {config.rpc_2}")
    BridgeLogger.info("Accounts:")
    BridgeLogger.info(f"  • Account 1: {config.account_address_1}")
    BridgeLogger.info(f"  • Account 2: {config.account_address_2}")
    BridgeLogger.info("Tokens:")
    BridgeLogger.info(f"  • L1 Token: {config.agg_erc20_l1}")
    BridgeLogger.info(f"  • L2 Token: {config.agg_erc20_l2 or 'Not deployed'}")
    BridgeLogger.info("========================================")
    print("")

def print_bridge_summary(bridge_tx_hash: str, claim_tx_hash: str, 
                        amount: int, token_address: str):
    """Print summary of bridge operation"""
    print("")
    BridgeLogger.info("========== BRIDGE OPERATION SUMMARY ==========")
    BridgeLogger.success("Bridge Transaction:")
    BridgeLogger.info(f"  ✅ Amount: {amount} tokens")
    BridgeLogger.info(f"  ✅ Token: {token_address}")
    BridgeLogger.info(f"  ✅ Bridge TX: {bridge_tx_hash}")
    
    if claim_tx_hash and claim_tx_hash != "N/A":
        BridgeLogger.success("Claim Transaction:")
        BridgeLogger.info(f"  ✅ Claim TX: {claim_tx_hash}")
    else:
        BridgeLogger.info("  ⚠️  Claim transaction: Auto-executed or not required")
    
    BridgeLogger.info("==============================================")
    print("")

# Version info
__version__ = "1.0.0"
__author__ = "Agglayer Team"

# Export all main classes and functions
__all__ = [
    # Core classes
    'NetworkID', 'BridgeConfig', 'BridgeLogger', 'BridgeEnvironment',
    'AggsandboxAPI', 'BridgeUtils', 'BRIDGE_CONFIG',
    
    # Operation classes
    'BridgeAsset', 'BridgeMessage', 'ClaimAsset', 'ClaimMessage',
    'BridgeAndCall', 'ClaimBridgeAndCall',
    
    # Utility functions
    'init_bridge_environment', 'print_test_config', 'print_bridge_summary'
]
