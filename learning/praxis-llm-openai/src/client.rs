use crate::types::{Content, Message, Tool, ToolCall, ToolChoice};
use crate::streaming::{StreamChunk, parse_sse_stream};
use crate::cache::{ResponseCache, cache_key};
use anyhow::{Context, Result};
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use std::time::Duration;

const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

/// OpenAI client (HTTP direct, no SDK)
pub struct OpenAIClient {
    http_client: reqwest::Client,
    base_url: String,
    cache: Option<ResponseCache>,
}

impl OpenAIClient {
    /// Create new client with API key (no cache)
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_cache(api_key, None)
    }
    
    /// Create new client with optional cache
    pub fn with_cache(api_key: impl Into<String>, cache_ttl: Option<Duration>) -> Result<Self> {
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
        
        let cache = cache_ttl.map(ResponseCache::new);
        
        Ok(Self {
            http_client,
            base_url: OPENAI_API_BASE.to_string(),
            cache,
        })
    }
    
    /// Get cache statistics (if cache enabled)
    pub fn cache_stats(&self) -> Option<crate::cache::CacheStats> {
        self.cache.as_ref().map(|c| c.stats())
    }
    
    /// Clear cache (if enabled)
    pub fn clear_cache(&self) {
        if let Some(cache) = &self.cache {
            cache.clear();
        }
    }
    
    /// Cleanup expired cache entries (if enabled)
    pub fn cleanup_cache(&self) {
        if let Some(cache) = &self.cache {
            cache.cleanup_expired();
        }
    }
    
    /// Send chat completion request (non-streaming)
    pub async fn chat_completion(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: ChatOptions,
    ) -> Result<ChatResponse> {
        let request = self.build_request(model, &messages, &options, false)?;
        
        // Try cache first (if enabled)
        if let Some(cache) = &self.cache {
            let key = self.generate_cache_key(model, &messages, &options)?;
            
            if let Some(cached_bytes) = cache.get(&key) {
                // Cache hit! Deserialize and return
                if let Ok(cached_response) = serde_json::from_slice::<ChatResponse>(&cached_bytes) {
                    return Ok(cached_response);
                }
                // If deserialization fails, invalidate and continue to API call
                cache.invalidate(&key);
            }
            
            // Cache miss - make API call
            let response = self.make_completion_request(&request).await?;
            
            // Store in cache for next time
            if let Ok(response_bytes) = serde_json::to_vec(&response) {
                cache.set(key, response_bytes);
            }
            
            return Ok(response);
        }
        
        // No cache - direct API call
        self.make_completion_request(&request).await
    }
    
    /// Internal method to make actual API call
    async fn make_completion_request(&self, request: &Value) -> Result<ChatResponse> {
        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.base_url))
            .json(request)
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
    
    /// Generate cache key from request parameters
    fn generate_cache_key(
        &self,
        model: &str,
        messages: &[Message],
        options: &ChatOptions,
    ) -> Result<String> {
        let messages_bytes = serde_json::to_vec(messages)?;
        let options_bytes = serde_json::to_vec(options)?;
        Ok(cache_key(model, &messages_bytes, &options_bytes))
    }
    
    /// Send chat completion request (streaming)
    /// Note: Streaming responses are NOT cached
    pub async fn chat_completion_stream(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: ChatOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        let request = self.build_request(model, &messages, &options, true)?;
        
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
        messages: &[Message],
        options: &ChatOptions,
        stream: bool,
    ) -> Result<Value> {
        // Convert our messages to OpenAI format
        let openai_messages: Vec<Value> = messages
            .iter()
            .map(|msg| self.convert_message(msg.clone()))
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    
    /// Reasoning content (o1 models only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    
    /// Reasoning tokens (o1 models only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
}

impl ChatResponse {
    /// Get first choice message content
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.message.content.as_deref())
    }
    
    /// Get reasoning content (o1 models only)
    pub fn reasoning(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.message.reasoning_content.as_deref())
    }
    
    /// Get first choice tool calls
    pub fn tool_calls(&self) -> Option<&[ToolCall]> {
        self.choices
            .first()
            .and_then(|c| c.message.tool_calls.as_deref())
    }
    
    /// Convert response to our Message type
    /// Handles both regular content and reasoning content (o1 models)
    pub fn to_message(&self) -> Option<Message> {
        let choice = self.choices.first()?;
        
        // Build content based on what's present
        let content = match (&choice.message.reasoning_content, &choice.message.content) {
            // Both reasoning and message present (o1 models)
            (Some(reasoning), Some(msg)) => {
                use crate::types::ContentPart;
                Some(Content::Parts(vec![
                    ContentPart::Reasoning { text: reasoning.clone() },
                    ContentPart::Text { text: msg.clone() },
                ]))
            }
            // Only reasoning (rare, but possible)
            (Some(reasoning), None) => {
                use crate::types::ContentPart;
                Some(Content::Parts(vec![
                    ContentPart::Reasoning { text: reasoning.clone() }
                ]))
            }
            // Only message (normal case)
            (None, Some(msg)) => {
                Some(Content::text(msg.clone()))
            }
            // Neither (only tool calls)
            (None, None) => None,
        };
        
        Some(Message::AI {
            content,
            tool_calls: choice.message.tool_calls.clone(),
            name: None,
        })
    }
}
