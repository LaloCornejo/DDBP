#!/bin/bash

# Create the network if it doesn't exist
echo "Creating MongoDB network..."
podman network create ddbp_mongo-network 2>/dev/null || true

# Rebuild the Rust application from the correct directory
echo "Building Rust application..."
cd rust-app && \
podman build -t docker.io/library/ddbp-rust-app:latest . && \
cd ..

# Start MongoDB containers and setup
echo "Starting MongoDB containers..."
podman compose up -d central-mongodb secondary-mongodb-1 secondary-mongodb-2 mongo-setup && \
sleep 15 && \
echo "Starting Rust application..." && \
podman run --name ddbp-rust-app \
    --network ddbp_mongo-network \
    -p 8000:8000 \
    -e MONGO_URI="mongodb://admin:password@central-mongodb:27017,secondary-mongodb-1:27017,secondary-mongodb-2:27017/social_media_db?replicaSet=rs0&authSource=admin" \
    -d \
    docker.io/library/ddbp-rust-app:latest

echo "Application startup complete!"
