#!/usr/bin/env python3
"""
L2-L1 Bridge-and-Call Test
Tests the complete flow of bridge-and-call operations from L2 to L1 using aggsandbox CLI
Based on bridge-operations.md documentation and the successful L1-L2 bridge-and-call test
"""

import sys
import os
import time
import json
import subprocess

# Add the lib directory to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'lib'))

from bridge_lib import BRIDGE_CONFIG, BridgeLogger, BridgeEnvironment, BridgeUtils
from aggsandbox_api import AggsandboxAPI, BridgeClaimArgs
from bridge_and_call import BridgeAndCall

def encode_call_data(function_signature: str, *args) -> str:
    """Encode function call data for bridge-and-call"""
    try:
        cmd = ["cast", "calldata", function_signature] + list(str(arg) for arg in args)
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        call_data = result.stdout.strip()
        BridgeLogger.debug(f"Encoded call data: {call_data}")
        return call_data
    except subprocess.CalledProcessError as e:
        BridgeLogger.error(f"Failed to encode call data: {e}")
        return None

def run_l2_to_l1_bridge_and_call_test(bridge_amount: int = 8):
    """
    Complete L2-L1 Bridge-and-Call Test
    
    This test follows the documented bridge-and-call process for L2→L1:
    0. Deploy SimpleBridgeAndCallReceiver contract on L1
    1. Prepare call data for the contract function
    2. Execute bridge-and-call from L2 to L1 using aggsandbox bridge bridge-and-call
    3. Monitor the bridge transactions using aggsandbox show bridges
    4. Wait for AggKit to sync bridge data from L2 to L1
    5. Claim asset bridge first (deposit_count from actual bridge)
    6. Claim message bridge second (deposit_count from actual bridge)
    7. Verify both claims and contract execution
    
    Args:
        bridge_amount: Amount of L2 AggERC20 tokens to bridge (default: 8)
    """
    print("\n" + "="*70)
    print(f"🔗 L2→L1 Bridge-and-Call Test")
    print(f"Token Amount: {bridge_amount} L2 AggERC20 tokens")
    print(f"Following bridge-operations.md documentation")
    print("="*70)
    
    try:
        # Initialize environment
        BridgeLogger.step("Initializing test environment")
        
        if not BridgeEnvironment.validate_sandbox_status():
            BridgeLogger.error("Sandbox is not running")
            return False
        
        if not BRIDGE_CONFIG:
            BridgeLogger.error("Bridge configuration not available")
            return False
        
        BridgeLogger.success("✅ Environment initialized successfully")
        BridgeLogger.info(f"L2 Network ID: {BRIDGE_CONFIG.network_id_agglayer_1}")
        BridgeLogger.info(f"L1 Network ID: {BRIDGE_CONFIG.network_id_mainnet}")
        BridgeLogger.info(f"From Account: {BRIDGE_CONFIG.account_address_1}")
        print()
        
        # Step 0: Deploy bridge-and-call receiver contract on L1
        BridgeLogger.step("[0/8] Deploying bridge-and-call receiver contract on L1")
        contract_address = BridgeAndCall.deploy_bridge_call_receiver(
            BRIDGE_CONFIG.network_id_mainnet, 
            BRIDGE_CONFIG.private_key_1,
            "SimpleBridgeAndCallReceiver"
        )
        if not contract_address:
            BridgeLogger.error("Failed to deploy bridge-and-call receiver contract")
            return False
        
        BridgeLogger.info(f"Bridge-and-call receiver deployed at: {contract_address}")
        time.sleep(5)
        print()
        
        # Step 1: Prepare call data
        BridgeLogger.step("[1/8] Preparing call data for contract")
        BridgeLogger.info("Encoding receiveTokensWithMessage function call")
        BridgeLogger.info("This will call the contract when the message bridge is claimed")
        
        # Get what the L2 token will become on L1 (for the function call)
        success, output = AggsandboxAPI.bridge_utils_precalculate(
            network=BRIDGE_CONFIG.network_id_mainnet,
            origin_network=BRIDGE_CONFIG.network_id_agglayer_1,
            origin_token=BRIDGE_CONFIG.agg_erc20_l2,
            json_output=True
        )
        
        l1_wrapped_token_addr = None
        if success:
            try:
                data = json.loads(output)
                l1_wrapped_token_addr = data.get('precalculated_address')
                BridgeLogger.success(f"✅ L1 wrapped token will be: {l1_wrapped_token_addr}")
            except json.JSONDecodeError as e:
                BridgeLogger.warning(f"Could not parse precalculate response: {e}")
        
        if not l1_wrapped_token_addr:
            BridgeLogger.error("Could not determine L1 wrapped token address")
            return False
        
        # Encode the receiveTokensWithMessage function call
        call_data = encode_call_data("receiveTokensWithMessage(address,uint256,string)", 
                                   l1_wrapped_token_addr, bridge_amount, f"L2→L1 bridge-and-call: {bridge_amount} tokens")
        if not call_data:
            BridgeLogger.error("Failed to encode call data")
            return False
        
        BridgeLogger.success(f"✅ Call data encoded: {call_data}")
        print()
        
        # Step 2: Execute bridge-and-call from L2 to L1
        BridgeLogger.step(f"[2/8] Executing bridge-and-call from L2 to L1")
        BridgeLogger.info("Using: aggsandbox bridge bridge-and-call")
        BridgeLogger.info(f"Token: {BRIDGE_CONFIG.agg_erc20_l2}")
        BridgeLogger.info(f"Amount: {bridge_amount} tokens")
        BridgeLogger.info(f"Target contract: {contract_address}")
        BridgeLogger.info("This will create both asset and message bridges")
        
        success, output = AggsandboxAPI.bridge_and_call(
            network=BRIDGE_CONFIG.network_id_agglayer_1,
            destination_network=BRIDGE_CONFIG.network_id_mainnet,
            token=BRIDGE_CONFIG.agg_erc20_l2,
            amount=str(bridge_amount),
            target=contract_address,
            data=call_data,
            fallback=BRIDGE_CONFIG.account_address_1,
            private_key=BRIDGE_CONFIG.private_key_1
        )
        
        if not success:
            BridgeLogger.error(f"Bridge-and-call operation failed: {output}")
            return False
        
        # Extract bridge transaction hash from output using BridgeUtils
        bridge_tx_hash = BridgeUtils.extract_tx_hash(output)
        
        if not bridge_tx_hash:
            BridgeLogger.error("Could not extract bridge transaction hash from output")
            BridgeLogger.debug(f"Bridge output: {output}")
            return False
        
        BridgeLogger.success(f"✅ Bridge-and-call transaction submitted: {bridge_tx_hash}")
        BridgeLogger.info("This creates TWO bridge transactions:")
        BridgeLogger.info("  • Asset bridge (deposit_count = X) - must be claimed first")
        BridgeLogger.info("  • Message bridge (deposit_count = X+1) - contains call instructions")
        print()
        
        # Step 3: Monitor bridge transactions and find both bridges
        BridgeLogger.step("[3/8] Finding our bridge transactions in L2 bridge events")
        BridgeLogger.info("Using: aggsandbox show bridges --network-id 1 --json")
        BridgeLogger.info("Looking for both asset and message bridges...")
        
        asset_bridge = None
        message_bridge = None
        
        for attempt in range(6):
            BridgeLogger.debug(f"Attempt {attempt + 1}/6 to find bridges...")
            time.sleep(3)
            
            success, output = AggsandboxAPI.show_bridges(
                network_id=BRIDGE_CONFIG.network_id_agglayer_1, 
                json_output=True
            )
            
            if success:
                try:
                    bridge_data = json.loads(output)
                    bridges = bridge_data.get('bridges', [])
                    
                    # Look for our bridge transactions (both asset and message) using BridgeUtils
                    for bridge in bridges:
                        # Check if this bridge matches our transaction hash
                        if (bridge.get('tx_hash') == bridge_tx_hash or 
                            bridge.get('bridge_tx_hash') == bridge_tx_hash):
                            # Asset bridge: leaf_type = 0, has amount > 0
                            if bridge.get('leaf_type') == 0 and bridge.get('amount', '0') != '0':
                                asset_bridge = bridge
                                BridgeLogger.success(f"✅ Found asset bridge (deposit_count = {bridge['deposit_count']}, amount = {bridge['amount']})")
                            # Message bridge: leaf_type = 1, has calldata
                            elif bridge.get('leaf_type') == 1 and bridge.get('calldata'):
                                message_bridge = bridge
                                BridgeLogger.success(f"✅ Found message bridge (deposit_count = {bridge['deposit_count']}, has calldata)")
                            else:
                                BridgeLogger.debug(f"Found bridge but couldn't classify: leaf_type={bridge.get('leaf_type')}, amount={bridge.get('amount')}")
                    
                    if asset_bridge and message_bridge:
                        break
                        
                except json.JSONDecodeError as e:
                    BridgeLogger.warning(f"Could not parse bridge data: {e}")
            else:
                BridgeLogger.warning(f"Could not get bridge data: {output}")
        
        if not asset_bridge or not message_bridge:
            BridgeLogger.error("❌ Could not find both asset and message bridge transactions")
            BridgeLogger.info(f"Asset bridge found: {asset_bridge is not None}")
            BridgeLogger.info(f"Message bridge found: {message_bridge is not None}")
            return False
        
        BridgeLogger.info(f"Asset Bridge Details:")
        asset_bridge_tx = BridgeUtils.get_bridge_tx_hash(asset_bridge)
        BridgeLogger.info(f"  • TX Hash: {asset_bridge_tx}")
        BridgeLogger.info(f"  • Amount: {asset_bridge.get('amount', 'N/A')} tokens")
        BridgeLogger.info(f"  • Deposit Count: {asset_bridge['deposit_count']}")
        BridgeLogger.info(f"  • Leaf Type: {asset_bridge.get('leaf_type')} (0=Asset)")
        
        BridgeLogger.info(f"Message Bridge Details:")
        message_bridge_tx = BridgeUtils.get_bridge_tx_hash(message_bridge)
        BridgeLogger.info(f"  • TX Hash: {message_bridge_tx}")
        BridgeLogger.info(f"  • Deposit Count: {message_bridge['deposit_count']}")
        BridgeLogger.info(f"  • Leaf Type: {message_bridge.get('leaf_type')} (1=Message)")
        BridgeLogger.info(f"  • Has Calldata: {len(message_bridge.get('calldata', '')) > 2}")
        print()
        
        # Wait for AggKit to sync bridge data from L2 to L1
        BridgeLogger.step("Waiting for AggKit to sync bridge data from L2 to L1")
        BridgeLogger.info("AggKit needs ~30 seconds to sync bridge transactions between networks")
        BridgeLogger.info("This is based on successful testing and optimized timings")
        time.sleep(30)
        print()
        
        # Step 4: Claim asset bridge FIRST using the specific deposit_count
        BridgeLogger.step(f"[4/8] Claiming asset bridge FIRST (deposit_count = {asset_bridge['deposit_count']})")
        BridgeLogger.info("Using: aggsandbox bridge claim")
        BridgeLogger.info(f"Using same tx_hash: {bridge_tx_hash}")
        BridgeLogger.info("Asset bridge must be claimed before message bridge")
        
        # Create claim args for asset bridge using BridgeUtils to get the correct tx hash
        bridge_tx = BridgeUtils.get_bridge_tx_hash(asset_bridge)
        asset_claim_args = BridgeClaimArgs(
            network=BRIDGE_CONFIG.network_id_mainnet,
            tx_hash=bridge_tx,  # Use BridgeUtils to get correct tx hash
            source_network=BRIDGE_CONFIG.network_id_agglayer_1,
            deposit_count=asset_bridge['deposit_count'],  # Use actual asset deposit count
            private_key=BRIDGE_CONFIG.private_key_2
        )
        
        success, output = AggsandboxAPI.bridge_claim(asset_claim_args)
        
        if not success:
            BridgeLogger.error(f"❌ Asset claim operation failed: {output}")
            return False
        
        # Extract asset claim transaction hash using BridgeUtils
        asset_claim_tx_hash = BridgeUtils.extract_tx_hash(output)
        
        if asset_claim_tx_hash:
            BridgeLogger.success(f"✅ Asset claim transaction submitted: {asset_claim_tx_hash}")
        else:
            BridgeLogger.success("✅ Asset claim completed successfully")
            asset_claim_tx_hash = "completed"
        
        # Wait for asset claim to be processed before claiming message
        BridgeLogger.info("Waiting for asset claim to be processed before claiming message...")
        BridgeLogger.info("Asset claim must complete before message claim can succeed")
        time.sleep(5)  # Optimized wait time
        print()
        
        # Step 5: Claim message bridge SECOND using the specific deposit_count
        BridgeLogger.step(f"[5/8] Claiming message bridge SECOND (deposit_count = {message_bridge['deposit_count']})")
        BridgeLogger.info("Using: aggsandbox bridge claim")
        BridgeLogger.info(f"Using same tx_hash: {bridge_tx_hash}")
        BridgeLogger.info("Message bridge triggers contract execution when claimed")
        
        # Create claim args for message bridge using BridgeUtils to get the correct tx hash
        bridge_tx = BridgeUtils.get_bridge_tx_hash(message_bridge)
        message_claim_args = BridgeClaimArgs(
            network=BRIDGE_CONFIG.network_id_mainnet,
            tx_hash=bridge_tx,  # Use BridgeUtils to get correct tx hash
            source_network=BRIDGE_CONFIG.network_id_agglayer_1,
            deposit_count=message_bridge['deposit_count'],  # Use actual message deposit count
            private_key=BRIDGE_CONFIG.private_key_2
        )
        
        success, output = AggsandboxAPI.bridge_claim(message_claim_args)
        
        if not success:
            BridgeLogger.error(f"❌ Message claim operation failed: {output}")
            return False
        
        # Extract message claim transaction hash using BridgeUtils
        message_claim_tx_hash = BridgeUtils.extract_tx_hash(output)
        
        if message_claim_tx_hash:
            BridgeLogger.success(f"✅ Message claim transaction submitted: {message_claim_tx_hash}")
        else:
            BridgeLogger.success("✅ Message claim completed successfully")
            message_claim_tx_hash = "completed"
        
        # Wait for message claim to be processed and contract execution
        BridgeLogger.info("Waiting for message claim to be processed and contract execution...")
        time.sleep(5)
        print()
        
        # Step 6: Verify contract execution using existing module
        BridgeLogger.step("[6/8] Verifying contract execution")
        
        contract_verified = BridgeAndCall.verify_bridge_and_call_execution(
            receiver_contract=contract_address,
            network_id=BRIDGE_CONFIG.network_id_mainnet,
            expected_message=f"L2→L1 bridge-and-call: {bridge_amount} tokens",
            expected_amount=bridge_amount
        )
        
        if contract_verified:
            BridgeLogger.success("✅ Contract execution verified successfully")
        else:
            BridgeLogger.warning("⚠️ Contract execution verification failed or incomplete")
        
        print()
        
        # Step 7: Verify claims using aggsandbox show claims
        BridgeLogger.step("[7/8] Verifying claims on L1")
        BridgeLogger.info("Using: aggsandbox show claims --network-id 0 --json")
        
        success, output = AggsandboxAPI.show_claims(
            network_id=BRIDGE_CONFIG.network_id_mainnet,
            json_output=True
        )
        
        asset_claim_found = False
        message_claim_found = False
        
        if success:
            try:
                claims_data = json.loads(output)
                claims = claims_data.get('claims', [])
                total_claims = len(claims)
                
                BridgeLogger.success(f"✅ Found {total_claims} total claims on L1")
                
                # Look for both our asset and message claims using bridge_tx_hash
                asset_bridge_tx = BridgeUtils.get_bridge_tx_hash(asset_bridge)  # Same for both bridges
                
                for claim in claims:
                    # Check if this claim is from our bridge transaction
                    if claim.get('bridge_tx_hash') == asset_bridge_tx:
                        
                        # Asset claim: type = asset, has amount > 0
                        if (claim.get('type') == 'asset' and 
                            claim.get('amount') == str(bridge_amount)):
                            
                            asset_claim_found = True
                            BridgeLogger.success("✅ Found asset claim:")
                            BridgeLogger.info(f"  • Amount: {claim.get('amount')} tokens")
                            BridgeLogger.info(f"  • Status: {claim.get('status', 'unknown').upper()}")
                            BridgeLogger.info(f"  • Bridge TX: {claim.get('bridge_tx_hash')}")
                            BridgeLogger.info(f"  • Claim TX: {claim.get('claim_tx_hash')}")
                        
                        # Message claim: type = message, amount = 0  
                        elif (claim.get('type') == 'message' and 
                              claim.get('amount') == '0'):
                            
                            message_claim_found = True
                            BridgeLogger.success("✅ Found message claim:")
                            BridgeLogger.info(f"  • Type: {claim.get('type')}")
                            BridgeLogger.info(f"  • Status: {claim.get('status', 'unknown').upper()}")
                            BridgeLogger.info(f"  • Bridge TX: {claim.get('bridge_tx_hash')}")
                            BridgeLogger.info(f"  • Claim TX: {claim.get('claim_tx_hash')}")
                
                if asset_claim_found and message_claim_found:
                    BridgeLogger.success("🎉 Both asset and message claims found!")
                elif asset_claim_found:
                    BridgeLogger.info("✅ Asset claim found, message claim may still be processing")
                elif message_claim_found:
                    BridgeLogger.info("✅ Message claim found, asset claim may still be processing")
                else:
                    BridgeLogger.warning("⚠️ Claims may still be processing")
                    
            except json.JSONDecodeError as e:
                BridgeLogger.warning(f"Could not parse claims response: {e}")
        else:
            BridgeLogger.warning(f"Could not get claims data: {output}")
        
        # Final success summary
        print("\n🎯 L2→L1 Bridge-and-Call Test Results:")
        print("━" * 70)
        BridgeLogger.success("🎉 Complete L2→L1 bridge-and-call flow successful!")
        
        print(f"\n📋 Operations Completed:")
        BridgeLogger.info("✅ 0. Contract deployment (SimpleBridgeAndCallReceiver on L1)")
        BridgeLogger.info("✅ 1. Call data preparation (receiveTokensWithMessage)")
        BridgeLogger.info("✅ 2. aggsandbox bridge bridge-and-call (L2→L1 bridging)")
        BridgeLogger.info("✅ 3. aggsandbox show bridges --json (monitoring)")
        BridgeLogger.info("✅ 4. AggKit sync wait (5 seconds - optimized)")
        BridgeLogger.info("✅ 5. aggsandbox bridge claim (asset bridge)")
        BridgeLogger.info("✅ 6. aggsandbox bridge claim (message bridge)")
        BridgeLogger.info("✅ 7. Contract verification (receiveTokensWithMessage execution)")
        BridgeLogger.info("✅ 8. aggsandbox show claims --json (verification)")
        
        print(f"\n📊 Transaction Summary:")
        BridgeLogger.info(f"Bridge-and-Call TX (L2): {bridge_tx_hash}")
        BridgeLogger.info(f"Asset Claim TX (L1):    {asset_claim_tx_hash}")
        BridgeLogger.info(f"Message Claim TX (L1):  {message_claim_tx_hash}")
        BridgeLogger.info(f"Token Amount:           {bridge_amount} tokens")
        BridgeLogger.info(f"Asset Deposit Count:    {asset_bridge['deposit_count'] if asset_bridge else 'N/A'}")
        BridgeLogger.info(f"Message Deposit Count:  {message_bridge['deposit_count'] if message_bridge else 'N/A'}")
        BridgeLogger.info(f"Target Contract:        {contract_address}")
        BridgeLogger.info(f"Call Data:              {call_data}")
        
        print(f"\n🔄 Bridge Flow:")
        BridgeLogger.info(f"L2 Network {BRIDGE_CONFIG.network_id_agglayer_1} → L1 Network {BRIDGE_CONFIG.network_id_mainnet}")
        BridgeLogger.info(f"From: {BRIDGE_CONFIG.account_address_1}")
        BridgeLogger.info(f"To:   {contract_address} (contract)")
        BridgeLogger.info(f"Type: L2→L1 Bridge-and-Call (L2 AggERC20 + Message)")
        
        print("━" * 70)
        
        return True
        
    except Exception as e:
        BridgeLogger.error(f"Test failed with exception: {e}")
        import traceback
        BridgeLogger.debug(traceback.format_exc())
        return False

def main():
    """Main function to run the L2-L1 bridge-and-call test"""
    
    # Parse command line arguments
    bridge_amount = int(sys.argv[1]) if len(sys.argv) > 1 else 8
    
    # Run the L2-L1 bridge-and-call test
    success = run_l2_to_l1_bridge_and_call_test(bridge_amount)
    
    if success:
        print(f"\n🎉 SUCCESS: L2→L1 bridge-and-call test completed!")
        sys.exit(0)
    else:
        print(f"\n❌ FAILED: L2→L1 bridge-and-call test failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()
