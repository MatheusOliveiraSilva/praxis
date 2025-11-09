//! Prelude module for convenient imports
//!
//! Import everything you need with:
//! ```rust
//! use praxis::prelude::*;
//! ```

pub use crate::{
    Graph, GraphBuilder, GraphConfig, GraphInput, GraphState, LLMConfig, ContextPolicy,
    StreamEvent, PersistenceConfig, PersistenceContext,
    ChatClient, ReasoningClient, LLMClient, OpenAIClient,
    ChatRequest, ChatOptions, Message, Content, Tool, ToolCall, ToolChoice,
    MCPClient, MCPToolExecutor,
    PersistenceClient, EventAccumulator,
    ContextStrategy, ContextWindow, DefaultContextStrategy,
};

