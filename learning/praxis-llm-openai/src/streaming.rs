// Streaming support (SSE parsing)

use anyhow::Result;
use futures::{Stream, StreamExt};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Streaming chunk from OpenAI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
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

impl StreamChunk {
    /// Get content delta from first choice
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.delta.content.as_deref())
    }
    
    /// Check if stream is done
    pub fn is_done(&self) -> bool {
        self.choices
            .first()
            .and_then(|c| c.finish_reason.as_ref())
            .is_some()
    }
}

/// Parse SSE stream from OpenAI
pub fn parse_sse_stream(
    response: Response,
) -> Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>> {
    let stream = response.bytes_stream();
    
    Box::pin(async_stream::stream! {
        let mut byte_chunks = Box::pin(stream);
        let mut buffer = String::new();
        
        while let Some(chunk_result) = byte_chunks.next().await {
            match chunk_result {
                Ok(bytes) => {
                    // Add to buffer
                    if let Ok(text) = std::str::from_utf8(&bytes) {
                        buffer.push_str(text);
                        
                        // Process complete lines
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();
                            
                            // Skip empty lines
                            if line.is_empty() {
                                continue;
                            }
                            
                            // SSE format: "data: {...}"
                            if let Some(data) = line.strip_prefix("data: ") {
                                // Check for [DONE] marker
                                if data == "[DONE]" {
                                    break;
                                }
                                
                                // Parse JSON chunk
                                match serde_json::from_str::<StreamChunk>(data) {
                                    Ok(chunk) => yield Ok(chunk),
                                    Err(e) => yield Err(anyhow::anyhow!("Failed to parse chunk: {}", e)),
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
