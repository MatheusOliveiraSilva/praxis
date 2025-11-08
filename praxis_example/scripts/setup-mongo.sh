#!/bin/bash

set -e

echo "MongoDB Setup for Praxis"
echo "========================="
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Error: Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if container exists
if docker ps -a --format '{{.Names}}' | grep -q '^praxis-mongo$'; then
    # Container exists, check if it's running
    if docker ps --format '{{.Names}}' | grep -q '^praxis-mongo$'; then
        echo "✓ MongoDB container 'praxis-mongo' is already running"
    else
        echo "Starting existing MongoDB container..."
        docker start praxis-mongo
        echo "✓ MongoDB container started"
    fi
else
    # Container doesn't exist, create it via docker-compose
    echo "Creating MongoDB container..."
    cd "$(dirname "$0")/.."
    docker-compose up -d
    echo "✓ MongoDB container created and started"
fi

# Wait for MongoDB to be ready
echo ""
echo "Waiting for MongoDB to be ready..."
for i in {1..30}; do
    if docker exec praxis-mongo mongosh -u admin -p password123 --eval "db.adminCommand('ping')" > /dev/null 2>&1; then
        echo "✓ MongoDB is ready"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "Error: MongoDB failed to start after 30 seconds"
        exit 1
    fi
    sleep 1
done

# Create indexes
echo ""
echo "Creating indexes..."
docker exec praxis-mongo mongosh -u admin -p password123 --authenticationDatabase admin praxis /docker-entrypoint-initdb.d/init-indexes.js

echo ""
echo "========================="
echo "MongoDB Setup Complete!"
echo "========================="
echo ""
echo "Connection URI: mongodb://admin:password123@localhost:27017"
echo "Database: praxis"
echo ""
echo "To stop MongoDB: ./scripts/stop-mongo.sh"
echo "To test connection: cargo run --bin test-mongo"

