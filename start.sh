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

# Function to wait for MongoDB container to be ready
wait_for_mongo_ready() {
  container_name=$1
  retries=10
  while [ $retries -gt 0 ]; do
    if podman exec $container_name mongosh --eval "db.runCommand({ ping: 1 })" > /dev/null 2>&1; then
      echo "MongoDB container $container_name is ready"
      return 0
    else
      echo "Waiting for MongoDB container $container_name to be ready..."
      sleep 5
      ((retries--))
    fi
  done
  echo "Error: MongoDB container $container_name is not ready after waiting"
  exit 1
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

# Wait for MongoDB containers to be ready
echo "Waiting for MongoDB containers to be ready..."
wait_for_mongo_ready central-mongodb
wait_for_mongo_ready secondary-mongodb-1
wait_for_mongo_ready secondary-mongodb-2

# Initialize the replica set with IP addresses
echo "Initializing replica set..."
podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --eval "
rs.initiate({
  _id: 'rs0',
  members: [
    { _id: 0, host: 'central-mongodb:27017' },
    { _id: 1, host: 'secondary-mongodb-1:27017' },
    { _id: 2, host: 'secondary-mongodb-2:27017' }
  ]
})
"

# Check replica set status
echo "Checking replica set status..."
sleep 10
podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --eval "rs.status()"
