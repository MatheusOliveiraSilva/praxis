use std::collections::HashMap;
use std::time::Instant;
use std::marker::PhantomData;

use crate::{DBMessage, MessageRole, MessageType};

/// Trait for extracting information from stream events
/// This allows EventAccumulator to work with any event type
pub trait StreamEventExtractor {
    fn is_reasoning(&self) -> bool;
    fn is_message(&self) -> bool;
    fn is_tool_call(&self) -> bool;
    
    fn reasoning_content(&self) -> Option<&str>;
    fn message_content(&self) -> Option<&str>;
    fn tool_call_info(&self) -> Option<(u32, Option<&str>, Option<&str>, Option<&str>)>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventType {
    Reasoning,
    Message,
    ToolCall,
}

impl EventType {
    fn from_event<E: StreamEventExtractor>(event: &E) -> Option<Self> {
        if event.is_reasoning() {
            Some(EventType::Reasoning)
        } else if event.is_message() {
            Some(EventType::Message)
        } else if event.is_tool_call() {
            Some(EventType::ToolCall)
        } else {
            None
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
/// 
/// Generic over event type to avoid circular dependencies
pub struct EventAccumulator<E> {
    thread_id: String,
    user_id: String,
    current_type: Option<EventType>,
    
    // Buffers for different event types
    reasoning_buffer: String,
    message_buffer: String,
    tool_calls: HashMap<String, ToolCallBuffer>,
    
    // Timing tracking
    current_start: Option<Instant>,
    
    // Phantom data to track event type
    _phantom: PhantomData<E>,
}

impl<E: StreamEventExtractor> EventAccumulator<E> {
    pub fn new(thread_id: String, user_id: String) -> Self {
        Self {
            thread_id,
            user_id,
            current_type: None,
            reasoning_buffer: String::new(),
            message_buffer: String::new(),
            tool_calls: HashMap::new(),
            current_start: None,
            _phantom: PhantomData,
        }
    }
    
    /// Push event and check for type transition (Observer Pattern)
    /// 
    /// Returns Some(DBMessage) when type changes, indicating the previous buffer is complete
    pub fn push_and_check_transition(&mut self, event: &E) -> Option<DBMessage> {
        let new_type = EventType::from_event(event)?;
        
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
                    reasoning_id: None,
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
                    reasoning_id: None,
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
    
    fn accumulate_event(&mut self, event: &E) {
        if let Some(content) = event.reasoning_content() {
            self.reasoning_buffer.push_str(content);
        }
        
        if let Some(content) = event.message_content() {
            self.message_buffer.push_str(content);
        }
        
        if let Some((index, id, name, arguments)) = event.tool_call_info() {
            let tool_call_id = id.map(String::from)
                .unwrap_or_else(|| format!("call_{}", index));
            
            let entry = self.tool_calls.entry(tool_call_id.clone())
                .or_insert_with(|| ToolCallBuffer {
                    tool_call_id: tool_call_id.clone(),
                    tool_name: String::new(),
                    arguments: String::new(),
                    started_at: Instant::now(),
                });
            
            if let Some(name) = name {
                entry.tool_name = name.to_string();
            }
            if let Some(args) = arguments {
                entry.arguments.push_str(args);
            }
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
                reasoning_id: None,
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
