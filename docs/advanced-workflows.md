# Advanced Workflows

Complex cross-chain scenarios and advanced bridging patterns using the Agglayer Sandbox.

## Multi-Chain DeFi Scenarios

### Cross-Chain Liquidity Provision

Bridge tokens and immediately provide liquidity on the destination chain:

```bash
# 1. Prepare liquidity provision call data
LIQUIDITY_DATA=$(cast calldata "addLiquidity(address,address,uint256,uint256,uint256,uint256,address,uint256)" \
  $TOKEN_A $TOKEN_B 1000000000000000000 1000000000000000000 0 0 $ACCOUNT_ADDRESS_1 $(date +%s))

# 2. Bridge tokens with liquidity provision
aggsandbox bridge bridge-and-call \
  --network-id 0 \
  --destination-network-id 1 \
  --token $AGG_ERC20_L1 \
  --amount 10 \
  --target $DEX_ROUTER_L2 \
  --data $LIQUIDITY_DATA \
  --fallback $ACCOUNT_ADDRESS_1

# 3. Execute two-phase claiming
aggsandbox bridge claim --network-id 1 --tx-hash <hash> --source-network-id 0 --deposit-count 0
aggsandbox bridge claim --network-id 1 --tx-hash <hash> --source-network-id 0 --deposit-count 1
```

### Cross-Chain Yield Farming

Bridge and immediately stake tokens in a yield farm:

```bash
# 1. Get staking contract call data
STAKE_DATA=$(cast calldata "stake(uint256)" 5000000000000000000)

# 2. Pre-calculate L2 token address
L2_TOKEN=$(aggsandbox bridge utils precalculate \
  --network-id 1 \
  --origin-network 0 \
  --origin-token $YIELD_TOKEN_L1 \
  --json | jq -r '.precalculated_address')

# 3. Bridge and stake in one operation
aggsandbox bridge bridge-and-call \
  --network-id 0 \
  --destination-network-id 1 \
  --token $YIELD_TOKEN_L1 \
  --amount 5 \
  --target $STAKING_CONTRACT_L2 \
  --data $STAKE_DATA \
  --fallback $ACCOUNT_ADDRESS_1
```

## Multi-L2 Triangular Bridging

Complex routing through multiple L2 chains:

### L1 → L2-1 → L2-2 Flow

```bash
# 1. Start multi-L2 environment
aggsandbox start --multi-l2 --detach

# 2. Bridge from L1 to L2-1
aggsandbox bridge asset \
  --network-id 0 \
  --destination-network-id 1 \
  --amount 2.0 \
  --token-address 0x0000000000000000000000000000000000000000

# 3. Claim on L2-1
aggsandbox bridge claim \
  --network-id 1 \
  --tx-hash <l1_to_l2_hash> \
  --source-network-id 0

# 4. Bridge from L2-1 to L2-2
aggsandbox bridge asset \
  --network-id 1 \
  --destination-network 2 \
  --amount 1.0 \
  --token-address 0x0000000000000000000000000000000000000000

# 5. Claim on L2-2
aggsandbox bridge claim \
  --network 2 \
  --tx-hash <l2_to_l2_hash> \
  --source-network 1
```

### Monitoring Multi-Chain State

```bash
# Monitor all networks simultaneously
aggsandbox events --network-id 0 --blocks 5 &
aggsandbox events --network-id 1 --blocks 5 &
aggsandbox events --network-id 2 --blocks 5 &

# Check bridge state across all networks
aggsandbox show bridges --network-id 0 --json > l1_bridges.json
aggsandbox show bridges --network-id 1 --json > l2_1_bridges.json
aggsandbox show bridges --network-id 2 --json > l2_2_bridges.json
```

## Batch Bridge Operations

### Sequential Asset Bridges

```bash
#!/bin/bash
# batch_bridge.sh - Bridge multiple assets sequentially

TOKENS=("0x0000000000000000000000000000000000000000" "$AGG_ERC20_L1" "$ANOTHER_TOKEN_L1")
AMOUNTS=("0.1" "100" "50")

for i in "${!TOKENS[@]}"; do
  echo "Bridging ${AMOUNTS[$i]} of ${TOKENS[$i]}"

  aggsandbox bridge asset \
    --network-id 0 \
    --destination-network-id 1 \
    --amount "${AMOUNTS[$i]}" \
    --token-address "${TOKENS[$i]}" \
    --to-address $ACCOUNT_ADDRESS_2

  # Wait for transaction confirmation
  sleep 5
done
```

### Automated Claim Processing

