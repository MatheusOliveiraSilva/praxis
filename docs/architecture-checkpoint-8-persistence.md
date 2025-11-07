# Architecture Checkpoint 8: Persistence Layer (praxis-persist)

**Status**: 📋 Ready for Implementation  
**Date**: 2025-11-07  
**Phase**: Database Integration & Context Management

---

## Overview

The **praxis-persist** crate provides MongoDB integration for conversation persistence, message storage, and intelligent context management with automatic summarization.

### Goals

1. **Durable storage**: Threads and messages persisted to MongoDB
2. **Fast retrieval**: Efficient queries for conversation history
3. **Smart context**: Token-aware context windows with auto-summarization
4. **Scalable**: Indexed for high-throughput reads/writes
5. **Observable**: Track runs, tool usage, token consumption

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      praxis-api (HTTP Layer)                │
│                                                             │
│  - Receives user message                                    │
│  - Calls persist.get_thread_context()                       │
│  - Spawns Graph execution                                   │
│  - After completion, calls persist.save_run()               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                   praxis-persist (This Crate)               │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  ThreadRepository                                    │  │
│  │  - create_thread()                                   │  │
│  │  - get_thread()                                      │  │
│  │  - list_threads(user_id)                             │  │
│  │  - update_summary()                                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  MessageRepository                                   │  │
│  │  - save_message()                                    │  │
│  │  - get_messages(thread_id)                           │  │
│  │  - get_messages_paginated()                          │  │
│  │  - count_messages(thread_id)                         │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  RunRepository                                       │  │
│  │  - save_run()                                        │  │
│  │  - get_run(run_id)                                   │  │
│  │  - list_runs(thread_id)                              │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  ContextManager                                      │  │
│  │  - get_context_window(thread_id, max_tokens)         │  │
│  │  - trigger_summarization_if_needed()                 │  │
│  │  - generate_summary()                                │  │
│  │  - inject_summary_to_system_prompt()                 │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  MessageAccumulator                                  │  │
│  │  - accumulate(StreamEvent)                           │  │
│  │  - finalize() -> Vec<Message>                        │  │
│  │  - Converts streaming events to DB messages          │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      MongoDB                                │
│                                                             │
│  Collections:                                               │
│  - threads        (conversation metadata + summaries)       │
│  - messages       (user/assistant messages with types)      │
│  - runs           (execution metadata, tokens, duration)    │
└─────────────────────────────────────────────────────────────┘
```

---

## MongoDB Schema

### Collection: `threads`

Represents a conversation thread.

```javascript
{
  "_id": ObjectId("..."),
  "user_id": "user_123",
  "created_at": ISODate("2025-11-07T10:30:00Z"),
  "updated_at": ISODate("2025-11-07T10:35:00Z"),
  "metadata": {
    "title": "Weather inquiry",
    "tags": ["weather", "san-francisco"]
  },
  
  // Summary for context management
  "summary": {
    "text": "User asked about weather in San Francisco. Assistant provided forecast using get_forecast tool.",
    "generated_at": ISODate("2025-11-07T10:33:00Z"),
    "total_tokens_before_summary": 5000,
    "messages_count": 8
  }
}
```

**Indexes:**
```javascript
db.threads.createIndex({ "user_id": 1, "created_at": -1 });
db.threads.createIndex({ "_id": 1, "user_id": 1 });  // Composite for security
```

---

### Collection: `messages`

All messages in a thread (user, assistant, reasoning, tools).

```javascript
// User message
{
  "_id": ObjectId("..."),
  "thread_id": ObjectId("..."),
  "user_id": "user_123",
  "role": "user",          // "user" | "assistant"
  "type": "message",       // "message" | "reasoning" | "tool_call" | "tool_result"
  "content": "What's the weather in San Francisco?",
  "created_at": ISODate("2025-11-07T10:30:00Z"),
  "run_id": "run_abc123"   // Links to runs collection
}

// Assistant reasoning (streaming)
{
  "_id": ObjectId("..."),
  "thread_id": ObjectId("..."),
  "user_id": "user_123",
  "role": "assistant",
  "type": "reasoning",
  "content": "I need to check the weather forecast for San Francisco...",
  "created_at": ISODate("2025-11-07T10:30:01Z"),
  "duration_ms": 500,      // Time spent reasoning
  "run_id": "run_abc123"
}

