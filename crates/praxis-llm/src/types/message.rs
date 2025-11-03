use serde::{Deserialize, Serialize};
use super::content::Content;
use super::tool::ToolCall;

/// Praxis message types (high-level, provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    /// System prompt (instructions)
    System {
        content: Content,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    
    /// User/Human message
    #[serde(rename = "user")]
    Human {
        content: Content,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    
    /// Assistant/AI message
    #[serde(rename = "assistant")]
    AI {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Content>,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    
    /// Tool result message
    Tool {
        tool_call_id: String,
        content: Content,
    },
}

impl Message {
    /// Create system message
    pub fn system(content: impl Into<Content>) -> Self {
        Self::System {
            content: content.into(),
            name: None,
        }
    }
    
    /// Create human message
    pub fn human(content: impl Into<Content>) -> Self {
        Self::Human {
            content: content.into(),
            name: None,
        }
    }
    
    /// Create AI message with text
    pub fn ai(content: impl Into<Content>) -> Self {
        Self::AI {
            content: Some(content.into()),
            tool_calls: None,
            name: None,
        }
    }
    
    /// Create AI message with tool calls
    pub fn ai_with_tools(tool_calls: Vec<ToolCall>) -> Self {
        Self::AI {
            content: None,
            tool_calls: Some(tool_calls),
            name: None,
        }
    }
    
    /// Create tool result message
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<Content>) -> Self {
        Self::Tool {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
        }
    }
    
    /// Get role as string
    pub fn role(&self) -> &str {
        match self {
            Self::System { .. } => "system",
            Self::Human { .. } => "user",
            Self::AI { .. } => "assistant",
            Self::Tool { .. } => "tool",
        }
    }
}
