# Troubleshooting Guide

Comprehensive troubleshooting guide for common issues and debugging techniques.

## Quick Diagnostics

### Health Check Commands

```bash
# Check overall sandbox status
aggsandbox status

# Test all connections
aggsandbox status

# Test RPC endpoints
curl -s -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}' \
  | jq -r '.result // "❌ L1 RPC failed"' | sed 's/^/✅ L1 block: /'

curl -s -X POST http://127.0.0.1:8546 \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}' \
  | jq -r '.result // "❌ L2 RPC failed"' | sed 's/^/✅ L2 block: /'

# Test bridge service
curl -s http://localhost:5577/health | jq '.' && echo "✅ Bridge service healthy" || echo "❌ Bridge service down"

# Check service logs
aggsandbox logs --tail 50

# Verify environment
source .env && echo "Environment loaded"
```

### System Requirements Check

```bash
# Verify all prerequisites
docker --version && echo "✅ Docker installed"
docker compose version && echo "✅ Docker Compose installed"
rustc --version && echo "✅ Rust installed"
make --version && echo "✅ Make installed"

# Check available resources
docker system df --format "table {{.Type}}\t{{.TotalCount}}\t{{.Size}}\t{{.Reclaimable}}"

# Check running container stats (timeout after 5 seconds)
timeout 5s docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}" || echo "Container stats unavailable"
```

## Common Issues

### Docker Issues

#### Docker Daemon Not Running

**Symptoms:**

- `Cannot connect to the Docker daemon`
- `docker: command not found`

**Solutions:**

```bash
# Check Docker status
docker --version

# Start Docker Desktop (macOS/Windows)
open -a Docker

# Start Docker daemon (Linux)
sudo systemctl start docker
sudo systemctl enable docker

# Verify Docker is running
docker run hello-world
```

#### Port Conflicts

**Symptoms:**

- `Port 8545 already in use`
- `Address already in use`

**Solutions:**

```bash
# Check what's using the ports
lsof -i :8545 -i :8546 -i :5577
netstat -tulpn | grep -E '(8545|8546|5577)'

# Kill conflicting processes
sudo kill -9 <PID>

# Or change ports in docker-compose.yml
services:
  anvil-l1:
    ports:
      - "18545:8545"  # Use different host port
```

#### Out of Disk Space

**Symptoms:**

- `No space left on device`
- Docker images failing to build

**Solutions:**

```bash
# Clean up Docker resources
docker system prune -a --volumes
docker image prune -a
docker container prune
docker volume prune

# Check disk usage
df -h
docker system df
```

#### Memory Issues

**Symptoms:**

- Services crashing unexpectedly
- Slow performance
- OOM (Out of Memory) errors

**Solutions:**

```bash
# Check memory usage
docker stats --no-stream
free -h

# Reduce resource limits in docker-compose.yml
deploy:
  resources:
    limits:
      memory: 512M
      cpus: '0.5'

# Restart with resource limits
aggsandbox stop --volumes
aggsandbox start --detach
```

### Network Connectivity Issues

#### RPC Connection Failures

**Symptoms:**

- `Connection refused`
- `Network unreachable`
- Bridge operations timing out

**Diagnosis:**

```bash
# Test RPC connectivity
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'

curl -X POST http://127.0.0.1:8546 \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'

# Check if services are running
aggsandbox logs anvil-l1
aggsandbox logs anvil-l2
```

**Solutions:**

```bash
# Restart specific services
docker compose restart anvil-l1 anvil-l2

# Check firewall settings
sudo ufw status  # Linux
# Ensure ports 8545, 8546, 5577 are accessible

# Verify network configuration
docker network ls
docker network inspect aggsandbox_default
```

#### Bridge Service Not Responding

**Symptoms:**

- `Bridge service not responding`
- API calls timing out
- `Connection refused` on port 5577

**Diagnosis:**

```bash
# Check bridge service health
curl -f http://localhost:5577/health
aggsandbox logs bridge-service

# Verify service is running
docker ps | grep bridge
```

**Solutions:**

```bash
# Restart bridge service
docker compose restart bridge-service

# Check database connectivity
aggsandbox logs bridge-db

# Verify bridge contracts are deployed
aggsandbox show bridges --network-id 0
```

