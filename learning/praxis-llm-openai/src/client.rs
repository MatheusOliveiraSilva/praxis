use crate::types::{Content, Message, Tool, ToolCall, ToolChoice};
use crate::streaming::{StreamChunk, parse_sse_stream};
use anyhow::{Context, Result};
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
    
    /// Send chat completion request (non-streaming)
    pub async fn chat_completion(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: ChatOptions,
    ) -> Result<ChatResponse> {
        let request = self.build_request(model, messages, options, false)?;
        
        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }
        
        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse response")?;
        
        Ok(chat_response)
    }
    
    /// Send chat completion request (streaming)
    pub async fn chat_completion_stream(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: ChatOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        let request = self.build_request(model, messages, options, true)?;
        
        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }
        
        // Parse SSE stream
        Ok(parse_sse_stream(response))
    }
    
    /// Build request payload
    fn build_request(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: ChatOptions,
        stream: bool,
    ) -> Result<Value> {
        // Convert our messages to OpenAI format
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
        
        // Add optional parameters
        if let Some(temp) = options.temperature {
            obj.insert("temperature".to_string(), serde_json::json!(temp));
        }
        if let Some(max_tokens) = options.max_tokens {
            obj.insert("max_tokens".to_string(), serde_json::json!(max_tokens));
        }
        if let Some(tools) = options.tools {
            obj.insert("tools".to_string(), serde_json::to_value(tools)?);
        }
        if let Some(tool_choice) = options.tool_choice {
            obj.insert("tool_choice".to_string(), serde_json::to_value(tool_choice)?);
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
                // Convert parts to OpenAI content array format
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

/// OpenAI Chat Completion Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl ChatResponse {
    /// Get first choice message content
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.message.content.as_deref())
    }
    
    /// Get first choice tool calls
    pub fn tool_calls(&self) -> Option<&[ToolCall]> {
        self.choices
            .first()
            .and_then(|c| c.message.tool_calls.as_deref())
    }
    
    /// Convert response to our Message type
    pub fn to_message(&self) -> Option<Message> {
        let choice = self.choices.first()?;
        
        Some(Message::AI {
            content: choice.message.content.as_ref().map(|c| Content::text(c.clone())),
            tool_calls: choice.message.tool_calls.clone(),
            name: None,
        })
    }
}
