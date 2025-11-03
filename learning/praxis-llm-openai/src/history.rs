// History reconstruction utilities
// Converts DB content_items to Message instances for LLM input

use crate::types::{Content, Message};
use serde::{Deserialize, Serialize};

/// Content item as stored in the database
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

/// Assistant message as stored in database
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

/// Reconstruct messages from DB content items
/// 
/// Key behavior:
/// - Reasoning items become separate AI messages (one per reasoning block)
/// - Message items become separate AI messages (one per message block)
/// - Tool calls/results are grouped into appropriate messages
/// 
/// This allows the LLM to see the reasoning from previous turns when generating new responses.
pub fn reconstruct_messages(content_items: Vec<ContentItem>) -> Vec<Message> {
    let mut messages = Vec::new();
    let mut current_tool_calls = Vec::new();
    
    // Sort by sequence
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
                // Flush any pending tool calls first
                if !current_tool_calls.is_empty() {
                    messages.push(Message::AI {
                        content: None,
                        tool_calls: Some(current_tool_calls.clone()),
                        name: None,
                    });
                    current_tool_calls.clear();
                }
                
                // Add reasoning as separate AI message
                messages.push(Message::AI {
                    content: Some(Content::text(content)),
                    tool_calls: None,
                    name: None,
                });
            }
            
            ContentItem::Message { content, .. } => {
                // Flush any pending tool calls first
                if !current_tool_calls.is_empty() {
                    messages.push(Message::AI {
                        content: None,
                        tool_calls: Some(current_tool_calls.clone()),
                        name: None,
                    });
                    current_tool_calls.clear();
                }
                
                // Add message as separate AI message
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
                // Accumulate tool calls
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
                // Flush pending tool calls first
                if !current_tool_calls.is_empty() {
                    messages.push(Message::AI {
                        content: None,
                        tool_calls: Some(current_tool_calls.clone()),
                        name: None,
                    });
                    current_tool_calls.clear();
                }
                
                // Add tool result message
                messages.push(Message::Tool {
                    tool_call_id,
                    content: Content::text(result),
                });
            }
        }
    }
    
    // Flush any remaining tool calls
    if !current_tool_calls.is_empty() {
        messages.push(Message::AI {
            content: None,
            tool_calls: Some(current_tool_calls),
            name: None,
        });
    }
    
    messages
}

/// Reconstruct full conversation history from database
/// 
/// Typical usage:
/// ```ignore
/// // Fetch from DB
/// let user_messages = db.get_user_messages(conversation_id);
/// let assistant_messages = db.get_assistant_messages(conversation_id);
/// 
/// // Reconstruct for LLM
/// let mut history = Vec::new();
/// for msg in user_messages {
///     history.push(Message::Human { content: msg.content, name: None });
///     
///     // Find corresponding assistant message
///     if let Some(assistant) = assistant_messages.iter().find(|a| a.follows(msg)) {
///         let reconstructed = reconstruct_messages(assistant.content_items.clone());
///         history.extend(reconstructed);
///     }
/// }
/// ```
pub fn reconstruct_conversation(
    user_messages: Vec<(String, i64)>,  // (content, timestamp)
    assistant_messages: Vec<AssistantMessage>,
) -> Vec<Message> {
    let mut history = Vec::new();
    
    // Interleave user and assistant messages by timestamp
    let mut user_idx = 0;
    let mut assistant_idx = 0;
    
    while user_idx < user_messages.len() || assistant_idx < assistant_messages.len() {
        let user_time = user_messages.get(user_idx).map(|(_, t)| *t);
        let assistant_time = assistant_messages.get(assistant_idx).map(|a| a.created_at);
        
        match (user_time, assistant_time) {
            (Some(ut), Some(at)) if ut <= at => {
                // Add user message
                let (content, _) = &user_messages[user_idx];
                history.push(Message::Human {
                    content: Content::text(content.clone()),
                    name: None,
                });
                user_idx += 1;
            }
            (Some(_), Some(_)) | (None, Some(_)) => {
                // Add assistant message(s)
                let assistant = &assistant_messages[assistant_idx];
                let reconstructed = reconstruct_messages(assistant.content_items.clone());
                history.extend(reconstructed);
                assistant_idx += 1;
            }
            (Some(_), None) => {
                // Add remaining user message
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reconstruct_reasoning_and_message() {
        let items = vec![
            ContentItem::Reasoning {
                content: "Let me think about this...".to_string(),
                sequence: 0,
                timestamp: None,
            },
            ContentItem::Message {
                content: "The answer is 42.".to_string(),
                sequence: 1,
                timestamp: None,
            },
        ];
        
        let messages = reconstruct_messages(items);
        
        assert_eq!(messages.len(), 2);
        
        // First message should be reasoning
        if let Message::AI { content, tool_calls, .. } = &messages[0] {
            assert_eq!(content.as_ref().unwrap().as_text(), Some("Let me think about this..."));
            assert!(tool_calls.is_none());
        } else {
            panic!("Expected AI message");
        }
        
        // Second message should be message
        if let Message::AI { content, tool_calls, .. } = &messages[1] {
            assert_eq!(content.as_ref().unwrap().as_text(), Some("The answer is 42."));
            assert!(tool_calls.is_none());
        } else {
            panic!("Expected AI message");
        }
    }
    
    #[test]
    fn test_reconstruct_with_tool_calls() {
        let items = vec![
            ContentItem::ToolCall {
                tool_call_id: "call_1".to_string(),
                tool_name: "calculator".to_string(),
                arguments: r#"{"expr": "2+2"}"#.to_string(),
                sequence: 0,
                timestamp: None,
            },
            ContentItem::ToolResult {
                tool_call_id: "call_1".to_string(),
                result: "4".to_string(),
                is_error: false,
                sequence: 1,
                duration_ms: Some(10),
            },
            ContentItem::Message {
                content: "The result is 4.".to_string(),
                sequence: 2,
                timestamp: None,
            },
        ];
        
        let messages = reconstruct_messages(items);
        
        assert_eq!(messages.len(), 3);
        
        // First: tool call
        if let Message::AI { tool_calls, .. } = &messages[0] {
            assert!(tool_calls.is_some());
        } else {
            panic!("Expected AI message with tool calls");
        }
        
        // Second: tool result
        if let Message::Tool { tool_call_id, .. } = &messages[1] {
            assert_eq!(tool_call_id, "call_1");
        } else {
            panic!("Expected Tool message");
        }
        
        // Third: final message
        if let Message::AI { content, .. } = &messages[2] {
            assert_eq!(content.as_ref().unwrap().as_text(), Some("The result is 4."));
        } else {
            panic!("Expected AI message");
        }
    }
}

