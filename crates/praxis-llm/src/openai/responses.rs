// OpenAI Responses API
// https://platform.openai.com/docs/api-reference/responses

use serde::{Deserialize, Serialize};

/// Reasoning effort level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

/// Summary mode for reasoning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SummaryMode {
    Auto,
    Detailed,
}

/// Reasoning configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningConfig {
    pub effort: ReasoningEffort,
    pub summary: SummaryMode,
}

impl ReasoningConfig {
    pub fn new(effort: ReasoningEffort, summary: SummaryMode) -> Self {
        Self { effort, summary }
    }
    
    pub fn low() -> Self {
        Self::new(ReasoningEffort::Low, SummaryMode::Auto)
    }
    
    pub fn medium() -> Self {
        Self::new(ReasoningEffort::Medium, SummaryMode::Auto)
    }
    
    pub fn high() -> Self {
        Self::new(ReasoningEffort::High, SummaryMode::Auto)
    }
}

/// Non-streaming response from /v1/responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsesResponse {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub status: String,
    pub model: String,
    pub output: Vec<OutputItem>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
}

/// Item in the output array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputItem {
    Reasoning {
        id: String,
        summary: Vec<SummaryText>,
    },
    Message {
        id: String,
        status: String,
        role: String,
        content: Vec<ContentItem>,
    },
}

/// Summary text for reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryText {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Content item in message output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentItem {
    OutputText {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Vec<serde_json::Value>>,
    },
    // Future: other content types (images, etc)
}

/// Usage stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<OutputTokensDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
}

// ============================================================================
// STREAMING TYPES
// ============================================================================

/// Streaming chunk from /v1/responses (with stream=true)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStreamChunk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Delta for streaming output
/// Note: The Responses API sends different structures without explicit type tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDelta {
    /// For reasoning deltas - can be text or structured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<serde_json::Value>,
    
    /// For message deltas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    
    /// Content can be text or structured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    
    /// Direct text field (sometimes used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    
    /// Type indicator (if present)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub delta_type: Option<String>,
}

impl ResponseStreamChunk {
    /// Extract text content from reasoning delta
    pub fn reasoning_text(&self) -> Option<String> {
        let delta = self.delta.as_ref()?;
        
        // Try as direct string
        if let Some(text) = delta.as_str() {
            return Some(text.to_string());
        }
        
        // Try as object
        let obj = delta.as_object()?;
        
        // Try direct text field
        if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
            return Some(text.to_string());
        }
        
        // Try summary field (can be array or object or string)
        if let Some(summary) = obj.get("summary") {
            // Try as array of objects with text field
            if let Some(arr) = summary.as_array() {
                for item in arr {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        return Some(text.to_string());
                    }
                }
            }
            // Try as direct string
            if let Some(text) = summary.as_str() {
                return Some(text.to_string());
            }
        }
        
        None
    }
    
    /// Extract text content from message delta
    pub fn message_text(&self) -> Option<String> {
        let delta = self.delta.as_ref()?;
        
        // Try as direct string
        if let Some(text) = delta.as_str() {
            return Some(text.to_string());
        }
        
        // Try as object
        let obj = delta.as_object()?;
        
        // Try direct text field
        if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
            return Some(text.to_string());
        }
        
        // Try content field (can be array or object or string)
        if let Some(content) = obj.get("content") {
            // Try as array of objects with text field
            if let Some(arr) = content.as_array() {
                for item in arr {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        return Some(text.to_string());
                    }
                }
            }
            // Try as direct string
            if let Some(text) = content.as_str() {
                return Some(text.to_string());
            }
        }
        
        None
    }
    
    /// Check if this chunk indicates completion
    pub fn is_done(&self) -> bool {
        self.status.as_deref() == Some("completed")
    }
}

impl ResponsesResponse {
    /// Get all reasoning text (concatenated)
    pub fn reasoning_text(&self) -> Option<String> {
        let reasoning_texts: Vec<String> = self
            .output
            .iter()
            .filter_map(|item| match item {
                OutputItem::Reasoning { summary, .. } => {
                    let text = summary
                        .iter()
                        .map(|s| s.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    if text.is_empty() {
                        None
                    } else {
                        Some(text)
                    }
                }
                _ => None,
            })
            .collect();
        
        if reasoning_texts.is_empty() {
            None
        } else {
            Some(reasoning_texts.join("\n"))
        }
    }
    
    /// Get all message text (concatenated)
    pub fn message_text(&self) -> Option<String> {
        let message_texts: Vec<String> = self
            .output
            .iter()
            .filter_map(|item| match item {
                OutputItem::Message { content, .. } => {
                    let text = content
                        .iter()
                        .filter_map(|c| match c {
                            ContentItem::OutputText { text, .. } => Some(text.as_str()),
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    if text.is_empty() {
                        None
                    } else {
                        Some(text)
                    }
                }
                _ => None,
            })
            .collect();
        
        if message_texts.is_empty() {
            None
        } else {
            Some(message_texts.join("\n"))
        }
    }
}

