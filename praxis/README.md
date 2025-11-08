# Praxis - AI Agent Framework for Rust 🦀

**High-performance framework for building AI agents in Rust with streaming, MCP tools, and smart persistence.**

[![Crates.io](https://img.shields.io/crates/v/praxis.svg)](https://crates.io/crates/praxis)
[![Documentation](https://docs.rs/praxis/badge.svg)](https://docs.rs/praxis)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

- 🚀 **Real-time streaming**: Token-by-token responses via Server-Sent Events (SSE)
- 🛠️ **MCP integration**: Model Context Protocol for external tools (filesystem, web search, APIs)
- 💾 **Smart persistence**: MongoDB with automatic conversation summarization
- ⚡ **Async/await**: Built on Tokio for high concurrency
- 🔧 **Type-safe**: Compile-time guarantees and excellent error messages
- 📊 **Scalable**: Stateless design for horizontal scaling

## 🚀 Quick Start

```bash
cargo add praxis
```

### Basic Example

```rust
use praxis::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create an agent with just MongoDB and OpenAI
    let agent = AgentBuilder::new()
        .mongodb("mongodb://localhost:27017", "praxis")
        .openai_key(std::env::var("OPENAI_API_KEY")?)
        .build()
        .await?;
    
    // Chat!
    let response = agent.chat("What is 2+2?").await?;
    println!("{}", response);
    
    Ok(())
}
```

### With MCP Tools

```rust
use praxis::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = AgentBuilder::new()
        .mongodb("mongodb://localhost:27017", "praxis")
        .openai_key(std::env::var("OPENAI_API_KEY")?)
        // Add external MCP tool servers
        .mcp_servers("http://localhost:8000/mcp")
        .build()
        .await?;
    
    // Agent can now use tools from MCP servers
    let response = agent.chat("List files in /tmp").await?;
    println!("{}", response);
    
    Ok(())
}
```

### Streaming Responses

```rust
use praxis::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = AgentBuilder::new()
        .mongodb("mongodb://localhost:27017", "praxis")
        .openai_key(std::env::var("OPENAI_API_KEY")?)
        .build()
        .await?;
    
    let mut stream = agent.chat_stream("Explain Rust ownership").await?;
    
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::Message { content } => print!("{}", content),
            StreamEvent::ToolCall { tool_name, .. } => {
                println!("\n[Using tool: {}]", tool_name);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

## 📦 What You Get

When you install `praxis`, you get access to all these sub-crates:

- **praxis-types**: Core types (`StreamEvent`, `GraphState`, configs)
- **praxis-graph**: Execution runtime (`Node`, `Graph`, `Router`)
- **praxis-llm**: LLM clients (OpenAI with streaming)
- **praxis-mcp**: MCP protocol integration
- **praxis-persist**: MongoDB persistence with context management

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│  Your Application                       │
│  (uses praxis::AgentBuilder)            │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│  Praxis Framework                       │
│  ┌───────────────────────────────────┐  │
│  │  Graph (Orchestration)            │  │
│  │  - LLM Node (reasoning/response)  │  │
│  │  - Tool Node (MCP execution)      │  │
│  │  - Router (flow control)          │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
           ↓              ↓
    ┌──────────┐    ┌──────────┐
    │ OpenAI   │    │ MCP      │
    │ API      │    │ Servers  │
    └──────────┘    └──────────┘
```

## 🎯 Use Cases

- **Chatbots**: Conversational AI with tool access
- **AI Assistants**: Code assistants, data analysts, customer support
- **Automation**: Task automation with LLM reasoning
- **Research**: Experiments with AI agent architectures

## 📚 Examples

See the [examples directory](https://github.com/your-org/praxis/tree/main/examples) for:
- Simple chatbot
- RAG (Retrieval Augmented Generation)
- Code assistant with GitHub integration
- Complete REST API with SSE streaming

## 🔧 Building an API Server

For a production-ready REST API with SSE streaming, check out the `praxis-api` example:

```bash
# Clone the repository
git clone https://github.com/your-org/praxis
cd praxis/praxis-api

# Run the API server
cargo run
```

Endpoints:
- `POST /v1/chat` - Create conversation thread
- `POST /v1/chat/{id}/stream` - Stream agent responses (SSE)
- `GET /v1/health` - Health check

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - HTTP framework
- [MongoDB Rust Driver](https://github.com/mongodb/mongo-rust-driver)
- [Model Context Protocol](https://modelcontextprotocol.io/)

---

**Made with ❤️ by the Praxis team**
