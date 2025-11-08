# Persistence Layer Enhancements

## Overview

This document describes the enhanced persistence layer implementation, focusing on builder patterns, real LLM-based summarization, and async background processing.

## Architecture

### Builder Pattern

The `PersistClient` uses a builder pattern for configuration:

```rust
pub struct PersistClientBuilder {
    mongodb_uri: Option<String>,
    database: Option<String>,
    max_tokens: usize,  // Default: 30_000
    llm_client: Option<Arc<dyn LLMClient>>,
    system_prompt_template: String,  // Default: embedded template
}
```

**Benefits**:
- Type safety: Required fields are enforced at build time
- Extensibility: Easy to add new optional configuration
- Readability: Clear, self-documenting API
- Flexibility: Supports both inline strings and file paths

**Example**:
```rust
let client = PersistClient::builder()
    .mongodb_uri("mongodb://localhost:27017")
    .database("praxis")
    .max_tokens(30_000)
    .llm_client(llm_client)
    .system_prompt_template_file("prompts/system.txt")?
    .build()
    .await?;
```

### Thread Model Enhancement

Added `last_summary_update` field to track incremental summarization:

```rust
pub struct Thread {
    pub id: ObjectId,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_summary_update: DateTime<Utc>,  // NEW
    pub metadata: ThreadMetadata,
    pub summary: Option<ThreadSummary>,
}
```

**Purpose**: 
- Tracks when the last summary was generated
- Enables incremental summarization (only new messages)
- Initialized to `created_at` when thread is created
- Updated whenever a new summary is generated

### Real LLM Summarization

Replaced stub implementation with actual LLM calls:

**Flow**:
1. Filter messages (only `MessageType::Message`, skip tool calls)
2. Format conversation as "Role: Content" pairs
3. Build summarization prompt using embedded template
4. Call LLM (GPT-4o-mini for cost efficiency)
5. Stream response and collect text
6. Create `ThreadSummary` with metadata

**Prompt Template**:
```text
Summarize the following conversation concisely, focusing on key information 
that will be useful for future interactions.

Previous summary (if any):
<previous_summary>

Recent conversation to summarize:
<conversation>

Generate a brief summary (max 500 words) capturing:
- Main topics discussed
- Important decisions or conclusions
- User preferences or context

Keep the summary factual and relevant for continuing the conversation.
```

### Async Background Summarization

**Problem**: Summarization can take 5-15 seconds, blocking context retrieval.

**Solution**: Fire-and-forget async tasks using `tokio::spawn`:

```rust
pub async fn get_context_window(
    &self,
    thread_id: ObjectId,
) -> Result<(Vec<Message>, String)> {
    // 1. Get thread
    let thread = self.thread_repo.get_thread(thread_id).await?;
    
    // 2. Get messages after last_summary_update
    let messages = self.message_repo
        .get_messages_after(thread_id, thread.last_summary_update)
        .await?;
    
    // 3. Count tokens
    let total_tokens = self.count_tokens(&messages)?;
    
    // 4. Check if summarization needed
    if total_tokens > self.max_tokens {
        // Trigger async summarization (fire-and-forget)
        let context = self.clone_for_async();
        let previous_summary = thread.summary.clone();
        let messages_clone = messages.clone();
        
        tokio::spawn(async move {
            if let Err(e) = context.generate_and_save_summary(
                thread_id,
                messages_clone,
                previous_summary,
            ).await {
                tracing::error!("Failed to generate summary: {}", e);
            }
        });
        
        // Return immediately with current data
        let system_prompt = self.build_system_prompt(thread.summary.as_ref());
        return Ok((messages, system_prompt));
    }
    
    // 5. Under limit, return normally
    let system_prompt = self.build_system_prompt(thread.summary.as_ref());
    Ok((messages, system_prompt))
}
```

**Key Properties**:
- **Non-blocking**: Returns immediately, summarization happens in background
- **Incremental**: Only messages after `last_summary_update` are processed
- **Resilient**: Errors are logged, not propagated to caller
- **Scalable**: Unlimited conversation length over time

**Behavior**:
1. **First call over limit**: Returns full message window, spawns summary task
2. **Subsequent calls**: Continue with old summary until new one completes
3. **After summary completes**: `last_summary_update` is updated
4. **Next call**: Only loads messages after new `last_summary_update`

### System Prompt Templates

**Embedded Default**:
```text
You are a helpful AI assistant.

Previous conversation summary:
<summary>

Use this context to assist the user effectively.
```

**Template Replacement**:
- `<summary>` placeholder is replaced with actual summary text
- If no summary exists yet, replaced with "Nao temos resumo ainda"
- Custom templates can be provided via builder

**Loading Custom Templates**:
```rust
// Inline string
.system_prompt_template("Your custom prompt with <summary>")

// From file
.system_prompt_template_file("prompts/system.txt")?
```

## Database Schema

### Updated `threads` Collection

