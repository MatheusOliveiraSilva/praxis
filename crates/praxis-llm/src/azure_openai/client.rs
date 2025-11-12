// Azure OpenAI-specific client implementation

use crate::openai::{ReasoningConfig, ResponsesResponse};
use crate::streaming::{parse_chat_sse_stream, StreamEvent};
use crate::traits::{
    ChatClient, ChatOptions, ChatRequest, ChatResponse, LLMClient, ReasoningClient,
    ResponseOptions, ResponseOutput, ResponseRequest, TokenUsage,
};
use crate::types::{Content, Message, ToolCall};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;

/// Azure OpenAI client (HTTP direct, no SDK)
/// 
/// Azure OpenAI uses a different endpoint structure and authentication method than OpenAI:
/// - URL: https://{resource}.openai.azure.com/openai/deployments/{deployment}/...
/// - Auth header: api-key instead of Authorization: Bearer
/// - Deployment name is passed via the model parameter in each request
#[derive(Debug)]
pub struct AzureOpenAIClient {
    http_client: reqwest::Client,
    endpoint: String,
    api_version: String,
}

impl AzureOpenAIClient {
    /// Create new Azure OpenAI client with builder pattern
    pub fn builder() -> AzureOpenAIClientBuilder {
        AzureOpenAIClientBuilder::default()
    }
    
    /// Build chat completion request payload
    fn build_chat_request(
        &self,
        _model: &str,
        messages: Vec<Message>,
        options: &ChatOptions,
        stream: bool,
    ) -> Result<Value> {
        let azure_messages: Vec<Value> = messages
            .into_iter()
            .map(|msg| self.convert_message(msg))
            .collect::<Result<Vec<_>>>()?;
        
        let mut request = serde_json::json!({
            "messages": azure_messages,
            "stream": stream,
        });
        
        let obj = request.as_object_mut().unwrap();
        
        // Check if it's an o1 or gpt-5 model (uses different parameter names)
        let is_reasoning_model = _model.starts_with("o1") || _model.starts_with("gpt-5");
        
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
    
    /// Build responses request payload for Azure
    /// Azure uses chat/completions format (messages) not responses format (input)
    fn build_response_request(
        &self,
        model: &str,
        input: Vec<Message>,
        reasoning: Option<&ReasoningConfig>,
        options: &ResponseOptions,
        stream: bool,
    ) -> Result<Value> {
        let azure_messages: Vec<Value> = input
            .into_iter()
            .map(|msg| self.convert_message(msg))
            .collect::<Result<Vec<_>>>()?;
        
        // Azure uses same format as chat/completions with "messages" not "input"
        let mut request = serde_json::json!({
            "messages": azure_messages,
            "stream": stream,
        });
        
        let obj = request.as_object_mut().unwrap();
        
        // Check if it's a reasoning model
        let is_reasoning_model = model.starts_with("o1") || model.starts_with("gpt-5");
        
        // Azure uses reasoning_effort directly (not a reasoning object like OpenAI /responses)
        if let Some(reasoning) = reasoning {
            // Convert reasoning config to reasoning_effort string
            let effort_str = match reasoning.effort {
                crate::openai::ReasoningEffort::Low => "low",
                crate::openai::ReasoningEffort::Medium => "medium",
                crate::openai::ReasoningEffort::High => "high",
            };
            obj.insert("reasoning_effort".to_string(), serde_json::json!(effort_str));
        }
        
        // For reasoning models, use max_completion_tokens
        if let Some(max_tokens) = options.max_output_tokens {
            let token_field = if is_reasoning_model {
                "max_completion_tokens"
            } else {
                "max_tokens"
            };
            obj.insert(token_field.to_string(), serde_json::json!(max_tokens));
        }
        
        // Only add temperature for non-reasoning models
        if let Some(temp) = options.temperature {
            if !is_reasoning_model {
                obj.insert("temperature".to_string(), serde_json::json!(temp));
            }
        }
        
        Ok(request)
    }
    
    /// Convert our Message type to Azure OpenAI format (same as OpenAI)
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
    
    /// Convert Content to Azure OpenAI format (string or array)
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
    
    /// Build the full URL for an Azure OpenAI endpoint
    /// The deployment_name comes from the model parameter in the request
    fn build_url(&self, deployment_name: &str, path: &str) -> String {
        format!(
            "{}/openai/deployments/{}/{}?api-version={}",
            self.endpoint, deployment_name, path, self.api_version
        )
    }
}

/// Builder for AzureOpenAIClient
#[derive(Default)]
pub struct AzureOpenAIClientBuilder {
    api_key: Option<String>,
    endpoint: Option<String>,
    api_version: Option<String>,
}

impl AzureOpenAIClientBuilder {
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
    
