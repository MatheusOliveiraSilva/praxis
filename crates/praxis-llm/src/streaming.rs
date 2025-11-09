use std::pin::Pin;
use anyhow::Result;
use reqwest::Response;
use std::collections::VecDeque;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};

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

pub fn parse_chat_sse_stream(
    response: Response,
) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
    let stream = response.bytes_stream();
    
    Box::pin(async_stream::stream! {
        let mut byte_chunks = Box::pin(stream);
        let mut buffer = VecDeque::with_capacity(8192);
        
        while let Some(chunk_result) = byte_chunks.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.extend(bytes);
                    
                    while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                        let line_bytes: Vec<u8> = buffer.drain(..=newline_pos).collect();
                        
                        if let Ok(line_str) = std::str::from_utf8(&line_bytes) {
                            let line = line_str.trim();
                            
                            if line.is_empty() {
                                continue;
                            }
                            
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    yield Ok(StreamEvent::Done { finish_reason: None });
                                    break;
                                }
                                
                                match serde_json::from_str::<ChatStreamChunk>(data) {
                                    Ok(chunk) => {
                                        for event in chunk.to_stream_events() {
                                            yield Ok(event);
                                        }
                                    }
                                    Err(e) => yield Err(anyhow::anyhow!("Failed to parse chat chunk: {}", e)),
                                }
                            }
                        }
                    }
                }
                Err(e) => yield Err(anyhow::anyhow!("Stream error: {}", e)),
            }
        }
    })
}

pub fn parse_response_sse_stream(
    response: Response,
) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
    let stream = response.bytes_stream();
    
    Box::pin(async_stream::stream! {
        let mut byte_chunks = Box::pin(stream);
        let mut buffer = VecDeque::with_capacity(8192);
        
        while let Some(chunk_result) = byte_chunks.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.extend(bytes);
                    
                    while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                        let line_bytes: Vec<u8> = buffer.drain(..=newline_pos).collect();
                        
                        if let Ok(line_str) = std::str::from_utf8(&line_bytes) {
                            let line = line_str.trim();
                            
                            if line.is_empty() {
                                continue;
                            }
                            
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    yield Ok(StreamEvent::Done { finish_reason: None });
                                    break;
                                }
                                
                                match serde_json::from_str::<ResponseStreamChunk>(data) {
                                    Ok(chunk) => {
                                        if chunk.is_done() {
                                            yield Ok(StreamEvent::Done {
                                                finish_reason: chunk.status.clone(),
                                            });
                                            continue;
                                        }
                                        
                                        let is_reasoning = chunk.output_index.map(|idx| idx == 0).unwrap_or(false);
                                        
                                        if is_reasoning {
                                            if let Some(text) = chunk.reasoning_text() {
                                                if !text.is_empty() {
                                                    yield Ok(StreamEvent::Reasoning {
                                                        content: text,
                                                    });
                                                }
                                            }
                                        } else {
                                            if let Some(text) = chunk.message_text() {
                                                if !text.is_empty() {
                                                    yield Ok(StreamEvent::Message {
                                                        content: text,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => yield Err(anyhow::anyhow!("Failed to parse response chunk: {}", e)),
                                }
                            }
                        }
                    }
                }
                Err(e) => yield Err(anyhow::anyhow!("Stream error: {}", e)),
            }
        }
    })
}

pub use ChatStreamChunk as StreamChunk;
pub fn parse_sse_stream(response: Response) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
    parse_chat_sse_stream(response)
}