```javascript
{
  "_id": ObjectId("..."),
  "user_id": "user_123",
  "created_at": ISODate("2025-01-01T00:00:00Z"),
  "updated_at": ISODate("2025-01-02T10:30:00Z"),
  "last_summary_update": ISODate("2025-01-02T09:00:00Z"),  // NEW
  "metadata": {
    "title": "Example Thread",
    "tags": ["support", "billing"]
  },
  "summary": {
    "text": "User inquired about billing issues...",
    "generated_at": ISODate("2025-01-02T09:00:00Z"),
    "total_tokens_before_summary": 32000,
    "messages_count": 45
  }
}
```

### `messages` Collection (unchanged)

Indexes support querying by timestamp:
- `{ thread_id: 1, created_at: 1 }` - for time-range queries

## Query: `get_messages_after`

New repository method to support incremental loading:

```rust
pub async fn get_messages_after(
    &self,
    thread_id: ObjectId,
    after: DateTime<Utc>,
) -> Result<Vec<Message>> {
    let filter = doc! {
        "thread_id": thread_id,
        "created_at": { "$gt": bson::DateTime::from_millis(after.timestamp_millis()) }
    };
    
    let messages = self.collection
        .find(filter)
        .sort(doc! { "created_at": 1 })
        .await?
        .try_collect()
        .await?;
    
    Ok(messages)
}
```

## Error Handling

Added `PersistError::LLM` variant for LLM-related errors:

```rust
#[derive(Error, Debug)]
pub enum PersistError {
    // ... existing variants
    
    #[error("LLM error: {0}")]
    LLM(#[from] anyhow::Error),
}
```

This allows `?` operator to work with `LLMClient` methods.

## Performance Considerations

### Summarization Costs

- **Model**: GPT-4o-mini (~$0.15/1M tokens input, $0.60/1M tokens output)
- **Frequency**: Only when context exceeds `max_tokens`
- **Input size**: Typically 30k-50k tokens (summary prompt + messages)
- **Output size**: ~500-1000 tokens (summary)
- **Cost per summary**: ~$0.005-0.010

### Token Counting

Currently uses a simple heuristic (4 chars ≈ 1 token). For production:
- Consider integrating `tiktoken-rs` for accurate counting
- Cache token counts in message documents
- Update counts incrementally

### Cloning for Async Tasks

```rust
fn clone_for_async(&self) -> Self {
    Self {
        thread_repo: self.thread_repo.clone(),  // Cheap: MongoDB Collection is Arc internally
        message_repo: self.message_repo.clone(),
        max_tokens: self.max_tokens,
        llm_client: Arc::clone(&self.llm_client),
        system_prompt_template: self.system_prompt_template.clone(),
    }
}
```

Repository clones are cheap because `mongodb::Collection<T>` uses `Arc` internally.

## Testing

### Unit Tests (Future Work)

- Template replacement logic
- Token counting accuracy
- Message filtering by timestamp

### Integration Tests

```rust
// Test builder API
let client = PersistClient::builder()
    .mongodb_uri("mongodb://localhost:27017")
    .database("test")
    .llm_client(mock_llm_client)
    .build()
    .await?;

// Test async summarization
// (Mock time advancement, trigger summarization, verify async update)
```

### Manual Testing

```bash
# 1. Start MongoDB
./scripts/setup-mongo.sh

# 2. Run test with real OpenAI API
OPENAI_API_KEY=sk-xxx cargo run --bin test-mongo

# 3. Verify summary in database
docker exec -it praxis-mongo mongosh -u admin -p password123 praxis
db.threads.findOne()  # Check summary and last_summary_update
```

## Migration Path

For existing databases without `last_summary_update`:

```javascript
// Add field to all existing threads
db.threads.updateMany(
  { last_summary_update: { $exists: false } },
  { $set: { last_summary_update: "$created_at" } }
);
```

Or handle in code:
```rust
let last_summary_update = thread.last_summary_update
    .unwrap_or(thread.created_at);
```

## Future Enhancements

1. **Configurable summarization model**: Allow users to choose between GPT-4o-mini, GPT-4o, Claude, etc.
2. **Batch summarization**: Summarize multiple threads in parallel
3. **Summary quality metrics**: Track summary helpfulness/relevance
4. **Compression strategies**: Hierarchical summaries for very long conversations
5. **Token counting**: Integrate `tiktoken-rs` for accurate counting
6. **Caching**: Cache generated summaries in Redis for faster access

## Summary

The enhanced persistence layer provides:
- ✅ **Builder pattern** for ergonomic, type-safe configuration
- ✅ **Real LLM summarization** using GPT-4o-mini
- ✅ **Async background processing** for non-blocking summarization
- ✅ **Incremental summarization** via `last_summary_update` tracking
- ✅ **Template system** with embedded defaults and custom overrides
- ✅ **Scalability** for arbitrarily long conversations

This foundation enables production-ready AI agents with conversation memory that scales indefinitely while maintaining context relevance.

