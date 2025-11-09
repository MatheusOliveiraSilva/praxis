# Praxis Architecture

**Version**: 2.0  
**Date**: 2025-11-09  
**Status**: Production Ready

---

## Overview

**Praxis** is a high-performance React agent framework for building AI agents with LLM integration, tool execution, and persistence. Built in Rust with Tokio for maximum performance.

### Core Philosophy

> "Praxis" = **action guided by reason**

- **Separation of Concerns**: Each crate has a single responsibility
- **Type Safety**: Strong typing throughout with zero-copy optimizations
- **Scalability**: Async, stateless, horizontally scalable
- **Developer Experience**: Simple API with powerful abstractions

---

## Crate Structure

Praxis is organized into focused crates:

### Main Crate: `praxis`
Aggregates all crates and provides a unified API. Use this for most applications.

```rust
use praxis::prelude::*;
```

### Core Crates

- **`praxis-graph`**: React agent orchestrator
  - Graph execution engine
  - Node abstraction (LLMNode, ToolNode)
  - Router pattern (SimpleRouter)
  - Types: `GraphState`, `GraphInput`, `StreamEvent`, `GraphConfig`
  
- **`praxis-llm`**: Provider-agnostic LLM client
  - OpenAI and Azure support
  - Streaming with zero-copy optimizations
  - Traits: `ChatClient`, `ReasoningClient`
  
- **`praxis-mcp`**: Model Context Protocol client
  - Tool discovery and execution
  - HTTP-based MCP servers
  
- **`praxis-persist`**: Persistence layer
  - Generic `PersistenceClient` trait
  - MongoDB implementation (optional feature)
  - Incremental saving with `EventAccumulator`
  
- **`praxis-context`**: Context management
  - Token counting with tiktoken
  - Automatic summarization
  - Context window strategies

---

## Architecture Diagram

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│  (praxis-api, custom apps)              │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         praxis (main crate)             │
│  Re-exports all sub-crates              │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│      praxis-graph (orchestrator)        │
│  - Graph execution                      │
│  - Node abstraction                     │
│  - Router pattern                       │
│  - StreamEvent types                    │
└─────────────────────────────────────────┘
         ↓              ↓              ↓
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│ praxis-llm  │ │ praxis-mcp  │ │praxis-persist│
│             │ │             │ │             │
│ LLM client  │ │ Tool exec   │ │ MongoDB     │
│ Streaming   │ │ MCP protocol│ │ Persistence │
└─────────────┘ └─────────────┘ └─────────────┘
                           ↓
                  ┌─────────────┐
                  │praxis-context│
                  │             │
                  │ Token count │
                  │ Summarize   │
                  └─────────────┘
```

---

## Key Design Decisions

### 1. Types in `praxis-graph`
All graph-related types (`GraphState`, `GraphInput`, `StreamEvent`, etc.) live in `praxis-graph`, not a separate `praxis-types` crate. This eliminates circular dependencies and keeps types close to where they're used.

### 2. Generic Event Accumulator
`EventAccumulator<E: StreamEventExtractor>` is generic over event type, allowing `praxis-persist` to work with any event type without depending on `praxis-graph`.

### 3. Separation of Persistence and Context
- **`praxis-persist`**: Pure data access layer (CRUD operations)
- **`praxis-context`**: Business logic (token counting, summarization)

### 4. Zero-Copy Streaming
SSE parsing uses `VecDeque<u8>` circular buffer for efficient memory usage.

### 5. Trait-Based Design
- `ChatClient` and `ReasoningClient` traits for LLM abstraction
- `PersistenceClient` trait for database abstraction
- `ContextStrategy` trait for context management

---

## Data Flow

```
1. User sends message
   ↓
2. API creates GraphInput with full message history
   ↓
3. ContextStrategy.get_context_window()
   - Fetches messages after last_summary_update
   - Calculates tokens
   - Generates summary if needed (fire-and-forget)
   - Returns system prompt + messages
   ↓
4. Graph.spawn_run(input, persistence_context)
   - Creates GraphState from input
   - Spawns async execution loop
   - Returns event receiver
   ↓
5. Execution Loop:
   - LLMNode.execute() → streams events
   - EventAccumulator processes events
   - On type transition → save to DB
   - Router decides next node
   - Repeat until END
   ↓
6. Events streamed to client via SSE
```

---

## Quick Start

```rust
use praxis::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create LLM client
    let llm_client = Arc::new(OpenAIClient::new(
        std::env::var("OPENAI_API_KEY")?
    )?);
    
    // Create MCP executor
    let mcp_executor = Arc::new(MCPToolExecutor::new());
    
    // Build graph
    let graph = GraphBuilder::new()
        .with_llm_client(llm_client)
        .with_mcp_executor(mcp_executor)
        .build()?;
    
    // Create input
    let input = GraphInput::new(
        "conv-123",
        vec![Message::Human {
            content: Content::text("Hello!"),
            name: None,
        }],
        LLMConfig::new("gpt-4o"),
    );
    
    // Execute and stream
    let mut events = graph.spawn_run(input, None);
    while let Some(event) = events.recv().await {
        match event {
            StreamEvent::Message { content } => print!("{}", content),
            StreamEvent::Done { .. } => break,
            _ => {}
        }
    }
    
    Ok(())
}
```

---

## Scalability Properties

- **Stateless**: No shared state between requests
- **Async**: Built on Tokio for high concurrency
- **Bounded Channels**: Backpressure-aware event streaming
- **Horizontal Scaling**: Can run multiple instances behind load balancer
- **Resource Limits**: Max iterations, timeouts, cancellation support

---

## See Also

- [Checkpoint Documents](./README.md#architecture-evolution) - Detailed evolution history
- [Main Crate README](../crates/praxis/README.md) - Usage examples
- [API Documentation](https://docs.rs/praxis) - Full API reference

---

**Last updated:** 2025-11-09
