use anyhow::Result;
use futures::Stream;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::buffer_utils::{SseLineParser, parse_sse_stream};

pub use crate::buffer_utils::{CircularLineBuffer, EventBatcher};

use crate::openai::ResponseStreamChunk;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Reasoning {
        content: String,
    },
    
    Message {
        content: String,
    },
    
    ToolCall {
        index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<String>,
    },
    
    Done {
        #[serde(skip_serializing_if = "Option::is_none")]
        finish_reason: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    pub index: u32,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    pub function: Option<FunctionDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDelta {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

impl ChatStreamChunk {
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.delta.content.as_deref())
    }
    
    pub fn is_done(&self) -> bool {
        self.choices
            .first()
            .and_then(|c| c.finish_reason.as_ref())
            .is_some()
    }
    
    fn to_stream_events(&self) -> Vec<StreamEvent> {
        let mut events = Vec::new();
        
        if let Some(choice) = self.choices.first() {
            if let Some(content) = &choice.delta.content {
                if !content.is_empty() {
                    events.push(StreamEvent::Message {
                        content: content.clone(),
                    });
                }
            }
            
            if let Some(tool_calls) = &choice.delta.tool_calls {
                for tc in tool_calls {
                    events.push(StreamEvent::ToolCall {
                        index: tc.index,
                        id: tc.id.clone(),
                        name: tc.function.as_ref().and_then(|f| f.name.clone()),
                        arguments: tc.function.as_ref().and_then(|f| f.arguments.clone()),
                    });
                }
            }
            
            if let Some(finish_reason) = &choice.finish_reason {
                events.push(StreamEvent::Done {
                    finish_reason: Some(finish_reason.clone()),
                });
            }
        }
        
        events
    }
}

/// Chat SSE parser (Strategy Pattern)
struct ChatSseParser;

impl SseLineParser for ChatSseParser {
    fn parse_data_line(&self, data: &str) -> Result<Vec<StreamEvent>> {
        let chunk: ChatStreamChunk = serde_json::from_str(data)
            .map_err(|e| anyhow::anyhow!("Failed to parse chat chunk: {}", e))?;
        
        Ok(chunk.to_stream_events())
    }
}

/// Response SSE parser (Strategy Pattern)
struct ResponseSseParser;

impl SseLineParser for ResponseSseParser {
    fn parse_data_line(&self, data: &str) -> Result<Vec<StreamEvent>> {
        let chunk: ResponseStreamChunk = serde_json::from_str(data)
            .map_err(|e| anyhow::anyhow!("Failed to parse response chunk: {}", e))?;
        
        let mut events = Vec::new();
        
        if chunk.is_done() {
            events.push(StreamEvent::Done {
                finish_reason: chunk.status.clone(),
            });
            return Ok(events);
        }
        
        let is_reasoning = chunk.output_index.map(|idx| idx == 0).unwrap_or(false);
        
        // Debug: log what we're receiving
        tracing::debug!(
            "ResponseStreamChunk - output_index: {:?}, is_reasoning: {}, delta: {:?}",
            chunk.output_index,
            is_reasoning,
            chunk.delta
        );
        
        if is_reasoning {
            if let Some(text) = chunk.reasoning_text() {
                if !text.is_empty() {
                    tracing::debug!("Emitting Reasoning event with {} chars", text.len());
                    events.push(StreamEvent::Reasoning { content: text });
                }
            }
        } else {
            if let Some(text) = chunk.message_text() {
                if !text.is_empty() {
                    tracing::debug!("Emitting Message event with {} chars", text.len());
                    events.push(StreamEvent::Message { content: text });
                }
            }
        }
        
        Ok(events)
    }
}

pub fn parse_chat_sse_stream(
    response: Response,
) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
    parse_sse_stream(response, ChatSseParser)
}

pub fn parse_response_sse_stream(
    response: Response,
) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
    parse_sse_stream(response, ResponseSseParser)
}

pub use ChatStreamChunk as StreamChunk;

/// Default SSE parser (uses chat parser for backwards compatibility)
pub fn parse_sse_stream_legacy(response: Response) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
    parse_chat_sse_stream(response)
}

