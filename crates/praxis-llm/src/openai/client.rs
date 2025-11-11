// OpenAI-specific client implementation

use crate::openai::{ReasoningConfig, ResponsesResponse};
use crate::streaming::{parse_chat_sse_stream, parse_response_sse_stream, StreamEvent};
use crate::traits::{
    ChatClient, ChatOptions, ChatRequest, ChatResponse, LLMClient, ReasoningClient,
    ResponseOptions, ResponseOutput, ResponseRequest, TokenUsage,
};
use crate::types::{Content, Message, ToolCall};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;

const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

/// OpenAI client (HTTP direct, no SDK)
pub struct OpenAIClient {
    http_client: reqwest::Client,
    base_url: String,
}

impl OpenAIClient {
    /// Create new client with API key
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let api_key = api_key.into();
        
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .context("Invalid API key format")?,
        );
        
        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            http_client,
            base_url: OPENAI_API_BASE.to_string(),
        })
    }
    
    /// Build chat completion request payload
    fn build_chat_request(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: &ChatOptions,
        stream: bool,
    ) -> Result<Value> {
        let openai_messages: Vec<Value> = messages
            .into_iter()
            .map(|msg| self.convert_message(msg))
            .collect::<Result<Vec<_>>>()?;
        
        let mut request = serde_json::json!({
            "model": model,
            "messages": openai_messages,
            "stream": stream,
        });
        
        let obj = request.as_object_mut().unwrap();
        
        // Check if it's an o1 or gpt-5 model (uses different parameter names)
        let is_reasoning_model = model.starts_with("o1") || model.starts_with("gpt-5");
        
        if let Some(temp) = options.temperature {
            // o1 and gpt-5 models don't support temperature
            if !is_reasoning_model {
            obj.insert("temperature".to_string(), serde_json::json!(temp));
            }
        }
        if let Some(max_tokens) = options.max_tokens {
            // o1 and gpt-5 use max_completion_tokens instead of max_tokens
            let token_field = if is_reasoning_model {
                "max_completion_tokens"
            } else {
                "max_tokens"
            };
            obj.insert(token_field.to_string(), serde_json::json!(max_tokens));
        }
        if let Some(ref reasoning_effort) = options.reasoning_effort {
            obj.insert("reasoning_effort".to_string(), serde_json::json!(reasoning_effort));
        }
        if let Some(tools) = &options.tools {
            obj.insert("tools".to_string(), serde_json::to_value(tools)?);
        }
        if let Some(tool_choice) = &options.tool_choice {
            obj.insert("tool_choice".to_string(), serde_json::to_value(tool_choice)?);
        }
        
        Ok(request)
    }
    
    /// Build responses request payload
    fn build_response_request(
        &self,
        model: &str,
        input: Vec<Message>,
        reasoning: Option<&ReasoningConfig>,
        options: &ResponseOptions,
        stream: bool,
    ) -> Result<Value> {
        let openai_messages: Vec<Value> = input
            .into_iter()
            .map(|msg| self.convert_message(msg))
            .collect::<Result<Vec<_>>>()?;
        
        let mut request = serde_json::json!({
            "model": model,
            "input": openai_messages,
            "stream": stream,
        });
        
        let obj = request.as_object_mut().unwrap();
        
        if let Some(reasoning) = reasoning {
            obj.insert("reasoning".to_string(), serde_json::to_value(reasoning)?);
        }
        if let Some(temp) = options.temperature {
            obj.insert("temperature".to_string(), serde_json::json!(temp));
        }
        if let Some(max_tokens) = options.max_output_tokens {
            obj.insert("max_output_tokens".to_string(), serde_json::json!(max_tokens));
        }
        
        Ok(request)
    }
    
    /// Convert our Message type to OpenAI format
    fn convert_message(&self, message: Message) -> Result<Value> {
        match message {
            Message::System { content, name } => {
                let mut obj = serde_json::json!({
                    "role": "system",
                    "content": self.convert_content(content)?,
                });
                if let Some(name) = name {
                    obj.as_object_mut().unwrap().insert("name".to_string(), serde_json::json!(name));
                }
                Ok(obj)
            }
            Message::Human { content, name } => {
                let mut obj = serde_json::json!({
                    "role": "user",
                    "content": self.convert_content(content)?,
                });
                if let Some(name) = name {
                    obj.as_object_mut().unwrap().insert("name".to_string(), serde_json::json!(name));
                }
                Ok(obj)
            }
            Message::AI { content, tool_calls, name } => {
                let mut obj = serde_json::json!({
                    "role": "assistant",
                });
                
                let map = obj.as_object_mut().unwrap();
                
                if let Some(content) = content {
                    map.insert("content".to_string(), self.convert_content(content)?);
                }
                
                if let Some(tool_calls) = tool_calls {
                    map.insert("tool_calls".to_string(), serde_json::to_value(tool_calls)?);
                }
                
                if let Some(name) = name {
                    map.insert("name".to_string(), serde_json::json!(name));
                }
                
                Ok(obj)
            }
            Message::Tool { tool_call_id, content } => {
                Ok(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tool_call_id,
                    "content": self.convert_content(content)?,
                }))
            }
        }
    }
    
    /// Convert Content to OpenAI format (string or array)
    fn convert_content(&self, content: Content) -> Result<Value> {
        match content {
            Content::Text(s) => Ok(serde_json::json!(s)),
            Content::Parts(parts) => {
                let converted: Vec<Value> = parts
                    .into_iter()
                    .map(|part| match part {
                        crate::types::ContentPart::Text { text } => {
                            serde_json::json!({
                                "type": "text",
                                "text": text,
                            })
                        }
                    })
                    .collect();
                Ok(serde_json::json!(converted))
            }
        }
    }
}

