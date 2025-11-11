use praxis_llm::ToolCall;

/// Graph output items from LLM execution
/// 
/// Represents structured outputs that can be persisted and traced separately.
/// This is distinct from `praxis_llm::openai::OutputItem` which is the raw API format.
#[derive(Debug, Clone)]
pub enum GraphOutput {
    /// Reasoning output from models like GPT-5, o1
    Reasoning {
        id: String,
        content: String,
    },
    /// Regular message output
    Message {
        id: String,
        content: String,
        tool_calls: Option<Vec<ToolCall>>,
    },
}

impl GraphOutput {
    pub fn reasoning(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Reasoning {
            id: id.into(),
            content: content.into(),
        }
    }
    
    pub fn message(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Message {
            id: id.into(),
            content: content.into(),
            tool_calls: None,
        }
    }
    
    pub fn message_with_tools(
        id: impl Into<String>,
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self::Message {
            id: id.into(),
            content: content.into(),
            tool_calls: Some(tool_calls),
        }
    }
    
    pub fn id(&self) -> &str {
        match self {
            Self::Reasoning { id, .. } => id,
            Self::Message { id, .. } => id,
        }
    }
    
    pub fn content(&self) -> &str {
        match self {
            Self::Reasoning { content, .. } => content,
            Self::Message { content, .. } => content,
        }
    }
}

