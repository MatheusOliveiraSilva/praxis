//! # Praxis - AI Agent Framework for Rust
//!
//! Praxis is a high-performance framework for building AI agents in Rust with:
//! - 🚀 **Real-time streaming** (token-by-token responses via SSE)
//! - 🛠️ **MCP integration** (Model Context Protocol for external tools)
//! - 💾 **Smart persistence** (MongoDB with auto-summarization)
//! - ⚡ **Async/await** (built on Tokio for scalability)
//! - 🔧 **Type-safe** (compile-time guarantees)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use praxis::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create an agent with MongoDB and OpenAI
//!     let agent = AgentBuilder::new()
//!         .mongodb("mongodb://localhost:27017", "praxis")
//!         .openai_key("sk-...")
//!         .mcp_servers("http://localhost:8000/mcp")
//!         .build()
//!         .await?;
//!     
//!     // Run a conversation
//!     let response = agent.chat("What's the weather in SF?").await?;
//!     println!("{}", response);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! Praxis consists of several composable crates:
//!
//! - **praxis-types**: Core types (StreamEvent, GraphState, configs)
//! - **praxis-graph**: Execution runtime (Node, Graph, Router)
//! - **praxis-llm**: LLM clients (OpenAI, with streaming support)
//! - **praxis-mcp**: MCP protocol integration for external tools
//! - **praxis-persist**: MongoDB persistence with context management
//!
//! ## Examples
//!
//! ### Simple Chat
//!
//! ```rust,no_run
//! use praxis::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let agent = AgentBuilder::new()
//!         .mongodb("mongodb://localhost:27017", "praxis")
//!         .openai_key(std::env::var("OPENAI_API_KEY")?)
//!         .build()
//!         .await?;
//!     
//!     let response = agent.chat("Hello!").await?;
//!     println!("{}", response);
//!     Ok(())
//! }
//! ```
//!
//! ### Streaming Response
//!
//! ```rust,no_run
//! use praxis::prelude::*;
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let agent = AgentBuilder::new()
//!         .mongodb("mongodb://localhost:27017", "praxis")
//!         .openai_key(std::env::var("OPENAI_API_KEY")?)
//!         .build()
//!         .await?;
//!     
//!     let mut stream = agent.chat_stream("Explain Rust ownership").await?;
//!     
//!     while let Some(event) = stream.next().await {
//!         match event? {
//!             StreamEvent::Message { content } => print!("{}", content),
//!             StreamEvent::ToolCall { tool_name, .. } => {
//!                 println!("\n[Using tool: {}]", tool_name);
//!             }
//!             _ => {}
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### With MCP Tools
//!
//! ```rust,no_run
//! use praxis::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let agent = AgentBuilder::new()
//!         .mongodb("mongodb://localhost:27017", "praxis")
//!         .openai_key(std::env::var("OPENAI_API_KEY")?)
//!         // Add MCP servers (comma-separated)
//!         .mcp_servers("http://localhost:8000/mcp,http://localhost:8001/mcp")
//!         .build()
//!         .await?;
//!     
//!     // Agent can now use tools from MCP servers
//!     let response = agent.chat("List files in /tmp").await?;
//!     println!("{}", response);
//!     Ok(())
//! }
//! ```
//!
//! ## Building an API Server
//!
//! For a complete REST API with SSE streaming, see the `praxis-api` example in the repository.
//!
//! ## Features
//!
//! - `full` (default): All features enabled
//! - `api-server`: Include HTTP server components (future)

// Re-export all public APIs
pub use praxis_types as types;
pub use praxis_graph as graph;
pub use praxis_llm as llm;
pub use praxis_mcp as mcp;
pub use praxis_persist as persist;

// Re-export commonly used types
pub use praxis_types::{StreamEvent, GraphState, GraphConfig, LLMConfig};
pub use praxis_graph::{Graph, Node};
pub use praxis_llm::{LLMClient, OpenAIClient, Message, Content};
pub use praxis_mcp::{MCPClient, MCPToolExecutor};
pub use praxis_persist::PersistClient;

/// High-level builder for creating AI agents
pub mod builder;

/// Convenient prelude with commonly used types
pub mod prelude {
    pub use crate::builder::AgentBuilder;
    pub use crate::types::{StreamEvent, GraphConfig, LLMConfig};
    pub use crate::llm::{Message, Content};
    pub use anyhow::Result;
}
