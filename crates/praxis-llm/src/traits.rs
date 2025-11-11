use crate::openai::{ReasoningConfig, ResponsesResponse};
use crate::streaming::StreamEvent;
use crate::types::{Message, Tool, ToolChoice};
use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Trait for chat-based LLM interactions (GPT-4, etc)
/// 
/// Provides both streaming and non-streaming completions for conversational use cases.
#[async_trait]
pub trait ChatClient: Send + Sync {
    /// Non-streaming chat completion
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    
    /// Streaming chat completion
    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>>;
}

/// Trait for reasoning-based LLM interactions (o1 models)
/// 
/// Provides access to models with extended reasoning capabilities.
#[async_trait]
pub trait ReasoningClient: Send + Sync {
    /// Non-streaming reasoning completion
    async fn reason(&self, request: ResponseRequest) -> Result<ResponseOutput>;
    
    /// Streaming reasoning completion
    async fn reason_stream(
        &self,
        request: ResponseRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>>;
}

/// Convenience trait for clients that support both chat and reasoning
pub trait LLMClient: ChatClient + ReasoningClient {}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub options: ChatOptions,
}

impl ChatRequest {
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            options: ChatOptions::default(),
        }
    }
    
    pub fn with_options(mut self, options: ChatOptions) -> Self {
        self.options = options;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub reasoning_effort: Option<String>,
}

impl ChatOptions {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }
    
    pub fn max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }
    
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }
    
    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }
    
    pub fn reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(effort.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<crate::types::ToolCall>>,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ResponseRequest {
    pub model: String,
    pub input: Vec<Message>,
    pub reasoning: Option<ReasoningConfig>,
    pub options: ResponseOptions,
}

impl ResponseRequest {
    pub fn new(model: impl Into<String>, input: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            input,
            reasoning: None,
            options: ResponseOptions::default(),
        }
    }
    
    pub fn with_reasoning(mut self, reasoning: ReasoningConfig) -> Self {
        self.reasoning = Some(reasoning);
        self
    }
    
    pub fn with_options(mut self, options: ResponseOptions) -> Self {
        self.options = options;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResponseOptions {
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<u32>,
}

impl ResponseOptions {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }
    
    pub fn max_output_tokens(mut self, tokens: u32) -> Self {
        self.max_output_tokens = Some(tokens);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ResponseOutput {
    pub reasoning: Option<String>,
    pub message: Option<String>,
    pub usage: Option<TokenUsage>,
    pub status: Option<String>,
    pub raw: ResponsesResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
}

