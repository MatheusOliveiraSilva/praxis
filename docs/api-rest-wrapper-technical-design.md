# API REST Wrapper - Technical Design

## Overview

The `praxis-api` crate provides a production-ready HTTP REST API with Server-Sent Events (SSE) streaming for real-time AI agent interactions.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                       HTTP Layer (Axum)                      │
├─────────────────────────────────────────────────────────────┤
│  Middleware Stack:                                           │
│  - Request Logging                                           │
│  - CORS                                                      │
│  - Compression (gzip)                                        │
│  - Timeout (300s for streaming)                             │
│  - Tracing                                                   │
├─────────────────────────────────────────────────────────────┤
│  Routes:                                                     │
│  - /health (health check)                                   │
│  - /threads (CRUD)                                          │
│  - /threads/:id/messages (list + streaming POST)           │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                       AppState (Arc)                         │
│  - Config (TOML + ENV)                                      │
│  - PersistClient (MongoDB)                                  │
│  - LLMClient (OpenAI)                                       │
│  - MCPToolExecutor (MCP tools)                              │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                  Business Logic Layer                        │
│  - Thread management                                         │
│  - Message persistence                                       │
│  - Graph execution                                           │
│  - SSE event conversion                                      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌───────────────────┬───────────────────┬─────────────────────┐
│   MongoDB         │   OpenAI API      │   MCP Servers       │
│   (Persistence)   │   (LLM)           │   (Tools)           │
└───────────────────┴───────────────────┴─────────────────────┘
```

## Configuration System

### Hierarchical Loading

1. `config/default.toml` - Base configuration
2. `config/{ENV}.toml` - Environment-specific overrides
3. Environment variables - Runtime overrides

**Priority**: ENV > TOML overrides > default.toml

### Configuration Structure

```rust
pub struct Config {
    pub server: ServerConfig,      // Host, port, workers
    pub cors: CorsConfig,          // CORS settings
    pub mongodb: MongoDbConfig,    // MongoDB connection
    pub llm: LlmConfig,            // LLM settings
    pub mcp: McpConfig,            // MCP servers
    pub logging: LoggingConfig,    // Logging format/level
    
    // Secrets (ENV only)
    pub mongodb_uri: String,
    pub openai_api_key: String,
}
```

### Environment Variable Mapping

- `SERVER_*` → `server.*`
- `MONGODB_*` → `mongodb.*`
- `LLM_*` → `llm.*`
- `MCP_*` → `mcp.*`
- `LOG_*` → `logging.*`

## Request Flow

### 1. Thread Creation

```
POST /threads
    ↓
ThreadHandler::create_thread
    ↓
PersistClient::threads().create_thread()
    ↓
MongoDB (threads collection)
    ↓
201 Created + ThreadResponse
```

### 2. Streaming Message (SSE)

```
POST /threads/:id/messages
    ↓
StreamHandler::send_message_stream
    ↓
1. Validate thread exists
2. Save user message to MongoDB
3. Get context window (messages + summary)
4. Create GraphInput with LLM messages
5. Spawn Graph execution (background task)
6. Convert Receiver<StreamEvent> to SSE stream
7. Map Graph events to SSE events
8. Save assistant responses asynchronously
    ↓
SSE Stream (text/event-stream)
    event: message
    event: tool_call
    event: tool_result
    event: done
```

### 3. Message Persistence Flow

```
User Message
    ↓
Save to MongoDB (sync)
    ↓
Graph Execution (spawn)
    ↓
Stream Events
    ↓
Assistant Message
    ↓
Save to MongoDB (async, fire-and-forget)
```

## Server-Sent Events (SSE)

### Event Types

| Event Type | Purpose | Data Schema |
|------------|---------|-------------|
| `message` | AI response chunk | `{"content": string}` |
| `reasoning` | Internal reasoning | `{"content": string}` |
| `tool_call` | Tool being called | `{"name": string, "arguments": string}` |
| `tool_result` | Tool execution result | `{"result": string}` |
| `done` | Stream completed | `{"status": "completed"}` |
| `error` | Error occurred | `{"error": string}` |
| `info` | Informational event | `{}` |

### Implementation

```rust
// 1. Graph returns Receiver<StreamEvent>
let event_receiver = graph.spawn_run(graph_input);

// 2. Convert to Stream using ReceiverStream
let event_stream = ReceiverStream::new(event_receiver);

// 3. Map to SSE events
let sse_stream = event_stream.map(move |event| {
    let sse_event = match event {
        GraphStreamEvent::Message { content, .. } => {
            Event::default()
                .event("message")
                .json_data(json!({"content": content}))
        },
        // ... other event types
    };
    Ok::<Event, Infallible>(sse_event.unwrap())
});

