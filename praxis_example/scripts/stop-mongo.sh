#!/bin/bash

set -e

echo "Stopping MongoDB"
echo "================"
echo ""

# Check if container exists and is running
if docker ps --format '{{.Names}}' | grep -q '^praxis-mongo$'; then
    if [ "$1" == "--clean" ]; then
        echo "Stopping MongoDB and removing volumes..."
        cd "$(dirname "$0")/.."
        docker-compose down -v
        echo "✓ MongoDB stopped and volumes removed"
    else
        echo "Stopping MongoDB..."
        cd "$(dirname "$0")/.."
        docker-compose down
        echo "✓ MongoDB stopped (data preserved)"
        echo ""
        echo "To also remove data volumes, use: $0 --clean"
    fi
else
    echo "MongoDB container is not running"
fi

