# Praxis API

HTTP REST API for Praxis AI Framework with Server-Sent Events (SSE) streaming support.

## Quick Start

### 1. Prerequisites

- MongoDB running (see `praxis_example` for setup scripts)
- OpenAI API key
- MCP servers configured (optional)

### 2. Configuration

Copy `.env.example` to `.env`:

```bash
cd crates/praxis-api
cp .env.example .env
```

Edit `.env`:

```bash
MONGODB_URI=mongodb://admin:password123@localhost:27017
OPENAI_API_KEY=sk-your-key-here
# Optional:
# SERVER_PORT=3000
# MCP_SERVERS=http://localhost:8000/mcp,http://localhost:8001/mcp
```

### 3. Run Server

```bash
cargo run --bin praxis-api
```

Server starts on `http://localhost:8000` by default.

## API Endpoints

### Health Check

```bash
GET /health
```

**Response:**
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

### Threads

#### Create Thread

```bash
POST /threads
Content-Type: application/json

{
  "user_id": "user_123",
  "metadata": {
    "title": "My Conversation",
    "tags": ["important", "support"]
  }
}
```

**Response (201 Created):**
```json
{
  "thread_id": "507f1f77bcf86cd799439011",
  "user_id": "user_123",
  "created_at": "2025-01-08T12:00:00Z",
  "updated_at": "2025-01-08T12:00:00Z",
  "metadata": {
    "title": "My Conversation",
    "tags": ["important", "support"]
  }
}
```

#### List Threads

```bash
GET /threads?user_id=user_123&limit=20
```

**Response:**
```json
{
  "threads": [
    {
      "thread_id": "507f1f77bcf86cd799439011",
      "user_id": "user_123",
      "created_at": "2025-01-08T12:00:00Z",
      "updated_at": "2025-01-08T12:00:00Z",
      "metadata": {
        "title": "My Conversation",
        "tags": []
      },
      "summary": {
        "text": "User asked about...",
        "generated_at": "2025-01-08T12:05:00Z",
        "total_tokens_before_summary": 25000,
        "messages_count": 45
      }
    }
  ],
  "has_more": false
}
```

#### Get Thread

```bash
GET /threads/{thread_id}
```

#### Delete Thread

```bash
DELETE /threads/{thread_id}
```

**Response:** 204 No Content

### Messages

#### List Messages

```bash
GET /threads/{thread_id}/messages?limit=50&before=message_id
```

**Response:**
```json
{
  "messages": [
    {
      "message_id": "507f1f77bcf86cd799439012",
      "thread_id": "507f1f77bcf86cd799439011",
      "role": "user",
      "message_type": "message",
      "content": "Hello, can you help me?",
      "created_at": "2025-01-08T12:01:00Z"
    },
    {
      "message_id": "507f1f77bcf86cd799439013",
      "thread_id": "507f1f77bcf86cd799439011",
      "role": "assistant",
      "message_type": "message",
      "content": "Of course! How can I assist you?",
      "created_at": "2025-01-08T12:01:05Z"
    }
  ],
  "has_more": false
}
```

#### Send Message (Streaming)

```bash
POST /threads/{thread_id}/messages
Content-Type: application/json
Accept: text/event-stream

{
  "user_id": "user_123",
  "content": "What's the weather like?"
}
```

**Response:** Server-Sent Events stream

```
event: message
data: {"content":"The"}

event: message
data: {"content":" weather"}

event: message
data: {"content":" is"}

event: tool_call
data: {"name":"get_weather","arguments":"{\"location\":\"New York\"}"}

event: tool_result
data: {"result":"Sunny, 72°F"}

event: message
data: {"content":"It's sunny and 72°F."}

event: done
data: {"status":"completed"}
```

## Server-Sent Events (SSE)

The API uses SSE for real-time streaming of AI responses, tool calls, and execution results.

### Event Types

- `message`: AI response chunk
- `reasoning`: Internal reasoning (if enabled)
- `tool_call`: Tool being called
- `tool_result`: Tool execution result
- `done`: Stream completed
- `error`: Error occurred
- `info`: Informational event

### Client Example (JavaScript)

```javascript
const evtSource = new EventSource(
  '/threads/507f1f77bcf86cd799439011/messages',
  {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      user_id: 'user_123',
      content: 'Hello!'
    })
  }
);

evtSource.addEventListener('message', (event) => {
  const data = JSON.parse(event.data);
  console.log('Message:', data.content);
});

evtSource.addEventListener('tool_call', (event) => {
  const data = JSON.parse(event.data);
  console.log('Tool:', data.name, data.arguments);
});

evtSource.addEventListener('done', () => {
  console.log('Stream completed');
  evtSource.close();
});

evtSource.addEventListener('error', (event) => {
  console.error('Error:', event.data);
  evtSource.close();
});
```

### Client Example (curl)

