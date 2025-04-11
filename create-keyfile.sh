#!/bin/bash

# Create a keyfile for MongoDB replica set authentication
echo "Creating MongoDB keyfile..."
openssl rand -base64 756 > mongo-keyfile
chmod 600 mongo-keyfile

echo "MongoDB keyfile created successfully!"