// 4. Return as Axum SSE
Ok(Sse::new(sse_stream))
```

## Error Handling

### ApiError Type

```rust
pub enum ApiError {
    ThreadNotFound(String),
    MessageNotFound(String),
    BadRequest(String),
    Database(mongodb::error::Error),
    Persist(praxis_persist::PersistError),
    Graph(anyhow::Error),
    Config(String),
    Internal,
}
```

### IntoResponse Implementation

All errors are automatically converted to HTTP responses:

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::ThreadNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            // ... log internal errors, return generic message
        };
        
        (status, Json(json!({"error": message}))).into_response()
    }
}
```

### Error Response Format

```json
{
  "error": "Thread not found: 507f1f77bcf86cd799439011"
}
```

## Middleware Stack

### Tower Layers (Outer to Inner)

1. **TraceLayer** - Request/response tracing
2. **CompressionLayer** - gzip compression
3. **CorsLayer** - CORS headers
4. **TimeoutLayer** - 300s timeout for streaming
5. **LoggingMiddleware** - Custom request logging

### Logging Middleware

```rust
pub async fn log_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();
    
    let response = next.run(req).await;
    
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration_ms = %start.elapsed().as_millis(),
        "Request processed"
    );
    
    response
}
```

## AppState Management

### Shared State Pattern

```rust
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub persist: Arc<PersistClient>,
    pub llm_client: Arc<dyn LLMClient>,
    pub mcp_executor: Arc<MCPToolExecutor>,
    pub graph: Arc<Graph>,  // ✨ Created once at startup
}
```

### Initialization (main.rs)

```rust
// 1. Create MCP executor
let mcp_executor = MCPToolExecutor::new();
// ... connect to MCP servers ...
let mcp_executor = Arc::new(mcp_executor);

// 2. Create Graph once (stateless, shared across all requests)
let graph = Graph::new(
    llm_client.clone(),
    Arc::clone(&mcp_executor),
    GraphConfig::default(),
);

// 3. Create AppState with shared Graph
let state = Arc::new(AppState::new(
    config,
    persist_client,
    llm_client,
    mcp_executor,
    graph,  // Passed in, will be Arc-wrapped
));
```

### Benefits

- **Thread-safe**: All fields wrapped in `Arc`
- **Cheap cloning**: Only increments reference count
- **Shared resources**: Single connection pool, single MCP executor, single Graph
- **Zero per-request allocation**: Graph reused across all requests
- **Type-safe**: Compile-time verification

### Performance Impact

**Before optimization**:
- Per request: ~72 bytes allocated (Graph struct + GraphConfig)
- 2x Arc::clone operations

**After optimization**:
- Per request: 0 bytes allocated
- 0 Arc::clone operations
- Graph reused via `state.graph.spawn_run(input)`

**Why it's safe**: The Graph is stateless. All conversation state lives in `GraphInput`, not the Graph itself.

## Async Patterns

### Fire-and-Forget Message Saving

```rust
GraphStreamEvent::Message { content, .. } => {
    // Clone what we need
    let persist = Arc::clone(&persist_client);
    let user_id = user_id_clone.clone();
    let content_clone = content.clone();
    
    // Spawn background task
    tokio::spawn(async move {
        let assistant_message = DBMessage { /* ... */ };
        
        if let Err(e) = persist.messages().save_message(assistant_message).await {
            tracing::error!("Failed to save assistant message: {}", e);
        }
    });
    
    // Continue streaming immediately
    Event::default().event("message").json_data(/* ... */)
}
```

**Why?**
- Don't block streaming on database writes
- User sees response immediately
- Errors are logged but don't interrupt stream

### Graph Execution in Background

```rust
// Graph::spawn_run already spawns a background task
pub fn spawn_run(&self, input: GraphInput) -> mpsc::Receiver<StreamEvent> {
    let (tx, rx) = mpsc::channel(1000);
    
    tokio::spawn(async move {
        if let Err(e) = Self::execute_loop(/* ... */).await {
            let _ = tx.send(StreamEvent::Error { /* ... */ }).await;
        }
    });
    
    rx  // Return receiver immediately
}
```

## Performance Considerations

### Connection Pooling

- MongoDB connection pool (configurable size)
- HTTP clients reused (OpenAI, MCP)
- All wrapped in `Arc` for sharing

### Streaming Efficiency

- Channel buffer: 1000 events
- gzip compression for large payloads
- Timeout: 300s for long-running agents

