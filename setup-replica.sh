#!/bin/bash

# Wait for MongoDB servers to be ready
sleep 30

echo "Initializing replica set..."

# Initialize replica set
mongosh --host central-mongodb --port 27017 -u admin -p password --authenticationDatabase admin --eval "
rs.initiate({
  _id: 'rs0',
  members: [
    { _id: 0, host: 'central-mongodb:27017', priority: 10 },
    { _id: 1, host: 'secondary-mongodb-1:27017', priority: 5 },
    { _id: 2, host: 'secondary-mongodb-2:27017', priority: 1 }
  ]
})
"

# Wait for primary election
sleep 10
echo "Checking replica set status..."
mongosh --host central-mongodb --port 27017 -u admin -p password --authenticationDatabase admin --eval "rs.status()"

# Create application user and database
echo "Creating application user and database..."
mongosh --host central-mongodb --port 27017 -u admin -p password --authenticationDatabase admin --eval "
use social_media_db;
db.createUser({
  user: 'admin',
  pwd: 'password',
  roles: [{ role: 'readWrite', db: 'social_media_db' }]
});
db.test.insertOne({ test: true });
"

echo "MongoDB replica set setup complete!"
