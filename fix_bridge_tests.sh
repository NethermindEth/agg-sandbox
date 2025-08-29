#!/bin/bash

# Script to fix bridge test timing and add transaction verification

echo "🔧 Fixing bridge test synchronization timing and transaction verification..."

# List of test files to update
test_files=(
    "test/L1-L2/test_bridge_and_call_and_claim.py"
    "test/L2-L1/test_bridge_asset_and_claim.py"
    "test/L2-L1/test_bridge_message_and_claim.py"
    "test/L2-L1/test_bridge_and_call_and_claim.py"
    "test/L2-L2/test_bridge_asset_and_claim.py"
    "test/L2-L2/test_bridge_message_and_claim.py"
    "test/L2-L2/test_bridge_and_call_and_claim.py"
)

for file in "${test_files[@]}"; do
    if [ -f "$file" ]; then
        echo "Updating: $file"
        
        # Update wait time from 10 to 15 seconds
        sed -i 's/AggKit needs ~10 seconds to sync bridge transactions between networks/AggKit needs ~15 seconds to sync bridge transactions and global exit root/g' "$file"
        sed -i 's/time\.sleep(10)/time.sleep(15)/g' "$file"
        
        echo "  ✅ Updated sync timing to 15 seconds"
    else
        echo "  ⚠️  File not found: $file"
    fi
done

echo ""
echo "✅ Bridge test timing fixes completed!"
echo "📋 Manual task remaining: Add transaction receipt verification to claim operations"
echo "   This requires careful placement in each test's claim logic."