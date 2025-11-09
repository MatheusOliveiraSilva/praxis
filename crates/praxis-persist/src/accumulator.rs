use praxis_types::StreamEvent;
use std::collections::HashMap;
use std::time::Instant;

use crate::{DBMessage, MessageRole, MessageType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventType {
    Reasoning,
    Message,
    ToolCall,
}

impl EventType {
    fn from_event(event: &StreamEvent) -> Option<Self> {
        match event {
            StreamEvent::Reasoning { .. } => Some(EventType::Reasoning),
            StreamEvent::Message { .. } => Some(EventType::Message),
            StreamEvent::ToolCall { .. } => Some(EventType::ToolCall),
            _ => None,
        }
    }
}

struct ToolCallBuffer {
    tool_call_id: String,
    tool_name: String,
    arguments: String,
    started_at: Instant,
}

/// Observer that accumulates streaming events and detects type transitions
/// 
/// When event type changes (Reasoning → Message → ToolCall), 
/// the previous buffer is finalized and ready to be persisted
pub struct EventAccumulator {
    thread_id: String,
    user_id: String,
    current_type: Option<EventType>,
    
    // Buffers for different event types
    reasoning_buffer: String,
    message_buffer: String,
    tool_calls: HashMap<String, ToolCallBuffer>,
    
    // Timing tracking
    current_start: Option<Instant>,
}

impl EventAccumulator {
    pub fn new(thread_id: String, user_id: String) -> Self {
        Self {
            thread_id,
            user_id,
            current_type: None,
            reasoning_buffer: String::new(),
            message_buffer: String::new(),
            tool_calls: HashMap::new(),
            current_start: None,
        }
    }
    
    /// Push event and check for type transition (Observer Pattern)
    /// 
    /// Returns Some(DBMessage) when type changes, indicating the previous buffer is complete
    pub fn push_and_check_transition(&mut self, event: StreamEvent) -> Option<DBMessage> {
        let new_type = EventType::from_event(&event)?;
        
        // Detect transition
        let transitioned = self.current_type.map_or(false, |prev| prev != new_type);
        
        let completed_message = if transitioned {
            // Finalize previous buffer before switching
            self.finalize_current_buffer()
        } else {
            None
        };
        
        // Update state
        if self.current_type.is_none() {
            self.current_start = Some(Instant::now());
        }
        
        // If we transitioned, reset the timer
        if transitioned {
            self.current_start = Some(Instant::now());
        }
        
        self.current_type = Some(new_type);
        
        // Accumulate new event
        self.accumulate_event(event);
        
        completed_message
    }
    
    fn finalize_current_buffer(&mut self) -> Option<DBMessage> {
        let duration_ms = self.current_start
            .map(|start| start.elapsed().as_millis() as u64);
        
        let message = match self.current_type? {
            EventType::Reasoning if !self.reasoning_buffer.is_empty() => {
                Some(DBMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    thread_id: self.thread_id.clone(),
                    user_id: self.user_id.clone(),
                    role: MessageRole::Assistant,
                    message_type: MessageType::Reasoning,
                    content: std::mem::take(&mut self.reasoning_buffer),
                    tool_call_id: None,
                    tool_name: None,
                    arguments: None,
                    created_at: chrono::Utc::now(),
                    duration_ms,
                })
            },
            EventType::Message if !self.message_buffer.is_empty() => {
                Some(DBMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    thread_id: self.thread_id.clone(),
                    user_id: self.user_id.clone(),
                    role: MessageRole::Assistant,
                    message_type: MessageType::Message,
                    content: std::mem::take(&mut self.message_buffer),
                    tool_call_id: None,
                    tool_name: None,
                    arguments: None,
                    created_at: chrono::Utc::now(),
                    duration_ms,
                })
            },
            EventType::ToolCall => {
                // Finalize tool calls
                self.finalize_tool_calls()
            },
            _ => None,
        };
        
        message
    }
    
    fn accumulate_event(&mut self, event: StreamEvent) {
        match event {
            StreamEvent::Reasoning { content, .. } => {
                self.reasoning_buffer.push_str(&content);
            },
            StreamEvent::Message { content, .. } => {
                self.message_buffer.push_str(&content);
            },
            StreamEvent::ToolCall { index, id, name, arguments, .. } => {
                let tool_call_id = id.unwrap_or_else(|| format!("call_{}", index));
                
                let entry = self.tool_calls.entry(tool_call_id.clone())
                    .or_insert_with(|| ToolCallBuffer {
                        tool_call_id: tool_call_id.clone(),
                        tool_name: String::new(),
                        arguments: String::new(),
                        started_at: Instant::now(),
                    });
                
                if let Some(name) = name {
                    entry.tool_name = name;
                }
                if let Some(args) = arguments {
                    entry.arguments.push_str(&args);
                }
            },
            _ => {},
        }
    }
    
    fn finalize_tool_calls(&mut self) -> Option<DBMessage> {
        // Take the first tool call and create a message for it
        // In a real scenario, you might want to handle multiple tool calls differently
        if let Some((_, tool_call)) = self.tool_calls.drain().next() {
            let duration_ms = tool_call.started_at.elapsed().as_millis() as u64;
            
            // Parse arguments as JSON
            let arguments = serde_json::from_str(&tool_call.arguments).ok();
            
            Some(DBMessage {
                id: uuid::Uuid::new_v4().to_string(),
                thread_id: self.thread_id.clone(),
                user_id: self.user_id.clone(),
                role: MessageRole::Assistant,
                message_type: MessageType::ToolCall,
                content: String::new(),
                tool_call_id: Some(tool_call.tool_call_id),
                tool_name: Some(tool_call.tool_name),
                arguments,
                created_at: chrono::Utc::now(),
                duration_ms: Some(duration_ms),
            })
        } else {
            None
        }
    }
    
    /// Called at end of stream to finalize any remaining buffer
    pub fn finalize(&mut self) -> Option<DBMessage> {
        self.finalize_current_buffer()
    }
}

