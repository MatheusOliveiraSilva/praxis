# Praxis Observability

Observability and tracing abstraction layer for Praxis AI agent framework.

## Overview

`praxis-observability` provides a trait-based abstraction for observing and tracing AI agent executions. It enables you to track LLM calls, tool executions, and overall agent behavior across different observability backends.

## Features

- **Backend Agnostic**: Trait-based design allows multiple observability providers
- **Langfuse Integration**: Built-in support for Langfuse tracing
- **Async Fire-and-Forget**: Non-blocking tracing that doesn't impact agent performance
- **Structured Data**: Rich context captured for each node execution
- **Extensible**: Easy to add custom observers for other platforms

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
praxis-observability = "0.1"
```

## Quick Start

### Using Langfuse Observer

```rust
use praxis_observability::LangfuseObserver;

// Initialize observer
let observer = LangfuseObserver::new(
    "your-public-key",
    "your-secret-key",
    "https://cloud.langfuse.com",
)?;

// Use with Praxis graph
let graph = Graph::builder()
    .llm_client(llm_client)
    .mcp_executor(mcp_executor)
    .with_observer(Arc::new(observer))
    .build()?;
```

### Implementing Custom Observer

```rust
use async_trait::async_trait;
use praxis_observability::{Observer, NodeObservation};

struct MyCustomObserver {
    // Your fields
}

#[async_trait]
impl Observer for MyCustomObserver {
    async fn trace_start(&self, run_id: String, conversation_id: String) -> anyhow::Result<()> {
        // Initialize trace
        Ok(())
    }
    
    async fn trace_llm_node(&self, observation: NodeObservation) -> anyhow::Result<()> {
        // Trace LLM execution
        Ok(())
    }
    
    async fn trace_tool_node(&self, observation: NodeObservation) -> anyhow::Result<()> {
        // Trace tool execution
        Ok(())
    }
    
    async fn trace_end(&self, run_id: String, status: String, total_duration_ms: u64) -> anyhow::Result<()> {
        // Finalize trace
        Ok(())
    }
}
```

## Configuration

### Environment Variables (Langfuse)

```bash
LANGFUSE_PUBLIC_KEY=pk-xxx
LANGFUSE_SECRET_KEY=sk-xxx
LANGFUSE_HOST=https://cloud.langfuse.com
```

### Configuration File (TOML)

```toml
[observability]
enabled = true
provider = "langfuse"

[observability.langfuse]
public_key = "${LANGFUSE_PUBLIC_KEY}"
secret_key = "${LANGFUSE_SECRET_KEY}"
host = "https://cloud.langfuse.com"
```

## Trace Structure

When using Langfuse, traces are organized as:

```
Trace (per graph execution run)
├── Span: LLM Node #1
│   ├── Input: Messages sent to LLM
│   └── Output: AI response (text or tool calls)
├── Span: Tool Node #1
│   ├── Input: Tool calls
│   └── Output: Tool results
├── Span: LLM Node #2
│   └── ...
└── Status: Success/Error
```

## Architecture

### Observer Pattern

The `Observer` trait defines the contract for tracing:

- **trace_start**: Initialize a new trace for a graph execution
- **trace_llm_node**: Record LLM node execution with input/output
- **trace_tool_node**: Record tool node execution with tool calls/results
- **trace_end**: Finalize trace with status and duration

### Fire-and-Forget Design

All tracing operations are executed asynchronously in background tasks:

```rust
if let Some(obs) = &observer {
    let obs_clone = Arc::clone(obs);
    tokio::spawn(async move {
        let _ = obs_clone.trace_llm_node(observation).await;
    });
}
```

This ensures tracing never blocks the main execution flow.

### Node Exit Triggers

Observability is triggered **immediately after each node exits** in the graph execution loop:

1. **Node Execution**: `node.execute()` completes
2. **Message Extraction**: New messages added by the node are extracted
3. **Persistence** (async): Messages are saved to the database (fire-and-forget)
4. **Observability** (async): Observation is sent to the observer (fire-and-forget)
5. **Graph Continues**: Next node is determined and executed

This design ensures:
- Complete messages are traced (no partial streaming chunks)
- Tracing happens in real-time as nodes complete
- No blocking on I/O operations

### Langfuse Batch Ingestion

The Langfuse implementation uses the [batch ingestion API](https://api.reference.langfuse.com) for optimal performance:

```rust
// Each event is wrapped in a batch
{
  "batch": [{
    "id": "event-uuid",
    "timestamp": "2025-11-10T21:00:00Z",
    "type": "generation-create",
    "body": {
      "id": "span-uuid",
      "traceId": "trace-uuid",
      "input": [{"role": "user", "content": "..."}],
      "output": {"role": "assistant", "content": "..."},
      // ... other fields
    }
  }]
}
```

**Event Types**:
- `trace-create`: Creates/updates a trace
- `generation-create`: Records an LLM generation (for LLM nodes)
- `span-create`: Records a span (for Tool nodes)

**Benefits**:
- Single endpoint for all event types
- Atomic operations per node
- Easy to batch multiple events in the future

## Data Captured

### LLM Node Observation

**Input Format** (complete message history):
```json
[
  {"role": "user", "content": "What is the weather in SF?"},
  {"role": "assistant", "content": "Let me check that for you."},
  {"role": "user", "content": "Thanks!"}
]
```

**Output Format** (new AI message):
```json
{
  "role": "assistant",
  "content": "The weather in SF is 72°F and sunny."
}
```

**Additional Fields**:
- **Duration**: Time taken for LLM call (ms)
- **Model**: LLM model identifier (e.g., `gpt-4o-mini`)
- **Token Usage**: Prompt, completion, and total tokens
- **Metadata**: Run ID, conversation ID, timestamps

### Tool Node Observation

**Input Format** (tool calls):
```json
{
  "tool_calls": [
    {
      "id": "call_abc123",
      "name": "get_weather",
      "arguments": "{\"location\": \"San Francisco\"}"
    }
  ]
}
```

**Output Format** (tool results):
```json
{
  "tool_results": [
    {
      "tool_call_id": "call_abc123",
      "name": "get_weather",
      "content": "{\"temperature\": 72, \"condition\": \"sunny\"}",
      "status": "success"
    }
  ]
}
```

**Additional Fields**:
- **Duration**: Time taken for tool execution (ms)
- **Status**: Success or error for each tool call
- **Metadata**: Run ID, conversation ID, timestamps

## Best Practices

1. **Always use Arc**: Wrap observers in `Arc` for sharing across async tasks
2. **Handle errors gracefully**: Observer failures should log but not crash the agent
3. **Avoid blocking**: Never block in observer implementations
4. **Batch when possible**: For high-volume scenarios, implement batching
5. **Add metadata**: Include custom tags/metadata for better filtering

## Examples

See the `examples/` directory for:

- `simple_trace.rs`: Basic observer usage
- `custom_observer.rs`: Implementing a custom observer
- `metadata.rs`: Adding custom metadata to traces

## License

MIT

