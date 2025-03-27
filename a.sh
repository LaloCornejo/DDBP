#!/bin/bash

# Clean up any existing Podman pods that may interfere
podman pod rm -f $(podman pod ls -q) 2>/dev/null
podman container rm -f $(podman container ls -q) 2>/dev/null

# Create a common network for all nodes
podman network create social-media-network

# Create the init.sql file for schema initialization
cat > init.sql << 'EOF'
CREATE TABLE IF NOT EXISTS posts (
    id UUID PRIMARY KEY,
    content TEXT NOT NULL,
    author TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE,
    origin_node TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    last_seen TIMESTAMP WITH TIME ZONE NOT NULL
);
EOF

# Number of database nodes to create
NODE_COUNT=3

# Creaate a central node 
echo "Starting central database node on port 6969..."
NODE_NAME="mother-node"
PORT=6969

podman run -d \
  --name $NODE_NAME \
  --network social-media-network \
  -p $PORT:6969 \
  -e POSTGRES_USER=admin \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=social_media \
  -v "$(pwd)/init.sql:/docker-entrypoint-initdb.d/init.sql:Z" \
  postgres:14


# Create and start each node
for i in $(seq 1 $NODE_COUNT); do
  NODE_NAME="pg-node$i"
  PORT=$((5431 + $i))

  echo "Starting database node $NODE_NAME on port $PORT..."

  # Run PostgreSQL container with a unique name and port mapping
  podman run -d \
    --name $NODE_NAME \
    --network social-media-network \
    -p $PORT:5432 \
    -e POSTGRES_USER=admin \
    -e POSTGRES_PASSWORD=password \
    -e POSTGRES_DB=social_media \
    -v "$(pwd)/init.sql:/docker-entrypoint-initdb.d/init.sql:Z" \
    postgres:14

  echo "Node $NODE_NAME started on port $PORT"
done

# Print connection information
echo "PostgreSQL cluster setup complete!"
echo "Node connection URLs:"
for i in $(seq 1 $NODE_COUNT); do
  NODE_NAME="pg-node$i"
  PORT=$((5431 + $i))
  echo "  - $NODE_NAME: postgres://admin:password@localhost:$PORT/social_media"
done
  echo "  - CENTRAL NODE: postgres://admin:password@localhost:6969/social_media"
