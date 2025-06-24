#!/bin/bash

# Function to print timestamped messages
echo_ts() {
    local green="\e[32m"
    local end_color="\e[0m"
    local timestamp
    timestamp=$(date +"[%Y-%m-%d %H:%M:%S]")

    echo -e "$green$timestamp$end_color $1" >&2
}

echo_ts "Starting Anvil instances and contract deployment..."

# Build and start the services
docker-compose up --build -d anvil-mainnet anvil-polygon

echo_ts "Waiting for Anvil instances to be ready..."

# Wait for both Anvil instances to be healthy and run deployment
docker-compose up --build contract-deployer

echo_ts "Deployment completed! Check the logs above for any errors."

# Copy the deployed .env file from the container to host
echo_ts "Copying contract addresses from container..."
if docker-compose ps -q contract-deployer > /dev/null 2>&1; then
    container_name=$(docker-compose ps -q contract-deployer)
    if [ ! -z "$container_name" ]; then
        # Try to copy the file from the stopped container
        docker cp "${container_name}:/app/.env" ./.env.deployed 2>/dev/null
        if [ $? -eq 0 ]; then
            echo_ts "✅ Contract addresses copied to .env.deployed"
        else
            echo_ts "⚠️  Could not copy .env file from container"
        fi
    fi
fi

# Optional: Show the logs from the deployment
echo_ts "Recent deployment logs:"
docker-compose logs --tail=50 contract-deployer

# Check if the deployed contract addresses file exists
if [ -f ".env.deployed" ]; then
    echo_ts "✅ Contract addresses have been saved to .env.deployed"
    echo_ts "📋 Deployed contract addresses:"
    echo ""
    grep -E "(FFLONK_VERIFIER|POLYGON_ZKEVM|POLYGON_ROLLUP_MANAGER)" .env.deployed | head -10
    echo ""
    echo_ts "📄 Full contract addresses available in: .env.deployed"
else
    echo_ts "⚠️  Contract addresses file (.env.deployed) not found. Check deployment logs for errors."
fi 