### Memory Management

- `Arc` prevents duplication
- Async tasks clean up after completion
- SSE streams closed on client disconnect

## Security Considerations

### Current Implementation (Development)

- ⚠️ **No authentication**: `user_id` in request body
- ⚠️ **No authorization**: Public API
- ⚠️ **No rate limiting**: Trust reverse proxy

### Production Recommendations

1. **Authentication**
   - JWT tokens or API keys
   - OAuth 2.0 for user auth

2. **Authorization**
   - Verify `user_id` ownership
   - Role-based access control

3. **Rate Limiting**
   - Per-user rate limits
   - Per-IP rate limits
   - Implement at reverse proxy level

4. **Input Validation**
   - Already implemented via `serde`
   - Add custom validators for business logic

5. **HTTPS**
   - Terminate TLS at reverse proxy (nginx, Caddy)
   - Never run directly on internet

## Monitoring & Observability

### Structured Logging

```rust
tracing::info!(
    method = %method,
    uri = %uri,
    status = %status,
    duration_ms = %duration.as_millis(),
    "Request processed"
);
```

**Formats**:
- `pretty`: Development (human-readable)
- `json`: Production (machine-parseable)

### Health Check

```
GET /health

{
  "status": "healthy",
  "version": "0.1.0",
  "services": {
    "mongodb": "connected",
    "mcp": "available"
  }
}
```

### Metrics (Future)

- Request count/duration (histogram)
- SSE connection count (gauge)
- Error rate (counter)
- Database query latency (histogram)

## Testing Strategy

### Unit Tests

- Configuration loading
- Error response formatting
- Route handlers (with mock state)

### Integration Tests

- Full HTTP tests with test server
- MongoDB integration tests
- SSE stream validation

### Manual Testing

Provided curl examples in README:

```bash
# Health check
curl http://localhost:8000/health

# Create thread
curl -X POST http://localhost:8000/threads \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user_123", "metadata": {}}'

# Stream messages
curl -N -X POST http://localhost:8000/threads/{id}/messages \
  -H "Accept: text/event-stream" \
  -d '{"user_id": "user_123", "content": "Hello!"}'
```

## Deployment

### Production Checklist

- [ ] Set `ENV=prod`
- [ ] Configure `config/prod.toml`
- [ ] Set environment variables
- [ ] Use reverse proxy (nginx/Caddy)
- [ ] Configure MongoDB replica set
- [ ] Set up monitoring
- [ ] Configure log aggregation
- [ ] Set up rate limiting
- [ ] Enable HTTPS
- [ ] Configure firewall rules

### Environment Variables (Production)

```bash
ENV=prod
MONGODB_URI=mongodb://...
OPENAI_API_KEY=sk-...
SERVER_PORT=8000
SERVER_HOST=127.0.0.1  # Bind to localhost, proxy handles public
LOG_LEVEL=info
LOG_FORMAT=json
MCP_SERVERS=http://mcp-server:8000/mcp
```

## Future Enhancements

### Phase 1 (Current)
- ✅ Basic CRUD
- ✅ SSE streaming
- ✅ Thread management
- ✅ Message persistence

### Phase 2 (Next)
- [ ] Authentication/Authorization
- [ ] Rate limiting
- [ ] Metrics/telemetry
- [ ] WebSocket support (alternative to SSE)

### Phase 3 (Future)
- [ ] Multi-model support (Anthropic, Gemini)
- [ ] Batch operations
- [ ] Webhook support
- [ ] GraphQL API
- [ ] Admin dashboard

## Technical Decisions

### Why Axum?

- Native `async/await` support
- Excellent type safety
- Tower middleware ecosystem
- SSE support out of the box

### Why SSE over WebSocket?

- Simpler protocol (HTTP)
- Better proxy compatibility
- Automatic reconnection in browsers
- Sufficient for one-way streaming

### Why TOML + ENV?

- Human-readable config files
- Easy to version control
- Environment-specific overrides
- Standard in Rust ecosystem

### Why Arc Everywhere?

- Thread-safe reference counting
- Cheap cloning for async tasks
- Shared resource pools
- Prevents data duplication
- Enables stateless service reuse (Graph, LLMClient, MCPToolExecutor)

## Conclusion

The `praxis-api` crate provides a solid foundation for production AI agent APIs with:

- **Real-time streaming** via SSE
- **Flexible configuration** (TOML + ENV)
- **Production-ready** error handling and logging
- **Scalable architecture** with async patterns
- **Type-safe** Rust implementation

Ready for deployment with appropriate security measures and monitoring.
