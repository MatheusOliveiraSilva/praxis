# Architecture Checkpoint 6: REST API Layer (praxis-api)

**Status**: 📋 Ready for Implementation  
**Date**: 2025-11-07  
**Phase**: Production-Ready HTTP Gateway

---

## Overview

The **praxis-api** crate provides a production-ready HTTP API gateway built on Axum, enabling real-time streaming chat with MCP-powered agents. It exposes the Praxis runtime via RESTful endpoints with Server-Sent Events (SSE) for streaming responses.

### Goals

1. **Production-ready**: Deploy-ready API with proper error handling, logging, graceful shutdown
2. **Real-time streaming**: SSE-based streaming for token-by-token responses
3. **Stateless**: All state in MongoDB, API servers are horizontally scalable
4. **Observable**: Structured logging, health checks, metrics hooks
5. **Simple auth**: API key validation (JWT/OAuth later)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Client (Browser/CLI)                │
│                                                             │
│  - Fetch API / axios                                        │
│  - EventSource (SSE connection)                             │
│  - Displays streaming reasoning + messages                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP/1.1
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      praxis-api (Axum)                      │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Router                                              │  │
│  │  - POST   /v1/chat                                   │  │
│  │  - POST   /v1/chat/{id}/stream                       │  │
│  │  - GET    /v1/chat/{id}                              │  │
│  │  - POST   /v1/agents                                 │  │
│  │  - GET    /v1/agents/{id}                            │  │
│  │  - GET    /v1/health                                 │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Middleware Stack                                    │  │
│  │  - Logging (tracing)                                 │  │
│  │  - CORS                                              │  │
│  │  - Request ID injection                              │  │
│  │  - Timeout (30s default)                             │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Handlers                                            │  │
│  │  - create_thread()                                   │  │
│  │  - stream_chat()      → Graph.spawn_run()            │  │
│  │  - get_thread_history()                              │  │
│  │  - create_agent()                                    │  │
│  │  - get_agent()                                       │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      praxis-graph + praxis-mcp              │
│                                                             │
│  - Graph orchestration                                      │
│  - MCPToolExecutor with multiple servers                    │
│  - Real-time event emission                                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      praxis-persist (MongoDB)               │
│                                                             │
│  - threads collection                                       │
│  - messages collection                                      │
└─────────────────────────────────────────────────────────────┘
```

---

## API Endpoints

### 1. Create Thread

**POST** `/v1/chat`

Create a new conversation thread.

**Request:**
```json
{
  "user_id": "user_123",
  "metadata": {
    "title": "Weather inquiry",
    "tags": ["weather", "san-francisco"]
  }
}
```

**Response:**
```json
{
  "thread_id": "thread_abc123",
  "user_id": "user_123",
  "created_at": "2025-11-07T10:30:00Z",
  "metadata": { ... }
}
```

**Status Codes:**
- `201 Created` - Thread created successfully
- `400 Bad Request` - Invalid input
- `500 Internal Server Error` - Server error

---

### 2. Stream Chat (SSE)

**POST** `/v1/chat/{thread_id}/stream`

Send a message and stream the agent's response in real-time via SSE.

**Request:**
```json
{
  "content": "What's the weather in San Francisco?",
  "agent_id": "agent_default",
  "llm_config": {
    "model": "gpt-4",
    "temperature": 0.7,
    "max_tokens": 4096
  }
}
```

**Response:** `text/event-stream` (SSE)

```
data: {"type":"init_stream","run_id":"run_xyz","thread_id":"thread_abc123","timestamp":1699363800000}

data: {"type":"reasoning","content":"I need to check the weather..."}

data: {"type":"tool_call","index":0,"id":"call_1","name":"get_forecast","arguments":"{\"location\":\"San Francisco\"}"}

data: {"type":"tool_result","tool_call_id":"call_1","result":"Sunny, 72°F","is_error":false,"duration_ms":250}

data: {"type":"message","content":"The weather in San Francisco is sunny with a temperature of 72°F."}