```bash
#!/bin/bash
# auto_claim.sh - Automatically claim all pending bridges

# Get all bridges for L1
BRIDGES=$(aggsandbox show bridges --network-id 0 --json)
TX_HASHES=$(echo $BRIDGES | jq -r '.bridges[].tx_hash')

for tx_hash in $TX_HASHES; do
  echo "Claiming bridge: $tx_hash"

  aggsandbox bridge claim \
    --network-id 1 \
    --tx-hash $tx_hash \
    --source-network-id 0

  sleep 3
done
```

## Fork Mode Advanced Scenarios

### Testing Against Mainnet State

```bash
# 1. Configure fork URLs for real networks
cat >> .env << EOF
FORK_URL_MAINNET=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY
FORK_URL_AGGLAYER_1=https://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY
EOF

# 2. Start in fork mode
aggsandbox start --fork --detach

# 3. Bridge real tokens with real state
aggsandbox bridge asset \
  --network-id 0 \
  --destination-network-id 1 \
  --amount 1000 \
  --token-address 0xA0b86a33E6776e39e6b37ddEC4F25B04Dd9Fc4DC  # Real USDC
```

### Fork Mode with Custom Block Numbers

```bash
# Fork from specific block heights
cat >> .env << EOF
FORK_BLOCK_MAINNET=18500000
FORK_BLOCK_AGGLAYER_1=50000000
EOF

# This ensures consistent state across test runs
aggsandbox start --fork --detach
```

## Complex Bridge-and-Call Patterns

### Multi-Step Contract Execution

```bash
# 1. Encode complex multi-step operation
MULTI_STEP_DATA=$(cast calldata "executeMultiStep(address[],bytes[])" \
  "[$CONTRACT_1,$CONTRACT_2,$CONTRACT_3]" \
  "[$CALL_DATA_1,$CALL_DATA_2,$CALL_DATA_3]")

# 2. Execute bridge-and-call with multi-step
aggsandbox bridge bridge-and-call \
  --network-id 0 \
  --destination-network-id 1 \
  --token $AGG_ERC20_L1 \
  --amount 20 \
  --target $MULTI_STEP_EXECUTOR_L2 \
  --data $MULTI_STEP_DATA \
  --fallback $ACCOUNT_ADDRESS_1
```

### Conditional Bridge Execution

```bash
# 1. Encode conditional logic
CONDITIONAL_DATA=$(cast calldata "executeIfCondition(uint256,bytes)" \
  1000000000000000000 \
  $SECONDARY_CALL_DATA)

# 2. Bridge with conditional execution
aggsandbox bridge bridge-and-call \
  --network-id 0 \
  --destination-network-id 1 \
  --token $AGG_ERC20_L1 \
  --amount 15 \
  --target $CONDITIONAL_EXECUTOR_L2 \
  --data $CONDITIONAL_DATA \
  --fallback $ACCOUNT_ADDRESS_1
```

## Automated Trading Strategies

### Cross-Chain Arbitrage

```bash
#!/bin/bash
# arbitrage.sh - Cross-chain arbitrage bot

# 1. Check price differences
L1_PRICE=$(curl -s "http://localhost:8545" -X POST -H "Content-Type: application/json" \
  --data '{"method":"eth_call","params":[{"to":"'$DEX_L1'","data":"'$(cast calldata "getPrice(address)" $TOKEN)'"},"latest"],"id":1,"jsonrpc":"2.0"}' | jq -r '.result')

L2_PRICE=$(curl -s "http://localhost:8546" -X POST -H "Content-Type: application/json" \
  --data '{"method":"eth_call","params":[{"to":"'$DEX_L2'","data":"'$(cast calldata "getPrice(address)" $WRAPPED_TOKEN)'"},"latest"],"id":1,"jsonrpc":"2.0"}' | jq -r '.result')

# 2. If profitable, execute arbitrage
if [ "$L1_PRICE" -lt "$L2_PRICE" ]; then
  echo "Arbitrage opportunity detected"

  # Bridge and immediately sell on L2
  SELL_DATA=$(cast calldata "swapExactTokensForETH(uint256,uint256,address[],address,uint256)" \
    1000000000000000000 0 "[$WRAPPED_TOKEN,$WETH]" $ACCOUNT_ADDRESS_1 $(date +%s))

  aggsandbox bridge bridge-and-call \
    --network-id 0 \
    --destination-network-id 1 \
    --token $TOKEN \
    --amount 1 \
    --target $DEX_L2 \
    --data $SELL_DATA \
    --fallback $ACCOUNT_ADDRESS_1
fi
```

### Dynamic Liquidity Management

