#!/bin/bash

# Create the network if it doesn't exist
echo "Creating MongoDB network..."
podman network create ddbp_mongo-network 2>/dev/null || true

# Check if there are changes in the Rust application
echo "Checking for code changes..."
CHANGED=$(git diff --name-only HEAD | grep -E '^rust-app/' || true)

if [ -n "$CHANGED" ]; then
  echo "Changes detected. Rebuilding the Rust application..."
  
  # Rebuild the Rust application
  cd rust-app && \
  podman build -t docker.io/library/ddbp-rust-app:latest . && \
  cd ..
else
  echo "No changes detected. Skipping rebuild."
fi

# Start MongoDB containers and setup
echo "Starting MongoDB containers..."
podman compose up -d central-mongodb secondary-mongodb-1 secondary-mongodb-2 mongo-setup

# Wait for MongoDB replica set to be fully initialized
echo "Waiting for MongoDB replica set to initialize..."
MAX_RETRIES=30
RETRY_COUNT=0
REPLICA_READY=false

while [ $RETRY_COUNT -lt $MAX_RETRIES ] && [ "$REPLICA_READY" = false ]; do
  # Check if mongo-setup container has exited successfully
  SETUP_STATUS=$(podman inspect -f '{{.State.ExitCode}}' mongo-setup 2>/dev/null || echo "-1")
  
  if [ "$SETUP_STATUS" = "0" ]; then
    # Verify replica set status
    if podman exec -it central-mongodb mongosh -u admin -p password --quiet --eval "rs.status().ok" | grep -q "1"; then
      echo "MongoDB replica set is ready!"
      REPLICA_READY=true
    else
      echo "MongoDB replica set initialization in progress..."
    fi
  else
    echo "Waiting for MongoDB setup to complete..."
  fi
  
  if [ "$REPLICA_READY" = false ]; then
    RETRY_COUNT=$((RETRY_COUNT+1))
    echo "Waiting for MongoDB replica set (attempt $RETRY_COUNT/$MAX_RETRIES)..."
    sleep 2
  fi
done

if [ "$REPLICA_READY" = false ]; then
  echo "MongoDB replica set failed to initialize after multiple attempts. Exiting."
  exit 1
fi

echo "Starting Rust application..." 

# Remove existing container if it exists
CONTAINER_EXISTS=$(podman ps -a --filter name=ddbp-rust-app -q)
if [ ! -z "$CONTAINER_EXISTS" ]; then
  echo "Removing existing Rust application container..."
  podman rm -f ddbp-rust-app >/dev/null 2>&1
fi

# Start the Rust application container
podman run --name ddbp-rust-app \
    --network ddbp_mongo-network \
    -p 8000:8080 \
    -e HOST=0.0.0.0 \
    -e MONGO_URI="mongodb://admin:password@central-mongodb:27017,secondary-mongodb-1:27017,secondary-mongodb-2:27017/social_media_db?replicaSet=rs0&authSource=admin" \
    -d \
    docker.io/library/ddbp-rust-app:latest

# Check health of the application
echo "Checking application health..."
MAX_HEALTH_RETRIES=30
HEALTH_RETRY_COUNT=0
APP_HEALTHY=false

while [ $HEALTH_RETRY_COUNT -lt $MAX_HEALTH_RETRIES ] && [ "$APP_HEALTHY" = false ]; do
  HEALTH_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/health 2>/dev/null || echo "000")
  
  if [ "$HEALTH_RESPONSE" = "200" ]; then
    echo "Application is healthy and ready to use!"
    APP_HEALTHY=true
  else
    HEALTH_RETRY_COUNT=$((HEALTH_RETRY_COUNT+1))
    echo "Waiting for application to become healthy (attempt $HEALTH_RETRY_COUNT/$MAX_HEALTH_RETRIES)..."
    sleep 2
  fi
done

if [ "$APP_HEALTHY" = false ]; then
  echo "Warning: Application failed to report healthy status."
  echo "Check application logs: podman logs ddbp-rust-app"
else
  echo "DDBP application is running at http://localhost:8000"
fi
