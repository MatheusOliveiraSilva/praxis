use axum::{
    extract::{Path, Query, State},
    Json,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::str::FromStr;

use praxis_persist::{Message, MessageRole, MessageType};
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
    let thread_object_id = ObjectId::from_str(&thread_id)
        .map_err(|_| ApiError::BadRequest("Invalid thread ID format".to_string()))?;
    
    // Check if thread exists
    let thread = state
        .persist
        .threads()
        .get_thread(thread_object_id)
        .await?;
    
    if thread.is_none() {
        return Err(ApiError::ThreadNotFound(thread_id));
    }
    
    let limit = query.limit.min(100); // Cap at 100
    
    let before_id = if let Some(before_str) = query.before {
        Some(ObjectId::from_str(&before_str)
            .map_err(|_| ApiError::BadRequest("Invalid message ID format".to_string()))?)
    } else {
        None
    };
    
    let messages = state
        .persist
        .messages()
        .get_messages_paginated(thread_object_id, limit, before_id)
        .await?;
    
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

fn message_to_response(message: Message) -> MessageResponse {
    MessageResponse {
        message_id: message.id.to_hex(),
        thread_id: message.thread_id.to_hex(),
        role: message.role,
        message_type: message.message_type,
        content: message.content,
        created_at: message.created_at,
    }
}