### Configuration Issues

#### Environment Variables Not Set

**Symptoms:**

- `Missing required variable`
- Commands failing with undefined variables

**Diagnosis:**

```bash
# Check environment variables
env | grep -E "(RPC_|NETWORK_|ACCOUNT_|PRIVATE_KEY)"

# Verify .env file exists and is sourced
ls -la .env
cat .env | head -10
```

**Solutions:**

```bash
# Source environment file
source .env

# Verify required variables
echo "RPC_1: $RPC_1"
echo "Network ID: $NETWORK_ID_MAINNET"
echo "Account: $ACCOUNT_ADDRESS_1"

# Reset environment if needed
cp .env.example .env
source .env
```

#### Contract Address Issues

**Symptoms:**

- `Contract not found`
- Bridge operations failing
- Invalid contract responses

**Diagnosis:**

```bash
# Check contract addresses are set
echo "L1 Bridge: $POLYGON_ZKEVM_BRIDGE_L1"
echo "L2 Bridge: $POLYGON_ZKEVM_BRIDGE_L2"

# Verify contracts exist
cast code $POLYGON_ZKEVM_BRIDGE_L1 --rpc-url $RPC_1
cast code $POLYGON_ZKEVM_BRIDGE_L2 --rpc-url $RPC_2
```

**Solutions:**

```bash
# Restart sandbox to redeploy contracts
aggsandbox stop --volumes
aggsandbox start --detach

# Wait for contract deployment
sleep 30
aggsandbox status
```

### Bridge Operation Issues

#### Bridge Transaction Failures

**Symptoms:**

- Transaction reverted
- Gas estimation failed
- `Insufficient funds`

**Diagnosis:**

```bash
# Check account balance
cast balance $ACCOUNT_ADDRESS_1 --rpc-url $RPC_1
cast balance $ACCOUNT_ADDRESS_2 --rpc-url $RPC_2

# Check token balance
cast call $AGG_ERC20_L1 "balanceOf(address)" $ACCOUNT_ADDRESS_1 --rpc-url $RPC_1

# Check token allowance
cast call $AGG_ERC20_L1 "allowance(address,address)" $ACCOUNT_ADDRESS_1 $POLYGON_ZKEVM_BRIDGE_L1 --rpc-url $RPC_1
```

**Solutions:**

```bash
# Ensure sufficient ETH balance
cast send --value 1ether $ACCOUNT_ADDRESS_1 --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1

# Approve tokens before bridging
cast send $AGG_ERC20_L1 "approve(address,uint256)" $POLYGON_ZKEVM_BRIDGE_L1 1000000000000000000000 --private-key $PRIVATE_KEY_1 --rpc-url $RPC_1

# Use higher gas limit
aggsandbox bridge asset --network-id 0 --destination-network-id 1 --amount 0.1 --token-address 0x0000000000000000000000000000000000000000 --gas-limit 3000000
```

#### Claim Operation Failures

**Symptoms:**

- `Bridge not found`
- `Invalid proof`
- `Already claimed`

**Diagnosis:**

```bash
# Check bridge exists
aggsandbox show bridges --network-id 0 --json | jq '.bridges[] | select(.tx_hash == "'$TX_HASH'")'

# Check claim status
aggsandbox show claims --network-id 1

# Verify proof data
aggsandbox show claim-proof --network-id 0 --leaf-index 0 --deposit-count 0
```

**Solutions:**

```bash
# Wait for bridge processing
sleep 30
aggsandbox show bridges --network-id 0

# Use correct deposit count for bridge-and-call
aggsandbox bridge claim --network-id 1 --tx-hash $TX_HASH --source-network-id 0 --deposit-count 0  # Asset bridge first
aggsandbox bridge claim --network-id 1 --tx-hash $TX_HASH --source-network-id 0 --deposit-count 1  # Message bridge second

# Regenerate proof if needed
LEAF_INDEX=$(aggsandbox show l1-info-tree-index --network-id 0 --deposit-count 0 --json | jq -r '.')
aggsandbox show claim-proof --network-id 0 --leaf-index $LEAF_INDEX --deposit-count 0
```

#### Bridge-and-Call Issues

**Symptoms:**

- Asset bridge works but message bridge fails
- `MessageFailed` error
- Contract call not executing