```bash
#!/bin/bash
# liquidity_manager.sh - Dynamic liquidity rebalancing

# 1. Check liquidity positions across chains
L1_LIQUIDITY=$(aggsandbox bridge utils get-mapped --network 0 --origin-network 0 --origin-token $LP_TOKEN --json | jq -r '.balance')
L2_LIQUIDITY=$(aggsandbox bridge utils get-mapped --network 1 --origin-network 0 --origin-token $LP_TOKEN --json | jq -r '.balance')

# 2. Rebalance if needed
THRESHOLD=1000000000000000000
if [ "$L1_LIQUIDITY" -gt $((L2_LIQUIDITY + THRESHOLD)) ]; then
  echo "Rebalancing liquidity from L1 to L2"

  REMOVE_LIQUIDITY_DATA=$(cast calldata "removeLiquidity(address,address,uint256,uint256,uint256,address,uint256)" \
    $TOKEN_A $TOKEN_B $((THRESHOLD/2)) 0 0 $ACCOUNT_ADDRESS_1 $(date +%s))

  aggsandbox bridge message \
    --network-id 0 \
    --destination-network-id 1 \
    --target $DEX_ROUTER_L1 \
    --data $REMOVE_LIQUIDITY_DATA
fi
```

## Testing and Simulation

### Load Testing Bridge Operations

```bash
#!/bin/bash
# load_test.sh - Load test bridge operations

CONCURRENT_BRIDGES=10
BRIDGE_AMOUNT="0.01"

for i in $(seq 1 $CONCURRENT_BRIDGES); do
  aggsandbox bridge asset \
    --network-id 0 \
    --destination-network-id 1 \
    --amount $BRIDGE_AMOUNT \
    --token-address 0x0000000000000000000000000000000000000000 &
done

wait
echo "All $CONCURRENT_BRIDGES bridges initiated"
```

### Integration Test Suite

```bash
#!/bin/bash
# integration_test.sh - Comprehensive integration tests

set -e

echo "Starting integration test suite..."

# 1. Test basic asset bridge
echo "Testing basic asset bridge..."
TX_HASH=$(aggsandbox bridge asset --network 0 --destination-network 1 --amount 0.1 --token-address 0x0000000000000000000000000000000000000000 --json | jq -r '.tx_hash')
aggsandbox bridge claim --network-id 1 --tx-hash $TX_HASH --source-network-id 0

# 2. Test ERC20 bridge
echo "Testing ERC20 bridge..."
TX_HASH=$(aggsandbox bridge asset --network 0 --destination-network 1 --amount 100 --token-address $AGG_ERC20_L1 --json | jq -r '.tx_hash')
aggsandbox bridge claim --network-id 1 --tx-hash $TX_HASH --source-network-id 0

# 3. Test message bridge
echo "Testing message bridge..."
MESSAGE_DATA=$(cast calldata "transfer(address,uint256)" $ACCOUNT_ADDRESS_1 1000000000000000000)
TX_HASH=$(aggsandbox bridge message --network 0 --destination-network 1 --target $TARGET_CONTRACT --data $MESSAGE_DATA --json | jq -r '.tx_hash')
aggsandbox bridge claim --network-id 1 --tx-hash $TX_HASH --source-network-id 0

# 4. Test bridge-and-call
echo "Testing bridge-and-call..."
TX_HASH=$(aggsandbox bridge bridge-and-call --network 0 --destination-network 1 --token $AGG_ERC20_L1 --amount 10 --target $L2_TOKEN --data $TRANSFER_DATA --fallback $ACCOUNT_ADDRESS_1 --json | jq -r '.tx_hash')
aggsandbox bridge claim --network-id 1 --tx-hash $TX_HASH --source-network-id 0 --deposit-count 0
aggsandbox bridge claim --network-id 1 --tx-hash $TX_HASH --source-network-id 0 --deposit-count 1

echo "All integration tests passed!"
```

## Performance Optimization

### Parallel Bridge Processing

```bash
#!/bin/bash
# parallel_bridge.sh - Process multiple bridges in parallel

declare -a BRIDGE_PIDS=()

# Start multiple bridge operations
for i in {1..5}; do
  aggsandbox bridge asset \
    --network-id 0 \
    --destination-network-id 1 \
    --amount 0.1 \
    --token-address 0x0000000000000000000000000000000000000000 &
  BRIDGE_PIDS+=($!)
done

# Wait for all bridges to complete
for pid in "${BRIDGE_PIDS[@]}"; do
  wait $pid
done

echo "All parallel bridges completed"
```

### Resource Monitoring

