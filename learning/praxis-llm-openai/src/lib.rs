// Core modules
pub mod types;
pub mod traits;
pub mod streaming;
pub mod openai;
pub mod mcp;
pub mod history;
pub mod client;

// Provider-agnostic types
pub use traits::{
    LLMClient, 
    ChatRequest, ChatResponse, ChatOptions,
    ResponseRequest, ResponseOutput, ResponseOptions,
    TokenUsage,
};

// Streaming events
pub use streaming::StreamEvent;

// OpenAI client
pub use openai::OpenAIClient;

// Reasoning config
pub use openai::{ReasoningConfig, ReasoningEffort, SummaryMode};

// Shared types
pub use types::{Message, Content, Tool, ToolCall, ToolChoice};

// History reconstruction
pub use history::{ContentItem, AssistantMessage, reconstruct_messages, reconstruct_conversation};