data: {"type":"end_stream","status":"success","total_duration_ms":1234}
```

**Status Codes:**
- `200 OK` - Streaming started
- `404 Not Found` - Thread not found
- `500 Internal Server Error` - Execution error (sent via SSE `error` event)

---

### 3. Get Thread History

**GET** `/v1/chat/{thread_id}`

Retrieve full conversation history for a thread.

**Query Parameters:**
- `limit` (optional): Max messages to return (default: 50)
- `before` (optional): Cursor for pagination (message_id)

**Response:**
```json
{
  "thread_id": "thread_abc123",
  "user_id": "user_123",
  "messages": [
    {
      "id": "msg_1",
      "role": "user",
      "type": "message",
      "content": "What's the weather?",
      "created_at": "2025-11-07T10:30:00Z"
    },
    {
      "id": "msg_2",
      "role": "assistant",
      "type": "reasoning",
      "content": "I need to check...",
      "created_at": "2025-11-07T10:30:01Z",
      "duration_ms": 500
    },
    {
      "id": "msg_3",
      "role": "assistant",
      "type": "message",
      "content": "The weather is sunny.",
      "created_at": "2025-11-07T10:30:02Z",
      "duration_ms": 1200
    }
  ],
  "has_more": false
}
```

**Status Codes:**
- `200 OK` - History retrieved
- `404 Not Found` - Thread not found
- `500 Internal Server Error` - Database error

---

### 4. Create Agent Config

**POST** `/v1/agents`

Register a new agent configuration.

**Request:**
```json
{
  "name": "weather-agent",
  "llm_config": {
    "model": "gpt-4",
    "temperature": 0.7,
    "max_tokens": 4096
  },
  "mcp_servers": "http://localhost:8000/mcp,http://localhost:8001/mcp",
  "system_prompt": "You are a helpful weather assistant.",
  "metadata": {
    "description": "Agent for weather queries"
  }
}
```

**Response:**
```json
{
  "agent_id": "agent_abc123",
  "name": "weather-agent",
  "created_at": "2025-11-07T10:30:00Z",
  "llm_config": { ... },
  "mcp_servers": "...",
  "system_prompt": "..."
}
```

**Status Codes:**
- `201 Created` - Agent created
- `400 Bad Request` - Invalid config
- `500 Internal Server Error` - Database error

---

### 5. Get Agent Config

**GET** `/v1/agents/{agent_id}`

Retrieve agent configuration.

**Response:**
```json
{
  "agent_id": "agent_abc123",
  "name": "weather-agent",
  "llm_config": { ... },
  "mcp_servers": "...",
  "system_prompt": "...",
  "created_at": "2025-11-07T10:30:00Z",
  "metadata": { ... }
}
```

**Status Codes:**
- `200 OK` - Agent found
- `404 Not Found` - Agent not found

---

### 6. Health Check

**GET** `/v1/health`

Check API health and dependencies.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 12345,
  "dependencies": {
    "mongodb": "connected",
    "mcp_servers": {
      "http://localhost:8000/mcp": "connected",
      "http://localhost:8001/mcp": "disconnected"
    }
  }
}
```

**Status Codes:**
- `200 OK` - All healthy
- `503 Service Unavailable` - Dependencies unhealthy

---

## Application State

The Axum application maintains shared state via `Arc`:

```rust
pub struct AppState {
    // LLM client (shared across all requests)
    pub llm_client: Arc<dyn LLMClient>,
    
    // MCP tool executor (pre-connected to all servers)
    pub mcp_executor: Arc<MCPToolExecutor>,
    
    // MongoDB client
    pub db_client: mongodb::Client,
    
    // Database name
    pub db_name: String,
    
    // API configuration
    pub config: ApiConfig,
}

pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    pub max_iterations: usize,
    pub cors_origins: Vec<String>,
}
```

---

## SSE Implementation

### Server Side

```rust
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use tokio_stream::wrappers::ReceiverStream;

pub async fn stream_chat(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
    Json(req): Json<StreamChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    // 1. Validate thread exists
    // 2. Create GraphInput
    // 3. Call graph.spawn_run() -> returns event_rx
    let event_rx = graph.spawn_run(input);
    
    // 4. Convert mpsc::Receiver to Stream
    let stream = ReceiverStream::new(event_rx);
    
    // 5. Map StreamEvent to SSE Event
    let sse_stream = stream.map(|event| {
        let json = serde_json::to_string(&event)?;
        Ok(Event::default().data(json))
    });
    
    // 6. Return SSE response with keep-alive
    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}
```

### Client Side (JavaScript)

