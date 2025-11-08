#!/bin/bash

# Praxis All-in-One Stop Script
# Stops all services gracefully

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Praxis AI Agent - Stop Script                   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Stop Web UI
if [ -f /tmp/praxis-ui.pid ]; then
    UI_PID=$(cat /tmp/praxis-ui.pid)
    echo -e "${YELLOW}Stopping Web UI (PID: $UI_PID)...${NC}"
    kill $UI_PID 2>/dev/null || true
    rm /tmp/praxis-ui.pid
    echo -e "${GREEN}✓ Web UI stopped${NC}"
else
    echo -e "${YELLOW}⚠ Web UI not running (no PID file)${NC}"
    # Kill by port
    lsof -ti:3000 | xargs kill -9 2>/dev/null || true
fi

# Stop Praxis API
if [ -f /tmp/praxis-api.pid ]; then
    API_PID=$(cat /tmp/praxis-api.pid)
    echo -e "${YELLOW}Stopping Praxis API (PID: $API_PID)...${NC}"
    kill $API_PID 2>/dev/null || true
    rm /tmp/praxis-api.pid
    echo -e "${GREEN}✓ Praxis API stopped${NC}"
else
    echo -e "${YELLOW}⚠ Praxis API not running (no PID file)${NC}"
    # Kill by port
    lsof -ti:8000 | xargs kill -9 2>/dev/null || true
fi

# Stop MCP Server
if [ -f /tmp/praxis-mcp.pid ]; then
    MCP_PID=$(cat /tmp/praxis-mcp.pid)
    echo -e "${YELLOW}Stopping MCP Server (PID: $MCP_PID)...${NC}"
    kill $MCP_PID 2>/dev/null || true
    rm /tmp/praxis-mcp.pid
    echo -e "${GREEN}✓ MCP Server stopped${NC}"
else
    echo -e "${YELLOW}⚠ MCP Server not running (no PID file)${NC}"
    # Kill by port
    lsof -ti:8005 | xargs kill -9 2>/dev/null || true
fi

# Stop MongoDB
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

if docker ps | grep -q praxis-mongo; then
    echo -e "${YELLOW}Stopping MongoDB...${NC}"
    cd "$PROJECT_ROOT/praxis_example"
    ./scripts/stop-mongo.sh
    echo -e "${GREEN}✓ MongoDB stopped${NC}"
else
    echo -e "${YELLOW}⚠ MongoDB not running${NC}"
fi

# Clean up log files
rm -f /tmp/praxis-*.log

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ✓ All Services Stopped                       ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

