# Testing Guide - Praxis API

Complete guide to test the Praxis API locally.

## Quick Start

### 1. Start MongoDB

```bash
cd praxis_example
./scripts/setup-mongo.sh
```

This will:
- Start MongoDB in Docker
- Create necessary indexes
- Confirm it's ready

### 2. Configure API

```bash
cd ../crates/praxis-api

# Copy and edit .env file
cp .env.example .env

# Edit .env and add your OpenAI API key:
# OPENAI_API_KEY=sk-your-key-here
```

**Important**: Get your OpenAI API key from https://platform.openai.com/api-keys

### 3. Start API Server

```bash
cargo run --bin praxis-api
```

You should see:
```
Starting Praxis API server
Config loaded: 0.0.0.0:8000
Initializing LLM client
Connecting to MCP servers
Connecting to MongoDB
MongoDB connected
Initializing Graph orchestrator
Server listening on 0.0.0.0:8000
Health check: http://0.0.0.0:8000/health
```

### 4. Run Automated Tests

In a **new terminal**:

```bash
cd crates/praxis-api
./test-api.sh
```

This script will:
1. Check if API is running
2. Test health endpoint
3. Create a thread
4. Get thread details
5. List threads
6. Send a message (streaming)
7. List messages

## Manual Testing

### Health Check

```bash
curl http://localhost:8000/health | jq '.'
```

**Expected Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "services": {
    "mongodb": "connected",
    "mcp": "available"
  }
}
```

### Create Thread

```bash
curl -X POST http://localhost:8000/threads \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "metadata": {
      "title": "My Test Conversation",
      "tags": ["test", "demo"]
    }
  }' | jq '.'
```

**Expected Response:**
```json
{
  "thread_id": "507f1f77bcf86cd799439011",
  "user_id": "user_123",
  "created_at": "2025-01-08T15:30:00Z",
  "updated_at": "2025-01-08T15:30:00Z",
  "metadata": {
    "title": "My Test Conversation",
    "tags": ["test", "demo"]
  }
}
```

**Save the thread_id for next steps!**

### Get Thread

```bash
# Replace with your thread_id
THREAD_ID="507f1f77bcf86cd799439011"

curl http://localhost:8000/threads/$THREAD_ID | jq '.'
```

### List Threads

```bash
curl "http://localhost:8000/threads?user_id=user_123&limit=10" | jq '.'
```

### Send Message (Streaming)

```bash
# Replace with your thread_id
THREAD_ID="507f1f77bcf86cd799439011"

curl -N -X POST http://localhost:8000/threads/$THREAD_ID/messages \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "user_id": "user_123",
    "content": "Hello! What is the capital of France?"
  }'
```

**Expected Output (streaming):**
```
event: message
data: {"content":"The"}

event: message
data: {"content":" capital"}

event: message
data: {"content":" of"}

event: message
data: {"content":" France"}

event: message
data: {"content":" is"}

event: message
data: {"content":" Paris"}

event: message
data: {"content":"."}

event: done
data: {"status":"completed"}
```

**Tip**: Press `Ctrl+C` to stop streaming.

### Send Message with Tool Call

If you have MCP servers configured:

```bash
curl -N -X POST http://localhost:8000/threads/$THREAD_ID/messages \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "user_id": "user_123",
    "content": "What is the weather like in New York?"
  }'
```

**Expected Output:**
```
event: message
data: {"content":"Let"}

event: message
data: {"content":" me"}

event: message
data: {"content":" check"}

event: tool_call
data: {"name":"get_weather","arguments":"{\"location\":\"New York\"}"}

event: tool_result
data: {"result":"Sunny, 72°F"}

event: message
data: {"content":"The"}

event: message
data: {"content":" weather"}

event: message
data: {"content":" in"}

event: message
data: {"content":" New"}

event: message
data: {"content":" York"}

event: message
data: {"content":" is"}

event: message
data: {"content":" sunny"}

event: message
data: {"content":" and"}

event: message
data: {"content":" 72°F"}

