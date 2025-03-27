#!/bin/bash

# Array of environment files
ENV_FILES=(.env .env1 .env2 .env3)

# Function to kill processes running on specified ports
kill_ports() {
    for env_file in "${ENV_FILES[@]}"; do
        port=$(grep 'PORT=' "$env_file" | cut -d'=' -f2)
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

# Start the API services with the specified environment files
for env_file in "${ENV_FILES[@]}"; do
    echo "Starting API server with environment file $env_file..."
    env $(cat "$env_file" | xargs) cargo run &
done

# Wait for all background processes to finish
wait
