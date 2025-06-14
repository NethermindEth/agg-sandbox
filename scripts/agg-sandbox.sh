#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting deployment script for Polygon ZkEVM contracts${NC}"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if docker-compose.yml exists
if [ ! -f "docker-compose.yml" ]; then
    echo "docker-compose.yml not found. Please run this script from the project root directory."
    exit 1
fi

# Stop any existing containers to ensure a clean state
echo -e "${GREEN}Stopping existing containers...${NC}"
docker-compose down

# Start Docker containers
echo -e "${GREEN}Starting Anvil instances...${NC}"
docker-compose up -d

# Wait for Anvil instances to be ready
echo -e "${GREEN}Waiting for Anvil instances to be ready...${NC}"
sleep 10

# Check if .env file exists, create if not
if [ ! -f ".env" ]; then
    echo "Creating .env file..."
    touch .env
fi

# Set Docker environment variable only if it doesn't exist
if ! grep -q "^DOCKER_ENV=" .env; then
    echo "DOCKER_ENV=true" >> .env
fi

# Run the deployment script
echo -e "${GREEN}Running deployment script...${NC}"
./scripts/deploy-contracts.sh .env

echo -e "${GREEN}Deployment complete!${NC}"
echo "Contract addresses have been saved to .env" 