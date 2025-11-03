// Provider-agnostic LLM client traits and request/response types

use crate::openai::{ReasoningConfig, ResponsesResponse};
use crate::streaming::StreamEvent;
use crate::types::{Message, Tool, ToolChoice};
use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Generic LLM client trait that can be implemented by different providers
/// (OpenAI, Azure, Anthropic, etc.)
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Send chat completion request (non-streaming)
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse>;
    
    /// Send chat completion request (streaming)
    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>>;
    
    /// Send response request with reasoning (non-streaming)
    async fn response(&self, request: ResponseRequest) -> Result<ResponseOutput>;
    
    /// Send response request with reasoning (streaming)
    async fn response_stream(
        &self,
        request: ResponseRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>>;
}

/// Provider-agnostic chat completion request
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

/// Chat completion options
#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
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
}

/// Provider-agnostic chat completion response
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<crate::types::ToolCall>>,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
    pub raw: serde_json::Value,
}

/// Provider-agnostic response request (with reasoning support)
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

/// Response API options
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

/// Provider-agnostic response output
#[derive(Debug, Clone)]
pub struct ResponseOutput {
    pub reasoning: Option<String>,
    pub message: Option<String>,
    pub usage: Option<TokenUsage>,
    pub status: Option<String>,
    pub raw: ResponsesResponse,
}

/// Token usage stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
}

