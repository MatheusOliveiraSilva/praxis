use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::{Stream, StreamExt};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use std::str::FromStr;
use chrono::Utc;

use praxis_types::{StreamEvent as GraphStreamEvent, GraphInput};
use praxis_llm::{Message as LLMMessage, Content};
use praxis_persist::{Message as DBMessage, MessageRole, MessageType};
use crate::{error::{ApiError, ApiResult}, state::AppState};

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub user_id: String,
    pub content: String,
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
    let thread_object_id = ObjectId::from_str(&thread_id)
        .map_err(|_| ApiError::BadRequest("Invalid thread ID format".to_string()))?;
    
    // 1. Check if thread exists
    let _thread = state
        .persist
        .threads()
        .get_thread(thread_object_id)
        .await?
        .ok_or_else(|| ApiError::ThreadNotFound(thread_id.clone()))?;
    
    // 2. Save user message to database
    let user_message = DBMessage {
        id: ObjectId::new(),
        thread_id: thread_object_id,
        user_id: req.user_id.clone(),
        role: MessageRole::User,
        message_type: MessageType::Message,
        content: req.content.clone(),
        created_at: Utc::now(),
        duration_ms: None,
    };
    
    state.persist.messages().save_message(user_message.clone()).await?;
    
    // 3. Get context window (with existing messages and summary)
    let (db_messages, system_prompt) = state
        .persist
        .context()
        .get_context_window(thread_object_id)
        .await?;
    
    // 4. Convert database messages to LLM messages for GraphInput
    let mut llm_messages = vec![
        LLMMessage::System {
            content: Content::text(system_prompt),
            name: None,
        }
    ];
    
    for msg in db_messages {
        let llm_msg = match msg.role {
            MessageRole::User => LLMMessage::Human {
                content: Content::text(msg.content),
                name: None,
            },
            MessageRole::Assistant => LLMMessage::AI {
                content: Some(Content::text(msg.content)),
                tool_calls: None,
                name: None,
            },
        };
        llm_messages.push(llm_msg);
    }
    
    // Add the new user message
    llm_messages.push(LLMMessage::Human {
        content: Content::text(req.content.clone()),
        name: None,
    });
    
    // 5. Create graph input
    let graph_input = GraphInput::new(
        thread_id.clone(),
        llm_messages.last().unwrap().clone(),
        state.config.llm.clone().into(),
    );
    
    // 6. Run graph (reuses shared Graph instance from AppState)
    let event_receiver = state.graph.spawn_run(graph_input);
    
    // 7. Clone state for async message saving
    let persist_client = Arc::clone(&state.persist);
    let user_id_clone = req.user_id.clone();
    
    // 8. Convert Receiver to Stream for SSE
    use tokio_stream::wrappers::ReceiverStream;
    let event_stream = ReceiverStream::new(event_receiver);
    
    // 9. Convert Graph events to SSE events
    let sse_stream = event_stream.map(move |event| {
        let sse_event = match event {
            GraphStreamEvent::Message { content, .. } => {
                // Save assistant message asynchronously
                let persist = Arc::clone(&persist_client);
                let user_id = user_id_clone.clone();
                let content_clone = content.clone();
                
                tokio::spawn(async move {
                    let assistant_message = DBMessage {
                        id: ObjectId::new(),
                        thread_id: thread_object_id,
                        user_id,
                        role: MessageRole::Assistant,
                        message_type: MessageType::Message,
                        content: content_clone,
                        created_at: Utc::now(),
                        duration_ms: None,
                    };
                    
                    if let Err(e) = persist.messages().save_message(assistant_message).await {
                        tracing::error!("Failed to save assistant message: {}", e);
                    }
                });
                
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

