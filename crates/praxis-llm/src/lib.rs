pub mod types;
pub mod traits;
pub mod streaming;
pub mod openai;
pub mod mcp;
pub mod history;
pub mod client;

pub use traits::{
    LLMClient, 
    ChatRequest, ChatResponse, ChatOptions,
    ResponseRequest, ResponseOutput, ResponseOptions,
    TokenUsage,
};

pub use streaming::StreamEvent;
pub use openai::OpenAIClient;
pub use openai::{ReasoningConfig, ReasoningEffort, SummaryMode};
pub use types::{Message, Content, Tool, ToolCall, ToolChoice};
pub use history::{ContentItem, AssistantMessage, reconstruct_messages, reconstruct_conversation};

