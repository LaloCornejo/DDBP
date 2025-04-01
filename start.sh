#!/bin/bash

# Function to check if a container is running
check_container_running() {
  container_name=$1
  if ! podman container exists $container_name; then
    echo "Error: Container $container_name does not exist"
    exit 1
  fi
  
  state=$(podman inspect -f '{{.State.Status}}' $container_name 2>/dev/null)
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

# -----------------------------------------------------------
# Create MongoDB pods and containers

echo "Creating central MongoDB pod and container..."
# Create a central MongoDB pod
podman pod create --name central-mongodb -p 27017:27017

# Run a MongoDB container in the central pod with replication enabled
podman run -d --pod central-mongodb --name central-mongodb \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=password \
  -v $(pwd)/mongo-keyfile:/etc/mongo-keyfile:Z \
  mongo:latest \
  mongod --replSet rs0 --keyFile /etc/mongo-keyfile --bind_ip_all

# ----------------------------------------------------------
echo "Creating first secondary MongoDB pod and container..."
# Create the first secondary MongoDB pod
podman pod create --name secondary-mongodb-1 -p 27018:27017

# Run a MongoDB container in the first secondary pod with replication enabled
podman run -d --pod secondary-mongodb-1 --name secondary-mongodb-1 \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=password \
  -v $(pwd)/mongo-keyfile:/etc/mongo-keyfile:Z \
  mongo:latest \
  mongod --replSet rs0 --keyFile /etc/mongo-keyfile --bind_ip_all

echo "Creating second secondary MongoDB pod and container..."
# Create the second secondary MongoDB pod
podman pod create --name secondary-mongodb-2 -p 27019:27017

# Run a MongoDB container in the second secondary pod with replication enabled
podman run -d --pod secondary-mongodb-2 --name secondary-mongodb-2 \
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
central_ip=$(podman inspect -f '{{.NetworkSettings.Networks.podman.IPAddress}}' central-mongodb)
secondary1_ip=$(podman inspect -f '{{.NetworkSettings.Networks.podman.IPAddress}}' secondary-mongodb-1)
secondary2_ip=$(podman inspect -f '{{.NetworkSettings.Networks.podman.IPAddress}}' secondary-mongodb-2)

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

echo "MongoDB replica set setup complete."
# echo "Connection string: mongodb://admin:password@localhost:27017,localhost:27018,localhost:27019/?replicaSet=rs0"
echo "To connect to the MongoDB replica set, use the following connection string:"
echo "mongodb://admin:password@${central_ip}:27017,${secondary1_ip}:27017,${secondary2_ip}:27017/?replicaSet=rs0"