```bash
curl -N -X POST http://localhost:8000/threads/507f1f77bcf86cd799439011/messages \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "user_id": "user_123",
    "content": "What is 2+2?"
  }'

# easy!
THREAD_ID=$(curl -s -X POST http://localhost:8000/threads \
  -H "Content-Type: application/json" \
  -d '{"user_id":"test_user","metadata":{"tags":[]}}' | grep -o '"thread_id":"[^"]*"' | cut -d'"' -f4) && \
echo "Thread criado: $THREAD_ID" && \
curl -N -X POST http://localhost:8000/threads/$THREAD_ID/messages \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "user_id": "test_user",
    "content": "Solve this problem: If a train travels 120 km in 2 hours, what is its average speed? And after that make a dynamic programming problem that can be solved in O(n) time and space complexity.",
    "llm_config": {
      "model": "gpt-5",
      "reasoning_effort": "medium"    }
  }'
```

## Configuration

### TOML Configuration Files

Place configuration files in `crates/praxis-api/config/`:

- `default.toml`: Base configuration
- `dev.toml`: Development overrides
- `prod.toml`: Production overrides

Set `ENV` environment variable to choose config: `ENV=prod cargo run`

**Example `config/default.toml`:**

```toml
[server]
host = "0.0.0.0"
port = 8000
workers = 0  # 0 = num_cpus

[cors]
enabled = true
origins = ["http://localhost:3000", "http://localhost:5173"]

[mongodb]
database = "praxis"
pool_size = 10
timeout_ms = 5000

[llm]
model = "gpt-4o-mini"
max_tokens = 30000
temperature = 0.7

[mcp]
servers = "http://localhost:8000/mcp"

[logging]
level = "info"
format = "pretty"  # or "json"
```

### Environment Variables

Environment variables override TOML settings:

- `MONGODB_URI` (required): MongoDB connection string
- `OPENAI_API_KEY` (required): OpenAI API key
- `SERVER_PORT`: Override server port
- `SERVER_HOST`: Override server host
- `LLM_MODEL`: Override LLM model
- `LOG_LEVEL`: Override logging level (`debug`, `info`, `warn`, `error`)
- `LOG_FORMAT`: Override logging format (`pretty`, `json`)
- `MCP_SERVERS`: Comma-separated MCP server URLs

## Architecture

### Request Flow

```
HTTP Request
    ↓
Middleware (logging, CORS, timeout)
    ↓
Route Handler
    ↓
AppState (shared resources)
    ↓
PersistClient (MongoDB)
    ↓
Graph Executor (AI agent)
    ↓
SSE Stream
    ↓
HTTP Response
```

### Components

- **`Config`**: Hierarchical configuration (TOML + ENV)
- **`AppState`**: Shared application state
  - `PersistClient`: MongoDB persistence
  - `LLMClient`: OpenAI client
  - `MCPToolExecutor`: Tool execution
- **`ApiError`**: Unified error handling with automatic HTTP mapping
- **Routes**: HTTP endpoint handlers
- **Handlers**: Business logic for streaming
- **Middleware**: Request logging, CORS, compression

## Error Handling

All errors return JSON with a simple format:

```json
{
  "error": "Thread not found: 507f1f77bcf86cd799439011"
}
```

**Status Codes:**

- `200 OK`: Success
- `201 Created`: Resource created
- `204 No Content`: Success with no body
- `400 Bad Request`: Invalid request
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server error

## Development

### Run Tests

```bash
cargo test --package praxis-api
```

### Check Code

```bash
cargo check --package praxis-api
cargo clippy --package praxis-api
```

### Format Code

```bash
cargo fmt --package praxis-api
```

### Watch Mode

```bash
cargo watch -x 'run --bin praxis-api'
```

## Production Deployment

### Docker (Coming Soon)

### Environment Setup

1. Set `ENV=prod`
2. Configure `config/prod.toml`
3. Set required environment variables
4. Use reverse proxy (nginx, Caddy) for HTTPS
5. Configure MongoDB replica set for production
6. Set up monitoring and logging

### Performance Tips

- Use connection pooling (already configured)
- Enable compression (already configured)
- Set appropriate timeout values
- Monitor SSE connection count
- Use MongoDB indexes (created by `praxis_example` setup scripts)
- Configure rate limiting at reverse proxy level

## Troubleshooting

### MongoDB Connection Failed

```
Failed to load configuration: MONGODB_URI environment variable is required
```

**Fix:** Set `MONGODB_URI` in `.env` file or environment.

### MCP Server Connection Failed

```
Failed to connect to MCP server http://localhost:8000/mcp: Connection refused
```

**Fix:** MCP servers are optional. Either:
1. Start the MCP server
2. Remove from `MCP_SERVERS` environment variable
3. Leave empty for no MCP tools

### Port Already in Use

```
Address already in use (os error 48)
```

**Fix:** Change port with `SERVER_PORT=3000` or stop the conflicting process.

## License

MIT

