#!/bin/bash
set -e

# Function to check if MongoDB server is ready
check_mongodb_ready() {
  local host=$1
  echo "Checking if $host is ready..."
  for i in {1..30}; do
    if mongosh --host $host --port 27017 -u admin -p password --authenticationDatabase admin --eval "db.adminCommand('ping')" &>/dev/null; then
      echo "$host is ready!"
      return 0
    fi
    echo "Waiting for $host to be ready (attempt $i/30)..."
    sleep 2
  done
  echo "$host is not ready after 30 attempts"
  return 1
}

# Function to check if replica set is initialized and has a primary
check_replica_set_ready() {
  echo "Checking if replica set is ready..."
  for i in {1..30}; do
    # Check if a primary exists
    local primary=$(mongosh --host central-mongodb --port 27017 -u admin -p password --authenticationDatabase admin --quiet --eval "rs.status().members.find(m => m.state === 1)?.name || ''")
    if [ ! -z "$primary" ]; then
      echo "Replica set is ready with primary: $primary"
      return 0
    fi
    echo "Waiting for replica set to elect a primary (attempt $i/30)..."
    sleep 3
  done
  echo "Replica set failed to initialize with a primary node"
  return 1
}

echo "Waiting for MongoDB servers to be ready..."
# Initial sleep to let MongoDB instances start
sleep 15

# Check each MongoDB server
check_mongodb_ready "central-mongodb" || exit 1
check_mongodb_ready "secondary-mongodb-1" || exit 1
check_mongodb_ready "secondary-mongodb-2" || exit 1

echo "All MongoDB servers are ready. Initializing replica set..."

# Initialize replica set with retry
for i in {1..5}; do
  echo "Attempting to initialize replica set (attempt $i/5)..."
  mongosh --host central-mongodb --port 27017 -u admin -p password --authenticationDatabase admin --eval "
  rs.initiate({
    _id: 'rs0',
    members: [
      { _id: 0, host: 'central-mongodb:27017', priority: 10 },
      { _id: 1, host: 'secondary-mongodb-1:27017', priority: 5 },
      { _id: 2, host: 'secondary-mongodb-2:27017', priority: 1 }
    ]
  })
  " && break
  
  echo "Replica set initialization failed. Retrying in 5 seconds..."
  sleep 5
done

# Wait for replica set to be ready with a primary
echo "Waiting for primary election..."
check_replica_set_ready || exit 1

# Print replica set status for verification
echo "Checking replica set status..."
mongosh --host central-mongodb --port 27017 -u admin -p password --authenticationDatabase admin --eval "rs.status()"

echo "Waiting for replica set to fully stabilize..."
sleep 10

# Create application user and database
echo "Creating application user and database..."
mongosh --host central-mongodb --port 27017 -u admin -p password --authenticationDatabase admin --eval "
use social_media_db;
db.createUser({
  user: 'admin',
  pwd: 'password',
  roles: [
    { role: 'readWrite', db: 'social_media_db' },
    { role: 'dbAdmin', db: 'social_media_db' },
    { role: 'userAdmin', db: 'social_media_db' }
  ]
});
db.createCollection('users');
db.createCollection('posts');
db.createCollection('comments');
db.test.insertOne({ test: true });
"

# Verify database setup
echo "Verifying database setup..."
mongosh --host central-mongodb --port 27017 -u admin -p password --authenticationDatabase admin --eval "
use social_media_db;
db.getCollectionNames();
"

echo "MongoDB replica set setup complete!"