```bash
#!/bin/bash
# monitor_resources.sh - Monitor sandbox resource usage

while true; do
  echo "=== $(date) ==="

  # Docker resource usage
  docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}"

  # Bridge service health
  curl -s http://localhost:5577/health | jq '.'

  # Network block heights
  echo "L1 Block: $(cast block-number --rpc-url http://localhost:8545)"
  echo "L2 Block: $(cast block-number --rpc-url http://localhost:8546)"

  sleep 30
done
```

## Custom Deployment Scenarios

### Deploy Custom Contracts

```bash
# 1. Deploy custom contract on L1
CUSTOM_CONTRACT_L1=$(cast send --private-key $PRIVATE_KEY_1 \
  --rpc-url http://localhost:8545 \
  --create $(cat custom_contract.bin))

# 2. Deploy corresponding contract on L2
CUSTOM_CONTRACT_L2=$(cast send --private-key $PRIVATE_KEY_1 \
  --rpc-url http://localhost:8546 \
  --create $(cat custom_contract.bin))

# 3. Test cross-chain interaction
INTERACTION_DATA=$(cast calldata "crossChainFunction(uint256)" 12345)
aggsandbox bridge message \
  --network-id 0 \
  --destination-network-id 1 \
  --target $CUSTOM_CONTRACT_L2 \
  --data $INTERACTION_DATA
```

### Multi-Contract Ecosystem

```bash
#!/bin/bash
# deploy_ecosystem.sh - Deploy complete DeFi ecosystem

echo "Deploying DeFi ecosystem..."

# Deploy core contracts
FACTORY_L1=$(cast send --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1 --create $(cat factory.bin))
ROUTER_L1=$(cast send --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1 --create $(cat router.bin) $FACTORY_L1)
STAKING_L1=$(cast send --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1 --create $(cat staking.bin))

FACTORY_L2=$(cast send --private-key $PRIVATE_KEY_1 --rpc-url $RPC_2 --create $(cat factory.bin))
ROUTER_L2=$(cast send --private-key $PRIVATE_KEY_1 --rpc-url $RPC_2 --create $(cat router.bin) $FACTORY_L2)
STAKING_L2=$(cast send --private-key $PRIVATE_KEY_1 --rpc-url $RPC_2 --create $(cat staking.bin))

echo "Ecosystem deployed successfully"
echo "L1 Factory: $FACTORY_L1"
echo "L2 Factory: $FACTORY_L2"
```

## Monitoring and Analytics

### Bridge Analytics Dashboard

```bash
#!/bin/bash
# analytics.sh - Generate bridge analytics

echo "=== Bridge Analytics ==="

# Total bridges by network
echo "L1 Bridges:"
aggsandbox show bridges --network-id 0 --json | jq '.bridges | length'

echo "L2 Bridges:"
aggsandbox show bridges --network-id 1 --json | jq '.bridges | length'

# Total volume
L1_VOLUME=$(aggsandbox show bridges --network-id 0 --json | jq '[.bridges[].amount | tonumber] | add')
L2_VOLUME=$(aggsandbox show bridges --network-id 1 --json | jq '[.bridges[].amount | tonumber] | add')

echo "L1 Bridge Volume: $L1_VOLUME"
echo "L2 Bridge Volume: $L2_VOLUME"

# Success rate
L1_CLAIMS=$(aggsandbox show claims --network-id 1 --json | jq '.claims | length')
L2_CLAIMS=$(aggsandbox show claims --network-id 0 --json | jq '.claims | length')

echo "L1→L2 Success Rate: $(echo "scale=2; $L1_CLAIMS * 100 / $(aggsandbox show bridges --network-id 0 --json | jq '.bridges | length')" | bc)%"
```

### Real-time Event Monitoring

```bash
#!/bin/bash
# event_monitor.sh - Real-time cross-chain event monitoring

while true; do
  echo "=== $(date) ==="

  # Monitor bridge events
  echo "Recent L1 Events:"
  aggsandbox events --network-id 0 --blocks 1

  echo "Recent L2 Events:"
  aggsandbox events --network-id 1 --blocks 1

  # Check for failed transactions
  FAILED_TXS=$(aggsandbox logs bridge-service | grep -i error | tail -5)
  if [ ! -z "$FAILED_TXS" ]; then
    echo "⚠️  Recent Errors:"
    echo "$FAILED_TXS"
  fi

  sleep 10
done
```

## Next Steps

- **[Configuration](configuration.md)** - Advanced environment setup
- **[Troubleshooting](troubleshooting.md)** - Debug complex scenarios
- **[Development](development.md)** - Contribute to the project
- **[CLI Reference](cli-reference.md)** - Complete command documentation
