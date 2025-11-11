use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export TokenUsage from praxis-llm to avoid duplication
pub use praxis_llm::TokenUsage;

/// Observation data captured during node execution
/// 
/// Contains all input/output information needed for tracing.
/// The structure varies based on node type (LLM vs Tool).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeObservation {
    /// Unique identifier for this observation/span
    pub span_id: String,
    
    /// Run identifier for the overall graph execution
    pub run_id: String,
    
    /// Conversation/thread identifier
    pub conversation_id: String,
    
    /// Node type: "llm" or "tool"
    pub node_type: String,
    
    /// Timestamp when node execution started
    pub started_at: chrono::DateTime<chrono::Utc>,
    
    /// Duration of node execution in milliseconds
    pub duration_ms: u64,
    
    /// Input/output data specific to node type
    pub data: NodeObservationData,
    
    /// Optional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Output from a node execution
/// 
/// Represents structured outputs that can be traced separately
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "output_type", rename_all = "snake_case")]
pub enum NodeOutput {
    /// Reasoning output from models like GPT-5, o1
    Reasoning {
        /// OpenAI reasoning ID (rs_xxx)
        id: String,
        /// Reasoning content/summary
        content: String,
    },
    /// Regular message output
    Message {
        /// OpenAI message ID (msg_xxx)
        id: String,
        /// Message content
        content: String,
    },
    /// Tool calls output
    ToolCalls {
        /// Tool call information
        calls: Vec<ToolCallInfo>,
    },
}

/// Node-specific observation data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeObservationData {
    /// LLM node execution data
    Llm {
        /// Messages sent to the LLM (input)
        input_messages: Vec<LangfuseMessage>,
        
        /// Structured outputs from the LLM (can be multiple: reasoning + message)
        outputs: Vec<NodeOutput>,
        
        /// Model identifier
        model: String,
        
        /// Token usage information
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<TokenUsage>,
    },
    
    /// Tool node execution data
    Tool {
        /// Tool calls that were executed (input)
        tool_calls: Vec<ToolCallInfo>,
        
        /// Results from tool executions (output)
        tool_results: Vec<ToolResultInfo>,
    },
}

/// Message format compatible with Langfuse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangfuseMessage {
    /// Message role: "system", "user", "assistant", "tool"
    pub role: String,
    
    /// Message content
    pub content: String,
    
    /// Optional message name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// Optional tool call ID (for tool messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    
    /// Optional tool calls (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

/// Tool call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    /// Tool call identifier
    pub id: String,
    
    /// Tool name
    pub name: String,
    
    /// Tool arguments as JSON
    pub arguments: serde_json::Value,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultInfo {
    /// Tool call identifier this result corresponds to
    pub tool_call_id: String,
    
    /// Tool name
    pub tool_name: String,
    
    /// Result content
    pub result: String,
    
    /// Whether the tool execution resulted in an error
    pub is_error: bool,
    
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// Context for managing trace and span IDs
/// 
/// Maintains state across the lifetime of a graph execution.
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Unique trace identifier
    pub trace_id: String,
    
    /// Run identifier
    pub run_id: String,
    
    /// Conversation identifier
    pub conversation_id: String,
    
    /// Timestamp when trace started
    pub started_at: chrono::DateTime<chrono::Utc>,
    
    /// Counter for generating span IDs
    pub span_counter: u32,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new(run_id: String, conversation_id: String) -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            run_id,
            conversation_id,
            started_at: chrono::Utc::now(),
            span_counter: 0,
        }
    }
    
    /// Generate a new unique span ID
    pub fn next_span_id(&mut self) -> String {
        self.span_counter += 1;
        format!("{}-span-{}", self.trace_id, self.span_counter)
    }
}

