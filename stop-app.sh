#!/bin/bash

echo "Stopping all application containers..."

# Stop all containers
podman stop $(podman ps -a -q --filter name='central-mongodb|secondary-mongodb-1|secondary-mongodb-2|ddbp-rust-app|mongo-setup') 2>/dev/null

echo "Removing stopped containers..."
# Remove all containers
podman rm $(podman ps -a -q --filter name='central-mongodb|secondary-mongodb-1|secondary-mongodb-2|ddbp-rust-app|mongo-setup') 2>/dev/null

echo "Cleaning up network..."
# Remove the network
podman network rm ddbp_mongo-network 2>/dev/null || true

echo "Cleanup complete!"
