#!/bin/bash
set -e  # Exit on error

# Start MongoDB replica set
echo "Starting MongoDB replica set..."
./start.sh

# Check if MongoDB primary is ready
echo "Verifying MongoDB replica set is ready..."
MAX_RETRIES=12
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
  if podman exec -it central-mongodb mongosh --quiet --eval "rs.status().members.filter(m => m.state === 1).length > 0" | grep -q "true"; then
    echo "MongoDB replica set is ready with a primary node!"
    break
  else
    echo "Waiting for MongoDB replica set to initialize (attempt $(($RETRY_COUNT+1))/$MAX_RETRIES)..."
    RETRY_COUNT=$(($RETRY_COUNT+1))
    sleep 5
  fi
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
  echo "Error: MongoDB replica set failed to initialize with a primary node."
  exit 1
fi

# Get the IP addresses of all MongoDB containers for direct connection
CENTRAL_IP=$(podman inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' central-mongodb)
SECONDARY1_IP=$(podman inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' secondary-mongodb-1)
SECONDARY2_IP=$(podman inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' secondary-mongodb-2)

echo "MongoDB IP addresses:"
echo "- Central MongoDB: $CENTRAL_IP"
echo "- Secondary MongoDB 1: $SECONDARY1_IP"
echo "- Secondary MongoDB 2: $SECONDARY2_IP"

# Set environment variables for Rust application to connect to MongoDB
export MONGO_URI="mongodb://admin:password@$CENTRAL_IP:27017/?replicaSet=rs0"
export RUST_LOG=info
export RUST_BACKTRACE=1

# Build the Rust application
echo "Building the Rust application..."
cargo build --release

# Check if the binary was built successfully
if [ ! -f "$(pwd)/target/release/ddbp" ]; then
  echo "Error: Failed to build the Rust application."
  exit 1
fi

# Ensure the binary has execute permissions
echo "Setting executable permissions on binary..."
chmod +x "$(pwd)/target/release/ddbp"

# Kill any existing process using port 8000
echo "Ensuring port 8000 is available..."
lsof -i :8000 -t | xargs kill -9 2>/dev/null || true

# Run the application directly on the host
echo "Running the Rust application directly on the host..."
echo "Connecting to MongoDB at $MONGO_URI"
echo "Application will be available at http://localhost:8000"

# Modify src/main.rs to use the container IP (only if needed)
echo "Checking if we need to update MongoDB connection string in source code..."
if grep -q "mongodb://admin:password@127.0.0.1:27017" src/main.rs || grep -q "mongodb://admin:password@localhost:27017" src/main.rs; then
  echo "Temporarily updating MongoDB connection string in source code..."
  sed -i.bak "s|mongodb://admin:password@127.0.0.1:27017|$MONGO_URI|g" src/main.rs
  sed -i.bak "s|mongodb://admin:password@localhost:27017|$MONGO_URI|g" src/main.rs
  
  # Re-build with updated connection string
  echo "Rebuilding application with updated connection string..."
  cargo build --release
  
  # Ensure the binary has execute permissions
  chmod +x "$(pwd)/target/release/ddbp"
fi

# Start the application in the background and save its PID
./target/release/ddbp > app.log 2>&1 &
APP_PID=$!

# Wait for application to start
echo "Waiting for application to start..."
MAX_WAIT=10
for i in $(seq 1 $MAX_WAIT); do
  if lsof -i :8000 > /dev/null 2>&1; then
    echo "Application is running on port 8000"
    break
  else
    echo "Waiting for application to start... (attempt $i/$MAX_WAIT)"
    sleep 2
    
    # Check if the process is still running
    if ! ps -p $APP_PID > /dev/null; then
      echo "Error: Process died unexpectedly. Check app.log for details."
      cat app.log
      exit 1
    fi
  fi
done

# Test if the application is responding
echo "Testing API health endpoint..."
curl -s http://localhost:8000/health || {
  echo "Could not connect to application. Check app.log for details."
  cat app.log
  echo "Process output:"
  ps -p $APP_PID
  exit 1
}

echo "Application is running successfully with PID $APP_PID"
echo "To stop the application, run: kill $APP_PID"
echo "To view logs, run: cat app.log or tail -f app.log"
