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

# Function to get container IP address
get_container_ip() {
  container_name=$1
  ip=$(podman inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $container_name 2>/dev/null)
  if [ -z "$ip" ]; then
    echo "Error: Could not get IP address for container $container_name"
    exit 1
  fi
  echo "$ip"
}

# Create a keyfile for MongoDB replica set authentication
echo "Creating MongoDB keyfile..."
openssl rand -base64 756 > mongo-keyfile
chmod 600 mongo-keyfile

# Create a custom network
echo "Creating custom network..."
# First remove any existing network
podman network rm -f mongo-network 2>/dev/null || true
# Then create a new network with a specific subnet
podman network create --driver bridge --subnet 10.89.0.0/24 mongo-network

# -----------------------------------------------------------
# Create MongoDB pods and containers

echo "Creating central MongoDB pod and container..."
# Create a central MongoDB pod with explicit host bindings
podman pod create --name central-mongodb --network mongo-network -p 127.0.0.1:27017:27017

# Run a MongoDB container in the central pod with replication enabled
# Removed the problematic --setParameter
podman run -d --pod central-mongodb --name central-mongodb \
  --network mongo-network \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=password \
  -v $(pwd)/mongo-keyfile:/etc/mongo-keyfile:Z \
  mongo:latest \
  mongod --replSet rs0 --keyFile /etc/mongo-keyfile --bind_ip_all --auth --port 27017

# ----------------------------------------------------------
echo "Creating first secondary MongoDB pod and container..."
# Create the first secondary MongoDB pod with explicit host bindings
podman pod create --name secondary-mongodb-1 --network mongo-network -p 127.0.0.1:27018:27017

# Run a MongoDB container in the first secondary pod with replication enabled
podman run -d --pod secondary-mongodb-1 --name secondary-mongodb-1 \
  --network mongo-network \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=password \
  -v $(pwd)/mongo-keyfile:/etc/mongo-keyfile:Z \
  mongo:latest \
  mongod --replSet rs0 --keyFile /etc/mongo-keyfile --bind_ip_all --auth --port 27017

echo "Creating second secondary MongoDB pod and container..."
# Create the second secondary MongoDB pod with explicit host bindings
podman pod create --name secondary-mongodb-2 --network mongo-network -p 127.0.0.1:27019:27017

# Run a MongoDB container in the second secondary pod with replication enabled
podman run -d --pod secondary-mongodb-2 --name secondary-mongodb-2 \
  --network mongo-network \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=password \
  -v $(pwd)/mongo-keyfile:/etc/mongo-keyfile:Z \
  mongo:latest \
  mongod --replSet rs0 --keyFile /etc/mongo-keyfile --bind_ip_all --auth --port 27017

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

# Get IP addresses of all MongoDB containers
CENTRAL_IP=$(get_container_ip central-mongodb)
SECONDARY1_IP=$(get_container_ip secondary-mongodb-1)
SECONDARY2_IP=$(get_container_ip secondary-mongodb-2)

echo "Using IP addresses for replica set configuration:"
echo "Central MongoDB: $CENTRAL_IP"
echo "Secondary MongoDB 1: $SECONDARY1_IP"
echo "Secondary MongoDB 2: $SECONDARY2_IP"

# Initialize replica set using container network IPs
podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --eval "
rs.initiate({
  _id: 'rs0',
  members: [
    { _id: 0, host: '$CENTRAL_IP:27017', priority: 10 },
    { _id: 1, host: '$SECONDARY1_IP:27017', priority: 5 },
    { _id: 2, host: '$SECONDARY2_IP:27017', priority: 1 }
  ]
})
"

# Check replica set status
echo "Checking replica set status..."
sleep 10
podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --eval "rs.status()"

# Wait for primary election
echo "Waiting for primary election (up to 30 seconds)..."
for i in {1..6}; do
  # Check if a primary has been elected
  is_primary=$(podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --quiet --eval "rs.status().members.filter(m => m.state === 1).length > 0")
  if [ "$is_primary" = "true" ]; then
    echo "Primary node elected successfully!"
    break
  fi
  echo "Waiting for primary election... (attempt $i/6)"
  sleep 5
done

# Final status check
echo "Final replica set status:"
podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --eval "rs.status()"

# Add a note about the connection string to use from the host
echo ""
echo "MongoDB replica set is now ready!"
echo "Use the following connection string in your application:"
echo "mongodb://admin:password@127.0.0.1:27017,127.0.0.1:27018,127.0.0.1:27019/?replicaSet=rs0&authSource=admin&directConnection=false&serverSelectionTimeoutMS=30000&connectTimeoutMS=20000"
echo ""

# Create a test user and database
podman exec -it central-mongodb mongosh --host localhost --port 27017 -u admin -p password --authenticationDatabase admin --eval "
use social_media_db;
db.createUser({
  user: 'app_user',
  pwd: 'app_password',
  roles: [{ role: 'readWrite', db: 'social_media_db' }]
});
db.test.insertOne({ test: true });
db.getCollectionNames();
"
