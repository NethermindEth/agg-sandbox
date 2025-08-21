#!/usr/bin/env python3
"""
Test script to verify the Python bridge library works correctly
"""

import sys
import os

# Add the lib directory to Python path
sys.path.insert(0, os.path.dirname(__file__))

def test_library_imports():
    """Test that all modules can be imported"""
    print("🔄 Testing Python bridge library imports...")
    
    try:
        # Test core library
        from bridge_lib import BridgeLogger, AggsandboxAPI, BridgeUtils, BRIDGE_CONFIG
        print("✅ Core library imported successfully")
        
        # Test bridge asset module
        from bridge_asset import BridgeAsset
        print("✅ Bridge asset module imported successfully")
        
        # Test bridge message module
        from bridge_message import BridgeMessage
        print("✅ Bridge message module imported successfully")
        
        # Test claim asset module
        from claim_asset import ClaimAsset
        print("✅ Claim asset module imported successfully")
        
        # Test claim message module
        from claim_message import ClaimMessage
        print("✅ Claim message module imported successfully")
        
        # Test bridge and call module
        from bridge_and_call import BridgeAndCall
        print("✅ Bridge and call module imported successfully")
        
        # Test claim bridge and call module
        from claim_bridge_and_call import ClaimBridgeAndCall
        print("✅ Claim bridge and call module imported successfully")
        
        return True
        
    except ImportError as e:
        print(f"❌ Import failed: {e}")
        return False

def test_environment_loading():
    """Test environment loading"""
    print("\n🔄 Testing environment loading...")
    
    try:
        from bridge_lib import BridgeEnvironment, BridgeLogger
        
        # Test environment loading
        config = BridgeEnvironment.load_environment()
        print("✅ Environment loaded successfully")
        
        # Test configuration
        print(f"✅ L1 Token: {config.agg_erc20_l1}")
        print(f"✅ Account 1: {config.account_address_1}")
        print(f"✅ Account 2: {config.account_address_2}")
        print(f"✅ Network IDs: L1={config.network_id_mainnet}, L2={config.network_id_agglayer_1}")
        
        return True
        
    except Exception as e:
        print(f"❌ Environment loading failed: {e}")
        return False

def test_logging():
    """Test logging functions"""
    print("\n🔄 Testing logging functions...")
    
    try:
        from bridge_lib import BridgeLogger
        
        BridgeLogger.step("Testing step message")
        BridgeLogger.info("Testing info message")
        BridgeLogger.success("Testing success message")
        BridgeLogger.warning("Testing warning message")
        BridgeLogger.debug("Testing debug message (only shows if DEBUG=1)")
        
        print("✅ All logging functions work correctly")
        return True
        
    except Exception as e:
        print(f"❌ Logging test failed: {e}")
        return False

def test_api_functions():
    """Test API functions"""
    print("\n🔄 Testing API functions...")
    
    try:
        from bridge_lib import AggsandboxAPI, BridgeEnvironment
        
        # Test sandbox status
        if BridgeEnvironment.validate_sandbox_status():
            print("✅ Sandbox status validation works")
        else:
            print("⚠️ Sandbox not running (expected in some environments)")
        
        # Test bridge data retrieval
        bridge_data = AggsandboxAPI.get_bridges(0)
        if bridge_data:
            bridge_count = len(bridge_data.get('bridges', []))
            print(f"✅ Bridge data retrieval works ({bridge_count} bridges found)")
        else:
            print("⚠️ Could not retrieve bridge data (may be normal if no bridges)")
        
        return True
        
    except Exception as e:
        print(f"❌ API test failed: {e}")
        return False

def main():
    """Run all tests"""
    print("="*60)
    print("🐍 Python Bridge Library Test Suite")
    print("="*60)
    
    tests = [
        ("Library Imports", test_library_imports),
        ("Environment Loading", test_environment_loading),
        ("Logging Functions", test_logging),
        ("API Functions", test_api_functions)
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        print(f"\n📋 Running {test_name}...")
        if test_func():
            passed += 1
            print(f"✅ {test_name}: PASSED")
        else:
            print(f"❌ {test_name}: FAILED")
    
    print("\n" + "="*60)
    print(f"🎯 Test Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed! Python bridge library is working correctly.")
        print("\n📝 Available modules:")
        print("  • BridgeAsset - Asset bridging operations")
        print("  • BridgeMessage - Message bridging operations")
        print("  • ClaimAsset - Asset claiming operations")
        print("  • ClaimMessage - Message claiming operations")
        print("  • BridgeAndCall - Bridge and call operations")
        print("  • ClaimBridgeAndCall - Bridge and call claiming operations")
        print("\n💡 Usage example:")
        print("  from bridge_lib import init_bridge_environment, BRIDGE_CONFIG")
        print("  from bridge_asset import BridgeAsset")
        print("  bridge_tx, claim_tx = BridgeAsset.execute_l1_to_l2_bridge(...)")
        return 0
    else:
        print("❌ Some tests failed. Check the output above for details.")
        return 1

if __name__ == "__main__":
    sys.exit(main())
