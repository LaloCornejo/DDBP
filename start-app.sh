#!/bin/bash

# Path definitions
TIMESTAMP_FILE=".last_build_timestamp"
RUST_APP_DIR="rust-app"

# Function to check if files have been modified after the last build
check_modified_files() {
  local lastBuildTime=0
  if [ -f "$TIMESTAMP_FILE" ]; then
    lastBuildTime=$(cat "$TIMESTAMP_FILE")
  fi

  local filesChanged=false
  
  # Check source directory with special attention to files that might contain profile pic URLs
  find "$RUST_APP_DIR/src" -type f -not -path "*/\.*" -print0 | while IFS= read -r -d '' file; do
    modTime=$(stat -f "%m" "$file")
    if [ $modTime -gt $lastBuildTime ]; then
      echo "Change detected in: $file"
      # Check specifically for profile picture URL changes in the file
      if grep -q "profile_picture\|pfp\|avatar" "$file"; then
        echo "⚠️"
      fi
      return 0  # Changes found
    fi
  done
  
  # Check Cargo files
  for file in "$RUST_APP_DIR/Cargo.toml" "$RUST_APP_DIR/Cargo.lock"; do
    if [ -f "$file" ]; then
      modTime=$(stat -f "%m" "$file")
      if [ $modTime -gt $lastBuildTime ]; then
        echo "Change detected in: $file"
        return 0  # Changes found
      fi
    fi
  done
  
  # Check Dockerfile
  if [ -f "$RUST_APP_DIR/Dockerfile" ]; then
    modTime=$(stat -f "%m" "$RUST_APP_DIR/Dockerfile")
    if [ $modTime -gt $lastBuildTime ]; then
      echo "Change detected in: $RUST_APP_DIR/Dockerfile"
      return 0  # Changes found
    fi
  fi
  
  return 1  # No changes found
}

# Perform aggressive cleanup
echo "Performing aggressive cleanup..."
podman rm -f $(podman ps -a -q) >/dev/null 2>&1 || true
podman network prune -f >/dev/null 2>&1 || true

# Check if there are changes in the Rust application
echo "Checking for code changes..."
if check_modified_files; then
  echo "Changes detected. Rebuilding the Rust application..."
  
  # Perform aggressive cleanup to ensure all caches and layers are removed
  echo "Performing thorough container and cache cleanup..."
  
  # Force remove the existing container and image to ensure clean rebuild
  podman rm -f ddbp-rust-app >/dev/null 2>&1 || true
  podman rmi -f docker.io/library/ddbp-rust-app:latest >/dev/null 2>&1 || true
  
  # Clean podman build cache - this is crucial for profile picture URL changes
  echo "Cleaning podman build cache..."
  # Use standard podman commands for cache cleaning on macOS
  podman system prune -a -f >/dev/null 2>&1 || true
  podman image prune -a -f >/dev/null 2>&1 || true
  
  # Rebuild the Rust application with aggressive no-cache settings
  echo "Building fresh container with no cache..."
  cd rust-app && \
  podman build --pull --no-cache --force-rm -t docker.io/library/ddbp-rust-app:latest . && \
  BUILD_SUCCESS=$?
  cd ..
  
  # Update the timestamp file if build was successful
  if [ $BUILD_SUCCESS -eq 0 ]; then
    echo $(date +%s) > "$TIMESTAMP_FILE"
    echo "Build completed successfully and timestamp updated."
    echo "---------------------------------------------"
    echo "✅"
    echo "---------------------------------------------"
  else
    echo "Build failed. Not updating timestamp."
    echo "Try running the build manually to diagnose the issue."
    exit 1
  fi
else
  echo "No changes detected. Skipping rebuild."
fi

# Start containers
echo "Starting containers..."
podman-compose up -d

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
