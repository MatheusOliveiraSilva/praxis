pub mod types;
pub mod traits;
pub mod streaming;
pub mod buffer_utils;
pub mod openai;
pub mod azure_openai;
pub mod config;

pub use traits::{
    ChatClient,
    ReasoningClient,
    LLMClient, 
    ChatRequest, ChatResponse, ChatOptions,
    ResponseRequest, ResponseOutput, ResponseOptions,
    TokenUsage,
};

pub use streaming::StreamEvent;
pub use streaming::{CircularLineBuffer, EventBatcher};
pub use openai::OpenAIClient;
pub use openai::{ReasoningConfig, ReasoningEffort, SummaryMode};
pub use azure_openai::AzureOpenAIClient;
pub use config::{
    ProviderType, ProviderConfig, ProviderDetails,
    OpenAIConfig, AzureConfig, ClientFactory,
};
pub use types::{Message, Content, Tool, ToolCall, ToolChoice};
