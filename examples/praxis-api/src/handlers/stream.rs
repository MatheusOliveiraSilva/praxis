use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::{Stream, StreamExt};
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use chrono::Utc;

use tokio_stream::wrappers::ReceiverStream;
use praxis::{StreamEvent as GraphStreamEvent, GraphInput, Message as LLMMessage, Content, DBMessage, MessageRole, MessageType, PersistenceContext, LLMConfig};
use crate::{error::{ApiError, ApiResult}, state::AppState};

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub user_id: String,
    pub content: String,
    pub llm_config: RequestLLMConfig,
}

/// LLM configuration sent per request
#[derive(Debug, Clone, Deserialize)]
pub struct RequestLLMConfig {
    pub model: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    8000
}

/// Send a message and stream the response using Server-Sent Events
#[utoipa::path(
    post,
    path = "/threads/{thread_id}/messages",
    request_body = SendMessageRequest,
    responses(
        (status = 200, description = "Streaming response", content_type = "text/event-stream"),
        (status = 404, description = "Thread not found")
    ),
    tag = "messages"
)]
pub async fn send_message_stream(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // 1. Check if thread exists
    let _thread = state
        .persist
        .get_thread(&thread_id)
        .await?
        .ok_or_else(|| ApiError::ThreadNotFound(thread_id.clone()))?;
    
    // 2. Save user message to database
    let user_message = DBMessage {
        id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread_id.clone(),
        user_id: req.user_id.clone(),
        role: MessageRole::User,
        message_type: MessageType::Message,
        content: req.content.clone(),
        tool_call_id: None,
        tool_name: None,
        arguments: None,
        reasoning_id: None,
        created_at: Utc::now(),
        duration_ms: None,
    };
    
    state.persist.save_message(user_message).await?;
    
    // 3. Get context using strategy (BEFORE Graph execution)
    let context_window = state.context_strategy
        .get_context_window(&thread_id, Arc::clone(&state.persist))
        .await?;
    
    // 4. Build full message history
    let mut messages = vec![
        LLMMessage::System {
            content: Content::text(context_window.system_prompt),
            name: None,
        }
    ];
    messages.extend(context_window.messages);
    messages.push(LLMMessage::Human {
        content: Content::text(req.content.clone()),
        name: None,
    });
    
    // 5. Create GraphInput with dynamic LLM config from request
    let llm_config = LLMConfig {
        model: req.llm_config.model.clone(),
        provider: praxis::Provider::OpenAI,
        temperature: Some(req.llm_config.temperature),
        max_tokens: Some(req.llm_config.max_tokens),
        reasoning_effort: req.llm_config.reasoning_effort.clone(),
    };
    
    let graph_input = GraphInput::new(
        thread_id.clone(),
        messages,
        llm_config,
    );
    
    // 6. Spawn Graph with PersistenceContext
    let event_receiver = state.graph.spawn_run(
        graph_input,
        Some(PersistenceContext {
            thread_id: thread_id.clone(),
            user_id: req.user_id.clone(),
        }),
    );
    
    // 7. Convert Receiver to Stream for SSE
    let event_stream = ReceiverStream::new(event_receiver);
    
    // 8. Convert Graph events to SSE events (Graph handles persistence automatically)
    let sse_stream = event_stream.map(move |event| {
        let sse_event = match event {
            GraphStreamEvent::Message { content, .. } => {
                Event::default()
                    .event("message")
                    .json_data(serde_json::json!({
                        "content": content
                    }))
            },
            GraphStreamEvent::ToolCall { name, arguments, .. } => {
                Event::default()
                    .event("tool_call")
                    .json_data(serde_json::json!({
                        "name": name,
                        "arguments": arguments
                    }))
            },
            GraphStreamEvent::ToolResult { result, .. } => {
                Event::default()
                    .event("tool_result")
                    .json_data(serde_json::json!({
                        "result": result
                    }))
            },
            GraphStreamEvent::Reasoning { content, .. } => {
                Event::default()
                    .event("reasoning")
                    .json_data(serde_json::json!({
                        "content": content
                    }))
            },
            GraphStreamEvent::Done { .. } => {
                Event::default()
                    .event("done")
                    .json_data(serde_json::json!({
                        "status": "completed"
                    }))
            },
            GraphStreamEvent::Error { message, .. } => {
                Event::default()
                    .event("error")
                    .json_data(serde_json::json!({
                        "error": message
                    }))
            },
            _ => {
                // Handle other event types (InitStream, EndStream)
                Event::default()
                    .event("info")
                    .json_data(serde_json::json!({}))
            },
        };
        
        Ok::<Event, Infallible>(sse_event.unwrap())
    });
    
    Ok(Sse::new(sse_stream))
}

