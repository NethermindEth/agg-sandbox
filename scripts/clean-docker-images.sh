#!/bin/bash

# Script to clean Docker images used by aggsandbox compose files
# This helps ensure fresh images are pulled when Docker caches become stale

set -e

echo "🧹 Cleaning Docker images for aggsandbox..."

# Extract unique image names from both compose files
IMAGES=(
    "ametelnethermind/aggsandbox-deployer:latest"
    "ametelnethermind/aggkit:latest"
)

# Stop and remove containers first
echo "🛑 Stopping and removing containers..."
docker-compose down --remove-orphans 2>/dev/null || true
docker-compose -f docker-compose.multi-l2.yml down --remove-orphans 2>/dev/null || true

# Remove the specific images
echo "🗑️  Removing Docker images..."
for image in "${IMAGES[@]}"; do
    if docker image inspect "$image" >/dev/null 2>&1; then
        echo "  - Removing $image"
        docker rmi "$image" || true
    else
        echo "  - $image not found locally"
    fi
done

# Clean up dangling images
echo "🧽 Cleaning up dangling images..."
docker image prune -f

# Optional: Clean up unused volumes (uncomment if needed)
# echo "📦 Cleaning up unused volumes..."
# docker volume prune -f

echo "✅ Docker cleanup complete!"
echo "💡 Next time you run 'aggsandbox start', fresh images will be pulled."
