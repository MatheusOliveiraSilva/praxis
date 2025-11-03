use crate::types::{Content, Message};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentItem {
    Reasoning {
        content: String,
        sequence: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<i64>,
    },
    
    Message {
        content: String,
        sequence: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<i64>,
    },
    
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        arguments: String,
        sequence: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<i64>,
    },
    
    ToolResult {
        tool_call_id: String,
        result: String,
        is_error: bool,
        sequence: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub run_id: String,
    pub content_items: Vec<ContentItem>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u32>,
    #[serde(default)]
    pub incomplete: bool,
}

pub fn reconstruct_messages(content_items: Vec<ContentItem>) -> Vec<Message> {
    let mut messages = Vec::new();
    let mut current_tool_calls = Vec::new();
    
    let mut items = content_items;
    items.sort_by_key(|item| match item {
        ContentItem::Reasoning { sequence, .. } => *sequence,
        ContentItem::Message { sequence, .. } => *sequence,
        ContentItem::ToolCall { sequence, .. } => *sequence,
        ContentItem::ToolResult { sequence, .. } => *sequence,
    });
    
    for item in items {
        match item {
            ContentItem::Reasoning { content, .. } => {
                if !current_tool_calls.is_empty() {
                    messages.push(Message::AI {
                        content: None,
                        tool_calls: Some(current_tool_calls.clone()),
                        name: None,
                    });
                    current_tool_calls.clear();
                }
                
                messages.push(Message::AI {
                    content: Some(Content::text(content)),
                    tool_calls: None,
                    name: None,
                });
            }
            
            ContentItem::Message { content, .. } => {
                if !current_tool_calls.is_empty() {
                    messages.push(Message::AI {
                        content: None,
                        tool_calls: Some(current_tool_calls.clone()),
                        name: None,
                    });
                    current_tool_calls.clear();
                }
                
                messages.push(Message::AI {
                    content: Some(Content::text(content)),
                    tool_calls: None,
                    name: None,
                });
            }
            
            ContentItem::ToolCall {
                tool_call_id,
                tool_name,
                arguments,
                ..
            } => {
                current_tool_calls.push(crate::types::ToolCall {
                    id: tool_call_id,
                    tool_type: "function".to_string(),
                    function: crate::types::FunctionCall {
                        name: tool_name,
                        arguments,
                    },
                });
            }
            
            ContentItem::ToolResult {
                tool_call_id,
                result,
                ..
            } => {
                if !current_tool_calls.is_empty() {
                    messages.push(Message::AI {
                        content: None,
                        tool_calls: Some(current_tool_calls.clone()),
                        name: None,
                    });
                    current_tool_calls.clear();
                }
                
                messages.push(Message::Tool {
                    tool_call_id,
                    content: Content::text(result),
                });
            }
        }
    }
    
    if !current_tool_calls.is_empty() {
        messages.push(Message::AI {
            content: None,
            tool_calls: Some(current_tool_calls),
            name: None,
        });
    }
    
    messages
}

pub fn reconstruct_conversation(
    user_messages: Vec<(String, i64)>,
    assistant_messages: Vec<AssistantMessage>,
) -> Vec<Message> {
    let mut history = Vec::new();
    let mut user_idx = 0;
    let mut assistant_idx = 0;
    
    while user_idx < user_messages.len() || assistant_idx < assistant_messages.len() {
        let user_time = user_messages.get(user_idx).map(|(_, t)| *t);
        let assistant_time = assistant_messages.get(assistant_idx).map(|a| a.created_at);
        
        match (user_time, assistant_time) {
            (Some(ut), Some(at)) if ut <= at => {
                let (content, _) = &user_messages[user_idx];
                history.push(Message::Human {
                    content: Content::text(content.clone()),
                    name: None,
                });
                user_idx += 1;
            }
            (Some(_), Some(_)) | (None, Some(_)) => {
                let assistant = &assistant_messages[assistant_idx];
                let reconstructed = reconstruct_messages(assistant.content_items.clone());
                history.extend(reconstructed);
                assistant_idx += 1;
            }
            (Some(_), None) => {
                let (content, _) = &user_messages[user_idx];
                history.push(Message::Human {
                    content: Content::text(content.clone()),
                    name: None,
                });
                user_idx += 1;
            }
            (None, None) => break,
        }
    }
    
    history
}

