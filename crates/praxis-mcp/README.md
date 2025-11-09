# praxis-mcp

Model Context Protocol (MCP) client for AI agent tool execution.

## Overview

This crate provides a high-level interface for connecting to MCP servers and executing tools within AI agent workflows.

## Features

- Connect to multiple MCP servers simultaneously
- Execute tools with structured arguments
- Full async/await support
- Error handling and timeouts
- Built on `rmcp` (Rust MCP SDK)

## Installation

```toml
[dependencies]
praxis-mcp = "0.1"
```

## Usage

### Basic Tool Execution

```rust
use praxis_mcp::{MCPExecutor, MCPConfig};

// Create executor with multiple servers
let executor = MCPExecutor::new(vec![
    MCPConfig {
        name: "weather".to_string(),
        url: "http://localhost:8005/mcp".to_string(),
    }
]).await?;

// Execute tool
let result = executor.execute_tool(
    "get_weather",
    serde_json::json!({"location": "San Francisco"})
).await?;

println!("Result: {}", result);
```

### Multi-Server Setup

```rust
use praxis_mcp::{MCPExecutor, MCPConfig};

let executor = MCPExecutor::new(vec![
    MCPConfig {
        name: "weather".to_string(),
        url: "http://localhost:8005/mcp".to_string(),
    },
    MCPConfig {
        name: "search".to_string(),
        url: "http://localhost:8006/mcp".to_string(),
    },
]).await?;

// Executor automatically routes to the correct server
let weather = executor.execute_tool("get_weather", args).await?;
let search = executor.execute_tool("web_search", args).await?;
```

### List Available Tools

```rust
let tools = executor.list_tools().await?;

for tool in tools {
    println!("Tool: {} - {}", tool.name, tool.description);
}
```

## MCP Protocol

The Model Context Protocol (MCP) is a standard for connecting AI agents to external tools and data sources. This crate implements the client side of the protocol.

Key concepts:
- **Server**: Hosts one or more tools
- **Tool**: A function that can be called with structured arguments
- **Resource**: Data that can be read (future support)

## Examples

See the `examples/` directory for complete examples:

```bash
# Run the simple MCP agent example
cargo run --example simple_mcp_agent
```

## Part of Praxis Framework

This crate is part of the [Praxis AI Agent Framework](https://github.com/matheussilva/praxis):

- [praxis-graph](https://crates.io/crates/praxis-graph) - React agent orchestrator
- [praxis-llm](https://crates.io/crates/praxis-llm) - LLM client (OpenAI, Azure)
- [praxis-types](https://crates.io/crates/praxis-types) - Core types and event model
- [praxis-persist](https://crates.io/crates/praxis-persist) - MongoDB persistence

## License

MIT