```javascript
const eventSource = new EventSource('/v1/chat/thread_123/stream', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    content: 'What is the weather?',
    agent_id: 'agent_default'
  })
});

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch (data.type) {
    case 'reasoning':
      console.log('Thinking:', data.content);
      break;
    case 'message':
      console.log('Response:', data.content);
      break;
    case 'tool_call':
      console.log('Calling tool:', data.name);
      break;
    case 'end_stream':
      eventSource.close();
      break;
  }
};

eventSource.onerror = (error) => {
  console.error('SSE error:', error);
  eventSource.close();
};
```

---

## Error Handling

### Error Response Format

All errors return JSON with consistent structure:

```json
{
  "error": {
    "code": "THREAD_NOT_FOUND",
    "message": "Thread 'thread_123' does not exist",
    "details": {
      "thread_id": "thread_123"
    }
  }
}
```

### Error Codes

- `INVALID_INPUT` - Request validation failed
- `THREAD_NOT_FOUND` - Thread ID not found in database
- `AGENT_NOT_FOUND` - Agent ID not found
- `MCP_CONNECTION_FAILED` - Cannot connect to MCP server
- `LLM_ERROR` - LLM API error
- `DATABASE_ERROR` - MongoDB operation failed
- `INTERNAL_ERROR` - Unexpected server error

### Error Type

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "THREAD_NOT_FOUND" | "AGENT_NOT_FOUND" => StatusCode::NOT_FOUND,
            "INVALID_INPUT" => StatusCode::BAD_REQUEST,
            "MCP_CONNECTION_FAILED" | "DATABASE_ERROR" => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        let body = serde_json::json!({ "error": self });
        (status, axum::Json(body)).into_response()
    }
}
```

---

## Middleware

### 1. Logging Middleware

Uses `tracing` for structured logging:

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/v1/chat", post(create_thread))
    .layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = %Uuid::new_v4(),
                )
            })
    );
```

**Log Format:**
```
INFO http_request{method=POST uri=/v1/chat/thread_123/stream request_id=abc-123}: Started processing
INFO http_request{method=POST uri=/v1/chat/thread_123/stream request_id=abc-123}: Graph execution completed duration_ms=1234
```

### 2. CORS Middleware

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([header::CONTENT_TYPE]);

let app = Router::new()
    .route("/v1/chat", post(create_thread))
    .layer(cors);
```

### 3. Timeout Middleware

```rust
use tower_http::timeout::TimeoutLayer;

let app = Router::new()
    .route("/v1/chat", post(create_thread))
    .layer(TimeoutLayer::new(Duration::from_secs(30)));
```

---

## Configuration

### Environment Variables

```bash
# Server
PRAXIS_API_HOST=0.0.0.0
PRAXIS_API_PORT=3000

# LLM
OPENAI_API_KEY=sk-...

# MCP Servers
MCP_SERVERS=http://localhost:8000/mcp,http://localhost:8001/mcp

# Database
MONGODB_URI=mongodb://localhost:27017
MONGODB_DATABASE=praxis

# Execution
MAX_ITERATIONS=10
REQUEST_TIMEOUT_SECS=30

# CORS
CORS_ORIGINS=http://localhost:3000,https://app.example.com

# Logging
RUST_LOG=info,praxis_api=debug
```

### Config File (Optional)

```toml
[server]
host = "0.0.0.0"
port = 3000

[llm]
provider = "openai"
api_key = "${OPENAI_API_KEY}"

[mcp]
servers = [
    "http://localhost:8000/mcp",
    "http://localhost:8001/mcp"
]

[database]
uri = "mongodb://localhost:27017"
database = "praxis"

[execution]
max_iterations = 10
request_timeout_secs = 30

[cors]
origins = ["http://localhost:3000"]
```

---

## Crate Structure

```
praxis-api/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Re-exports
│   ├── main.rs             # Binary entry point
│   ├── server.rs           # Axum server setup
│   ├── state.rs            # AppState definition
│   ├── config.rs           # Configuration loading
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── chat.rs         # Chat endpoints
│   │   ├── agents.rs       # Agent endpoints
│   │   └── health.rs       # Health check
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── create_thread.rs
│   │   ├── stream_chat.rs
│   │   ├── get_history.rs
│   │   └── agent_crud.rs
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── logging.rs
│   │   └── error.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── request.rs      # Request DTOs
│   │   └── response.rs     # Response DTOs
│   └── error.rs            # ApiError type
└── examples/
    └── simple_server.rs    # Standalone example
