use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use praxis_llm::types::FunctionCall;

/// Database-agnostic message model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBMessage {
    pub id: String,
    pub thread_id: String,
    pub user_id: String,
    pub role: MessageRole,
    pub message_type: MessageType,
    pub content: String,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
    pub arguments: Option<serde_json::Value>,
    pub reasoning_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub duration_ms: Option<u64>,
}

impl Default for DBMessage {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: String::new(),
            user_id: String::new(),
            role: MessageRole::Assistant,
            message_type: MessageType::Message,
            content: String::new(),
            tool_call_id: None,
            tool_name: None,
            arguments: None,
            reasoning_id: None,
            created_at: Utc::now(),
            duration_ms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Message,
    Reasoning,
    ToolCall,
    ToolResult,
}

// Conversion: DBMessage → praxis_llm::Message
impl TryFrom<DBMessage> for praxis_llm::Message {
    type Error = anyhow::Error;
    
    fn try_from(msg: DBMessage) -> Result<Self, Self::Error> {
        match (msg.role, msg.message_type) {
            (MessageRole::User, MessageType::Message) => {
                Ok(praxis_llm::Message::Human {
                    content: praxis_llm::Content::text(msg.content),
                    name: None,
                })
            },
            (MessageRole::Assistant, MessageType::Message) => {
                Ok(praxis_llm::Message::AI {
                    content: Some(praxis_llm::Content::text(msg.content)),
                    tool_calls: None,
                    name: None,
                })
            },
            (MessageRole::Assistant, MessageType::ToolCall) => {
                // Tool call - construct ToolCall struct
                if let (Some(tool_call_id), Some(tool_name), Some(arguments)) = 
                    (msg.tool_call_id, msg.tool_name, msg.arguments) {
                    Ok(praxis_llm::Message::AI {
                        content: None,
                        tool_calls: Some(vec![praxis_llm::ToolCall {
                            id: tool_call_id,
                            tool_type: "function".to_string(),
                            function: FunctionCall {
                                name: tool_name,
                                arguments: serde_json::to_string(&arguments)
                                    .unwrap_or_else(|_| "{}".to_string()),
                            },
                        }]),
                        name: None,
                    })
                } else {
                    Err(anyhow::anyhow!("Invalid tool call message: missing required fields"))
                }
            },
            (_, MessageType::ToolResult) => {
                // Tool result
                if let Some(tool_call_id) = msg.tool_call_id {
                    Ok(praxis_llm::Message::Tool {
                        tool_call_id,
                        content: praxis_llm::Content::text(msg.content),
                    })
                } else {
                    Err(anyhow::anyhow!("Invalid tool result message: missing tool_call_id"))
                }
            },
            // Skip reasoning messages (not sent to LLM)
            (_, MessageType::Reasoning) => {
                Err(anyhow::anyhow!("Reasoning messages are not converted to LLM messages"))
            },
            // Handle other combinations that shouldn't happen
            _ => {
                Err(anyhow::anyhow!("Invalid message role/type combination"))
            },
        }
    }
}