event: done
data: {"status":"completed"}
```

### List Messages

```bash
curl "http://localhost:8000/threads/$THREAD_ID/messages?limit=50" | jq '.'
```

**Expected Response:**
```json
{
  "messages": [
    {
      "message_id": "507f1f77bcf86cd799439012",
      "thread_id": "507f1f77bcf86cd799439011",
      "role": "user",
      "message_type": "message",
      "content": "Hello! What is the capital of France?",
      "created_at": "2025-01-08T15:31:00Z"
    },
    {
      "message_id": "507f1f77bcf86cd799439013",
      "thread_id": "507f1f77bcf86cd799439011",
      "role": "assistant",
      "message_type": "message",
      "content": "The capital of France is Paris.",
      "created_at": "2025-01-08T15:31:05Z"
    }
  ],
  "has_more": false
}
```

### Delete Thread

```bash
curl -X DELETE http://localhost:8000/threads/$THREAD_ID
```

**Expected**: HTTP 204 No Content (no body)

## Testing with MCP Tools

### 1. Start Python Weather MCP Server

In a separate terminal:

```bash
cd mcp_servers/weather
uv run python weather.py
```

Server will start on `http://localhost:8000/mcp`

### 2. Update API Configuration

Edit `crates/praxis-api/.env`:

```bash
MCP_SERVERS=http://localhost:8000/mcp
```

### 3. Restart API

```bash
# Stop current API (Ctrl+C in API terminal)
# Start again
cargo run --bin praxis-api
```

You should see:
```
Connecting to MCP servers...
Connected to MCP server: http://localhost:8000/mcp
```

### 4. Test Tool Calling

```bash
curl -N -X POST http://localhost:8000/threads/$THREAD_ID/messages \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "user_id": "user_123",
    "content": "What is the weather in San Francisco?"
  }'
```

## Testing with HTTPie (Alternative)

If you prefer HTTPie over curl:

```bash
# Install HTTPie
brew install httpie  # macOS
# or
pip install httpie

# Health check
http GET localhost:8000/health

# Create thread
http POST localhost:8000/threads \
  user_id=user_123 \
  metadata:='{"title":"Test","tags":[]}'

# Send message
http --stream POST localhost:8000/threads/$THREAD_ID/messages \
  user_id=user_123 \
  content="Hello!"
```

## Testing with Postman

1. Import the collection (coming soon)
2. Set environment variables:
   - `base_url`: `http://localhost:8000`
   - `user_id`: `user_123`
   - `thread_id`: (will be set after creating thread)

## Common Issues

### "Connection refused"

**Problem**: API not running
**Solution**: Start the API with `cargo run --bin praxis-api`

### "MONGODB_URI environment variable is required"

**Problem**: .env file not configured
**Solution**: 
```bash
cd crates/praxis-api
cp .env.example .env
# Edit .env and set MONGODB_URI
```

### "Failed to connect to MongoDB"

**Problem**: MongoDB not running
**Solution**:
```bash
cd praxis_example
./scripts/setup-mongo.sh
```

### "OPENAI_API_KEY must be set"

**Problem**: OpenAI API key not configured
**Solution**: Get key from https://platform.openai.com/api-keys and add to .env:
```bash
OPENAI_API_KEY=sk-your-key-here
```

### Streaming not working in browser

**Problem**: Browser EventSource doesn't support POST with body
**Solution**: Use curl, HTTPie, or a custom JavaScript client:

```javascript
// Custom streaming client
fetch('http://localhost:8000/threads/THREAD_ID/messages', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'text/event-stream',
  },
  body: JSON.stringify({
    user_id: 'user_123',
    content: 'Hello!'
  })
}).then(response => {
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  
  function readChunk() {
    reader.read().then(({done, value}) => {
      if (done) return;
      console.log(decoder.decode(value));
      readChunk();
    });
  }
  
  readChunk();
});
```

## Performance Testing

### Simple Load Test with `ab` (Apache Bench)

```bash
# Install ab
brew install httpd  # macOS

# Test health endpoint (100 requests, 10 concurrent)
ab -n 100 -c 10 http://localhost:8000/health

# Test thread creation (POST requests)
ab -n 50 -c 5 -p thread.json -T application/json http://localhost:8000/threads
```

Create `thread.json`:
```json
{
  "user_id": "load_test_user",
  "metadata": {"title": "Load Test"}
}
```

### Load Test with `wrk`

```bash
# Install wrk
brew install wrk  # macOS

# Test health endpoint
wrk -t4 -c100 -d30s http://localhost:8000/health
```

## Next Steps

1. Try different prompts and see how the agent responds
2. Test with your own MCP servers
3. Integrate with a frontend (React, Vue, etc.)
4. Add authentication to the API
5. Deploy to production

## Useful Commands

```bash
# View API logs in real-time
cargo run --bin praxis-api | jq '.'  # If using json format

# Check MongoDB contents
docker exec -it praxis-mongo mongosh -u admin -p password123 praxis
# Then: db.threads.find().pretty()

# Stop all services
# API: Ctrl+C in terminal
# MongoDB: cd praxis_example && ./scripts/stop-mongo.sh
```

## License

MIT

