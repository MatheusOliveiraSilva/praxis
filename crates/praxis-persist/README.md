# praxis-persist

MongoDB persistence layer for AI agent conversations with intelligent context management.

## Overview

This crate provides a complete persistence solution for AI agent conversations:

- **Thread Management**: Create, list, and delete conversation threads
- **Message Storage**: Store user and assistant messages
- **Context Management**: Token-aware context window management
- **Auto-Summarization**: Automatic summarization when context exceeds limits

## Features

- Full async MongoDB integration
- Token-aware context management
- Automatic context summarization
- Thread-based conversation organization
- Built-in error handling

## Installation

```toml
[dependencies]
praxis-persist = "0.1"
```

## Usage

### Basic Setup

```rust
use praxis_persist::PersistClient;

// Connect to MongoDB
let client = PersistClient::connect("mongodb://localhost:27017")
    .await?;
```

### Thread Management

```rust
// Create new thread
let thread = client.create_thread(
    "user123",
    Some("Weather Discussion")
).await?;

println!("Thread ID: {}", thread.thread_id);

// List user's threads
let threads = client.list_threads("user123", None, None).await?;

// Delete thread
client.delete_thread(&thread.thread_id, "user123").await?;
```

### Message Storage

```rust
use praxis_persist::Message;

// Store user message
let message = client.store_message(
    &thread_id,
    "user123",
    "user",
    "What's the weather in SF?"
).await?;

// Store assistant message
let response = client.store_message(
    &thread_id,
    "user123",
    "assistant",
    "The weather in San Francisco is sunny, 72°F."
).await?;

// List thread messages
let messages = client.list_messages(&thread_id, "user123").await?;

for msg in messages {
    println!("{}: {}", msg.role, msg.content);
}
```

### Context Management

```rust
use praxis_persist::ContextManager;

// Get context-aware message history
let context = client.get_context(
    &thread_id,
    "user123",
    4000  // max tokens
).await?;

// Returns messages that fit within token limit
// Automatically summarizes old messages if needed
```

### Message Accumulator

Convert streaming events into persisted messages:

```rust
use praxis_persist::MessageAccumulator;
use praxis_types::StreamEvent;

let mut accumulator = MessageAccumulator::new();

// Process streaming events
while let Some(event) = stream.next().await {
    if let Some(message) = accumulator.process(event?) {
        // Complete message ready to persist
        client.store_message(
            &thread_id,
            &user_id,
            &message.role,
            &message.content
        ).await?;
    }
}
```

## Context Management Strategy

The context manager uses a token-aware approach:

1. **Calculate total tokens** in conversation history
2. **If under limit**: Return all messages
3. **If over limit**: 
   - Summarize old messages
   - Keep recent messages intact
   - Inject summary as system message

This ensures:
- No context window overflow
- Important recent context preserved
- Cost-effective token usage

## Part of Praxis Framework

This crate is part of the [Praxis AI Agent Framework](https://github.com/matheussilva/praxis):

- [praxis-graph](https://crates.io/crates/praxis-graph) - React agent orchestrator
- [praxis-llm](https://crates.io/crates/praxis-llm) - LLM client (OpenAI, Azure)
- [praxis-types](https://crates.io/crates/praxis-types) - Core types and event model
- [praxis-mcp](https://crates.io/crates/praxis-mcp) - MCP client

## License

MIT

