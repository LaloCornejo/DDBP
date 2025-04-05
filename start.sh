#!/bin/bash

# Function to check if a container is running
check_container_running() {
  container_name=$1
  if ! podman container exists $container_name; then
    echo "Error: Container $container_name does not exist"
    exit 1
  fi
  
  state=$(podman inspect --format '{{.State.Status}}' $container_name 2>/dev/null)
  if [ "$state" != "running" ]; then
    echo "Error: Container $container_name is not running. State: $state"
    echo "Logs for $container_name:"
    podman logs $container_name
    exit 1
  fi
}

# Clean up any existing containers and pods
echo "Cleaning up existing containers and pods..."
podman pod rm -f central-mongodb 2>/dev/null || true
podman pod rm -f secondary-mongodb-1 2>/dev/null || true
podman pod rm -f secondary-mongodb-2 2>/dev/null || true
podman container rm -f central-mongodb 2>/dev/null || true
podman container rm -f secondary-mongodb-1 2>/dev/null || true
podman container rm -f secondary-mongodb-2 2>/dev/null || true

# Create a keyfile for MongoDB replica set authentication
echo "Creating MongoDB keyfile..."
openssl rand -base64 756 > mongo-keyfile
chmod 600 mongo-keyfile

# Create a custom network
echo "Creating custom network..."
podman network create mongo-network || true

# -----------------------------------------------------------
# Create MongoDB pods and containers

echo "Creating central MongoDB pod and container..."
# Create a central MongoDB pod
podman pod create --name central-mongodb --network mongo-network -p 27017:27017

# Run a MongoDB container in the central pod with replication enabled
podman run -d --pod central-mongodb --name central-mongodb \
  --network mongo-network \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=password \
  -v $(pwd)/mongo-keyfile:/etc/mongo-keyfile:Z \
  mongo:latest \
  mongod --replSet rs0 --keyFile /etc/mongo-keyfile --bind_ip_all

# ----------------------------------------------------------
echo "Creating first secondary MongoDB pod and container..."
# Create the first secondary MongoDB pod
podman pod create --name secondary-mongodb-1 --network mongo-network -p 27018:27017

# Run a MongoDB container in the first secondary pod with replication enabled
podman run -d --pod secondary-mongodb-1 --name secondary-mongodb-1 \
  --network mongo-network \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=password \
  -v $(pwd)/mongo-keyfile:/etc/mongo-keyfile:Z \
  mongo:latest \
  mongod --replSet rs0 --keyFile /etc/mongo-keyfile --bind_ip_all

echo "Creating second secondary MongoDB pod and container..."
# Create the second secondary MongoDB pod
podman pod create --name secondary-mongodb-2 --network mongo-network -p 27019:27017

# Run a MongoDB container in the second secondary pod with replication enabled
podman run -d --pod secondary-mongodb-2 --name secondary-mongodb-2 \
  --network mongo-network \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=password \
  -v $(pwd)/mongo-keyfile:/etc/mongo-keyfile:Z \
  mongo:latest \
  mongod --replSet rs0 --keyFile /etc/mongo-keyfile --bind_ip_all

# ---------------------------------------------------------
# Wait for MongoDB containers to initialize
echo "Waiting for MongoDB containers to initialize (30 seconds)..."
sleep 30

# Check if all containers are running
echo "Checking if containers are running..."
check_container_running central-mongodb
check_container_running secondary-mongodb-1
check_container_running secondary-mongodb-2

# Get IP addresses of the pods
echo "Getting container IP addresses..."
# Fixed the inspect syntax - using --format instead of -f and getting IP from correct location
central_ip=$(podman inspect --format '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' central-mongodb)
secondary1_ip=$(podman inspect --format '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' secondary-mongodb-1)
secondary2_ip=$(podman inspect --format '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' secondary-mongodb-2)

# Fallback to pod IP if container IP is empty
if [ -z "$central_ip" ]; then
  central_ip=$(podman pod inspect --format '{{.NetworkSettings.IPs}}' central-mongodb | awk '{print $1}')
fi
if [ -z "$secondary1_ip" ]; then
  secondary1_ip=$(podman pod inspect --format '{{.NetworkSettings.IPs}}' secondary-mongodb-1 | awk '{print $1}')
fi
if [ -z "$secondary2_ip" ]; then
  secondary2_ip=$(podman pod inspect --format '{{.NetworkSettings.IPs}}' secondary-mongodb-2 | awk '{print $1}')
fi

# Second fallback if needed
if [ -z "$central_ip" ]; then
  central_ip=$(podman inspect --format '{{.NetworkSettings.IPAddress}}' central-mongodb)
fi
if [ -z "$secondary1_ip" ]; then
  secondary1_ip=$(podman inspect --format '{{.NetworkSettings.IPAddress}}' secondary-mongodb-1)
fi
if [ -z "$secondary2_ip" ]; then
  secondary2_ip=$(podman inspect --format '{{.NetworkSettings.IPAddress}}' secondary-mongodb-2)
fi

echo "Central MongoDB IP: $central_ip"
echo "Secondary MongoDB 1 IP: $secondary1_ip"
echo "Secondary MongoDB 2 IP: $secondary2_ip"

# Verify admin user creation and authentication
echo "Verifying MongoDB authentication..."
podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --eval 'db.runCommand({ connectionStatus: 1 })'

# Initialize the replica set with IP addresses
echo "Initializing replica set..."
podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --eval "
rs.initiate({
  _id: 'rs0',
  members: [
    { _id: 0, host: '${central_ip}:27017' },
    { _id: 1, host: '${secondary1_ip}:27017' },
    { _id: 2, host: '${secondary2_ip}:27017' }
  ]
})
"

# Check replica set status
echo "Checking replica set status..."
sleep 10
podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --eval "rs.status()"

# Ping test for each MongoDB container
echo "Pinging MongoDB containers to verify network connectivity..."

echo "Pinging Central MongoDB..."
ping -c 4 $central_ip

echo "Pinging Secondary MongoDB 1..."
ping -c 4 $secondary1_ip

echo "Pinging Secondary MongoDB 2..."
ping -c 4 $secondary2_ip

echo "MongoDB replica set setup complete."
echo "To connect to the MongoDB replica set, use the following connection string:"
echo "mongodb://admin:password@${central_ip}:27017,${secondary1_ip}:27017,${secondary2_ip}:27017/?replicaSet=rs0"
