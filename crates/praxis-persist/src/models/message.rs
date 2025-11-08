use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub thread_id: ObjectId,
    pub user_id: String,
    pub role: MessageRole,
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Message,
    Reasoning,
    ToolCall,
    ToolResult,
}

