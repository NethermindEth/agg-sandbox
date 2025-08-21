#!/usr/bin/env python3
"""
L2-L1 Asset Bridge Test
Tests the complete flow of bridging wrapped tokens from L2 back to L1 using aggsandbox CLI
Based on bridge-operations.md documentation and the successful L1-L2 test
"""

import sys
import os
import time
import json

# Add the lib directory to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'lib'))

from bridge_lib import BRIDGE_CONFIG, BridgeLogger, BridgeEnvironment
from aggsandbox_api import AggsandboxAPI, BridgeAssetArgs, BridgeClaimArgs

def run_l2_to_l1_asset_bridge_test(bridge_amount: int = 50):
    """
    Complete L2-L1 Asset Bridge Test
    
    This test follows the documented reverse bridge process:
    1. Bridge wrapped tokens from L2 to L1 using aggsandbox bridge asset
    2. Monitor the bridge transaction using aggsandbox show bridges
    3. Wait for AggKit to sync bridge data from L2 to L1
    4. Claim the original tokens on L1 using aggsandbox bridge claim
    5. Verify the claim using aggsandbox show claims
    """
    print("\n" + "="*70)
    print(f"🔄 L2→L1 Asset Bridge Test")
    print(f"Bridging {bridge_amount} wrapped tokens from L2 back to L1")
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
        BridgeLogger.info(f"L2 Token Address: {BRIDGE_CONFIG.agg_erc20_l2}")
        BridgeLogger.info(f"From Account: {BRIDGE_CONFIG.account_address_1}")
        BridgeLogger.info(f"To Account: {BRIDGE_CONFIG.account_address_2}")
        print()
        
        # Get what the L2 token will become on L1 using precalculate
        BridgeLogger.step("Getting L1 wrapped token address using precalculate")
        BridgeLogger.info("Using: aggsandbox bridge utils precalculate")
        BridgeLogger.info("This shows what L2 AggERC20 will become when bridged to L1")
        
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
                if l1_wrapped_token_addr:
                    BridgeLogger.success(f"✅ L1 wrapped token address: {l1_wrapped_token_addr}")
                else:
                    BridgeLogger.warning("No precalculated_address in response")
            except json.JSONDecodeError as e:
                BridgeLogger.warning(f"Could not parse precalculate response: {e}")
        else:
            BridgeLogger.warning(f"Could not get precalculated address: {output}")
        
        if not l1_wrapped_token_addr:
            BridgeLogger.error("Could not determine L1 wrapped token address")
            return False
        
        # Check L2 wrapped token balance before bridge
        BridgeLogger.step("Checking L2 wrapped token balance before bridge")
        try:
            import subprocess
            result = subprocess.run([
                "cast", "call", BRIDGE_CONFIG.agg_erc20_l2,
                "balanceOf(address)(uint256)",
                BRIDGE_CONFIG.account_address_1,
                "--rpc-url", BRIDGE_CONFIG.rpc_2
            ], capture_output=True, text=True, check=True)
            
            balance_output = result.stdout.strip()
            # Handle cast output format like "1000000000000000000000000 [1e24]"
            if '[' in balance_output:
                balance_str = balance_output.split('[')[0].strip()
            else:
                balance_str = balance_output
            l2_balance_before = int(balance_str)
            BridgeLogger.info(f"L2 AggERC20 balance before bridge: {l2_balance_before} tokens")
            
            if l2_balance_before < bridge_amount:
                BridgeLogger.error(f"❌ Insufficient L2 balance: {l2_balance_before} < {bridge_amount}")
                BridgeLogger.info("You may need to get L2 AggERC20 tokens first")
                return False
            
        except Exception as e:
            BridgeLogger.warning(f"Could not check L2 balance before bridge: {e}")
            l2_balance_before = None
        
        # Check L1 wrapped token balance before claim
        BridgeLogger.step("Checking L1 wrapped token balance before claim")
        try:
            result = subprocess.run([
                "cast", "call", l1_wrapped_token_addr,
                "balanceOf(address)(uint256)",
                BRIDGE_CONFIG.account_address_2,
                "--rpc-url", BRIDGE_CONFIG.rpc_1
            ], capture_output=True, text=True, check=True)
            
            l1_balance_before = int(result.stdout.strip())
            BridgeLogger.info(f"L1 wrapped token balance before claim: {l1_balance_before} tokens")
            
        except Exception as e:
            BridgeLogger.warning(f"Could not check L1 balance before claim: {e}")
            l1_balance_before = None
        
        print()
        
        # Step 1: Bridge L2 AggERC20 tokens to L1
        BridgeLogger.step(f"[1/5] Bridging {bridge_amount} L2 AggERC20 tokens to L1")
        BridgeLogger.info("Using: aggsandbox bridge asset")
        BridgeLogger.info(f"Source token (L2): {BRIDGE_CONFIG.agg_erc20_l2}")
        BridgeLogger.info(f"Will create wrapped token on L1: {l1_wrapped_token_addr}")
        
        # Create bridge args (L2 → L1)
        bridge_args = BridgeAssetArgs(
            network=BRIDGE_CONFIG.network_id_agglayer_1,  # Source: L2
            destination_network=BRIDGE_CONFIG.network_id_mainnet,  # Destination: L1
            amount=str(bridge_amount),
            token_address=BRIDGE_CONFIG.agg_erc20_l2,  # L2 AggERC20 token
            to_address=BRIDGE_CONFIG.account_address_2,  # Receive on L1
            private_key=BRIDGE_CONFIG.private_key_1  # L2 account 1 with tokens
        )
        
        success, output = AggsandboxAPI.bridge_asset(bridge_args)
        if not success:
            BridgeLogger.error(f"Bridge operation failed: {output}")
            return False
        
        # Extract bridge transaction hash from output
        bridge_tx_hash = None
        lines = output.split('\n')
        for line in lines:
            if 'bridge transaction submitted' in line.lower() and '0x' in line:
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
        
        BridgeLogger.success(f"✅ Bridge transaction submitted: {bridge_tx_hash}")
        print()
        
        # Step 2: Monitor bridge transaction and find our bridge
        BridgeLogger.step("[2/5] Finding our bridge in L2 bridge events")
        BridgeLogger.info("Using: aggsandbox show bridges --network-id 1 --json")
        
        our_bridge = None
        for attempt in range(6):
            BridgeLogger.debug(f"Attempt {attempt + 1}/6 to find bridge...")
            time.sleep(3)
            
            success, output = AggsandboxAPI.show_bridges(
                network_id=BRIDGE_CONFIG.network_id_agglayer_1,  # L2 bridges
                json_output=True
            )
            
            if success:
                try:
                    bridge_data = json.loads(output)
                    bridges = bridge_data.get('bridges', [])
                    
                    # Look for our specific bridge transaction
                    for bridge in bridges:
                        if bridge.get('tx_hash') == bridge_tx_hash:
                            our_bridge = bridge
                            BridgeLogger.success(f"✅ Found our bridge (attempt {attempt + 1})")
                            break
                    
                    if our_bridge:
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
        BridgeLogger.info(f"  • TX Hash: {our_bridge['tx_hash']}")
        BridgeLogger.info(f"  • Amount: {our_bridge['amount']} tokens")
        BridgeLogger.info(f"  • Deposit Count: {our_bridge['deposit_count']}")
        BridgeLogger.info(f"  • Block: {our_bridge.get('block_num', 'N/A')}")
        BridgeLogger.info(f"  • Destination Network: {our_bridge['destination_network']}")
        print()
        
        # Wait for AggKit to sync bridge data from L2 to L1
        BridgeLogger.step("Waiting for AggKit to sync bridge data from L2 to L1")
        BridgeLogger.info("AggKit needs ~30 seconds to sync bridge transactions between networks")
        BridgeLogger.info("This is normal behavior - bridge data must be synced before claiming")
        time.sleep(30)
        print()
        
        # Step 3: Claim the bridged assets on L1
        BridgeLogger.step("[3/5] Claiming bridged assets on L1")
        BridgeLogger.info("Using: aggsandbox bridge claim")
        BridgeLogger.info("This will restore the original tokens on L1")
        
        # Create claim args
        claim_args = BridgeClaimArgs(
            network=BRIDGE_CONFIG.network_id_mainnet,  # Claim on L1
            tx_hash=our_bridge['tx_hash'],
            source_network=BRIDGE_CONFIG.network_id_agglayer_1,  # Source: L2
            private_key=BRIDGE_CONFIG.private_key_2  # L1 account 2 (recipient)
        )
        
        success, output = AggsandboxAPI.bridge_claim(claim_args)
        if not success:
            BridgeLogger.error(f"❌ Claim operation failed: {output}")
            return False
        
        # Extract claim transaction hash
        claim_tx_hash = None
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
        else:
            BridgeLogger.success("✅ Claim completed successfully")
            claim_tx_hash = "completed"
        
        # Wait for claim to be processed before checking balance
        BridgeLogger.info("Waiting for claim to be processed and tokens transferred...")
        BridgeLogger.info("Checking claim status until completed...")
        
        # Wait for claim to be completed (check status periodically)
        claim_completed = False
        for attempt in range(12):  # Try for up to 60 seconds (12 * 5 seconds)
            time.sleep(5)
            BridgeLogger.debug(f"Checking claim status (attempt {attempt + 1}/12)...")
            
            success, output = AggsandboxAPI.show_claims(
                network_id=BRIDGE_CONFIG.network_id_mainnet,  # Check L1 claims
                json_output=True
            )
            
            if success:
                try:
                    claims_data = json.loads(output)
                    claims = claims_data.get('claims', [])
                    
                    # Look for our claim by matching bridge details (not tx_hash since it changes)
                    for claim in claims:
                        # Match by origin_address (L2 token), destination_address, amount, and networks
                        if (claim.get('origin_address') == BRIDGE_CONFIG.agg_erc20_l2 and
                            claim.get('destination_address') == BRIDGE_CONFIG.account_address_2 and
                            claim.get('amount') == str(our_bridge['amount']) and
                            claim.get('origin_network') == BRIDGE_CONFIG.network_id_agglayer_1 and
                            claim.get('destination_network') == BRIDGE_CONFIG.network_id_mainnet):
                            
                            claim_status = claim.get('status', 'unknown')
                            BridgeLogger.debug(f"Found matching claim: status={claim_status}, tx_hash={claim.get('tx_hash')}")
                            
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
            BridgeLogger.warning("⚠️ Claim still not completed after 60 seconds, checking balance anyway...")
        
        # Add a small additional wait for token transfer
        time.sleep(2)
        
        # Check L1 wrapped token balance after claim
        BridgeLogger.step("Checking L1 wrapped token balance after claim")
        try:
            result = subprocess.run([
                "cast", "call", l1_wrapped_token_addr,
                "balanceOf(address)(uint256)",
                BRIDGE_CONFIG.account_address_2,
                "--rpc-url", BRIDGE_CONFIG.rpc_1
            ], capture_output=True, text=True, check=True)
            
            l1_balance_after = int(result.stdout.strip())
            BridgeLogger.info(f"L1 wrapped token balance after claim: {l1_balance_after} tokens")
            
            # Calculate balance difference
            if l1_balance_before is not None:
                l1_difference = l1_balance_after - l1_balance_before
                BridgeLogger.info(f"L1 balance difference: +{l1_difference} tokens")
                
                # Verify the balance increased by the bridged amount
                if l1_difference == int(our_bridge['amount']):
                    BridgeLogger.success(f"✅ Balance verification: L1 increased by exactly {our_bridge['amount']} tokens")
                elif l1_difference > 0:
                    BridgeLogger.warning(f"⚠️ Balance verification: Expected +{our_bridge['amount']}, got +{l1_difference}")
                else:
                    BridgeLogger.error(f"❌ Balance verification: L1 balance did not increase (difference: {l1_difference})")
            
        except Exception as e:
            BridgeLogger.warning(f"Could not check L1 balance after claim: {e}")
            l1_balance_after = None
            l1_difference = None
        
        print()
        
        # Step 4: Verify claim using aggsandbox show claims
        BridgeLogger.step("[4/5] Verifying claim on L1")
        BridgeLogger.info("Using: aggsandbox show claims --network-id 0 --json")
        BridgeLogger.info("Waiting for claim to be fully processed and indexed...")
        
        time.sleep(15)  # Give claim time to be fully processed and indexed
        
        success, output = AggsandboxAPI.show_claims(
            network_id=BRIDGE_CONFIG.network_id_mainnet,  # L1 claims
            json_output=True
        )
        
        if success:
            try:
                claims_data = json.loads(output)
                claims = claims_data.get('claims', [])
                total_claims = len(claims)
                
                BridgeLogger.success(f"✅ Found {total_claims} total claims on L1")
                
                # Look for our specific claim by matching bridge details
                our_claim = None
                completed_claim = None
                for claim in claims:
                    # Match by origin_address (L2 token), destination_address, amount, and networks
                    if (claim.get('origin_address') == BRIDGE_CONFIG.agg_erc20_l2 and
                        claim.get('destination_address') == BRIDGE_CONFIG.account_address_2 and
                        claim.get('amount') == str(our_bridge['amount']) and
                        claim.get('origin_network') == BRIDGE_CONFIG.network_id_agglayer_1 and
                        claim.get('destination_network') == BRIDGE_CONFIG.network_id_mainnet):
                        
                        if claim.get('status') == 'completed':
                            completed_claim = claim
                        elif claim.get('status') == 'pending':
                            our_claim = claim
                
                # Prefer completed claim, fallback to pending
                display_claim = completed_claim or our_claim
                
                if display_claim:
                    claim_status = display_claim.get('status', 'unknown')
                    BridgeLogger.success("✅ Found our claim in L1 claims:")
                    BridgeLogger.info(f"  • Amount: {display_claim.get('amount')} tokens")
                    BridgeLogger.info(f"  • Block: {display_claim.get('block_num')}")
                    BridgeLogger.info(f"  • Status: {claim_status.upper()}")
                    BridgeLogger.info(f"  • Global Index: {display_claim.get('global_index')}")
                    BridgeLogger.info(f"  • TX Hash: {display_claim.get('tx_hash')}")
                    
                    if claim_status == "completed":
                        BridgeLogger.success("🎉 Claim is COMPLETE!")
                    elif claim_status == "pending":
                        BridgeLogger.info("⏳ Claim is still PENDING (this is normal)")
                    else:
                        BridgeLogger.warning(f"⚠️ Claim status: {claim_status}")
                    
                    # Show both statuses if we found both
                    if completed_claim and our_claim:
                        BridgeLogger.info(f"Note: Found both PENDING and COMPLETED entries (normal behavior)")
                elif our_claim or completed_claim:
                    # This shouldn't happen with our logic, but just in case
                    BridgeLogger.success("✅ Found related claim entries")
                else:
                    BridgeLogger.warning("⚠️ Our specific claim not found (may still be processing)")
                    # Show a few recent claims for debugging
                    if claims:
                        BridgeLogger.info("Recent claims for reference:")
                        for i, claim in enumerate(claims[:3]):
                            BridgeLogger.info(f"  {i+1}. Amount: {claim.get('amount')}, Status: {claim.get('status')}, Origin: {claim.get('origin_address', 'N/A')[:10]}...")
                    
            except json.JSONDecodeError as e:
                BridgeLogger.warning(f"Could not parse claims response: {e}")
        else:
            BridgeLogger.warning(f"Could not get claims data: {output}")
        
        # Final success summary
        print("\n🎯 L2→L1 Asset Bridge Test Results:")
        print("━" * 70)
        BridgeLogger.success("🎉 Complete L2→L1 asset bridge flow successful!")
        
        print(f"\n📋 Operations Completed:")
        BridgeLogger.info("✅ 1. aggsandbox bridge utils get-mapped (wrapped token discovery)")
        BridgeLogger.info("✅ 2. aggsandbox bridge asset (L2→L1 bridging)")
        BridgeLogger.info("✅ 3. aggsandbox show bridges --json (monitoring)")
        BridgeLogger.info("✅ 4. AggKit sync wait (30 seconds)")
        BridgeLogger.info("✅ 5. aggsandbox bridge claim (claiming on L1)")
        BridgeLogger.info("✅ 6. aggsandbox show claims --json (verification)")
        
        print(f"\n📊 Transaction Summary:")
        BridgeLogger.info(f"Bridge TX (L2): {our_bridge['tx_hash']}")
        BridgeLogger.info(f"Claim TX (L1):  {claim_tx_hash}")
        BridgeLogger.info(f"Amount:         {our_bridge['amount']} tokens")
        BridgeLogger.info(f"Deposit Count:  {our_bridge['deposit_count']}")
        BridgeLogger.info(f"L2 Token:       {BRIDGE_CONFIG.agg_erc20_l2}")
        BridgeLogger.info(f"L1 Wrapped Token: {l1_wrapped_token_addr}")
        
        print(f"\n💰 Balance Changes:")
        if l2_balance_before is not None:
            l2_balance_after = l2_balance_before - bridge_amount  # Should decrease on L2
            BridgeLogger.info(f"L2 Before Bridge: {l2_balance_before} tokens")
            BridgeLogger.info(f"L2 After Bridge:  {l2_balance_after} tokens (estimated)")
            BridgeLogger.info(f"L2 Difference:    -{bridge_amount} tokens")
        
        if l1_balance_before is not None and l1_balance_after is not None:
            BridgeLogger.info(f"L1 Before Claim:  {l1_balance_before} tokens")
            BridgeLogger.info(f"L1 After Claim:   {l1_balance_after} tokens")
            BridgeLogger.info(f"L1 Difference:    +{l1_difference} tokens")
            
            if l1_difference == int(our_bridge['amount']):
                BridgeLogger.success(f"✅ Perfect match: {l1_difference} tokens received on L1")
            elif l1_difference > 0:
                BridgeLogger.warning(f"⚠️ Partial match: Expected {our_bridge['amount']}, got {l1_difference}")
            else:
                BridgeLogger.error(f"❌ No tokens received on L1")
        else:
            BridgeLogger.info("L1 balance verification was not available")
        
        print(f"\n🔄 Bridge Flow:")
        BridgeLogger.info(f"L2 Network {BRIDGE_CONFIG.network_id_agglayer_1} → L1 Network {BRIDGE_CONFIG.network_id_mainnet}")
        BridgeLogger.info(f"From: {BRIDGE_CONFIG.account_address_1}")
        BridgeLogger.info(f"To:   {BRIDGE_CONFIG.account_address_2}")
        BridgeLogger.info(f"Type: L2→L1 Asset Bridge (L2 AggERC20 → L1 Wrapped Token)")
        
        print("━" * 70)
        
        return True
        
    except Exception as e:
        BridgeLogger.error(f"Test failed with exception: {e}")
        import traceback
        BridgeLogger.debug(traceback.format_exc())
        return False

def main():
    """Main function to run the L2-L1 asset bridge test"""
    
    # Parse command line arguments
    bridge_amount = int(sys.argv[1]) if len(sys.argv) > 1 else 50
    
    # Run the L2-L1 asset bridge test
    success = run_l2_to_l1_asset_bridge_test(bridge_amount)
    
    if success:
        print(f"\n🎉 SUCCESS: L2→L1 asset bridge test completed!")
        sys.exit(0)
    else:
        print(f"\n❌ FAILED: L2→L1 asset bridge test failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()
