# Counter Contract Bridge and Call Example

This document demonstrates the complete workflow for deploying a Counter contract and using `bridgeAndCall` functionality with the Agglayer sandbox CLI.

## Prerequisites

- Fresh sandbox environment running (`aggsandbox start --detach`)
- Foundry/cast installed
- CLI tools built and available

## Step 1: Deploy Counter Contract on L2

Deploy the Counter contract to L2 using the pre-compiled bytecode:

```bash
# Source environment variables
source .env

# Deploy Counter contract on L2 using cast
COUNTER_L2=$(forge create agglayer-contracts/src/mocks/Counter.sol:Counter \
 --rpc-url $RPC_2 \
 --private-key $PRIVATE_KEY_1 \
 --value 0.1ether \
 --broadcast \
 --json | jq -r '.deployedTo')

 echo "Counter deployed at: $COUNTER_L2"
```

## Step 2: Verify Counter Deployment

Check that the contract is deployed and working:

```bash
# Check initial counter value (should be 0)
cast call --rpc-url $RPC_2 $COUNTER_L2 "getCount()"

# Expected output: 0x0000000000000000000000000000000000000000000000000000000000000000
```

## Step 3: Create Callada

```bash
INCREMENT_DATA=$(cast calldata "increment()")
```

## Step 4: Execute bridgeAndCall Operation

Use CLI to bridge ETH and call the Counter's increment function:

```bash
# Bridge 0.01 ETH and call increment() function using CLI
aggsandbox bridge bridge-and-call \
  --network-id 0 \
  --destination-network-id 1 \
  --token 0x0000000000000000000000000000000000000000 \
  --amount 10000000000000000 \
  --target $COUNTER_L2 \
  --fallback $ACCOUNT_ADDRESS_1 \
  --data $INCREMENT_DATA \
  --msg-value 10000000000000000
```

Note the transaction hash from the output
```bash
aggsandbox bridge claim   --network-id 1   --tx-hash <tx_hash>  --source-network-id 0 --deposit-count 0
```
```bash
aggsandbox bridge claim   --network-id 1   --tx-hash <tx_hash>  --source-network-id 0 --deposit-count 1
```
## Step 5: Verify Incremented Counter Value

Check that the counter value has been incremented:

```bash
# Check counter value after bridgeAndCall operation (should be 1)
cast call --rpc-url $RPC_2 $COUNTER_L2 "getCount()"

# Expected output: 0x0000000000000000000000000000000000000000000000000000000000000001
```
