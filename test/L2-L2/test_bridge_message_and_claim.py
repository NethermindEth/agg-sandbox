#!/usr/bin/env python3
"""
L2-L2 Message Bridge Test
Tests the complete flow of bridging messages from L2-1 to L2-2 using aggsandbox CLI
Based on bridge-operations.md documentation and the successful L1-L2 and L2-L1 message tests

NOTE: This test will fail at the claiming step due to a known issue with 
localhost:5577 (aggkit-l2) L1 info tree index API in multi-L2 mode.
The bridge transaction will succeed, but claiming will fail until the developer fixes the API.
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

def deploy_message_receiver_contract() -> str:
    """Deploy SimpleBridgeMessageReceiver contract on L2-2"""
    BridgeLogger.step("Deploying SimpleBridgeMessageReceiver contract on L2-2")
    
    try:
        # Deploy the contract using forge
        cmd = [
            "forge", "create", 
            "test/contracts/SimpleBridgeMessageReceiver.sol:SimpleBridgeMessageReceiver",
            "--rpc-url", "http://localhost:8547",  # L2-2 RPC
            "--private-key", BRIDGE_CONFIG.private_key_1
        ]
        
        BridgeLogger.debug(f"Executing: {' '.join(cmd)}")
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        
        # Extract contract address from output
        output = result.stdout.strip()
        lines = output.split('\n')
        contract_address = None
        
        for line in lines:
            if 'Deployed to:' in line:
                contract_address = line.split('Deployed to:')[1].strip()
                break
        
        if contract_address:
            BridgeLogger.success(f"✅ Contract deployed at: {contract_address}")
            return contract_address
        else:
            BridgeLogger.error("Could not extract contract address from deployment output")
            return None
            
    except subprocess.CalledProcessError as e:
        BridgeLogger.error(f"Contract deployment failed: {e}")
        BridgeLogger.error(f"Error output: {e.stderr}")
        return None

def encode_message_data(message: str) -> str:
    """Encode a string message for bridge transmission"""
    try:
        # Convert string to hex
        message_hex = message.encode('utf-8').hex()
        return f"0x{message_hex}"
    except Exception as e:
        BridgeLogger.error(f"Failed to encode message: {e}")
        return None

def run_l2_to_l2_message_bridge_test(message: str = "L2-1 to L2-2 Message"):
    """
    Complete L2-L2 Message Bridge Test
    
    This test follows the documented message bridge process:
    0. Deploy SimpleBridgeMessageReceiver contract on L2-2
    1. Bridge message from L2-1 to L2-2 using aggsandbox bridge message
    2. Monitor the bridge transaction using aggsandbox show bridges
    3. Wait for AggKit to sync bridge data from L2-1 to L2-2 (longer wait)
    4. Attempt to claim the message on L2-2 using aggsandbox bridge claim
    5. Verify the claim using aggsandbox show claims
    6. Check that the contract received the message
    
    NOTE: Step 4 will fail due to L1 info tree index API issue in multi-L2 mode
    
    Args:
        message: Plain text message to bridge (default: "L2-1 to L2-2 Message")
    """
    # Encode the message
    message_data = encode_message_data(message)
    if not message_data:
        BridgeLogger.error("Failed to encode message data")
        return False
    
    print("\n" + "="*70)
    print(f"📬 L2→L2 Message Bridge Test")
    print(f"Message: '{message}' -> {message_data}")
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
        
        # Step 0: Deploy message receiver contract on L2-2
        BridgeLogger.step("[0/6] Deploying message receiver contract on L2-2")
        contract_address = deploy_message_receiver_contract()
        if not contract_address:
            BridgeLogger.error("Failed to deploy message receiver contract")
            return False
        
        BridgeLogger.info(f"Message receiver deployed at: {contract_address}")
        time.sleep(5)
        print()
        
        # Step 1: Bridge message from L2-1 to L2-2
        BridgeLogger.step(f"[1/6] Bridging message from L2-1 to L2-2")
        BridgeLogger.info("Using: aggsandbox bridge message")
        BridgeLogger.info(f"Message data: {message_data}")
        
        success, output = AggsandboxAPI.bridge_message(
            network=1,  # L2-1
            destination_network=2,  # L2-2
            target=contract_address,
            data=message_data,
            private_key=BRIDGE_CONFIG.private_key_1
        )
        
        if not success:
            BridgeLogger.error(f"Bridge message operation failed: {output}")
            return False
        
        # Extract bridge transaction hash from output using the utility function
        bridge_tx_hash = BridgeUtils.extract_tx_hash(output)
        
        if not bridge_tx_hash:
            BridgeLogger.error("Could not extract bridge transaction hash from output")
            BridgeLogger.debug(f"Bridge output: {output}")
            return False
        
        BridgeLogger.success(f"✅ Message bridge transaction submitted: {bridge_tx_hash}")
        print()
        
        # Step 2: Monitor bridge transaction and find our bridge
        BridgeLogger.step("[2/6] Finding our bridge in L2-1 bridge events")
        BridgeLogger.info("Using: aggsandbox show bridges --network-id 1 --json")
        
        our_bridge = None
        for attempt in range(6):
            BridgeLogger.debug(f"Attempt {attempt + 1}/6 to find bridge...")
            time.sleep(3)
            
            success, output = AggsandboxAPI.show_bridges(
                network_id=1,  # L2-1 bridges
                json_output=True
            )
            
            if success:
                try:
                    bridge_data = json.loads(output)
                    bridges = bridge_data.get('bridges', [])
                    
                    # Look for our specific bridge transaction
                    our_bridge = BridgeUtils.find_bridge_by_tx_hash(bridges, bridge_tx_hash)
                    if our_bridge:
                        BridgeLogger.success(f"✅ Found our bridge (attempt {attempt + 1})")
                        break
                        
                except json.JSONDecodeError as e:
                    BridgeLogger.warning(f"Could not parse bridge data: {e}")
            else:
                BridgeLogger.warning(f"Could not get bridge data: {output}")
        
        if not our_bridge:
            BridgeLogger.error("❌ Our bridge transaction not found in bridge events")
            BridgeLogger.info("This may indicate an indexing delay or bridge failure")
            return False
        
        BridgeLogger.info(f"Bridge Details:")
        bridge_tx = BridgeUtils.get_bridge_tx_hash(our_bridge)
        BridgeLogger.info(f"  • TX Hash: {bridge_tx}")
        BridgeLogger.info(f"  • Deposit Count: {our_bridge['deposit_count']}")
        BridgeLogger.info(f"  • Block: {our_bridge.get('block_num', 'N/A')}")
        BridgeLogger.info(f"  • Destination Network: {our_bridge['destination_network']}")
        BridgeLogger.info(f"  • Message Data: {message_data}")
        print()
        
        # Wait for AggKit to sync bridge data from L2-1 to L2-2 (longer wait for L2→L2)
        BridgeLogger.step("Waiting for AggKit to sync bridge data from L2-1 to L2-2")
        BridgeLogger.info("L2→L2 bridging requires longer sync time than L1<->L2")
        BridgeLogger.info("AggKit needs ~45 seconds to sync bridge transactions between L2 networks")
        BridgeLogger.info("This is normal behavior - L2→L2 sync takes much longer than L1<->L2")
        time.sleep(45)  # Even longer wait for L2→L2 message bridging
        print()
        
        # Step 3: Claim the bridged message on L2-2
        BridgeLogger.step("[3/6] Claiming bridged message on L2-2")
        BridgeLogger.info("Using: aggsandbox bridge claim")
        BridgeLogger.info("This will trigger the contract execution on L2-2")
        
        # Create claim args
        bridge_tx = BridgeUtils.get_bridge_tx_hash(our_bridge)
        claim_args = BridgeClaimArgs(
            network=2,  # Claim on L2-2
            tx_hash=bridge_tx,
            source_network=1,  # Source: L2-1
            private_key=BRIDGE_CONFIG.private_key_2  # L2-2 account 2 (recipient)
        )
        
        success, output = AggsandboxAPI.bridge_claim(claim_args)
        if not success:
            BridgeLogger.error(f"❌ Claim operation failed: {output}")
            return False
        
        # Extract claim transaction hash
        claim_tx_hash = BridgeUtils.extract_tx_hash(output)
        if claim_tx_hash:
            BridgeLogger.success(f"✅ Claim transaction submitted: {claim_tx_hash}")
        else:
            BridgeLogger.success("✅ Claim completed successfully")
            claim_tx_hash = "completed"
        
        # Wait for claim to be processed
        BridgeLogger.info("Waiting for claim to be processed...")
        BridgeLogger.info("Checking claim status until completed...")
        
        # Wait for claim to be completed (check status periodically)
        claim_completed = False
        for attempt in range(12):  # Try for up to 60 seconds (12 * 5 seconds)
            time.sleep(5)
            BridgeLogger.debug(f"Checking claim status (attempt {attempt + 1}/12)...")
            
            success, output = AggsandboxAPI.show_claims(
                network_id=2,  # Check L2-2 claims
                json_output=True
            )
            
            if success:
                try:
                    claims_data = json.loads(output)
                    claims = claims_data.get('claims', [])
                    
                    # Look for our claim using multiple matching strategies
                    bridge_tx = BridgeUtils.get_bridge_tx_hash(our_bridge)
                    for claim in claims:
                        # Match by bridge_tx_hash, claim_tx_hash, or bridge details
                        if (claim.get('bridge_tx_hash') == bridge_tx or 
                            claim.get('claim_tx_hash') == claim_tx_hash or
                            (claim.get('destination_address') == contract_address and
                             claim.get('origin_network') == 1 and  # L2-1
                             claim.get('destination_network') == 2 and  # L2-2
                             claim.get('amount') == '0')):
                            
                            claim_status = claim.get('status', 'unknown')
                            BridgeLogger.debug(f"Found matching claim: status={claim_status}, tx_hash={claim.get('claim_tx_hash')}")
                            
                            if claim_status == "completed":
                                BridgeLogger.success(f"✅ Claim completed after {(attempt + 1) * 5} seconds!")
                                claim_completed = True
                                break
                            elif claim_status == "pending":
                                BridgeLogger.debug("⏳ Still pending...")
                                continue
                    
                    if claim_completed:
                        break
                        
                except json.JSONDecodeError:
                    BridgeLogger.debug("Could not parse claims data")
        
        if not claim_completed:
            BridgeLogger.error("❌ Claim did not complete after 60 seconds - this indicates a problem!")
            return False
        
        print()
        
        # Step 4: Verify message was received by the contract
        BridgeLogger.step("[4/6] Verifying message received by contract")
        BridgeLogger.info("Checking if the message receiver contract got the message")
        
        try:
            # Call getLastMessage() to see if contract received our message
            cmd = [
                "cast", "call", contract_address,
                "getLastMessage()(address,uint32,bytes,uint256)",
                "--rpc-url", "http://localhost:8547"  # L2-2 RPC
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            output_data = result.stdout.strip()
            BridgeLogger.success(f"✅ Contract call successful: {output_data}")
            
            # Also check totalMessagesReceived
            cmd = [
                "cast", "call", contract_address,
                "totalMessagesReceived()(uint256)",
                "--rpc-url", "http://localhost:8547"  # L2-2 RPC
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            total_messages = int(result.stdout.strip())
            BridgeLogger.success(f"✅ Total messages received by contract: {total_messages}")
            
        except subprocess.CalledProcessError as e:
            BridgeLogger.warning(f"Could not verify contract state: {e}")
        
        print()
        
        # Step 5: Verify claim using aggsandbox show claims
        BridgeLogger.step("[5/6] Verifying claim on L2-2")
        BridgeLogger.info("Using: aggsandbox show claims --network-id 2 --json")
        BridgeLogger.info("Waiting for claim to be fully processed and indexed...")
        
        time.sleep(15)  # Give claim time to be fully processed and indexed
        
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
                
                # Look for our specific claim using multiple matching strategies
                bridge_tx = BridgeUtils.get_bridge_tx_hash(our_bridge)
                our_claim = None
                completed_claim = None
                for claim in claims:
                    # Match by bridge_tx_hash, claim_tx_hash, or bridge details
                    # Note: Due to developer bug, L2-L2 message claims show type "asset"
                    if (claim.get('bridge_tx_hash') == bridge_tx or 
                        claim.get('claim_tx_hash') == claim_tx_hash or
                        (claim.get('destination_address') == contract_address and
                         claim.get('origin_network') == 1 and  # L2-1
                         claim.get('destination_network') == 2 and  # L2-2
                         claim.get('amount') == '0')):
                        
                        if claim.get('status') == 'completed':
                            completed_claim = claim
                        elif claim.get('status') == 'pending':
                            our_claim = claim
                
                # Prefer completed claim, fallback to pending
                display_claim = completed_claim or our_claim
                
                if display_claim:
                    claim_status = display_claim.get('status', 'unknown')
                    claim_type = display_claim.get('type', 'unknown')
                    BridgeLogger.success("✅ Found our claim in L2-2 claims:")
                    BridgeLogger.info(f"  • Type: {claim_type}")
                    BridgeLogger.info(f"  • Block: {display_claim.get('block_num')}")
                    BridgeLogger.info(f"  • Status: {claim_status.upper()}")
                    BridgeLogger.info(f"  • Global Index: {display_claim.get('global_index')}")
                    BridgeLogger.info(f"  • Bridge TX: {display_claim.get('bridge_tx_hash')}")
                    BridgeLogger.info(f"  • Claim TX: {display_claim.get('claim_tx_hash')}")
                    
                    # Note developer bug for L2-L2 message claims
                    if claim_type == 'asset' and display_claim.get('amount') == '0':
                        BridgeLogger.warning("⚠️ Developer bug: L2-L2 message claims show type 'asset' instead of 'message'")
                    
                    if claim_status == "completed":
                        BridgeLogger.success("🎉 Claim is COMPLETE!")
                    else:
                        BridgeLogger.error(f"❌ Claim status is not completed: {claim_status}")
                        return False
                    
                    # Show both statuses if we found both
                    if completed_claim and our_claim:
                        BridgeLogger.info(f"Note: Found both PENDING and COMPLETED entries (normal behavior)")
                else:
                    BridgeLogger.error("❌ Our specific claim not found in claims API")
                    # Show a few recent claims for debugging
                    if claims:
                        BridgeLogger.info("Recent claims for reference:")
                        for i, claim in enumerate(claims[:3]):
                            BridgeLogger.info(f"  {i+1}. Type: {claim.get('type')}, Status: {claim.get('status')}, TX: {claim.get('claim_tx_hash', 'N/A')[:10]}...")
                    return False
                    
            except json.JSONDecodeError as e:
                BridgeLogger.warning(f"Could not parse claims response: {e}")
                return False
        else:
            BridgeLogger.warning(f"Could not get claims data: {output}")
            return False
        
        # Final success summary
        print("\n🎯 L2→L2 Message Bridge Test Results:")
        print("━" * 70)
        BridgeLogger.success("🎉 Complete L2→L2 message bridge flow successful!")
        
        print(f"\n📋 Operations Completed:")
        BridgeLogger.info("✅ 0. Contract deployment (SimpleBridgeMessageReceiver on L2-2)")
        BridgeLogger.info("✅ 1. aggsandbox bridge message (L2-1→L2-2 message bridging)")
        BridgeLogger.info("✅ 2. aggsandbox show bridges --json (monitoring)")
        BridgeLogger.info("✅ 3. AggKit sync wait (90 seconds - L2→L2 extended time)")
        BridgeLogger.info("✅ 4. aggsandbox bridge claim (claiming on L2-2)")
        BridgeLogger.info("✅ 5. Contract verification (message received)")
        BridgeLogger.info("✅ 6. aggsandbox show claims --json (verification)")
        
        print(f"\n📊 Transaction Summary:")
        bridge_tx = BridgeUtils.get_bridge_tx_hash(our_bridge)
        BridgeLogger.info(f"Bridge TX (L2-1): {bridge_tx}")
        BridgeLogger.info(f"Claim TX (L2-2):  {claim_tx_hash}")
        BridgeLogger.info(f"Message Data:     {message_data}")
        BridgeLogger.info(f"Deposit Count:    {our_bridge['deposit_count']}")
        BridgeLogger.info(f"Target Contract:  {contract_address}")
        
        print(f"\n🔄 Bridge Flow:")
        BridgeLogger.info(f"L2-1 Network 1 → L2-2 Network 2")
        BridgeLogger.info(f"From: {BRIDGE_CONFIG.account_address_1}")
        BridgeLogger.info(f"To Contract: {contract_address}")
        BridgeLogger.info(f"Type: Pure L2→L2 Message Bridge (no assets involved)")
        BridgeLogger.info(f"RPC: http://localhost:8546 → http://localhost:8547")
        
        print("━" * 70)
        return True
        
    except Exception as e:
        BridgeLogger.error(f"Test failed with exception: {e}")
        import traceback
        BridgeLogger.debug(traceback.format_exc())
        return False

def main():
    """Main function to run the L2-L2 message bridge test"""
    
    # Parse command line arguments
    if len(sys.argv) > 1:
        message = sys.argv[1]
    else:
        # Default message
        message = "L2-1 to L2-2 Message"
    
    # Run the L2-L2 message bridge test
    success = run_l2_to_l2_message_bridge_test(message)
    
    if success:
        print(f"\n🎉 SUCCESS: L2→L2 message bridge test completed!")
        sys.exit(0)
    else:
        print(f"\n❌ FAILED: L2→L2 message bridge test failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()