// Tool call
{
  "_id": ObjectId("..."),
  "thread_id": ObjectId("..."),
  "user_id": "user_123",
  "role": "assistant",
  "type": "tool_call",
  "tool_call_id": "call_1",
  "tool_name": "get_forecast",
  "arguments": {
    "location": "San Francisco, CA"
  },
  "created_at": ISODate("2025-11-07T10:30:02Z"),
  "run_id": "run_abc123"
}

// Tool result
{
  "_id": ObjectId("..."),
  "thread_id": ObjectId("..."),
  "user_id": "user_123",
  "role": "assistant",
  "type": "tool_result",
  "tool_call_id": "call_1",
  "result": "Sunny, 72°F with light winds",
  "is_error": false,
  "duration_ms": 250,
  "created_at": ISODate("2025-11-07T10:30:03Z"),
  "run_id": "run_abc123"
}

// Assistant final message
{
  "_id": ObjectId("..."),
  "thread_id": ObjectId("..."),
  "user_id": "user_123",
  "role": "assistant",
  "type": "message",
  "content": "The weather in San Francisco is sunny with a temperature of 72°F.",
  "created_at": ISODate("2025-11-07T10:30:04Z"),
  "duration_ms": 1200,     // Total response time
  "run_id": "run_abc123"
}
```

**Indexes:**
```javascript
db.messages.createIndex({ "thread_id": 1, "created_at": 1 });
db.messages.createIndex({ "user_id": 1, "created_at": -1 });
db.messages.createIndex({ "run_id": 1 });
db.messages.createIndex({ "type": 1, "created_at": -1 });
```

---

### Collection: `runs`

Execution metadata for each Graph run.

```javascript
{
  "_id": "run_abc123",     // String ID (UUID)
  "thread_id": ObjectId("..."),
  "user_id": "user_123",
  "status": "success",     // "success" | "error" | "cancelled"
  "started_at": ISODate("2025-11-07T10:30:00Z"),
  "completed_at": ISODate("2025-11-07T10:30:04Z"),
  "duration_ms": 4000,
  
  // Token usage
  "tokens": {
    "prompt_tokens": 150,
    "completion_tokens": 80,
    "reasoning_tokens": 45,
    "total_tokens": 275
  },
  
  // LLM config used
  "llm_config": {
    "model": "gpt-4",
    "temperature": 0.7,
    "max_tokens": 4096
  },
  
  // Tools called during this run
  "tools_used": [
    {
      "name": "get_forecast",
      "duration_ms": 250,
      "success": true
    }
  ],
  
  // Optional: store events for debugging
  "events": [
    // Array of StreamEvent (optional)
  ],
  
  // Error details (if status = "error")
  "error": {
    "message": "LLM API error",
    "code": "LLM_ERROR"
  }
}
```

**Indexes:**
```javascript
db.runs.createIndex({ "thread_id": 1, "started_at": -1 });
db.runs.createIndex({ "user_id": 1, "started_at": -1 });
db.runs.createIndex({ "status": 1, "started_at": -1 });
```

---

## Repository Interfaces

### ThreadRepository

```rust
use mongodb::{Client, Collection, bson::oid::ObjectId};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: ThreadMetadata,
    pub summary: Option<ThreadSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMetadata {
    pub title: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSummary {
    pub text: String,
    pub generated_at: DateTime<Utc>,
    pub total_tokens_before_summary: usize,
    pub messages_count: usize,
}

pub struct ThreadRepository {
    collection: Collection<Thread>,
}

impl ThreadRepository {
    pub fn new(client: &Client, db_name: &str) -> Self {
        let collection = client.database(db_name).collection("threads");
        Self { collection }
    }
    
    /// Create a new thread
    pub async fn create_thread(
        &self,
        user_id: String,
        metadata: ThreadMetadata,
    ) -> Result<Thread> {
        let thread = Thread {
            id: ObjectId::new(),
            user_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata,
            summary: None,
        };
        
        self.collection.insert_one(&thread, None).await?;
        Ok(thread)
    }
    
    /// Get thread by ID
    pub async fn get_thread(&self, thread_id: ObjectId) -> Result<Option<Thread>> {
        let filter = doc! { "_id": thread_id };
        Ok(self.collection.find_one(filter, None).await?)
    }
    
    /// List threads for a user
    pub async fn list_threads(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<Thread>> {
        let filter = doc! { "user_id": user_id };
        let options = FindOptions::builder()
            .sort(doc! { "updated_at": -1 })
            .limit(limit)
            .build();
        
        let cursor = self.collection.find(filter, options).await?;
        cursor.try_collect().await
    }
    
    /// Update thread summary
    pub async fn update_summary(
        &self,
        thread_id: ObjectId,
        summary: ThreadSummary,
    ) -> Result<()> {
        let filter = doc! { "_id": thread_id };
        let update = doc! {
            "$set": {
                "summary": bson::to_bson(&summary)?,
                "updated_at": Utc::now()
            }
        };
        
        self.collection.update_one(filter, update, None).await?;
        Ok(())
    }
    
    /// Touch thread (update updated_at)
    pub async fn touch_thread(&self, thread_id: ObjectId) -> Result<()> {
        let filter = doc! { "_id": thread_id };
        let update = doc! { "$set": { "updated_at": Utc::now() } };
        self.collection.update_one(filter, update, None).await?;
        Ok(())
    }
}
```

---

### MessageRepository

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub thread_id: ObjectId,
    pub user_id: String,
    pub role: MessageRole,
    #[serde(rename = "type")]
    pub message_type: MessageType,
    #[serde(flatten)]
    pub data: MessageData,
    pub created_at: DateTime<Utc>,
    pub run_id: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Message,
    Reasoning,
    ToolCall,
    ToolResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageData {
    Text {
        content: String,
    },
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        tool_call_id: String,
        result: String,
        is_error: bool,
    },
}

pub struct MessageRepository {
    collection: Collection<Message>,
}

impl MessageRepository {
    pub fn new(client: &Client, db_name: &str) -> Self {
        let collection = client.database(db_name).collection("messages");
        Self { collection }
    }
    
    /// Save a single message
    pub async fn save_message(&self, message: Message) -> Result<ObjectId> {
        self.collection.insert_one(&message, None).await?;
        Ok(message.id)
    }
    
    /// Save multiple messages (batch)
    pub async fn save_messages(&self, messages: Vec<Message>) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }
        self.collection.insert_many(messages, None).await?;
        Ok(())
    }
    
    /// Get all messages for a thread
    pub async fn get_messages(&self, thread_id: ObjectId) -> Result<Vec<Message>> {
        let filter = doc! { "thread_id": thread_id };
        let options = FindOptions::builder()
            .sort(doc! { "created_at": 1 })
            .build();
        
        let cursor = self.collection.find(filter, options).await?;
        cursor.try_collect().await
    }
    
    /// Get messages with pagination
    pub async fn get_messages_paginated(
        &self,
        thread_id: ObjectId,
        limit: i64,
        before: Option<ObjectId>,
    ) -> Result<Vec<Message>> {
        let mut filter = doc! { "thread_id": thread_id };
        if let Some(before_id) = before {
            filter.insert("_id", doc! { "$lt": before_id });
        }
        
        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .build();
        
        let cursor = self.collection.find(filter, options).await?;
        let mut messages: Vec<Message> = cursor.try_collect().await?;
        messages.reverse(); // Return in chronological order
        Ok(messages)
    }
    
    /// Count messages in a thread
    pub async fn count_messages(&self, thread_id: ObjectId) -> Result<u64> {
        let filter = doc! { "thread_id": thread_id };
        self.collection.count_documents(filter, None).await
    }
    
    /// Get messages by run_id
    pub async fn get_messages_by_run(&self, run_id: &str) -> Result<Vec<Message>> {
        let filter = doc! { "run_id": run_id };
        let options = FindOptions::builder()
            .sort(doc! { "created_at": 1 })
            .build();
        
        let cursor = self.collection.find(filter, options).await?;
        cursor.try_collect().await
    }
}
```

---

### RunRepository

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    #[serde(rename = "_id")]
    pub id: String,  // UUID
    pub thread_id: ObjectId,
    pub user_id: String,
    pub status: RunStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub tokens: Option<TokenUsage>,
    pub llm_config: LLMConfig,
    pub tools_used: Vec<ToolUsage>,
    pub events: Option<Vec<serde_json::Value>>,  // Optional: store events
    pub error: Option<RunError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Success,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub reasoning_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsage {
    pub name: String,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunError {
    pub message: String,
    pub code: String,
}

pub struct RunRepository {
    collection: Collection<Run>,
}

impl RunRepository {
    pub fn new(client: &Client, db_name: &str) -> Self {
        let collection = client.database(db_name).collection("runs");
        Self { collection }
    }
    
    /// Create a new run (start tracking)
    pub async fn create_run(
        &self,
        run_id: String,
        thread_id: ObjectId,
        user_id: String,
        llm_config: LLMConfig,
    ) -> Result<()> {
        let run = Run {
            id: run_id,
            thread_id,
            user_id,
            status: RunStatus::Success,  // Will update on completion
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            tokens: None,
            llm_config,
            tools_used: vec![],
            events: None,
            error: None,
        };
        
        self.collection.insert_one(&run, None).await?;
        Ok(())
    }
    
    /// Complete a run (success)
    pub async fn complete_run(
        &self,
        run_id: &str,
        duration_ms: u64,
        tokens: Option<TokenUsage>,
        tools_used: Vec<ToolUsage>,
    ) -> Result<()> {
        let filter = doc! { "_id": run_id };
        let update = doc! {
            "$set": {
                "status": "success",
                "completed_at": Utc::now(),
                "duration_ms": duration_ms as i64,
                "tokens": bson::to_bson(&tokens)?,
                "tools_used": bson::to_bson(&tools_used)?,
            }
        };
        
        self.collection.update_one(filter, update, None).await?;
        Ok(())
    }
    
    /// Fail a run (error)
    pub async fn fail_run(
        &self,
        run_id: &str,
        error: RunError,
    ) -> Result<()> {
        let filter = doc! { "_id": run_id };
        let update = doc! {
            "$set": {
                "status": "error",
                "completed_at": Utc::now(),
                "error": bson::to_bson(&error)?,
            }
        };
        
        self.collection.update_one(filter, update, None).await?;
        Ok(())
    }
    
    /// Get run by ID
    pub async fn get_run(&self, run_id: &str) -> Result<Option<Run>> {
        let filter = doc! { "_id": run_id };
        Ok(self.collection.find_one(filter, None).await?)
    }
    
    /// List runs for a thread
    pub async fn list_runs(&self, thread_id: ObjectId) -> Result<Vec<Run>> {
        let filter = doc! { "thread_id": thread_id };
        let options = FindOptions::builder()
            .sort(doc! { "started_at": -1 })
            .build();
        
        let cursor = self.collection.find(filter, options).await?;
        cursor.try_collect().await
    }
}
```

---

## Context Management

### ContextManager

Intelligent context window management with automatic summarization.

```rust
use tiktoken_rs::cl100k_base;  // OpenAI tokenizer

pub struct ContextManager {
    message_repo: MessageRepository,
    thread_repo: ThreadRepository,
    llm_client: Arc<dyn LLMClient>,
    max_tokens: usize,  // e.g., 8000 tokens
}

impl ContextManager {
    pub fn new(
        message_repo: MessageRepository,
        thread_repo: ThreadRepository,
        llm_client: Arc<dyn LLMClient>,
        max_tokens: usize,
    ) -> Self {
        Self {
            message_repo,
            thread_repo,
            llm_client,
            max_tokens,
        }
    }
    
    /// Get context window for a thread
    /// Returns: (messages, system_prompt_with_summary)
    pub async fn get_context_window(
        &self,
        thread_id: ObjectId,
    ) -> Result<(Vec<praxis_llm::Message>, String)> {
        // 1. Get thread (check for existing summary)
        let thread = self.thread_repo.get_thread(thread_id).await?
            .ok_or_else(|| anyhow::anyhow!("Thread not found"))?;
        
        // 2. Get all messages
        let messages = self.message_repo.get_messages(thread_id).await?;
        
        // 3. Count total tokens
        let total_tokens = self.count_tokens(&messages)?;
        
        // 4. If under limit, return all messages
        if total_tokens <= self.max_tokens {
            let llm_messages = self.convert_to_llm_messages(messages);
            let system_prompt = self.build_system_prompt(thread.summary);
            return Ok((llm_messages, system_prompt));
        }
        
        // 5. Over limit - trigger summarization if not already done
        let summary = if thread.summary.is_none() {
            // Generate summary
            let summary = self.generate_summary(&messages).await?;
            self.thread_repo.update_summary(thread_id, summary.clone()).await?;
            summary
        } else {
            thread.summary.unwrap()
        };
        
        // 6. Keep only recent messages after summary
        let messages_after_summary = self.get_messages_after_summary(&messages, &summary);
        
        // 7. Build system prompt with summary
        let system_prompt = self.build_system_prompt(Some(summary));
        let llm_messages = self.convert_to_llm_messages(messages_after_summary);
        
        Ok((llm_messages, system_prompt))
    }
    
    /// Count tokens in messages
    fn count_tokens(&self, messages: &[Message]) -> Result<usize> {
        let bpe = cl100k_base().map_err(|e| anyhow::anyhow!("Tokenizer error: {}", e))?;
        
        let mut total_tokens = 0;
        for msg in messages {
            let text = match &msg.data {
                MessageData::Text { content } => content.clone(),
                MessageData::ToolCall { tool_name, arguments, .. } => {
                    format!("{}: {}", tool_name, arguments)
                }
                MessageData::ToolResult { result, .. } => result.clone(),
            };
            
            let tokens = bpe.encode_with_special_tokens(&text);
            total_tokens += tokens.len();
        }
        
        Ok(total_tokens)
    }
    
    /// Generate summary of messages
    async fn generate_summary(&self, messages: &[Message]) -> Result<ThreadSummary> {
        // Build conversation text
        let conversation = messages.iter()
            .filter(|m| matches!(m.message_type, MessageType::Message))
            .map(|m| {
                let content = match &m.data {
                    MessageData::Text { content } => content,
                    _ => "",
                };
                format!("{}: {}", m.role, content)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        // Summary prompt
        let summary_prompt = format!(
            "Summarize the following conversation concisely:\n\n{}\n\nSummary:",
            conversation
        );
        
        // Call LLM to generate summary
        let request = ChatRequest::new(
            "gpt-4".to_string(),
            vec![praxis_llm::Message::Human {
                content: praxis_llm::Content::text(summary_prompt),
                name: None,
            }],
        );
        
        // Note: Use non-streaming version for summary
        let response = self.llm_client.chat_completion(request).await?;
        let summary_text = response.content;
        
        let total_tokens = self.count_tokens(messages)?;
        
        Ok(ThreadSummary {
            text: summary_text,
            generated_at: Utc::now(),
            total_tokens_before_summary: total_tokens,
            messages_count: messages.len(),
        })
    }
    
    /// Get messages after summary was generated
    fn get_messages_after_summary(
        &self,
        messages: &[Message],
        summary: &ThreadSummary,
    ) -> Vec<Message> {
        messages.iter()
            .filter(|m| m.created_at > summary.generated_at)
            .cloned()
            .collect()
    }
    
    /// Build system prompt with optional summary
    fn build_system_prompt(&self, summary: Option<ThreadSummary>) -> String {
        let base_prompt = "You are a helpful AI assistant.";
        
        if let Some(summary) = summary {
            format!(
                "{}\n\nConversation summary (previous messages):\n{}",
                base_prompt,
                summary.text
            )
        } else {
            base_prompt.to_string()
        }
    }
    
    /// Convert DB messages to LLM messages
    fn convert_to_llm_messages(&self, messages: Vec<Message>) -> Vec<praxis_llm::Message> {
        messages.into_iter()
            .filter_map(|msg| {
                match msg.message_type {
                    MessageType::Message => {
                        let content = match msg.data {
                            MessageData::Text { content } => content,
                            _ => return None,
                        };
                        
                        Some(match msg.role {
                            MessageRole::User => praxis_llm::Message::Human {
                                content: praxis_llm::Content::text(content),
                                name: None,
                            },
                            MessageRole::Assistant => praxis_llm::Message::AI {
                                content: Some(content),
                                tool_calls: None,
                                name: None,
                            },
                        })
                    }
                    _ => None,  // Skip reasoning, tool calls for now
                }
            })
            .collect()
    }
}
```

---

## MessageAccumulator

Accumulates streaming events and converts them into database messages.

```rust
use praxis_types::StreamEvent;
use std::collections::HashMap;

pub struct MessageAccumulator {
    run_id: String,
    thread_id: ObjectId,
    user_id: String,
    
    // Buffers
    reasoning_buffer: String,
    message_buffer: String,
    tool_calls: HashMap<String, ToolCallBuffer>,
    
    // Timestamps
    reasoning_start: Option<Instant>,
    message_start: Option<Instant>,
    
    // Finalized messages
    messages: Vec<Message>,
}

struct ToolCallBuffer {
    tool_call_id: String,
    tool_name: String,
    arguments: String,
    result: Option<(String, bool)>,  // (result, is_error)
    call_time: Instant,
    result_time: Option<Instant>,
}

impl MessageAccumulator {
    pub fn new(run_id: String, thread_id: ObjectId, user_id: String) -> Self {
        Self {
            run_id,
            thread_id,
            user_id,
            reasoning_buffer: String::new(),
            message_buffer: String::new(),
            tool_calls: HashMap::new(),
            reasoning_start: None,
            message_start: None,
            messages: vec![],
        }
    }
    
    /// Accumulate a streaming event
    pub fn accumulate(&mut self, event: StreamEvent) {
        match event {
            StreamEvent::Reasoning { content } => {
                if self.reasoning_start.is_none() {
                    self.reasoning_start = Some(Instant::now());
                }
                self.reasoning_buffer.push_str(&content);
            }
            
            StreamEvent::Message { content } => {
                if self.message_start.is_none() {
                    self.message_start = Some(Instant::now());
                }
                self.message_buffer.push_str(&content);
            }
            
            StreamEvent::ToolCall { index, id, name, arguments } => {
                let tool_call_id = id.unwrap_or_else(|| format!("call_{}", index));
                let entry = self.tool_calls.entry(tool_call_id.clone())
                    .or_insert_with(|| ToolCallBuffer {
                        tool_call_id: tool_call_id.clone(),
                        tool_name: String::new(),
                        arguments: String::new(),
                        result: None,
                        call_time: Instant::now(),
                        result_time: None,
                    });
                
                if let Some(name) = name {
                    entry.tool_name = name;
                }
                if let Some(args) = arguments {
                    entry.arguments.push_str(&args);
                }
            }
            
            StreamEvent::ToolResult { tool_call_id, result, is_error, .. } => {
                if let Some(entry) = self.tool_calls.get_mut(&tool_call_id) {
                    entry.result = Some((result, is_error));
                    entry.result_time = Some(Instant::now());
                }
            }
            
            _ => {}  // Ignore other events
        }
    }
    
    /// Finalize and return messages for database
    pub fn finalize(mut self) -> Vec<Message> {
        // Finalize reasoning
        if !self.reasoning_buffer.is_empty() {
            let duration_ms = self.reasoning_start
                .map(|start| start.elapsed().as_millis() as u64);
            
            self.messages.push(Message {
                id: ObjectId::new(),
                thread_id: self.thread_id,
                user_id: self.user_id.clone(),
                role: MessageRole::Assistant,
                message_type: MessageType::Reasoning,
                data: MessageData::Text {
                    content: self.reasoning_buffer,
                },
                created_at: Utc::now(),
                run_id: self.run_id.clone(),
                duration_ms,
            });
        }
        
        // Finalize tool calls and results
        for (_, tool_call) in self.tool_calls {
            // Tool call message
            self.messages.push(Message {
                id: ObjectId::new(),
                thread_id: self.thread_id,
                user_id: self.user_id.clone(),
                role: MessageRole::Assistant,
                message_type: MessageType::ToolCall,
                data: MessageData::ToolCall {
                    tool_call_id: tool_call.tool_call_id.clone(),
                    tool_name: tool_call.tool_name,
                    arguments: serde_json::from_str(&tool_call.arguments).unwrap_or_default(),
                },
                created_at: Utc::now(),
                run_id: self.run_id.clone(),
                duration_ms: None,
            });
            
            // Tool result message
            if let Some((result, is_error)) = tool_call.result {
                let duration_ms = tool_call.result_time
                    .map(|rt| rt.duration_since(tool_call.call_time).as_millis() as u64);
                
                self.messages.push(Message {
                    id: ObjectId::new(),
                    thread_id: self.thread_id,
                    user_id: self.user_id.clone(),
                    role: MessageRole::Assistant,
                    message_type: MessageType::ToolResult,
                    data: MessageData::ToolResult {
                        tool_call_id: tool_call.tool_call_id,
                        result,
                        is_error,
                    },
                    created_at: Utc::now(),
                    run_id: self.run_id.clone(),
                    duration_ms,
                });
            }
        }
        
        // Finalize message
        if !self.message_buffer.is_empty() {
            let duration_ms = self.message_start
                .map(|start| start.elapsed().as_millis() as u64);
            
            self.messages.push(Message {
                id: ObjectId::new(),
                thread_id: self.thread_id,
                user_id: self.user_id.clone(),
                role: MessageRole::Assistant,
                message_type: MessageType::Message,
                data: MessageData::Text {
                    content: self.message_buffer,
                },
                created_at: Utc::now(),
                run_id: self.run_id.clone(),
                duration_ms,
            });
        }
        
        self.messages
    }
}
```

---

## Integration with praxis-api

### Request Flow

```rust
// In praxis-api handler
pub async fn stream_chat(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
    Json(req): Json<StreamChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event>>>, ApiError> {
    let thread_id = ObjectId::parse_str(&thread_id)?;
    
    // 1. Get context window (with auto-summarization if needed)
    let (messages, system_prompt) = state.context_manager
        .get_context_window(thread_id)
        .await?;
    
    // 2. Add user message to DB
    let user_message = Message {
        id: ObjectId::new(),
        thread_id,
        user_id: req.user_id.clone(),
        role: MessageRole::User,
        message_type: MessageType::Message,
        data: MessageData::Text {
            content: req.content.clone(),
        },
        created_at: Utc::now(),
        run_id: run_id.clone(),
        duration_ms: None,
    };
    state.message_repo.save_message(user_message).await?;
    
    // 3. Build GraphInput with messages + system prompt
    let mut all_messages = vec![
        praxis_llm::Message::System {
            content: system_prompt,
        }
    ];
    all_messages.extend(messages);
    all_messages.push(praxis_llm::Message::Human {
        content: praxis_llm::Content::text(req.content),
        name: None,
    });
    
    let input = GraphInput::new(
        thread_id.to_string(),
        all_messages,
        req.llm_config,
    );
    
    // 4. Create run tracking
    state.run_repo.create_run(
        run_id.clone(),
        thread_id,
        req.user_id.clone(),
        req.llm_config,
    ).await?;
    
    // 5. Spawn graph execution
    let event_rx = state.graph.spawn_run(input);
    
    // 6. Create accumulator for this run
    let mut accumulator = MessageAccumulator::new(
        run_id.clone(),
        thread_id,
        req.user_id.clone(),
    );
    
    // 7. Stream events + accumulate
    let stream = ReceiverStream::new(event_rx)
        .map(move |event| {
            // Accumulate event
            accumulator.accumulate(event.clone());
            
            // Forward to client
            let json = serde_json::to_string(&event)?;
            Ok(Event::default().data(json))
        });
    
    // 8. After stream completes, persist messages
    tokio::spawn(async move {
        // Finalize messages
        let messages = accumulator.finalize();
        
        // Save to DB
        state.message_repo.save_messages(messages).await?;
        
        // Complete run
        state.run_repo.complete_run(
            &run_id,
            duration_ms,
            Some(token_usage),
            tools_used,
        ).await?;
        
        // Touch thread
        state.thread_repo.touch_thread(thread_id).await?;
    });
    
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
```

---

## Crate Structure

```
praxis-persist/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API
│   ├── repositories/
│   │   ├── mod.rs
│   │   ├── thread.rs           # ThreadRepository
│   │   ├── message.rs          # MessageRepository
│   │   └── run.rs              # RunRepository
│   ├── models/
│   │   ├── mod.rs
│   │   ├── thread.rs           # Thread, ThreadMetadata, ThreadSummary
│   │   ├── message.rs          # Message, MessageRole, MessageType
│   │   └── run.rs              # Run, RunStatus, TokenUsage
│   ├── context/
│   │   ├── mod.rs
│   │   └── manager.rs          # ContextManager
│   ├── accumulator.rs          # MessageAccumulator
│   ├── client.rs               # MongoDB client wrapper
│   └── error.rs                # Persistence errors
└── examples/
    └── simple_persistence.rs   # Standalone example
```

---

## Dependencies

```toml
[dependencies]
# Praxis core
praxis-types = { path = "../praxis-types" }
praxis-llm = { path = "../praxis-llm" }

# MongoDB
mongodb = "3.1"
bson = "2.13"

# Tokenizer (for context management)
tiktoken-rs = "0.6"

# Async runtime
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Date/time
chrono = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = "2.0"

# Logging
tracing = "0.1"
```

---

## Implementation Phases

### Phase 1: MongoDB Setup ✓
- [ ] MongoDB client connection
- [ ] Database initialization
- [ ] Collection creation
- [ ] Index creation
- [ ] Connection pooling

### Phase 2: Basic Repositories ✓
- [ ] ThreadRepository (CRUD)
- [ ] MessageRepository (CRUD)
- [ ] RunRepository (CRUD)
- [ ] Basic model types
- [ ] Error handling

### Phase 3: MessageAccumulator ✓
- [ ] Event accumulation logic
- [ ] Buffer management
- [ ] Finalization to DB messages
- [ ] Timestamp tracking
- [ ] Duration calculation

### Phase 4: Context Manager ✓
- [ ] Token counting with tiktoken
- [ ] Context window retrieval
- [ ] Summary generation
- [ ] System prompt building
- [ ] Message filtering after summary

### Phase 5: Integration ✓
- [ ] Integrate with praxis-api
- [ ] Request flow with context loading
- [ ] Post-execution persistence
- [ ] Run tracking
- [ ] Error handling

### Phase 6: Optimization ✓
- [ ] Batch message inserts
- [ ] Query optimization
- [ ] Index tuning
- [ ] Caching layer (optional)
- [ ] Connection reuse

---

## Testing Strategy

### Unit Tests
- Repository CRUD operations
- MessageAccumulator logic
- Context token counting
- Summary generation

### Integration Tests
```rust
#[tokio::test]
async fn test_full_persistence_flow() {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await.unwrap();
    
    // Create repositories
    let thread_repo = ThreadRepository::new(&client, "test_db");
    let message_repo = MessageRepository::new(&client, "test_db");
    
    // Create thread
    let thread = thread_repo.create_thread(
        "user_123".to_string(),
        ThreadMetadata::default(),
    ).await.unwrap();
    
    // Save messages
    let message = Message { /* ... */ };
    message_repo.save_message(message).await.unwrap();
    
    // Retrieve messages
    let messages = message_repo.get_messages(thread.id).await.unwrap();
    assert_eq!(messages.len(), 1);
}
```

### Load Tests
- Concurrent writes (100+ threads writing)
- Message retrieval speed
- Context window generation performance
- Summarization latency

---

## Next Steps

After praxis-persist is complete:
1. **Performance monitoring** - Query metrics, slow query log
2. **Backup strategy** - MongoDB backup/restore
3. **Multi-tenancy** - Workspace isolation
4. **Advanced summarization** - Hierarchical summaries
5. **Analytics** - User engagement, tool usage stats

---

## Questions to Address During Implementation

1. Should we store raw StreamEvents in runs collection for debugging?
2. Summarization trigger: based on token count or message count?
3. Should we cache recent context windows in Redis?
4. How to handle concurrent writes to same thread?
5. Purge old messages? Archive strategy?

---

**Ready to implement Phase 1 when you are!** 🚀

