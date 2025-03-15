#!/bin/bash

# Ports to be used for the APIs
PORTS=(3000 3001 3002 6969)

# Function to kill processes running on specified ports
kill_ports() {
    for port in "${PORTS[@]}"; do
        echo "Checking port $port..."
        pid=$(lsof -ti tcp:$port)
        if [ -n "$pid" ]; then
            echo "Killing process $pid on port $port..."
            kill -9 $pid
        else
            echo "No process found on port $port."
        fi
    done
}

# Kill processes running on the specified ports
kill_ports

# Start the API services on the specified ports
for port in "${PORTS[@]}"; do
    echo "Starting API server on port $port..."
    DATABASE_URL=postgres://admin:password@localhost:5432/social_media \
    HOST=0.0.0.0 \
    PORT=$port \
    NODE_ID=node$port \
    CLUSTER_NODES=http://localhost:3001,http://localhost:3002,http://localhost:6969 \
    cargo run &
done

# Wait for all background processes to finish
wait