```

---

## Dependencies

```toml
[dependencies]
# Praxis core
praxis-types = { path = "../praxis-types" }
praxis-graph = { path = "../praxis-graph" }
praxis-llm = { path = "../praxis-llm" }
praxis-mcp = { path = "../praxis-mcp" }
praxis-persist = { path = "../praxis-persist" }

# HTTP framework
axum = { version = "0.7", features = ["macros"] }
tower = "0.5"
tower-http = { version = "0.5", features = ["trace", "cors", "timeout"] }

# Async runtime
tokio = { workspace = true, features = ["full"] }
tokio-stream = "0.1"

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error handling
anyhow = { workspace = true }
thiserror = "2.0"

# MongoDB
mongodb = "3.1"

# Config
config = "0.14"
dotenv = "0.15"

# Utils
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { workspace = true }
```

---

## Implementation Phases

### Phase 1: Basic Server ✓
- [ ] Axum server setup with graceful shutdown
- [ ] AppState with Arc-wrapped dependencies
- [ ] Basic routing structure
- [ ] Health check endpoint
- [ ] Logging middleware
- [ ] CORS middleware

### Phase 2: Chat Endpoints ✓
- [ ] POST /v1/chat - create thread (stub, no DB yet)
- [ ] POST /v1/chat/{id}/stream - SSE streaming
- [ ] Graph integration (spawn_run + event forwarding)
- [ ] Error handling and mapping
- [ ] Request/response models

### Phase 3: Agent Management ✓
- [ ] POST /v1/agents - create agent config (stub)
- [ ] GET /v1/agents/{id} - get agent config
- [ ] Agent config validation
- [ ] MCP server string parsing and validation

### Phase 4: Database Integration ✓
- [ ] Replace thread stubs with MongoDB calls
- [ ] GET /v1/chat/{id} - fetch history from DB
- [ ] Message persistence after stream completion
- [ ] Agent config persistence

### Phase 5: Production Hardening ✓
- [ ] Request timeout enforcement
- [ ] Proper error responses for all cases
- [ ] Structured logging for all operations
- [ ] Health check with dependency status
- [ ] Configuration from env/file
- [ ] Example client (HTML + JS)

---

## Testing Strategy

### Unit Tests
- Request/response model validation
- Error type conversion
- Configuration parsing

### Integration Tests
```rust
#[tokio::test]
async fn test_stream_chat_success() {
    let app = create_test_app().await;
    
    let response = app
        .post("/v1/chat/test_thread/stream")
        .json(&json!({
            "content": "Hello",
            "agent_id": "test_agent"
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    
    // Parse SSE stream and validate events
    let events: Vec<StreamEvent> = parse_sse_stream(response).await;
    assert!(events.iter().any(|e| matches!(e, StreamEvent::InitStream { .. })));
    assert!(events.iter().any(|e| matches!(e, StreamEvent::EndStream { .. })));
}
```

### Load Tests
- Concurrent SSE connections (100+ simultaneous streams)
- Request throughput (requests/sec)
- Memory usage under load
- Graceful degradation

---

## Deployment

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --package praxis-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/praxis-api /usr/local/bin/
EXPOSE 3000
CMD ["praxis-api"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: praxis-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: praxis-api
  template:
    metadata:
      labels:
        app: praxis-api
    spec:
      containers:
      - name: praxis-api
        image: praxis-api:latest
        ports:
        - containerPort: 3000
        env:
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: praxis-secrets
              key: openai-api-key
        - name: MONGODB_URI
          valueFrom:
            secretKeyRef:
              name: praxis-secrets
              key: mongodb-uri
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "512Mi"
            cpu: "1000m"
```

---

## Next Steps

After praxis-api is complete:
1. **praxis-dx** - Developer experience tools (CLI, AgentBuilder, config files)
2. **praxis-persist** - Full MongoDB integration with context management
3. **Authentication** - JWT/OAuth support
4. **Metrics** - Prometheus metrics export
5. **Rate Limiting** - Per-user/API key limits
6. **WebSocket Support** - Bidirectional streaming
7. **Multi-tenancy** - Workspace/organization support

---

## Questions to Address During Implementation

1. Should we buffer SSE events if client is slow? (Backpressure handling)
2. How to handle client disconnect mid-stream? (Cancel graph execution?)
3. Should streaming errors close the SSE connection or send error event?
4. Rate limiting strategy: per IP, per user, per API key?
5. How to version the API? (v1, v2 in path, or header-based?)

---

**Ready to implement Phase 1 when you are!** 🚀

