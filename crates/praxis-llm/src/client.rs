// Re-export for backwards compatibility and convenience

// Provider-agnostic client
pub use crate::openai::OpenAIClient;

// Kept for backwards compatibility with existing code
pub use crate::traits::{ChatOptions, ChatRequest, ChatResponse};
pub use crate::openai::responses::Usage;
pub use crate::types::ToolCall;

// These types are kept for legacy compatibility
use serde::{Deserialize, Serialize};

/// Legacy: OpenAI Chat Completion Response
/// Kept for backwards compatibility with existing code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyChatResponse {
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

impl LegacyChatResponse {
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
    pub fn to_message(&self) -> Option<crate::types::Message> {
        use crate::types::{Content, Message};
        let choice = self.choices.first()?;
        
        Some(Message::AI {
            content: choice.message.content.as_ref().map(|c| Content::text(c.clone())),
            tool_calls: choice.message.tool_calls.clone(),
            name: None,
        })
    }
}