**Diagnosis:**

```bash
# Check both bridge transactions exist
aggsandbox show bridges --network-id 0 --json | jq '.bridges[] | select(.tx_hash == "'$TX_HASH'")'

# Verify asset bridge claimed first
aggsandbox show claims --network-id 1 --json | jq '.claims[] | select(.global_index == "0")'

# Check target contract exists
cast code $TARGET_CONTRACT --rpc-url $RPC_2
```

**Solutions:**

```bash
# Always claim asset bridge before message bridge
aggsandbox bridge claim --network-id 1 --tx-hash $TX_HASH --source-network-id 0 --deposit-count 0

# Wait for confirmation before claiming message
sleep 10
aggsandbox bridge claim --network-id 1 --tx-hash $TX_HASH --source-network-id 0 --deposit-count 1

# Verify target contract and call data
cast call $TARGET_CONTRACT "your_function()" --rpc-url $RPC_2
```

### Fork Mode Issues

#### Fork URL Connection Issues

**Symptoms:**

- `Fork URL validation failed`
- `Rate limited`
- `Invalid API key`

**Diagnosis:**

```bash
# Test fork URL manually
curl -X POST $FORK_URL_MAINNET \
  -H "Content-Type: application/json" \
  --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}'

# Check rate limits
curl -I $FORK_URL_MAINNET
```

**Solutions:**

```bash
# Verify API keys are correct
echo "Mainnet URL: $FORK_URL_MAINNET"
echo "L2 URL: $FORK_URL_AGGLAYER_1"

# Use different RPC provider
FORK_URL_MAINNET=https://eth-mainnet.g.infura.io/v3/YOUR_PROJECT_ID

# Reduce rate limiting
export RPC_RATE_LIMIT=5
```

#### Fork State Issues

**Symptoms:**

- Inconsistent state across chains
- Outdated blockchain data
- Fork behavior different from mainnet

**Solutions:**

```bash
# Pin to specific block numbers
FORK_BLOCK_MAINNET=18500000
FORK_BLOCK_AGGLAYER_1=50000000

# Restart with clean state
aggsandbox stop --volumes
aggsandbox start --fork --detach

# Verify fork block numbers
cast block-number --rpc-url $RPC_1
cast block-number --rpc-url $RPC_2
```

## Advanced Debugging

### Enable Debug Logging

```bash
# Enable verbose logging
aggsandbox start --detach --verbose

# Check detailed logs
aggsandbox logs --follow --verbose

# Enable debug mode
LOG_LEVEL=debug aggsandbox start --detach
```

### Service-Specific Debugging

#### Anvil Node Issues

```bash
# Check Anvil logs
aggsandbox logs anvil-l1
aggsandbox logs anvil-l2

# Test Anvil directly
cast block-number --rpc-url http://localhost:8545
cast accounts --rpc-url http://localhost:8545

# Restart Anvil nodes
docker compose restart anvil-l1 anvil-l2
```

#### Bridge Service Debugging

```bash
# Check bridge service logs
aggsandbox logs bridge-service | grep -i error
aggsandbox logs bridge-service | tail -100

# Check database logs
aggsandbox logs bridge-db

# Verify API endpoints
curl http://localhost:5577/health
curl http://localhost:5577/bridge/v1/bridges?network_id=0
```

#### Database Issues

```bash
# Check database connectivity
aggsandbox logs bridge-db

# Reset database
docker compose down
docker volume rm aggsandbox_db-data
aggsandbox start --detach
```

### Performance Issues

#### Slow Performance

**Symptoms:**

- Transactions taking too long
- High CPU/memory usage
- Timeouts

**Diagnosis:**

```bash
# Monitor resource usage
docker stats --no-stream
htop  # or top

# Check network latency
ping localhost
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:5577/health
```

**Solutions:**

```bash
# Reduce resource usage
# In docker-compose.yml:
deploy:
  resources:
    limits:
      memory: 1g
      cpus: '0.5'

# Increase timeouts
HTTP_TIMEOUT=60
BRIDGE_TIMEOUT_SECONDS=600

# Use faster block times for development
ANVIL_BLOCK_TIME=1
```

#### Memory Leaks