// ============================================================================
// TRAIT IMPLEMENTATIONS
// ============================================================================

#[async_trait]
impl ChatClient for OpenAIClient {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let payload = self.build_chat_request(
            &request.model,
            request.messages,
            &request.options,
            false,
        )?;
        
        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }
        
        let raw: OpenAIChatResponse = response
            .json()
            .await
            .context("Failed to parse response")?;
        
        // Convert to provider-agnostic response
        let choice = raw.choices.first();
        Ok(ChatResponse {
            content: choice.and_then(|c| c.message.content.clone()),
            tool_calls: choice.and_then(|c| c.message.tool_calls.clone()),
            usage: Some(TokenUsage {
                input_tokens: raw.usage.prompt_tokens,
                output_tokens: raw.usage.completion_tokens,
                total_tokens: raw.usage.total_tokens,
                reasoning_tokens: None,
            }),
            finish_reason: choice.and_then(|c| c.finish_reason.clone()),
            raw: serde_json::to_value(raw)?,
        })
    }
    
    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let payload = self.build_chat_request(
            &request.model,
            request.messages,
            &request.options,
            true,
        )?;
        
        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }
        
        Ok(parse_chat_sse_stream(response))
    }
    }
    
#[async_trait]
impl ReasoningClient for OpenAIClient {
    async fn reason(&self, request: ResponseRequest) -> Result<ResponseOutput> {
        let payload = self.build_response_request(
            &request.model,
            request.input,
            request.reasoning.as_ref(),
            &request.options,
            false,
        )?;
        
        let response = self
            .http_client
            .post(format!("{}/responses", self.base_url))
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }
        
        let raw: ResponsesResponse = response
            .json()
            .await
            .context("Failed to parse response")?;
        
        // Convert to provider-agnostic response
        Ok(ResponseOutput {
            reasoning: raw.reasoning_text(),
            message: raw.message_text(),
            usage: Some(TokenUsage {
                input_tokens: raw.usage.input_tokens,
                output_tokens: raw.usage.output_tokens,
                total_tokens: raw.usage.total_tokens,
                reasoning_tokens: raw.usage.output_tokens_details
                    .as_ref()
                    .and_then(|d| d.reasoning_tokens),
            }),
            status: Some(raw.status.clone()),
            raw,
        })
    }
    
    async fn reason_stream(
        &self,
        request: ResponseRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let payload = self.build_response_request(
            &request.model,
            request.input,
            request.reasoning.as_ref(),
            &request.options,
            true,
        )?;
        
        let response = self
            .http_client
            .post(format!("{}/responses", self.base_url))
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }
        
        Ok(parse_response_sse_stream(response))
    }
}

// OpenAI supports both chat and reasoning
impl LLMClient for OpenAIClient {}

// ============================================================================
// OPENAI-SPECIFIC RESPONSE TYPES (for Chat Completions)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

