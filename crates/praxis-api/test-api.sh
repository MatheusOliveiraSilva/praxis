#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# API base URL
API_URL="http://localhost:8000"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   Praxis API Test Suite${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if server is running
echo -e "${YELLOW}Checking if API is running...${NC}"
if ! curl -s "${API_URL}/health" > /dev/null 2>&1; then
    echo -e "${RED}Error: API is not running!${NC}"
    echo "Please start the server first:"
    echo "  cd crates/praxis-api"
    echo "  cargo run --bin praxis-api"
    exit 1
fi
echo -e "${GREEN}✓ API is running${NC}"
echo ""

# 1. Health Check
echo -e "${BLUE}1. Testing Health Check${NC}"
echo -e "${YELLOW}GET /health${NC}"
curl -s "${API_URL}/health" | jq '.'
echo ""
echo ""

# 2. Create Thread
echo -e "${BLUE}2. Creating a new thread${NC}"
echo -e "${YELLOW}POST /threads${NC}"
THREAD_RESPONSE=$(curl -s -X POST "${API_URL}/threads" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "test_user_123",
    "metadata": {
      "title": "Test Conversation",
      "tags": ["test", "demo"]
    }
  }')

echo "$THREAD_RESPONSE" | jq '.'
THREAD_ID=$(echo "$THREAD_RESPONSE" | jq -r '.thread_id')
echo -e "${GREEN}✓ Thread created: ${THREAD_ID}${NC}"
echo ""
echo ""

# 3. Get Thread
echo -e "${BLUE}3. Getting thread details${NC}"
echo -e "${YELLOW}GET /threads/${THREAD_ID}${NC}"
curl -s "${API_URL}/threads/${THREAD_ID}" | jq '.'
echo ""
echo ""

# 4. List Threads
echo -e "${BLUE}4. Listing threads for user${NC}"
echo -e "${YELLOW}GET /threads?user_id=test_user_123${NC}"
curl -s "${API_URL}/threads?user_id=test_user_123&limit=10" | jq '.'
echo ""
echo ""

# 5. Send Message (non-streaming for now)
echo -e "${BLUE}5. Sending a message (streaming will be tested separately)${NC}"
echo -e "${YELLOW}POST /threads/${THREAD_ID}/messages${NC}"
echo -e "${YELLOW}Note: This will stream, showing first few events...${NC}"
echo ""

# Start streaming in background, capture first few events
curl -N -X POST "${API_URL}/threads/${THREAD_ID}/messages" \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "user_id": "test_user_123",
    "content": "Hello! Can you help me with a simple math problem? What is 2+2?"
  }' 2>/dev/null | head -n 20 &

STREAM_PID=$!
sleep 3
kill $STREAM_PID 2>/dev/null
wait $STREAM_PID 2>/dev/null

echo ""
echo -e "${GREEN}✓ Message sent (streaming started)${NC}"
echo ""
echo ""

# Wait a bit for message to be saved
echo -e "${YELLOW}Waiting 2 seconds for message to be processed...${NC}"
sleep 2
echo ""

# 6. List Messages
echo -e "${BLUE}6. Listing messages in thread${NC}"
echo -e "${YELLOW}GET /threads/${THREAD_ID}/messages${NC}"
curl -s "${API_URL}/threads/${THREAD_ID}/messages?limit=20" | jq '.'
echo ""
echo ""

# 7. Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}   Test Suite Completed!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "Thread ID: ${YELLOW}${THREAD_ID}${NC}"
echo ""
echo -e "${BLUE}Manual Testing Commands:${NC}"
echo ""
echo -e "${YELLOW}# Stream a full conversation:${NC}"
echo "curl -N -X POST '${API_URL}/threads/${THREAD_ID}/messages' \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -H 'Accept: text/event-stream' \\"
echo "  -d '{\"user_id\": \"test_user_123\", \"content\": \"Your message here\"}'"
echo ""
echo -e "${YELLOW}# Get thread details:${NC}"
echo "curl '${API_URL}/threads/${THREAD_ID}' | jq '.'"
echo ""
echo -e "${YELLOW}# List all messages:${NC}"
echo "curl '${API_URL}/threads/${THREAD_ID}/messages' | jq '.'"
echo ""

