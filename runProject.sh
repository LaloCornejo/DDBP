#!/bin/bash
echo "Start.sh script running..."
exec ./start.sh
echo "Done with start.sh script."

# Build the Rust application
echo "Building the Rust application..."
cargo build --release

# Run the Rust application in a Podman container
echo "Running the Rust application in a Podman container..."
podman run -it --rm \
  --network mongo-network \
  -v $(pwd)/target/release/ddbp:/usr/local/bin/ddbp \
  -v $(pwd)/.env:/usr/local/bin/.env \
  ddbp:latest /usr/local/bin/ddbp

echo "Application is running. You can access it at http://127.0.0.1:8000"
