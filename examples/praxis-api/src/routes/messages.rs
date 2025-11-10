use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use praxis::{DBMessage, MessageRole, MessageType};
use crate::{error::{ApiError, ApiResult}, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub message_id: String,
    pub thread_id: String,
    pub role: MessageRole,
    pub message_type: MessageType,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub before: Option<String>,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize)]
pub struct ListMessagesResponse {
    pub messages: Vec<MessageResponse>,
    pub has_more: bool,
}

/// List messages in a thread
#[utoipa::path(
    get,
    path = "/threads/{thread_id}/messages",
    params(
        ("thread_id" = String, Path, description = "Thread ID"),
        ("limit" = Option<i64>, Query, description = "Maximum number of messages (default: 50)"),
        ("before" = Option<String>, Query, description = "Get messages before this message ID")
    ),
    responses(
        (status = 200, description = "List of messages", body = ListMessagesResponse),
        (status = 404, description = "Thread not found")
    ),
    tag = "messages"
)]
pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> ApiResult<Json<ListMessagesResponse>> {
    // Check if thread exists
    let thread = state
        .persist
        .get_thread(&thread_id)
        .await?;
    
    if thread.is_none() {
        return Err(ApiError::ThreadNotFound(thread_id));
    }
    
    let limit = query.limit.min(100); // Cap at 100
    
    // Get all messages for the thread (PersistenceClient doesn't have pagination yet)
    // TODO: Add pagination support to PersistenceClient trait
    let all_messages = state
        .persist
        .get_messages(&thread_id)
        .await?;
    
    // Simple pagination: if before is specified, filter messages before that ID
    let messages: Vec<DBMessage> = if let Some(before_str) = query.before {
        let before_idx = all_messages.iter()
            .position(|m| m.id == before_str)
            .unwrap_or(all_messages.len());
        all_messages.into_iter()
            .take(before_idx)
            .take(limit as usize)
            .collect()
    } else {
        all_messages.into_iter()
            .take(limit as usize)
            .collect()
    };
    
    let has_more = messages.len() as i64 == limit;
    let message_responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(message_to_response)
        .collect();
    
    Ok(Json(ListMessagesResponse {
        messages: message_responses,
        has_more,
    }))
}

fn message_to_response(message: DBMessage) -> MessageResponse {
    MessageResponse {
        message_id: message.id,
        thread_id: message.thread_id,
        role: message.role,
        message_type: message.message_type,
        content: message.content,
        created_at: message.created_at,
    }
}

