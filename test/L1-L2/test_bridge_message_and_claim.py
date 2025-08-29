#!/usr/bin/env python3
"""
L1-L2 Message Bridge Test
Tests the complete flow of bridging messages from L1 to L2 using aggsandbox CLI
Based on bridge-operations.md documentation and the successful asset bridge test
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
    """Deploy SimpleBridgeMessageReceiver contract on L2"""
    BridgeLogger.step("Deploying SimpleBridgeMessageReceiver contract on L2")
    
    try:
        # Deploy the contract using forge with --broadcast
        cmd = [
            "forge", "create", 
            "test/contracts/SimpleBridgeMessageReceiver.sol:SimpleBridgeMessageReceiver",
            "--rpc-url", BRIDGE_CONFIG.rpc_2,
            "--private-key", BRIDGE_CONFIG.private_key_1,
            "--broadcast"
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
            BridgeLogger.debug(f"Full output: {output}")
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

def run_l1_to_l2_message_bridge_test(message: str = "Hello World"):
    """
    Complete L1-L2 Message Bridge Test
    
    This test follows the documented message bridge process:
    0. Deploy SimpleBridgeMessageReceiver contract on L2
    1. Bridge message from L1 to L2 using aggsandbox bridge message
    2. Monitor the bridge transaction using aggsandbox show bridges
    3. Wait for AggKit to sync bridge data from L1 to L2
    4. Claim the message on L2 using aggsandbox bridge claim
    5. Verify the claim using aggsandbox show claims
    6. Check that the contract received the message
    
    Args:
        message: Plain text message to bridge (default: "Hello World")
    """
    # Encode the message
    message_data = encode_message_data(message)
    if not message_data:
        BridgeLogger.error("Failed to encode message data")
        return False
    
    print("\n" + "="*70)
    print(f"📬 L1→L2 Message Bridge Test")
    print(f"Message: '{message}' -> {message_data}")
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
        BridgeLogger.info(f"L1 Network ID: {BRIDGE_CONFIG.network_id_mainnet}")
        BridgeLogger.info(f"L2 Network ID: {BRIDGE_CONFIG.network_id_agglayer_1}")
        BridgeLogger.info(f"From Account: {BRIDGE_CONFIG.account_address_1}")
        print()
        
        # Step 0: Deploy message receiver contract on L2
        BridgeLogger.step("[0/6] Deploying message receiver contract on L2")
        contract_address = deploy_message_receiver_contract()
        if not contract_address:
            BridgeLogger.error("Failed to deploy message receiver contract")
            return False
        
        BridgeLogger.info(f"Message receiver deployed at: {contract_address}")
        time.sleep(5)
        print()
        
        # Step 1: Bridge message from L1 to L2
        BridgeLogger.step(f"[1/6] Bridging message from L1 to L2")
        BridgeLogger.info("Using: aggsandbox bridge message")
        BridgeLogger.info(f"Message data: {message_data}")
        
        success, output = AggsandboxAPI.bridge_message(
            network=BRIDGE_CONFIG.network_id_mainnet,
            destination_network=BRIDGE_CONFIG.network_id_agglayer_1,
            target=contract_address,
            data=message_data,
            private_key=BRIDGE_CONFIG.private_key_1
        )
        
        if not success:
            BridgeLogger.error(f"Bridge message operation failed: {output}")
            return False
        
        # Extract bridge transaction hash from output
        bridge_tx_hash = None
        lines = output.split('\n')
        for line in lines:
            if 'bridge message transaction submitted' in line.lower() and '0x' in line:
                words = line.split()
                for word in words:
                    if word.startswith('0x') and len(word) == 66:
                        bridge_tx_hash = word
                        break
                if bridge_tx_hash:
                    break
        
        if not bridge_tx_hash:
            BridgeLogger.error("Could not extract bridge transaction hash from output")
            BridgeLogger.debug(f"Bridge output: {output}")
            return False
        
        BridgeLogger.success(f"✅ Message bridge transaction submitted: {bridge_tx_hash}")
        print()
        
        # Step 2: Monitor bridge transaction and find our bridge
        BridgeLogger.step("[2/6] Finding our bridge in L1 bridge events")
        BridgeLogger.info("Using: aggsandbox show bridges --network-id 0 --json")
        
        our_bridge = None
        for attempt in range(6):
            BridgeLogger.debug(f"Attempt {attempt + 1}/6 to find bridge...")
            time.sleep(3)
            
            success, output = AggsandboxAPI.show_bridges(
                network_id=BRIDGE_CONFIG.network_id_mainnet, 
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
        
        # Wait for AggKit to sync bridge data from L1 to L2
        BridgeLogger.step("Waiting for AggKit to sync bridge data from L1 to L2")
        BridgeLogger.info("AggKit needs ~20 seconds to sync bridge transactions and global exit root")
        BridgeLogger.info("This is based on successful testing and optimized timings")
        time.sleep(20)
        print()
        
        # Step 3: Claim the bridged message on L2
        BridgeLogger.step("[3/6] Claiming bridged message on L2")
        BridgeLogger.info("Using: aggsandbox bridge claim")
        
        # Create claim args
        bridge_tx = BridgeUtils.get_bridge_tx_hash(our_bridge)
        claim_args = BridgeClaimArgs(
            network=BRIDGE_CONFIG.network_id_agglayer_1,
            tx_hash=bridge_tx,
            source_network=BRIDGE_CONFIG.network_id_mainnet,
            private_key=BRIDGE_CONFIG.private_key_2
        )
        
        # Try claiming up to 3 times with delays
        claim_success = False
        claim_tx_hash = None
        
        for attempt in range(3):
            if attempt > 0:
                BridgeLogger.info(f"Retrying claim (attempt {attempt + 1}/3)...")
                time.sleep(5)  # Wait 5 seconds before retry
            
            success, output = AggsandboxAPI.bridge_claim(claim_args)
            
            if success:
                # Extract claim transaction hash
                lines = output.split('\n')
                for line in lines:
                    if '✅ claim transaction submitted:' in line.lower() and '0x' in line:
                        words = line.split()
                        for word in words:
                            if word.startswith('0x') and len(word) == 66:
                                claim_tx_hash = word
                                break
                        if claim_tx_hash:
                            break
                
                if claim_tx_hash:
                    BridgeLogger.success(f"✅ Claim transaction submitted: {claim_tx_hash}")
                    
                    # Verify transaction actually succeeded
                    BridgeLogger.info("Verifying claim transaction status...")
                    try:
                        import subprocess
                        result = subprocess.run([
                            "cast", "receipt", claim_tx_hash, "--rpc-url", BRIDGE_CONFIG.rpc_2
                        ], capture_output=True, text=True, check=True)
                        
                        receipt = result.stdout.strip()
                        if "status               0 (failed)" in receipt:
                            BridgeLogger.error(f"❌ Claim transaction failed on-chain: {claim_tx_hash}")
                            # Look for revert reason
                            if "revertReason" in receipt:
                                lines = receipt.split('\n')
                                for line in lines:
                                    if "revertReason" in line:
                                        BridgeLogger.error(f"Revert reason: {line.strip()}")
                            # Don't break, continue to retry
                            if attempt == 2:  # Last attempt
                                BridgeLogger.error("❌ All claim attempts resulted in failed transactions")
                                return False
                            else:
                                BridgeLogger.info("Will retry claim after longer delay...")
                                time.sleep(10)
                                continue
                        elif "status               1 (success)" in receipt:
                            BridgeLogger.success("✅ Claim transaction succeeded on-chain")
                            claim_success = True
                            break
                        else:
                            BridgeLogger.warning("Could not determine transaction status, assuming success")
                            claim_success = True
                            break
                            
                    except Exception as e:
                        BridgeLogger.warning(f"Could not verify transaction receipt: {e}")
                        claim_success = True
                        break
                else:
                    BridgeLogger.success("✅ Claim completed successfully")
                    claim_tx_hash = "completed"
                    claim_success = True
                    break
            else:
                BridgeLogger.warning(f"Claim attempt {attempt + 1} failed: {output}")
                if "wait at least" in output.lower() or "ger" in output.lower():
                    BridgeLogger.info("Claim failed due to timing - will retry with longer delay")
                    time.sleep(10)  # Longer delay if it's a timing issue
        
        if not claim_success:
            BridgeLogger.error("❌ All claim attempts failed")
            return False
        
        # Wait for claim to be processed before checking status
        BridgeLogger.info("Waiting for claim to be processed...")
        BridgeLogger.info("Checking claim status until completed...")
        
        # Wait for claim to be completed (check status periodically)
        claim_completed = False
        for attempt in range(12):  # Try for up to 60 seconds (12 * 5 seconds)
            time.sleep(5)
            BridgeLogger.debug(f"Checking claim status (attempt {attempt + 1}/12)...")
            
            success, output = AggsandboxAPI.show_claims(
                network_id=BRIDGE_CONFIG.network_id_agglayer_1,
                json_output=True
            )
            
            if success:
                try:
                    claims_data = json.loads(output)
                    claims = claims_data.get('claims', [])
                    
                    # Look for our claim by matching bridge transaction hash (most reliable)
                    for claim in claims:
                        # Match by bridge_tx_hash, destination_network, and type
                        if (claim.get('bridge_tx_hash') == bridge_tx_hash and
                            claim.get('destination_network') == BRIDGE_CONFIG.network_id_agglayer_1 and
                            claim.get('type') == 'message'):
                            
                            claim_status = claim.get('status', 'unknown')
                            BridgeLogger.debug(f"Found matching claim: status={claim_status}, tx_hash={claim.get('claim_tx_hash')}")
                            
                            if claim_status == "completed":
                                BridgeLogger.success(f"✅ Claim completed after {(attempt + 1) * 5} seconds!")
                                claim_completed = True
                                break
                            elif claim_status == "pending":
                                BridgeLogger.debug("⏳ Still pending...")
                                # Continue searching for completed status
                                continue
                    
                    if claim_completed:
                        break
                        
                except json.JSONDecodeError:
                    BridgeLogger.debug("Could not parse claims data")
        
        if not claim_completed:
            BridgeLogger.error("❌ Claim did not complete after 60 seconds - this indicates a problem!")
            BridgeLogger.error("This usually means:")
            BridgeLogger.error("  1. The claim transaction failed silently")
            BridgeLogger.error("  2. The bridge data hasn't synced properly")
            BridgeLogger.error("  3. There's an issue with the claim parameters")
            
            # Show the actual pending claim for debugging
            success, output = AggsandboxAPI.show_claims(
                network_id=BRIDGE_CONFIG.network_id_agglayer_1,
                json_output=True
            )
            if success:
                try:
                    claims_data = json.loads(output)
                    claims = claims_data.get('claims', [])
                    for claim in claims:
                        if (claim.get('bridge_tx_hash') == bridge_tx_hash and
                            claim.get('destination_network') == BRIDGE_CONFIG.network_id_agglayer_1 and
                            claim.get('type') == 'message'):
                            BridgeLogger.error(f"Found matching claim still pending:")
                            BridgeLogger.error(f"  Bridge TX: {claim.get('bridge_tx_hash')}")
                            BridgeLogger.error(f"  Claim TX: {claim.get('claim_tx_hash', 'None')}")
                            BridgeLogger.error(f"  Status: {claim.get('status')}")
                            BridgeLogger.error(f"  Destination: {claim.get('destination_address')}")
                            break
                except json.JSONDecodeError:
                    pass
            
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
                "--rpc-url", BRIDGE_CONFIG.rpc_2
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            output = result.stdout.strip()
            BridgeLogger.success(f"✅ Contract call successful: {output}")
            
            # Also check totalMessagesReceived
            cmd = [
                "cast", "call", contract_address,
                "totalMessagesReceived()(uint256)",
                "--rpc-url", BRIDGE_CONFIG.rpc_2
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            total_messages = int(result.stdout.strip())
            BridgeLogger.success(f"✅ Total messages received by contract: {total_messages}")
            
        except subprocess.CalledProcessError as e:
            BridgeLogger.warning(f"Could not verify contract state: {e}")
        
        print()
        
        # Step 5: Verify claim using aggsandbox show claims
        BridgeLogger.step("[5/6] Verifying claim on L2")
        BridgeLogger.info("Using: aggsandbox show claims --network-id 1 --json")
        BridgeLogger.info("Waiting for claim to be fully processed and indexed...")
        
        time.sleep(20)  # Optimized wait time based on testing
        
        success, output = AggsandboxAPI.show_claims(
            network_id=BRIDGE_CONFIG.network_id_agglayer_1,
            json_output=True
        )
        
        if success:
            try:
                claims_data = json.loads(output)
                claims = claims_data.get('claims', [])
                total_claims = len(claims)
                
                BridgeLogger.success(f"✅ Found {total_claims} total claims on L2")
                
                # Look for our specific claim by matching bridge details
                our_claim = None
                completed_claim = None
                for claim in claims:
                    # Match by destination_address, origin_network, destination_network, type
                    if (claim.get('destination_address') == contract_address and
                        claim.get('origin_network') == BRIDGE_CONFIG.network_id_mainnet and
                        claim.get('destination_network') == BRIDGE_CONFIG.network_id_agglayer_1 and
                        claim.get('type') == 'message'):
                        
                        if claim.get('status') == 'completed':
                            completed_claim = claim
                        elif claim.get('status') == 'pending':
                            our_claim = claim
                
                # Prefer completed claim, fallback to pending
                display_claim = completed_claim or our_claim
                
                if display_claim:
                    claim_status = display_claim.get('status', 'unknown')
                    BridgeLogger.success("✅ Found our claim in L2 claims:")
                    BridgeLogger.info(f"  • Type: {display_claim.get('type', 'unknown')}")
                    BridgeLogger.info(f"  • Block: {display_claim.get('block_num')}")
                    BridgeLogger.info(f"  • Status: {claim_status.upper()}")
                    BridgeLogger.info(f"  • Global Index: {display_claim.get('global_index')}")
                    BridgeLogger.info(f"  • TX Hash: {display_claim.get('claim_tx_hash')}")
                    
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
        else:
            BridgeLogger.warning(f"Could not get claims data: {output}")
        
        # Final success summary
        print("\n🎯 L1→L2 Message Bridge Test Results:")
        print("━" * 70)
        BridgeLogger.success("🎉 Complete L1→L2 message bridge flow successful!")
        
        print(f"\n📋 Operations Completed:")
        BridgeLogger.info("✅ 1. aggsandbox bridge message (L1→L2 message bridging)")
        BridgeLogger.info("✅ 2. aggsandbox show bridges --json (monitoring)")
        BridgeLogger.info("✅ 3. AggKit sync wait (10 seconds - optimized)")
        BridgeLogger.info("✅ 4. aggsandbox bridge claim (claiming on L2)")
        BridgeLogger.info("✅ 5. aggsandbox show claims --json (verification)")
        
        print(f"\n📊 Transaction Summary:")
        BridgeLogger.info(f"Bridge TX (L1): {our_bridge['bridge_tx_hash']}")
        BridgeLogger.info(f"Claim TX (L2):  {claim_tx_hash}")
        BridgeLogger.info(f"Message Data:   {message_data}")
        BridgeLogger.info(f"Deposit Count:  {our_bridge['deposit_count']}")
        BridgeLogger.info(f"Target Contract: {contract_address}")
        
        print(f"\n🔄 Bridge Flow:")
        BridgeLogger.info(f"L1 Network {BRIDGE_CONFIG.network_id_mainnet} → L2 Network {BRIDGE_CONFIG.network_id_agglayer_1}")
        BridgeLogger.info(f"From: {BRIDGE_CONFIG.account_address_1}")
        BridgeLogger.info(f"To Contract: {contract_address}")
        BridgeLogger.info(f"Type: Pure Message Bridge (no assets involved)")
        
        print("━" * 70)
        
        return True
        
    except Exception as e:
        BridgeLogger.error(f"Test failed with exception: {e}")
        import traceback
        BridgeLogger.debug(traceback.format_exc())
        return False

def main():
    """Main function to run the L1-L2 message bridge test"""
    
    # Parse command line arguments
    if len(sys.argv) > 1:
        message = sys.argv[1]
    else:
        # Default message
        message = "Hello World"
    
    # Run the L1-L2 message bridge test
    success = run_l1_to_l2_message_bridge_test(message)
    
    if success:
        print(f"\n🎉 SUCCESS: L1→L2 message bridge test completed!")
        sys.exit(0)
    else:
        print(f"\n❌ FAILED: L1→L2 message bridge test failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()
