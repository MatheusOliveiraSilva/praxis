#[cfg(feature = "mongodb")]
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::{DBMessage, MessageRole, MessageType, Thread as DBThread, ThreadMetadata, ThreadSummary};

/// MongoDB-specific Message model (uses ObjectId)
#[cfg(feature = "mongodb")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoMessage {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub thread_id: ObjectId,
    pub user_id: String,
    pub role: MessageRole,
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// MongoDB-specific Thread model (uses ObjectId)
#[cfg(feature = "mongodb")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoThread {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: ThreadMetadata,
    pub last_summary_update: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ThreadSummary>,
}

// Conversions between database-agnostic and MongoDB-specific models

#[cfg(feature = "mongodb")]
impl From<DBMessage> for MongoMessage {
    fn from(msg: DBMessage) -> Self {
        // Parse thread_id as ObjectId, or create new if invalid
        let thread_id = ObjectId::parse_str(&msg.thread_id)
            .unwrap_or_else(|_| ObjectId::new());
        
        // Parse id as ObjectId, or create new if invalid
        let id = ObjectId::parse_str(&msg.id)
            .unwrap_or_else(|_| ObjectId::new());
        
        Self {
            id,
            thread_id,
            user_id: msg.user_id,
            role: msg.role,
            message_type: msg.message_type,
            content: msg.content,
            tool_call_id: msg.tool_call_id,
            tool_name: msg.tool_name,
            arguments: msg.arguments,
            created_at: msg.created_at,
            duration_ms: msg.duration_ms,
        }
    }
}

#[cfg(feature = "mongodb")]
impl From<MongoMessage> for DBMessage {
    fn from(msg: MongoMessage) -> Self {
        Self {
            id: msg.id.to_hex(),
            thread_id: msg.thread_id.to_hex(),
            user_id: msg.user_id,
            role: msg.role,
            message_type: msg.message_type,
            content: msg.content,
            tool_call_id: msg.tool_call_id,
            tool_name: msg.tool_name,
            arguments: msg.arguments,
            created_at: msg.created_at,
            duration_ms: msg.duration_ms,
        }
    }
}

#[cfg(feature = "mongodb")]
impl From<MongoThread> for DBThread {
    fn from(thread: MongoThread) -> Self {
        Self {
            id: thread.id.to_hex(),
            user_id: thread.user_id,
            created_at: thread.created_at,
            updated_at: thread.updated_at,
            metadata: thread.metadata,
            last_summary_update: thread.last_summary_update,
            summary: thread.summary,
        }
    }
}

