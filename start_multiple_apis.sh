#!/bin/bash
# Start multiple API services connecting to different database nodes

# Number of API services to start
SERVICE_COUNT=3

# Function to wait for PostgreSQL to be ready
wait_for_postgres() {
  local container_name=$1
  local port=$2
  until podman exec -it $container_name pg_isready -h localhost -p $port; do
    echo "Waiting for PostgreSQL on port $port..."
    sleep 2
  done
}

for i in $(seq 1 $SERVICE_COUNT); do
  PORT=$((3000 + $i - 1))
  DB_PORT=$((5431 + $i))
  NODE_ID="node$i"
  DB_CONTAINER="pg-node$i"
  
  # Build a comma-separated list of all database URLs
  DB_URLS=""
  for j in $(seq 1 $SERVICE_COUNT); do
    DB_PORT_J=$((5431 + $j))
    if [ -n "$DB_URLS" ]; then
      DB_URLS="$DB_URLS,"
    fi
    DB_URLS="${DB_URLS}postgres://admin:password@localhost:$DB_PORT_J/social_media"
  done
  
  # Build a comma-separated list of all API node URLs
  API_NODES=""
  for j in $(seq 1 $SERVICE_COUNT); do
    API_PORT=$((3000 + $j - 1))
    if [ -n "$API_NODES" ]; then
      API_NODES="$API_NODES,"
    fi
    API_NODES="${API_NODES}http://localhost:$API_PORT"
  done
  
  echo "Waiting for PostgreSQL on port $DB_PORT..."
  wait_for_postgres $DB_CONTAINER $DB_PORT
  
  echo "Starting API service $NODE_ID on port $PORT connecting to database on port $DB_PORT..."
  
  # Start the API service in the background
  DATABASE_URL="postgres://admin:password@localhost:$DB_PORT/social_media" \
  DATABASE_URLS="$DB_URLS" \
  NODE_ID="$NODE_ID" \
  PORT="$PORT" \
  CLUSTER_NODES="$API_NODES" \
  cargo run &> api_service_$NODE_ID.log &
  
  # Sleep to prevent all services from starting at exactly the same time
  sleep 2
done

echo "All API services started."
echo "Press Ctrl+C to stop all services."

# Wait for user to press Ctrl+C
wait
