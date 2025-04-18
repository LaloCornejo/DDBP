#!/bin/bash

# Default mode
DEV_MODE=false
FORCE_REBUILD=false
BUILD_CACHE_DIR="$HOME/.cache/ddbp-build"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --dev)
      DEV_MODE=true
      shift
      ;;
    --rebuild)
      FORCE_REBUILD=true
      shift
      ;;
    --help)
      echo "Usage: ./fast-start-app.sh [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --dev        Run in development mode (faster startup, hot reloading)"
      echo "  --rebuild    Force rebuild of the application"
      echo "  --help       Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

# Create build cache directory if it doesn't exist
mkdir -p "$BUILD_CACHE_DIR"

# Cache file to store last build timestamp
LAST_BUILD_FILE="$BUILD_CACHE_DIR/last_build_timestamp"
RUST_APP_DIR="rust-app"

# Create the network if it doesn't exist
echo "Creating MongoDB network..."
podman network create ddbp_mongo-network 2>/dev/null || true

# Check if we need to rebuild the Rust application
REBUILD_NEEDED=false
if [ "$FORCE_REBUILD" = true ]; then
  echo "Forced rebuild requested"
  REBUILD_NEEDED=true
else
  # Get the latest modification time of any file in the rust-app directory
  if [ -d "$RUST_APP_DIR" ]; then
    LATEST_MOD_TIME=$(find "$RUST_APP_DIR" -type f -not -path "*/target/*" -not -path "*/.*" -printf '%T@\n' 2>/dev/null | sort -r | head -n 1)
    
    if [ ! -f "$LAST_BUILD_FILE" ]; then
      echo "No previous build timestamp found"
      REBUILD_NEEDED=true
    else
      LAST_BUILD_TIME=$(cat "$LAST_BUILD_FILE")
      if (( $(echo "$LATEST_MOD_TIME > $LAST_BUILD_TIME" | bc -l) )); then
        echo "Source files have changed since last build"
        REBUILD_NEEDED=true
      else
        echo "No changes detected since last build"
      fi
    fi
  else
    echo "Rust application directory not found"
    exit 1
  fi
fi

# Check if image exists
IMAGE_EXISTS=$(podman images -q docker.io/library/ddbp-rust-app:latest 2>/dev/null)
if [ -z "$IMAGE_EXISTS" ]; then
  echo "Docker image doesn't exist, rebuild required"
  REBUILD_NEEDED=true
fi

# Rebuild the Rust application if needed
if [ "$REBUILD_NEEDED" = true ]; then
  echo "Building Rust application..."
  
  BUILD_ARGS=""
  if [ "$DEV_MODE" = true ]; then
    BUILD_ARGS="--build-arg MODE=debug"
    echo "Building in debug/development mode"
  else
    BUILD_ARGS="--build-arg MODE=release"
    echo "Building in release mode"
  fi
  
  cd "$RUST_APP_DIR" && \
  # Use buildkit for better caching
  podman build $BUILD_ARGS -t docker.io/library/ddbp-rust-app:latest . && \
  cd .. && \
  # Store current timestamp for future comparison
  date +%s > "$LAST_BUILD_FILE"
  
  if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
  fi
else
  echo "Skipping build, using cached image"
fi

# Start MongoDB containers and setup
echo "Starting MongoDB containers..."
podman compose up -d central-mongodb secondary-mongodb-1 secondary-mongodb-2 mongo-setup

# Check if mongo-setup container is healthy before proceeding
echo "Waiting for MongoDB setup to complete..."
MAX_RETRIES=10
RETRY_COUNT=0
SETUP_COMPLETE=false

while [ $RETRY_COUNT -lt $MAX_RETRIES ] && [ "$SETUP_COMPLETE" = false ]; do
  RETRY_COUNT=$((RETRY_COUNT+1))
  
  # Check if mongo-setup container has exited successfully
  SETUP_STATUS=$(podman inspect -f '{{.State.Status}}' mongo-setup 2>/dev/null || echo "not_found")
  
  if [ "$SETUP_STATUS" = "exited" ]; then
    EXIT_CODE=$(podman inspect -f '{{.State.ExitCode}}' mongo-setup)
    if [ "$EXIT_CODE" = "0" ]; then
      SETUP_COMPLETE=true
      echo "MongoDB setup completed successfully"
    else
      echo "MongoDB setup failed with exit code $EXIT_CODE"
      exit 1
    fi
  else
    echo "Waiting for MongoDB setup to complete... ($RETRY_COUNT/$MAX_RETRIES)"
    sleep 3
  fi
done

if [ "$SETUP_COMPLETE" = false ]; then
  echo "MongoDB setup timed out after $MAX_RETRIES attempts"
  exit 1
fi

# Remove existing container if it exists
CONTAINER_EXISTS=$(podman ps -a --filter name=ddbp-rust-app -q)
if [ ! -z "$CONTAINER_EXISTS" ]; then
  echo "Removing existing Rust application container..."
  podman rm -f ddbp-rust-app >/dev/null 2>&1
fi

# Start the Rust application
echo "Starting Rust application..."
RUN_ARGS=""
if [ "$DEV_MODE" = true ]; then
  # For dev mode, you might want to add volume mounts or environment variables
  RUN_ARGS="-e DEV_MODE=true"
  echo "Running in development mode"
fi

podman run --name ddbp-rust-app \
    --network ddbp_mongo-network \
    -p 8000:8000 \
    -e MONGO_URI="mongodb://admin:password@central-mongodb:27017,secondary-mongodb-1:27017,secondary-mongodb-2:27017/social_media_db?replicaSet=rs0&authSource=admin" \
    $RUN_ARGS \
    -d \
    docker.io/library/ddbp-rust-app:latest

echo "Application startup complete!"
echo ""
if [ "$DEV_MODE" = true ]; then
  echo "Running in DEVELOPMENT mode"
else
  echo "Running in PRODUCTION mode"
fi
echo "Access the application at http://localhost:8000"

