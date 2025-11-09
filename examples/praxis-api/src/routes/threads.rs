use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use praxis_persist::{ThreadMetadata, Thread, ThreadSummary};
use crate::{error::{ApiError, ApiResult}, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateThreadRequest {
    pub user_id: String,
    #[serde(default)]
    pub metadata: ThreadMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadResponse {
    pub thread_id: String,
    pub user_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: ThreadMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ThreadSummaryResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreadSummaryResponse {
    pub text: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub total_tokens_before_summary: usize,
    pub messages_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct ListThreadsQuery {
    pub user_id: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct ListThreadsResponse {
    pub threads: Vec<ThreadResponse>,
    pub has_more: bool,
}

/// Create a new thread
#[utoipa::path(
    post,
    path = "/threads",
    request_body = CreateThreadRequest,
    responses(
        (status = 201, description = "Thread created", body = ThreadResponse),
        (status = 400, description = "Invalid request")
    ),
    tag = "threads"
)]
pub async fn create_thread(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateThreadRequest>,
) -> ApiResult<(StatusCode, Json<ThreadResponse>)> {
    let thread = state
        .persist
        .create_thread(&req.user_id, req.metadata)
        .await?;
    
    Ok((StatusCode::CREATED, Json(thread_to_response(thread))))
}

/// List threads for a user
#[utoipa::path(
    get,
    path = "/threads",
    params(
        ("user_id" = String, Query, description = "User ID to filter threads"),
        ("limit" = Option<i64>, Query, description = "Maximum number of threads to return (default: 20)")
    ),
    responses(
        (status = 200, description = "List of threads", body = ListThreadsResponse),
        (status = 400, description = "Invalid request")
    ),
    tag = "threads"
)]
pub async fn list_threads(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListThreadsQuery>,
) -> ApiResult<Json<ListThreadsResponse>> {
    let limit = query.limit.min(100); // Cap at 100
    
    let threads = state
        .persist
        .list_threads(&query.user_id, Some(limit), None)
        .await?;
    
    let has_more = threads.len() as i64 == limit;
    let thread_responses: Vec<ThreadResponse> = threads
        .into_iter()
        .map(thread_to_response)
        .collect();
    
    Ok(Json(ListThreadsResponse {
        threads: thread_responses,
        has_more,
    }))
}

/// Get a specific thread by ID
#[utoipa::path(
    get,
    path = "/threads/{thread_id}",
    params(
        ("thread_id" = String, Path, description = "Thread ID")
    ),
    responses(
        (status = 200, description = "Thread details", body = ThreadResponse),
        (status = 404, description = "Thread not found")
    ),
    tag = "threads"
)]
pub async fn get_thread(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
) -> ApiResult<Json<ThreadResponse>> {
    let thread = state
        .persist
        .get_thread(&thread_id)
        .await?
        .ok_or_else(|| ApiError::ThreadNotFound(thread_id))?;
    
    Ok(Json(thread_to_response(thread)))
}

/// Delete a thread
#[utoipa::path(
    delete,
    path = "/threads/{thread_id}",
    params(
        ("thread_id" = String, Path, description = "Thread ID")
    ),
    responses(
        (status = 204, description = "Thread deleted"),
        (status = 404, description = "Thread not found")
    ),
    tag = "threads"
)]
pub async fn delete_thread(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
) -> ApiResult<StatusCode> {
    // Check if thread exists
    let thread = state
        .persist
        .get_thread(&thread_id)
        .await?;
    
    if thread.is_none() {
        return Err(ApiError::ThreadNotFound(thread_id.clone()));
    }
    
    let user_id = thread.unwrap().user_id;
    
    // Delete the thread (includes deleting messages in MongoDB implementation)
    state.persist.delete_thread(&thread_id, &user_id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

fn thread_to_response(thread: Thread) -> ThreadResponse {
    ThreadResponse {
        thread_id: thread.id,
        user_id: thread.user_id,
        created_at: thread.created_at,
        updated_at: thread.updated_at,
        metadata: thread.metadata,
        summary: thread.summary.map(summary_to_response),
    }
}

fn summary_to_response(summary: ThreadSummary) -> ThreadSummaryResponse {
    ThreadSummaryResponse {
        text: summary.text,
        generated_at: summary.generated_at,
        total_tokens_before_summary: summary.total_tokens_before_summary,
        messages_count: summary.messages_count,
    }
}

