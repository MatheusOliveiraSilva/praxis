# praxis-types

Core types and event model for the Praxis AI agent framework.

## Overview

This crate provides the foundational types used across all Praxis crates:

- **Event Model**: `StreamEvent` enum for real-time streaming
- **Configuration**: Shared configuration types
- **Message Types**: Structured message representations
- **Common Traits**: Shared traits for extensibility

## Features

- Zero-copy event streaming with `StreamEvent`
- Serde serialization support
- Chrono-based timestamps
- UUID-based identifiers

## Installation

```toml
[dependencies]
praxis-types = "0.1"
```

## Usage

### StreamEvent

The core event type for streaming AI agent responses:

```rust
use praxis_types::StreamEvent;

match event {
    StreamEvent::Message { content, .. } => {
        println!("Message: {}", content);
    }
    StreamEvent::Reasoning { content, .. } => {
        println!("Reasoning: {}", content);
    }
    StreamEvent::ToolCall { name, arguments, .. } => {
        println!("Calling tool: {} with {}", name, arguments);
    }
    StreamEvent::ToolResult { result, .. } => {
        println!("Tool result: {}", result);
    }
    StreamEvent::Done { .. } => {
        println!("Stream complete");
    }
}
```

### Configuration

```rust
use praxis_types::Config;

let config = Config {
    model: "gpt-4o".to_string(),
    temperature: 0.7,
    max_tokens: Some(1000),
};
```

## Event Model

Praxis uses a canonical event model for streaming:

- **Message**: Assistant's response content (token-by-token)
- **Reasoning**: Internal reasoning/thinking process
- **ToolCall**: Function/tool invocation request
- **ToolResult**: Result from tool execution
- **Done**: Stream completion marker

This unified event model works across all components (LLM, Graph, API).

## Part of Praxis Framework

This crate is part of the [Praxis AI Agent Framework](https://github.com/matheussilva/praxis):

- [praxis-graph](https://crates.io/crates/praxis-graph) - React agent orchestrator
- [praxis-llm](https://crates.io/crates/praxis-llm) - LLM client (OpenAI, Azure)
- [praxis-mcp](https://crates.io/crates/praxis-mcp) - Model Context Protocol client
- [praxis-persist](https://crates.io/crates/praxis-persist) - MongoDB persistence

## License

MIT

