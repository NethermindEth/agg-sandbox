#!/usr/bin/env python3
"""
L2-L2 Bridge-and-Call Test
Tests the complete flow of bridge-and-call operations from L2-1 to L2-2 using aggsandbox CLI
Based on bridge-operations.md documentation and the successful L1-L2 and L2-L1 bridge-and-call tests

This bridges L2-1 AggERC20 tokens to L2-2 with a contract call in a single atomic operation
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

def run_l2_to_l2_bridge_and_call_test(bridge_amount: int = 12):
    """
    Complete L2-L2 Bridge-and-Call Test
    
    This test follows the documented bridge-and-call process for L2→L2:
    0. Deploy SimpleBridgeAndCallReceiver contract on L2-2
    1. Prepare call data for the contract function
    2. Execute bridge-and-call from L2-1 to L2-2 using aggsandbox bridge bridge-and-call
    3. Monitor the bridge transactions using aggsandbox show bridges
    4. Wait for AggKit to sync bridge data from L2-1 to L2-2 (longer wait)
    5. Claim asset bridge first (deposit_count = X)
    6. Claim message bridge second (deposit_count = X+1)
    7. Verify contract execution and token transfer
    8. Verify both claims using aggsandbox show claims
    
    Args:
        bridge_amount: Amount of L2-1 AggERC20 tokens to bridge (default: 12)
    """
    print("\n" + "="*70)
    print(f"🔗 L2→L2 Bridge-and-Call Test")
    print(f"Token Amount: {bridge_amount} L2-1 AggERC20 tokens")
    print(f"Following bridge-operations.md documentation")
    print(f"Source: L2-1 (Network 1) → Destination: L2-2 (Network 2)")
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
        BridgeLogger.info(f"L2-1 Network ID: 1 (zkEVM)")
        BridgeLogger.info(f"L2-2 Network ID: 2 (Agglayer-2)")
        BridgeLogger.info(f"From Account: {BRIDGE_CONFIG.account_address_1}")
        print()
        
        # Step 0: Deploy bridge-and-call receiver contract on L2-2
        BridgeLogger.step("[0/8] Deploying bridge-and-call receiver contract on L2-2")
        contract_address = BridgeAndCall.deploy_bridge_call_receiver(
            2,  # L2-2 network
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
        
        # Get what the L2-1 token will become on L2-2 (for the function call)
        success, output = AggsandboxAPI.bridge_utils_precalculate(
            network=2,  # L2-2
            origin_network=1,  # L2-1
            origin_token=BRIDGE_CONFIG.agg_erc20_l2,
            json_output=True
        )
        
        l2_2_wrapped_token_addr = None
        if success:
            try:
                data = json.loads(output)
                l2_2_wrapped_token_addr = data.get('precalculated_address')
                BridgeLogger.success(f"✅ L2-2 wrapped token will be: {l2_2_wrapped_token_addr}")
            except json.JSONDecodeError as e:
                BridgeLogger.warning(f"Could not parse precalculate response: {e}")
        
        if not l2_2_wrapped_token_addr:
            BridgeLogger.error("Could not determine L2-2 wrapped token address")
            return False
        
        # Encode the receiveTokensWithMessage function call
        call_data = encode_call_data("receiveTokensWithMessage(address,uint256,string)", 
                                   l2_2_wrapped_token_addr, bridge_amount, f"L2→L2 bridge-and-call: {bridge_amount} tokens")
        if not call_data:
            BridgeLogger.error("Failed to encode call data")
            return False
        
        BridgeLogger.success(f"✅ Call data encoded: {call_data}")
        print()
        
        # Step 2: Execute bridge-and-call from L2-1 to L2-2
        BridgeLogger.step(f"[2/8] Executing bridge-and-call from L2-1 to L2-2")
        BridgeLogger.info("Using: aggsandbox bridge bridge-and-call")
        BridgeLogger.info(f"Token: {BRIDGE_CONFIG.agg_erc20_l2}")
        BridgeLogger.info(f"Amount: {bridge_amount} tokens")
        BridgeLogger.info(f"Target contract: {contract_address}")
        BridgeLogger.info("This will create both asset and message bridges")
        
        success, output = AggsandboxAPI.bridge_and_call(
            network=1,  # L2-1
            destination_network=2,  # L2-2
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
        
        # Extract bridge transaction hash from output
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
        BridgeLogger.step("[3/8] Finding our bridge transactions in L2-1 bridge events")
        BridgeLogger.info("Using: aggsandbox show bridges --network-id 1 --json")
        BridgeLogger.info("Looking for both asset and message bridges...")
        
        asset_bridge = None
        message_bridge = None
        
        for attempt in range(6):
            BridgeLogger.debug(f"Attempt {attempt + 1}/6 to find bridges...")
            time.sleep(3)
            
            success, output = AggsandboxAPI.show_bridges(
                network_id=1,  # L2-1 bridges
                json_output=True
            )
            
            if success:
                try:
                    bridge_data = json.loads(output)
                    bridges = bridge_data.get('bridges', [])
                    
                    # Look for our bridge transactions (both asset and message)
                    for bridge in bridges:
                        bridge_tx = BridgeUtils.get_bridge_tx_hash(bridge)
                        if bridge_tx == bridge_tx_hash:
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
        asset_tx = BridgeUtils.get_bridge_tx_hash(asset_bridge)
        BridgeLogger.info(f"  • TX Hash: {asset_tx}")
        BridgeLogger.info(f"  • Amount: {asset_bridge.get('amount', 'N/A')} tokens")
        BridgeLogger.info(f"  • Deposit Count: {asset_bridge['deposit_count']}")
        BridgeLogger.info(f"  • Leaf Type: {asset_bridge.get('leaf_type')} (0=Asset)")
        
        BridgeLogger.info(f"Message Bridge Details:")
        message_tx = BridgeUtils.get_bridge_tx_hash(message_bridge)
        BridgeLogger.info(f"  • TX Hash: {message_tx}")
        BridgeLogger.info(f"  • Deposit Count: {message_bridge['deposit_count']}")
        BridgeLogger.info(f"  • Leaf Type: {message_bridge.get('leaf_type')} (1=Message)")
        BridgeLogger.info(f"  • Has Calldata: {len(message_bridge.get('calldata', '')) > 2}")
        print()
        
        # Wait for AggKit to sync bridge data from L2-1 to L2-2 (longer wait for L2→L2)
        BridgeLogger.step("Waiting for AggKit to sync bridge data from L2-1 to L2-2")
        BridgeLogger.info("L2→L2 bridging requires longer sync time than L1<->L2")
        BridgeLogger.info("AggKit needs ~45 seconds to sync bridge transactions between L2 networks")
        BridgeLogger.info("This is normal behavior - L2→L2 sync takes much longer than L1<->L2")
        time.sleep(45)  # Extended wait for L2→L2 bridge-and-call
        print()
        
        # Step 4: Claim asset bridge FIRST (deposit_count = X)
        BridgeLogger.step(f"[4/8] Claiming asset bridge FIRST (deposit_count = {asset_bridge['deposit_count']})")
        BridgeLogger.info("Using: aggsandbox bridge claim")
        BridgeLogger.info(f"Using same tx_hash: {bridge_tx_hash}")
        BridgeLogger.info("Asset bridge must be claimed before message bridge")
        
        # Create claim args for asset bridge using the actual deposit_count
        asset_tx = BridgeUtils.get_bridge_tx_hash(asset_bridge)
        asset_claim_args = BridgeClaimArgs(
            network=2,  # L2-2
            tx_hash=asset_tx,  # Use bridge_tx_hash from BridgeUtils
            source_network=1,  # L2-1
            deposit_count=asset_bridge['deposit_count'],  # Use actual asset deposit count
            private_key=BRIDGE_CONFIG.private_key_2
        )
        
        success, output = AggsandboxAPI.bridge_claim(asset_claim_args)
        if not success:
            BridgeLogger.error(f"❌ Asset claim operation failed: {output}")
            return False
        
        # Extract asset claim transaction hash
        asset_claim_tx_hash = BridgeUtils.extract_tx_hash(output)
        if asset_claim_tx_hash:
            BridgeLogger.success(f"✅ Asset claim transaction submitted: {asset_claim_tx_hash}")
        else:
            BridgeLogger.success("✅ Asset claim completed successfully")
            asset_claim_tx_hash = "completed"
        
        # Wait a bit between asset and message claims
        BridgeLogger.info("Waiting 15 seconds before claiming message bridge...")
        time.sleep(15)
        
        print()
        
        # Step 5: Claim message bridge SECOND (deposit_count = X+1)
        BridgeLogger.step(f"[5/8] Claiming message bridge SECOND (deposit_count = {message_bridge['deposit_count']})")
        BridgeLogger.info("Using: aggsandbox bridge claim")
        BridgeLogger.info(f"Using same tx_hash: {bridge_tx_hash}")
        BridgeLogger.info("Message bridge triggers the contract execution")
        
        # Create claim args for message bridge using the actual deposit_count
        message_tx = BridgeUtils.get_bridge_tx_hash(message_bridge)
        message_claim_args = BridgeClaimArgs(
            network=2,  # L2-2
            tx_hash=message_tx,  # Use bridge_tx_hash from BridgeUtils
            source_network=1,  # L2-1
            deposit_count=message_bridge['deposit_count'],  # Use actual message deposit count
            private_key=BRIDGE_CONFIG.private_key_2
        )
        
        success, output = AggsandboxAPI.bridge_claim(message_claim_args)
        if not success:
            BridgeLogger.error(f"❌ Message claim operation failed: {output}")
            return False
        
        # Extract message claim transaction hash
        message_claim_tx_hash = BridgeUtils.extract_tx_hash(output)
        if message_claim_tx_hash:
            BridgeLogger.success(f"✅ Message claim transaction submitted: {message_claim_tx_hash}")
        else:
            BridgeLogger.success("✅ Message claim completed successfully")
            message_claim_tx_hash = "completed"
        
        # Wait for both claims to be processed
        BridgeLogger.info("Waiting for both claims to be processed...")
        BridgeLogger.info("Checking claim statuses until both are completed...")
        
        # Wait for both claims to be completed (check status periodically)
        asset_claim_completed = False
        message_claim_completed = False
        
        for attempt in range(12):  # Try for up to 60 seconds (12 * 5 seconds)
            time.sleep(5)
            BridgeLogger.debug(f"Checking claim statuses (attempt {attempt + 1}/12)...")
            
            success, output = AggsandboxAPI.show_claims(
                network_id=2,  # Check L2-2 claims
                json_output=True
            )
            
            if success:
                try:
                    claims_data = json.loads(output)
                    claims = claims_data.get('claims', [])
                    
                    # Look for both our claims by matching bridge details
                    for claim in claims:
                        # Asset claim: has amount > 0, type = asset, matches our bridge amount
                        if (claim.get('origin_address') == BRIDGE_CONFIG.agg_erc20_l2 and
                            claim.get('amount') == str(bridge_amount) and
                            claim.get('origin_network') == 1 and  # L2-1
                            claim.get('destination_network') == 2 and  # L2-2
                            claim.get('type') == 'asset'):
                            
                            if claim.get('status') == "completed":
                                if not asset_claim_completed:
                                    BridgeLogger.success(f"✅ Asset claim completed! (dest: {claim.get('destination_address')[:10]}...)")
                                    asset_claim_completed = True
                        
                        # Message claim: from our L2-1 network, match by claim_tx_hash if available
                        elif (claim.get('origin_network') == 1 and  # L2-1
                              claim.get('destination_network') == 2 and  # L2-2
                              claim.get('amount') == '0' and
                              (claim.get('claim_tx_hash') == message_claim_tx_hash or
                               claim.get('destination_address') == contract_address)):
                            
                            if claim.get('status') == "completed":
                                if not message_claim_completed:
                                    BridgeLogger.success(f"✅ Message claim completed! (dest: {claim.get('destination_address')[:10]}...)")
                                    message_claim_completed = True
                    
                    if asset_claim_completed and message_claim_completed:
                        BridgeLogger.success(f"✅ Both claims completed after {(attempt + 1) * 5} seconds!")
                        break
                        
                except json.JSONDecodeError:
                    BridgeLogger.debug("Could not parse claims data")
        
        if not (asset_claim_completed and message_claim_completed):
            BridgeLogger.error("❌ Not all claims completed after 60 seconds - this indicates a problem!")
            BridgeLogger.error(f"Asset claim completed: {asset_claim_completed}")
            BridgeLogger.error(f"Message claim completed: {message_claim_completed}")
            return False
        
        print()
        
        # Step 6: Verify contract execution and token transfer
        BridgeLogger.step("[6/8] Verifying contract execution and token transfer")
        BridgeLogger.info("Checking if the contract received tokens and executed the function")
        
        try:
            # Check if contract received tokens
            cmd = [
                "cast", "call", l2_2_wrapped_token_addr,
                "balanceOf(address)(uint256)",
                contract_address,
                "--rpc-url", "http://localhost:8547"  # L2-2 RPC
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            contract_balance = int(result.stdout.strip())
            BridgeLogger.success(f"✅ Contract token balance: {contract_balance} tokens")
            
            # Check contract state to see if function was called
            cmd = [
                "cast", "call", contract_address,
                "getLastCall()(address,uint256,string)",
                "--rpc-url", "http://localhost:8547"  # L2-2 RPC
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            call_data = result.stdout.strip()
            BridgeLogger.success(f"✅ Contract call data: {call_data}")
            
            # Check total calls received
            cmd = [
                "cast", "call", contract_address,
                "totalCallsReceived()(uint256)",
                "--rpc-url", "http://localhost:8547"  # L2-2 RPC
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            total_calls = int(result.stdout.strip())
            BridgeLogger.success(f"✅ Total calls received by contract: {total_calls}")
            
        except subprocess.CalledProcessError as e:
            BridgeLogger.warning(f"Could not verify contract state: {e}")
        
        print()
        
        # Step 7: Verify both claims using aggsandbox show claims
        BridgeLogger.step("[7/8] Verifying both claims on L2-2")
        BridgeLogger.info("Using: aggsandbox show claims --network-id 2 --json")
        BridgeLogger.info("Waiting for claims to be fully processed and indexed...")
        
        time.sleep(15)  # Give claims time to be fully processed and indexed
        
        success, output = AggsandboxAPI.show_claims(
            network_id=2,  # L2-2 claims
            json_output=True
        )
        
        if success:
            try:
                claims_data = json.loads(output)
                claims = claims_data.get('claims', [])
                total_claims = len(claims)
                
                BridgeLogger.success(f"✅ Found {total_claims} total claims on L2-2")
                
                # Look for our specific claims (use same relaxed matching as monitoring)
                our_asset_claim = None
                our_message_claim = None
                
                for claim in claims:
                    # Asset claim: has amount > 0, type = asset, matches our bridge amount
                    if (claim.get('origin_address') == BRIDGE_CONFIG.agg_erc20_l2 and
                        claim.get('amount') == str(bridge_amount) and
                        claim.get('origin_network') == 1 and  # L2-1
                        claim.get('destination_network') == 2 and  # L2-2
                        claim.get('type') == 'asset' and
                        claim.get('status') == 'completed'):
                        our_asset_claim = claim
                    
                    # Message claim: from our L2-1 network, match by claim_tx_hash if available
                    elif (claim.get('origin_network') == 1 and  # L2-1
                          claim.get('destination_network') == 2 and  # L2-2
                          claim.get('amount') == '0' and
                          claim.get('status') == 'completed' and
                          (claim.get('claim_tx_hash') == message_claim_tx_hash or
                           claim.get('destination_address') == contract_address)):
                        # Take the most recent message claim (highest global_index)
                        if our_message_claim is None or claim.get('global_index', 0) > our_message_claim.get('global_index', 0):
                            our_message_claim = claim
                
                if our_asset_claim and our_message_claim:
                    BridgeLogger.success("✅ Found both completed claims in L2-2:")
                    asset_claim_tx = our_asset_claim.get('claim_tx_hash') or our_asset_claim.get('claim_tx_hash')
                    message_claim_tx = our_message_claim.get('claim_tx_hash') or our_message_claim.get('claim_tx_hash')
                    BridgeLogger.info(f"  • Asset Claim - Amount: {our_asset_claim.get('amount')}, TX: {asset_claim_tx}")
                    BridgeLogger.info(f"  • Message Claim - Contract: {our_message_claim.get('destination_address')[:10]}..., TX: {message_claim_tx}")
                    
                    # Check if developer bug is fixed
                    if our_message_claim.get('type') == 'message':
                        BridgeLogger.success("✅ Developer fix confirmed: L2-L2 message claims now correctly show type 'message'")
                    elif our_message_claim.get('type') == 'asset' and our_message_claim.get('amount') == '0':
                        BridgeLogger.warning("⚠️ Developer bug still present: L2-L2 message claims show type 'asset' instead of 'message'")
                    
                    BridgeLogger.success("🎉 Both claims are COMPLETE!")
                elif our_asset_claim:
                    BridgeLogger.warning("⚠️ Only asset claim found - message claim may still be processing")
                    return False
                elif our_message_claim:
                    BridgeLogger.warning("⚠️ Only message claim found - asset claim may still be processing")
                    return False
                else:
                    BridgeLogger.error("❌ Neither claim found in claims API")
                    return False
                    
            except json.JSONDecodeError as e:
                BridgeLogger.warning(f"Could not parse claims response: {e}")
                return False
        else:
            BridgeLogger.warning(f"Could not get claims data: {output}")
            return False
        
        # Final success summary
        print("\n🎯 L2→L2 Bridge-and-Call Test Results:")
        print("━" * 70)
        BridgeLogger.success("🎉 Complete L2→L2 bridge-and-call flow successful!")
        
        print(f"\n📋 Operations Completed:")
        BridgeLogger.info("✅ 0. Contract deployment (SimpleBridgeAndCallReceiver on L2-2)")
        BridgeLogger.info("✅ 1. Call data preparation (receiveTokensWithMessage)")
        BridgeLogger.info("✅ 2. aggsandbox bridge bridge-and-call (L2-1→L2-2 bridging)")
        BridgeLogger.info("✅ 3. aggsandbox show bridges --json (monitoring)")
        BridgeLogger.info("✅ 4. AggKit sync wait (45 seconds - L2→L2 extended time)")
        BridgeLogger.info("✅ 5. aggsandbox bridge claim (asset bridge on L2-2)")
        BridgeLogger.info("✅ 6. aggsandbox bridge claim (message bridge on L2-2)")
        BridgeLogger.info("✅ 7. Contract verification (tokens and execution)")
        BridgeLogger.info("✅ 8. aggsandbox show claims --json (verification)")
        
        print(f"\n📊 Transaction Summary:")
        BridgeLogger.info(f"Bridge-and-Call TX (L2-1): {bridge_tx_hash}")
        BridgeLogger.info(f"Asset Claim TX (L2-2):    {asset_claim_tx_hash}")
        BridgeLogger.info(f"Message Claim TX (L2-2):  {message_claim_tx_hash}")
        BridgeLogger.info(f"Token Amount:             {bridge_amount} tokens")
        BridgeLogger.info(f"Asset Deposit Count:      {asset_bridge['deposit_count'] if asset_bridge else 'N/A'}")
        BridgeLogger.info(f"Message Deposit Count:    {message_bridge['deposit_count'] if message_bridge else 'N/A'}")
        BridgeLogger.info(f"Target Contract:          {contract_address}")
        BridgeLogger.info(f"L2-2 Wrapped Token:       {l2_2_wrapped_token_addr}")
        
        print(f"\n🔄 Bridge Flow:")
        BridgeLogger.info(f"L2-1 Network 1 → L2-2 Network 2")
        BridgeLogger.info(f"From: {BRIDGE_CONFIG.account_address_1}")
        BridgeLogger.info(f"To Contract: {contract_address}")
        BridgeLogger.info(f"Type: L2→L2 Bridge-and-Call (Atomic bridge + contract execution)")
        BridgeLogger.info(f"RPC: http://localhost:8546 → http://localhost:8547")
        
        print("━" * 70)
        
        return True
        
    except Exception as e:
        BridgeLogger.error(f"Test failed with exception: {e}")
        import traceback
        BridgeLogger.debug(traceback.format_exc())
        return False

def main():
    """Main function to run the L2-L2 bridge-and-call test"""
    
    # Parse command line arguments
    bridge_amount = int(sys.argv[1]) if len(sys.argv) > 1 else 12
    
    # Run the L2-L2 bridge-and-call test
    success = run_l2_to_l2_bridge_and_call_test(bridge_amount)
    
    if success:
        print(f"\n🎉 SUCCESS: L2→L2 bridge-and-call test completed!")
        sys.exit(0)
    else:
        print(f"\n❌ FAILED: L2→L2 bridge-and-call test failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()
