use serde::{Deserialize, Serialize};

/// Extended StreamEvent that includes both LLM streaming events and Graph orchestration events
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

