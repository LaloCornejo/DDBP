#!/bin/bash

# Connect to the primary MongoDB instance and check replica set status
podman exec -it central-mongodb mongosh -u admin -p password --authenticationDatabase admin --eval 'rs.status()'
