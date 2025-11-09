use serde::{Deserialize, Serialize};

/// Unified StreamEvent for Graph orchestration
/// 
/// Includes both LLM streaming events and Graph-specific orchestration events.
/// This is the canonical event type used throughout the Graph execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Graph execution started
    InitStream {
        run_id: String,
        conversation_id: String,
        timestamp: i64,
    },
    
    /// Internal reasoning from LLM (streamed token-by-token)
    Reasoning {
        content: String,
    },
    
    /// Response message from LLM (streamed token-by-token)
    Message {
        content: String,
    },
    
    /// LLM decided to call a tool (streamed incrementally)
    ToolCall {
        index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<String>,
    },
    
    /// Tool execution completed
    ToolResult {
        tool_call_id: String,
        result: String,
        is_error: bool,
        duration_ms: u64,
    },
    
    /// LLM streaming completed
    Done {
        #[serde(skip_serializing_if = "Option::is_none")]
        finish_reason: Option<String>,
    },
    
    /// Fatal error occurred
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        node_id: Option<String>,
    },
    
    /// Graph execution completed
    EndStream {
        status: String,
        total_duration_ms: u64,
    },
}

/// Automatic conversion from LLM StreamEvent to Graph StreamEvent
impl From<praxis_llm::StreamEvent> for StreamEvent {
    fn from(event: praxis_llm::StreamEvent) -> Self {
        match event {
            praxis_llm::StreamEvent::Reasoning { content } => {
                Self::Reasoning { content }
            }
            praxis_llm::StreamEvent::Message { content } => {
                Self::Message { content }
            }
            praxis_llm::StreamEvent::ToolCall {
                index,
                id,
                name,
                arguments,
            } => Self::ToolCall {
                index,
                id,
                name,
                arguments,
            },
            praxis_llm::StreamEvent::Done { finish_reason } => {
                Self::Done { finish_reason }
            }
        }
    }
}

/// Implementation of StreamEventExtractor for praxis-persist compatibility
impl praxis_persist::StreamEventExtractor for StreamEvent {
    fn is_reasoning(&self) -> bool {
        matches!(self, Self::Reasoning { .. })
    }
    
    fn is_message(&self) -> bool {
        matches!(self, Self::Message { .. })
    }
    
    fn is_tool_call(&self) -> bool {
        matches!(self, Self::ToolCall { .. })
    }
    
    fn reasoning_content(&self) -> Option<&str> {
        match self {
            Self::Reasoning { content } => Some(content),
            _ => None,
        }
    }
    
    fn message_content(&self) -> Option<&str> {
        match self {
            Self::Message { content } => Some(content),
            _ => None,
        }
    }
    
    fn tool_call_info(&self) -> Option<(u32, Option<&str>, Option<&str>, Option<&str>)> {
        match self {
            Self::ToolCall { index, id, name, arguments } => {
                Some((
                    *index,
                    id.as_deref(),
                    name.as_deref(),
                    arguments.as_deref(),
                ))
            },
            _ => None,
        }
    }
}

