#!/bin/bash

# Praxis All-in-One Startup Script
# Starts all services in the correct order

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Praxis AI Agent - Startup Script                ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to check if port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1 ; then
        return 0
    else
        return 1
    fi
}

# Function to wait for service
wait_for_service() {
    local name=$1
    local url=$2
    local max_attempts=30
    local attempt=1
    
    echo -e "${YELLOW}⏳ Waiting for $name to be ready...${NC}"
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ $name is ready!${NC}"
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done
    
    echo -e "${RED}✗ $name failed to start after $max_attempts seconds${NC}"
    return 1
}

# 1. Check prerequisites
echo -e "${BLUE}[1/5] Checking prerequisites...${NC}"

command -v docker >/dev/null 2>&1 || { echo -e "${RED}✗ Docker is not installed${NC}"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}✗ Rust/Cargo is not installed${NC}"; exit 1; }
command -v node >/dev/null 2>&1 || { echo -e "${RED}✗ Node.js is not installed${NC}"; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo -e "${RED}✗ Python 3 is not installed${NC}"; exit 1; }
command -v uv >/dev/null 2>&1 || { echo -e "${RED}✗ uv is not installed (pip install uv)${NC}"; exit 1; }

echo -e "${GREEN}✓ All prerequisites met${NC}"
echo ""

# 2. Start MongoDB
echo -e "${BLUE}[2/5] Starting MongoDB...${NC}"

if check_port 27017; then
    echo -e "${YELLOW}⚠ MongoDB already running on port 27017${NC}"
else
    cd "$PROJECT_ROOT/praxis_example"
    ./scripts/setup-mongo.sh
    wait_for_service "MongoDB" "mongodb://admin:password123@localhost:27017" || exit 1
fi
echo ""

# 3. Start MCP Weather Server
echo -e "${BLUE}[3/5] Starting MCP Weather Server...${NC}"

if check_port 8005; then
    echo -e "${YELLOW}⚠ MCP Server already running on port 8005${NC}"
else
    cd "$PROJECT_ROOT/mcp_servers/weather"
    PORT=8005 uv run python weather.py > /tmp/praxis-mcp.log 2>&1 &
    MCP_PID=$!
    echo $MCP_PID > /tmp/praxis-mcp.pid
    wait_for_service "MCP Server" "http://localhost:8005/mcp" || exit 1
    echo -e "${GREEN}✓ MCP Server started (PID: $MCP_PID)${NC}"
fi
echo ""

# 4. Start Praxis API
echo -e "${BLUE}[4/5] Starting Praxis API...${NC}"

# Check for .env file
if [ ! -f "$PROJECT_ROOT/crates/praxis-api/.env" ]; then
    echo -e "${RED}✗ Missing .env file in crates/praxis-api/${NC}"
    echo -e "${YELLOW}Please create it with:${NC}"
    echo -e "  OPENAI_API_KEY=your-key-here"
    echo -e "  MONGODB_URI=mongodb://admin:password123@localhost:27017"
    echo -e "  MCP_SERVERS=http://localhost:8005/mcp"
    exit 1
fi

if check_port 8000; then
    echo -e "${YELLOW}⚠ Praxis API already running on port 8000${NC}"
else
    cd "$PROJECT_ROOT/crates/praxis-api"
    cargo run --release --bin praxis-api > /tmp/praxis-api.log 2>&1 &
    API_PID=$!
    echo $API_PID > /tmp/praxis-api.pid
    wait_for_service "Praxis API" "http://localhost:8000/health" || exit 1
    echo -e "${GREEN}✓ Praxis API started (PID: $API_PID)${NC}"
fi
echo ""

# 5. Start Web UI (optional)
echo -e "${BLUE}[5/5] Starting Web UI...${NC}"

if check_port 3000; then
    echo -e "${YELLOW}⚠ Web UI already running on port 3000${NC}"
else
    cd "$PROJECT_ROOT/agent_ui"
    
    # Check if node_modules exists
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}📦 Installing frontend dependencies...${NC}"
        npm install
    fi
    
    npm run dev > /tmp/praxis-ui.log 2>&1 &
    UI_PID=$!
    echo $UI_PID > /tmp/praxis-ui.pid
    sleep 5  # Next.js takes a bit longer to start
    echo -e "${GREEN}✓ Web UI started (PID: $UI_PID)${NC}"
fi
echo ""

# Summary
echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                   🎉 All Services Running!                 ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Services:${NC}"
echo -e "  • MongoDB:         ${GREEN}mongodb://localhost:27017${NC}"
echo -e "  • MCP Server:      ${GREEN}http://localhost:8005/mcp${NC}"
echo -e "  • Praxis API:      ${GREEN}http://localhost:8000${NC}"
echo -e "  • Web UI:          ${GREEN}http://localhost:3000${NC}"
echo ""
echo -e "${BLUE}Logs:${NC}"
echo -e "  • MCP Server:      tail -f /tmp/praxis-mcp.log"
echo -e "  • Praxis API:      tail -f /tmp/praxis-api.log"
echo -e "  • Web UI:          tail -f /tmp/praxis-ui.log"
echo ""
echo -e "${BLUE}Stop all services:${NC}"
echo -e "  ./scripts/stop-all.sh"
echo ""
echo -e "${YELLOW}Test the API:${NC}"
echo -e "  curl http://localhost:8000/health"
echo ""
echo -e "${YELLOW}Open Web UI:${NC}"
echo -e "  open http://localhost:3000"
echo ""

