# Praxis

High-performance React agent framework for building AI agents with LLM integration, tool execution, and persistence.

## Overview

Praxis is a comprehensive framework for building production-ready AI agents that can reason, execute tools, persist conversations, and manage context automatically.

## Quick Start

```rust
use praxis::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let llm_client = Arc::new(OpenAIClient::new(
        std::env::var("OPENAI_API_KEY")?
    )?);
    
    let mcp_executor = Arc::new(MCPToolExecutor::new());
    
    let graph = GraphBuilder::new()
        .with_llm_client(llm_client)
        .with_mcp_executor(mcp_executor)
        .build()?;
    
    let input = GraphInput::new(
        "conv-123",
        vec![Message::Human {
            content: Content::text("Hello!"),
            name: None,
        }],
        LLMConfig::new("gpt-4o"),
    );
    
    let mut events = graph.spawn_run(input, None);
    while let Some(event) = events.recv().await {
        if let StreamEvent::Message { content } = event {
            print!("{}", content);
        }
    }
    
    Ok(())
}
```

## Architecture

Praxis consists of focused crates:

- **`praxis-graph`**: React agent orchestrator
- **`praxis-llm`**: LLM client (OpenAI, Azure)
- **`praxis-mcp`**: Model Context Protocol client
- **`praxis-persist`**: Persistence layer (MongoDB)
- **`praxis-context`**: Context management

## Features

- ✅ Real-time streaming with zero-copy optimizations
- ✅ Incremental persistence with MongoDB
- ✅ Automatic context summarization
- ✅ MCP-based tool execution
- ✅ Strong typing throughout
- ✅ Built on Tokio for high performance

## Documentation

See [docs/](https://github.com/matheussilva/praxis/tree/main/docs) for detailed architecture documentation.

## License

MIT

