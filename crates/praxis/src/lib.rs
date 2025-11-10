//! # Praxis
//!
//! High-performance React agent framework for building AI agents with LLM integration,
//! tool execution, and persistence.
//!
//! ## Overview
//!
//! Praxis is a comprehensive framework for building production-ready AI agents that can:
//!
//! - **Reason and respond** using LLMs (OpenAI, Azure)
//! - **Execute tools** via MCP (Model Context Protocol)
//! - **Persist conversations** with MongoDB (or other backends)
//! - **Manage context** with automatic summarization
//! - **Stream responses** in real-time
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use praxis::prelude::*;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create LLM client
//!     let llm_client = Arc::new(OpenAIClient::new(
//!         std::env::var("OPENAI_API_KEY")?
//!     )?);
//!
//!     // Create MCP executor
//!     let mcp_executor = Arc::new(MCPToolExecutor::new());
//!
//!     // Build graph
//!     let graph = GraphBuilder::new()
//!         .with_llm_client(llm_client)
//!         .with_mcp_executor(mcp_executor)
//!         .build()?;
//!
//!     // Create input
//!     let input = GraphInput::new(
//!         "conversation-123",
//!         vec![Message::Human {
//!             content: Content::text("Hello!"),
//!             name: None,
//!         }],
//!         LLMConfig::new("gpt-4o"),
//!     );
//!
//!     // Execute and stream events
//!     let mut events = graph.spawn_run(input, None);
//!     while let Some(event) = events.recv().await {
//!         match event {
//!             StreamEvent::Message { content } => println!("{}", content),
//!             StreamEvent::Done { .. } => break,
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! Praxis is organized into focused crates:
//!
//! - **`praxis-graph`**: React agent orchestrator with graph execution
//! - **`praxis-llm`**: Provider-agnostic LLM client (OpenAI, Azure)
//! - **`praxis-mcp`**: Model Context Protocol client and executor
//! - **`praxis-persist`**: Persistence layer with MongoDB support
//! - **`praxis-context`**: Context management and summarization
//!
//! ## Features
//!
//! - ✅ **Streaming**: Real-time event streaming with zero-copy optimizations
//! - ✅ **Persistence**: Incremental saving with MongoDB backend
//! - ✅ **Context Management**: Automatic summarization and token counting
//! - ✅ **Tool Execution**: MCP-based tool integration
//! - ✅ **Type Safety**: Strong typing throughout the framework
//! - ✅ **Async**: Built on Tokio for high performance
//!
//! ## License
//!
//! MIT

pub mod prelude;

pub use praxis_graph::{
    Graph, GraphBuilder, GraphConfig, GraphInput, GraphState, LLMConfig, ContextPolicy,
    StreamEvent, PersistenceConfig, PersistenceContext,
};

pub use praxis_llm::{
    ChatClient, ReasoningClient, LLMClient,
    OpenAIClient,
    ChatRequest, ChatOptions, ResponseRequest, ResponseOptions,
    Message, Content, Tool, ToolCall, ToolChoice,
    ReasoningConfig, ReasoningEffort, SummaryMode,
};

pub use praxis_mcp::{
    MCPClient, MCPToolExecutor, ToolResponse,
};

pub use praxis_persist::{
    PersistenceClient, EventAccumulator, StreamEventExtractor,
    DBMessage, MessageRole, MessageType, Thread, ThreadMetadata, ThreadSummary, PersistError,
};

#[cfg(feature = "mongodb")]
pub use praxis_persist::MongoPersistenceClient;

pub use praxis_context::{
    ContextStrategy, ContextWindow, DefaultContextStrategy,
};