    /// Set the Azure OpenAI endpoint (base URL)
    /// Example: "https://my-resource.openai.azure.com"
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }
    
    pub fn api_version(mut self, api_version: impl Into<String>) -> Self {
        self.api_version = Some(api_version.into());
        self
    }
    
    pub fn build(self) -> Result<AzureOpenAIClient> {
        let api_key = self.api_key.context("API key is required")?;
        let endpoint = self.endpoint.context("Endpoint is required")?;
        let api_version = self.api_version.context("API version is required")?;
        
        // Remove trailing slash from endpoint
        let endpoint = endpoint.trim_end_matches('/').to_string();
        
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "api-key",
            HeaderValue::from_str(&api_key)
                .context("Invalid API key format")?,
        );
        
        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(AzureOpenAIClient {
            http_client,
            endpoint,
            api_version,
        })
    }
}

// ============================================================================
// TRAIT IMPLEMENTATIONS
// ============================================================================

#[async_trait]
impl ChatClient for AzureOpenAIClient {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let deployment_name = &request.model;
        
        let payload = self.build_chat_request(
            &request.model,
            request.messages,
            &request.options,
            false,
        )?;
        
        let url = self.build_url(deployment_name, "chat/completions");
        
        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Azure OpenAI API error ({}): {}", status, error_text);
        }
        
        let raw: AzureChatResponse = response
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
        let deployment_name = &request.model;
        
        let payload = self.build_chat_request(
            &request.model,
            request.messages,
            &request.options,
            true,
        )?;
        
        let url = self.build_url(deployment_name, "chat/completions");
        
        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Azure OpenAI API error ({}): {}", status, error_text);
        }
        
        Ok(parse_chat_sse_stream(response))
    }
}

#[async_trait]
impl ReasoningClient for AzureOpenAIClient {
    async fn reason(&self, request: ResponseRequest) -> Result<ResponseOutput> {
        let deployment_name = &request.model;
        
        let payload = self.build_response_request(
            &request.model,
            request.input,
            request.reasoning.as_ref(),
            &request.options,
            false,
        )?;
        
        // Azure OpenAI uses chat/completions for reasoning models (gpt-5, o1)
        // not a separate /responses endpoint like OpenAI
        let url = self.build_url(deployment_name, "chat/completions");
        
        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Azure OpenAI API error ({}): {}", status, error_text);
        }
        
        // Azure returns chat/completions format, not responses format
        let chat_response: AzureChatResponse = response
            .json()
            .await
            .context("Failed to parse response")?;
        
        // Extract content from the first choice
        let message_content = chat_response.choices
            .first()
            .and_then(|c| c.message.content.clone());
        
        // For reasoning models, Azure may include reasoning in the response
        // For now, we'll use the message content
        let reasoning_content = None; // Azure doesn't separate reasoning in the same way
        
        // Create a synthetic ResponsesResponse for compatibility
        let raw = ResponsesResponse {
            id: chat_response.id.clone(),
            object: "response".to_string(),
            created_at: chat_response.created,
            status: "completed".to_string(),
            model: chat_response.model.clone(),
            output: vec![],
            usage: crate::openai::responses::Usage {
                input_tokens: chat_response.usage.prompt_tokens,
                output_tokens: chat_response.usage.completion_tokens,
                total_tokens: chat_response.usage.total_tokens,
                output_tokens_details: None,
            },
            reasoning: None,
        };
        
        // Convert to provider-agnostic response
        Ok(ResponseOutput {
            reasoning: reasoning_content,
            message: message_content,
            usage: Some(TokenUsage {
                input_tokens: chat_response.usage.prompt_tokens,
                output_tokens: chat_response.usage.completion_tokens,
                total_tokens: chat_response.usage.total_tokens,
                reasoning_tokens: None,
            }),
            status: Some("completed".to_string()),
            raw,
        })
    }
    
    async fn reason_stream(
        &self,
        request: ResponseRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let deployment_name = &request.model;
        
        let payload = self.build_response_request(
            &request.model,
            request.input,
            request.reasoning.as_ref(),
            &request.options,
            true,
        )?;
        
        // Azure OpenAI uses chat/completions for reasoning models (gpt-5, o1)
        // not a separate /responses endpoint like OpenAI
        let url = self.build_url(deployment_name, "chat/completions");
        
        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Azure OpenAI API error ({}): {}", status, error_text);
        }
        
        // Azure uses chat/completions format for streaming too
        Ok(parse_chat_sse_stream(response))
    }
}

// Azure OpenAI supports both chat and reasoning
impl LLMClient for AzureOpenAIClient {}

// ============================================================================
// AZURE-SPECIFIC RESPONSE TYPES (for Chat Completions)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AzureChatResponse {
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
