#!/bin/bash

# Perform aggressive cleanup
echo "Performing aggressive cleanup..."
podman rm -f $(podman ps -a -q) >/dev/null 2>&1 || true
podman network prune -f >/dev/null 2>&1 || true

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

# Start containers
echo "Starting containers..."
podman compose up -d

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

echo "DDBP application is running at http://localhost:8000"