```bash
# Monitor memory usage over time
while true; do
  docker stats --no-stream | grep aggsandbox
  sleep 10
done

# Restart services periodically
docker compose restart bridge-service
```

## Error Codes Reference

### Common Error Codes

| Error Code                | Description              | Solution                                      |
| ------------------------- | ------------------------ | --------------------------------------------- |
| `0x37e391c3`              | MessageFailed            | Don't manually claim BridgeExtension messages |
| `0xe450d38c`              | ERC20InsufficientBalance | Check token balance and allowance             |
| `0x0595ea2e`              | Invalid Network ID       | Use correct network IDs (0, 1, 2)             |
| Gas estimation failed     | Insufficient gas         | Increase `--gas-limit` parameter              |
| Connection refused        | Service not running      | Check service status and restart              |
| Docker daemon not running | Docker not started       | Start Docker service                          |

### Bridge-Specific Errors

| Error                              | Cause                     | Solution                                 |
| ---------------------------------- | ------------------------- | ---------------------------------------- |
| Bridge not found                   | Transaction not processed | Wait for processing, check tx hash       |
| Invalid proof                      | Wrong proof parameters    | Regenerate proof with correct values     |
| Already claimed                    | Bridge already processed  | Check claims status                      |
| Insufficient allowance             | Token not approved        | Approve bridge contract                  |
| Asset bridge must be claimed first | Wrong claiming order      | Claim asset bridge before message bridge |

## Recovery Procedures

### Clean Restart

```bash
# Complete environment reset
aggsandbox stop --volumes
docker system prune -f
rm -rf ./data ./logs
cp .env.example .env
source .env
aggsandbox start --detach
```

### Partial Recovery

```bash
# Restart only problematic services
docker compose restart bridge-service
docker compose restart anvil-l1

# Reset specific components
docker compose down bridge-service
docker compose up -d bridge-service
```

### Data Recovery

```bash
# Backup important data
mkdir -p ./backup
cp .env ./backup/
docker cp $(docker ps -qf name=bridge-db):/var/lib/postgresql/data ./backup/db-data

# Restore from backup
docker cp ./backup/db-data $(docker ps -qf name=bridge-db):/var/lib/postgresql/data
```

## Getting Help

### Gather Debug Information

```bash
#!/bin/bash
# debug_info.sh - Collect debugging information

echo "=== Agglayer Sandbox Debug Information ==="
echo "Date: $(date)"
echo

echo "=== System Information ==="
uname -a
docker --version
docker compose version

echo "=== Service Status ==="
aggsandbox status

echo "=== Container Status ==="
docker ps --filter name=aggsandbox

echo "=== Recent Logs ==="
aggsandbox logs --tail 20

echo "=== Environment Variables ==="
env | grep -E "(RPC_|NETWORK_|ACCOUNT_)" | head -10

echo "=== Resource Usage ==="
docker stats --no-stream
```

### Support Channels

1. **CLI Help**: `aggsandbox --help`
2. **Command Help**: `aggsandbox <command> --help`
3. **GitHub Issues**: Report bugs and request features
4. **Documentation**: Check other documentation files
5. **Community**: Engage with the community for help

### Before Reporting Issues

1. **Search existing issues** on GitHub
2. **Run debug information script** above
3. **Try clean restart** procedure
4. **Check documentation** for known solutions
5. **Provide minimal reproduction** steps

## Prevention Tips

### Best Practices

1. **Always source `.env`** before operations
2. **Check service status** before bridging
3. **Use appropriate gas limits** for operations
4. **Monitor resource usage** during heavy testing
5. **Keep Docker updated** to latest stable version
6. **Backup important configurations** before changes

### Regular Maintenance

```bash
# Weekly maintenance script
#!/bin/bash
# maintenance.sh

echo "Performing weekly maintenance..."

# Clean up Docker resources
docker system prune -f

# Update images
docker compose pull

# Restart services
aggsandbox stop
aggsandbox start --detach

# Verify health
aggsandbox status
curl -f http://localhost:5577/health

echo "Maintenance completed"
```

## Next Steps

- **[Configuration](configuration.md)** - Advanced configuration options
- **[Development](development.md)** - Contributing and development setup
- **[CLI Reference](cli-reference.md)** - Complete command reference
- **[Bridge Operations](bridge-operations.md)** - Bridge operation guides
