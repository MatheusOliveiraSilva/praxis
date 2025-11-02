# Architecture Checkpoint 3: StreamEvent & Persistence

**Status**: ✅ Finalized  
**Date**: 2025-11-02  
**Phase**: Conceptual exploration

---

## StreamEvent Definition

**StreamEvent** is the event model transmitted via bounded channels during Graph execution, streamed to clients via SSE.

### Core Principles

1. **Real-time streaming**: Events emitted as they happen (token-by-token)
2. **Type-safe**: Rust enum with specific variants
3. **SSE compatible**: JSON serialization without nested "data" wrapper
4. **Observability**: Optional node events for debugging

---

## Event Structure

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "init_stream")]
    InitStream {
        run_id: String,
        conversation_id: String,
        timestamp: i64,
    },
    
    #[serde(rename = "reasoning")]
    Reasoning {
        content: String,
    },
    
    #[serde(rename = "message")]
    Message {
        content: String,
    },
    
    #[serde(rename = "tool_call")]
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
        timestamp: i64,
    },
    
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_call_id: String,
        result: serde_json::Value,
        is_error: bool,
        duration_ms: u64,
    },
    
    #[serde(rename = "node_enter")]
    NodeEnter {
        node_id: String,
        node_type: String,
        timestamp: i64,
    },
    
    #[serde(rename = "node_exit")]
    NodeExit {
        node_id: String,
        duration_ms: u64,
    },
    
    #[serde(rename = "error")]
    Error {
        message: String,
        node_id: Option<String>,
        error_code: Option<String>,
    },
    
    #[serde(rename = "end_stream")]
    EndStream {
        status: StreamStatus,
        total_duration_ms: u64,
        tokens_used: Option<TokenUsage>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamStatus {
    Success,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub reasoning_tokens: u32,
}
```

---

## Serialization Format

### Serde Configuration

Using `#[serde(tag = "type")]` produces flat JSON (no nested "data" field):

```rust
let event = StreamEvent::Reasoning { 
    content: "Pensando...".to_string() 
};

let json = serde_json::to_string(&event)?;
// {"type":"reasoning","content":"Pensando..."}
```

### SSE Format

When sent via Server-Sent Events, the framework adds `data:` prefix:

```
data: {"type":"reasoning","content":"Pensando..."}

data: {"type":"message","content":"Hello"}

```

Browser EventSource automatically parses and removes the `data:` prefix.

---

## Event Types

### 1. InitStream
Marks the beginning of execution.

**Fields:**
- `run_id`: Unique execution ID (for tracing)
- `conversation_id`: Conversation identifier
- `timestamp`: Unix timestamp (ms)

**When emitted:** First event after Graph.spawn_run()

---

### 2. Reasoning
Internal thought process of LLM (streamed token-by-token).

**Fields:**
- `content`: Chunk of reasoning text

**Semantic meaning:** Internal reasoning (may be hidden from user in UI)

**Example:**
```
Reasoning { content: "Let me " }
Reasoning { content: "think " }
Reasoning { content: "about this..." }
```

---

### 3. Message
Response to user (streamed token-by-token).

**Fields:**
- `content`: Chunk of message text

**Semantic meaning:** Final response (always visible to user)

**Example:**
```
Message { content: "The answer " }
Message { content: "is " }
Message { content: "42." }
```

---

### 4. ToolCall
LLM decided to call a tool.

**Fields:**
- `tool_call_id`: Unique ID to correlate with result
- `tool_name`: Name of the tool (e.g., "calculator")
- `arguments`: JSON arguments for the tool
- `timestamp`: When tool was called

**When emitted:** After LLMNode generates tool_call in response

**Example:**
```json
{
  "type": "tool_call",
  "tool_call_id": "call_abc123",
  "tool_name": "calculator",
  "arguments": { "expression": "2+2" },
  "timestamp": 1699999999600
}
```

---

### 5. ToolResult
Result of tool execution.

**Fields:**
- `tool_call_id`: Correlates with ToolCall
- `result`: JSON result from tool
- `is_error`: True if tool failed
- `duration_ms`: Execution time

**When emitted:** After ToolNode executes tool

**Example (success):**
```json
{
  "type": "tool_result",
  "tool_call_id": "call_abc123",
  "result": { "answer": 4 },
  "is_error": false,
  "duration_ms": 15
}
```

**Example (error):**
```json
{
  "type": "tool_result",
  "tool_call_id": "call_abc123",
  "result": { "error": "Service unavailable" },
  "is_error": true,
  "duration_ms": 2000
}
```

---

### 6. NodeEnter / NodeExit (Optional)
Debugging/observability events.

**Configuration:** Only emitted if `GraphConfig.emit_node_events = true`

**NodeEnter fields:**
- `node_id`: Identifier (e.g., "llm_node")
- `node_type`: Type (e.g., "LLM")
- `timestamp`: Entry time

**NodeExit fields:**
- `node_id`: Identifier
- `duration_ms`: How long node took

**Usage:** Track execution flow, identify bottlenecks

---

### 7. Error
Fatal error that stops execution.

**Fields:**
- `message`: Error description
- `node_id`: Which node failed (optional)
- `error_code`: Machine-readable code (optional)

**When emitted:** 
- LLM service down
- Timeout exceeded
- State corruption
- Any app-level failure

**Difference from ToolResult.is_error:**
- Tool failure → ToolResult with is_error=true (execution continues)
- App failure → Error event (execution stops)

---

### 8. EndStream
Marks end of execution.

**Fields:**
- `status`: Success, Error, or Cancelled
- `total_duration_ms`: Total execution time
- `tokens_used`: Token statistics (optional)

**When emitted:** 
- Loop completes normally (Success)
- Fatal error occurs (Error)
- Client cancels request (Cancelled)

---

## Persistence Model

### Challenge: Multiple Content Items

LLM can emit multiple reasoning/message blocks in any order during execution:

```
Execution flow:
  LLMNode (1st call):
    Reasoning → "Let me use calculator..."
    Message → "I'll calculate this."
    ToolCall → calculator(2+2)
  
  ToolNode:
    ToolResult → 4
  
  LLMNode (2nd call, after tool):
    Reasoning → "The result is 4."
    Message → "The answer is 4."

Result: 6 content items in sequence
```

### Design Decision: Flat List (Not Grouped Blocks)

**Why flat list?**
- ✅ Simple frontend rendering (iterate in order)
- ✅ Simple DB queries (sort by created_at or sequence)
- ✅ Easy for fine-tuning ML models (direct format)
- ✅ Natural ordering (sequence number + timestamp)
- ✅ Easy to add new content types

**Alternative considered:** Grouped blocks (reasoning_blocks[], message_blocks[])
- ❌ Complex frontend (merge/ordering logic)
- ❌ Complex queries (unpacking, aggregations)
- ❌ Less intuitive ("what was the 3rd thing sent?")

### ContentItem Enum

Generic enum for all content types in execution order:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    #[serde(rename = "reasoning")]
    Reasoning {
        sequence: u32,
        content: String,
        timestamp: i64,
    },
    
    #[serde(rename = "message")]
    Message {
        sequence: u32,
        content: String,
        timestamp: i64,
    },
    
    #[serde(rename = "tool_call")]
    ToolCall {
        sequence: u32,
        tool_call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
        timestamp: i64,
    },
    
    #[serde(rename = "tool_result")]
    ToolResult {
        sequence: u32,
        tool_call_id: String,
        result: serde_json::Value,
        is_error: bool,
        duration_ms: u64,
        timestamp: i64,
    },
}

impl ContentItem {
    pub fn sequence(&self) -> u32 {
        match self {
            Self::Reasoning { sequence, .. } => *sequence,
            Self::Message { sequence, .. } => *sequence,
            Self::ToolCall { sequence, .. } => *sequence,
            Self::ToolResult { sequence, .. } => *sequence,
        }
    }
    
    pub fn timestamp(&self) -> i64 {
        match self {
            Self::Reasoning { timestamp, .. } => *timestamp,
            Self::Message { timestamp, .. } => *timestamp,
            Self::ToolCall { timestamp, .. } => *timestamp,
            Self::ToolResult { timestamp, .. } => *timestamp,
        }
    }
}
```

---

## MessageAccumulator (In-Memory)

Accumulates streaming events into flat list of content items:

```rust
pub struct MessageAccumulator {
    // Flat list of content items (in order)
    pub content_items: Vec<ContentItem>,
    
    // Current streaming state
    current_item_type: Option<ContentItemType>,
    current_buffer: String,
    current_start_timestamp: Option<i64>,
    
    // Sequence counter (auto-increment)
    sequence_counter: u32,
    
    // Metadata
    pub run_id: String,
    pub conversation_id: String,
    pub start_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq)]
enum ContentItemType {
    Reasoning,
    Message,
}
```

### Accumulation Logic (Generic Finalization)

Key insight: Finalize current item whenever content type changes (reasoning→message or message→reasoning).

```rust
impl MessageAccumulator {
    pub fn new(run_id: String, conversation_id: String, start_timestamp: i64) -> Self {
        Self {
            content_items: Vec::new(),
            current_item_type: None,
            current_buffer: String::new(),
            current_start_timestamp: None,
            sequence_counter: 0,
            run_id,
            conversation_id,
            start_timestamp,
        }
    }
    
    /// Finalizes current item if type changed (generic for reasoning/message)
    fn finalize_current_if_different(&mut self, new_type: Option<ContentItemType>, timestamp: i64) {
        // No current item? Nothing to finalize
        if self.current_item_type.is_none() {
            return;
        }
        
        // Type changed (or ending)? Finalize
        if self.current_item_type != new_type {
            let item_type = self.current_item_type.take().unwrap();
            let content = std::mem::take(&mut self.current_buffer);
            let start = self.current_start_timestamp.take().unwrap();
            
            if !content.is_empty() {
                let item = match item_type {
                    ContentItemType::Reasoning => ContentItem::Reasoning {
                        sequence: self.sequence_counter,
                        content,
                        timestamp: start,
                    },
                    ContentItemType::Message => ContentItem::Message {
                        sequence: self.sequence_counter,
                        content,
                        timestamp: start,
                    },
                };
                
                self.content_items.push(item);
                self.sequence_counter += 1;
            }
        }
    }
    
    pub fn process_event(&mut self, event: &StreamEvent, timestamp: i64) {
        match event {
            StreamEvent::Reasoning { content } => {
                // Finalize previous item if different type
                self.finalize_current_if_different(Some(ContentItemType::Reasoning), timestamp);
                
                // Start new item if needed
                if self.current_item_type.is_none() {
                    self.current_item_type = Some(ContentItemType::Reasoning);
                    self.current_start_timestamp = Some(timestamp);
                }
                
                // Append chunk to buffer
                self.current_buffer.push_str(content);
            }
            
            StreamEvent::Message { content } => {
                // Finalize previous item if different type (could be reasoning)
                self.finalize_current_if_different(Some(ContentItemType::Message), timestamp);
                
                // Start new item if needed
                if self.current_item_type.is_none() {
                    self.current_item_type = Some(ContentItemType::Message);
                    self.current_start_timestamp = Some(timestamp);
                }
                
                // Append chunk to buffer
                self.current_buffer.push_str(content);
            }
            
            StreamEvent::ToolCall { tool_call_id, tool_name, arguments, timestamp } => {
                // Finalize any pending text item
                self.finalize_current_if_different(None, *timestamp);
                
                // Tool call is atomic (no chunks)
                let item = ContentItem::ToolCall {
                    sequence: self.sequence_counter,
                    tool_call_id: tool_call_id.clone(),
                    tool_name: tool_name.clone(),
                    arguments: arguments.clone(),
                    timestamp: *timestamp,
                };
                
                self.content_items.push(item);
                self.sequence_counter += 1;
            }
            
            StreamEvent::ToolResult { tool_call_id, result, is_error, duration_ms } => {
                // Tool result is atomic
                let item = ContentItem::ToolResult {
                    sequence: self.sequence_counter,
                    tool_call_id: tool_call_id.clone(),
                    result: result.clone(),
                    is_error: *is_error,
                    duration_ms: *duration_ms,
                    timestamp,
                };
                
                self.content_items.push(item);
                self.sequence_counter += 1;
            }
            
            _ => {}
        }
    }
    
    pub fn finalize(mut self, end_timestamp: i64, tokens_used: Option<TokenUsage>) -> AssistantMessage {
        // Finalize any pending item
        self.finalize_current_if_different(None, end_timestamp);
        
        AssistantMessage {
            id: uuid::Uuid::new_v4().to_string(),
            conversation_id: self.conversation_id,
            run_id: self.run_id,
            role: "assistant".to_string(),
            
            // Flat ordered list
            content_items: self.content_items,
            
            // Metadata
            created_at: self.start_timestamp,
            completed_at: Some(end_timestamp),
            duration_ms: (end_timestamp - self.start_timestamp) as u64,
            tokens_used,
            
            incomplete: false,
        }
    }
}
```

---

## Persistence Strategy: Hybrid

### Approach

1. **During execution:** Accumulate in memory (no DB writes)
2. **On completion (EndStream):** Save entire message once
3. **On cancellation:** Save partial message with `incomplete = true`

### Execution Flow

```
┌──────────────────────────────────────────┐
│         Graph Execution Loop             │
│                                          │
│  accumulator = MessageAccumulator::new() │
│                                          │
│  loop {                                  │
│    check_cancellation()                  │
│    node.execute() → events               │
│                                          │
│    for event in events {                 │
│      event_tx.send(event) ─────→ Client │
│      accumulator.process(event)          │
│    }                                     │
│  }                                       │
│                                          │
│  // Completed                            │
│  msg = accumulator.finalize()            │
│  db.save(msg).await  ← ONE WRITE         │
└──────────────────────────────────────────┘

On cancellation:
  msg = accumulator.finalize()
  msg.incomplete = true
  db.save(msg).await
```

### Benefits

- ✅ Efficient: Single DB write (normal case)
- ✅ Robust: Cancellation saves partial state
- ✅ Simple frontend: Just renders chunks
- ✅ Rich metadata: Duration per block, timestamps

---

## MongoDB Schema

### AssistantMessage Document

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    #[serde(rename = "_id")]
    pub id: String,
    
    pub conversation_id: String,
    pub run_id: String,
    pub role: String,  // "assistant"
    
    // Flat list of content items (ordered by sequence)
    pub content_items: Vec<ContentItem>,
    
    // Metadata
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: u64,
    pub tokens_used: Option<TokenUsage>,
    
    // Status
    pub incomplete: bool,
}
```

**Note:** `ContentItem` enum already contains all information (reasoning, message, tool_call, tool_result) in a unified structure.

### Example Document (Flat List)

```json
{
  "_id": "msg_abc123",
  "conversation_id": "conv_xyz",
  "run_id": "run_789",
  "role": "assistant",
  
  "content_items": [
    {
      "type": "reasoning",
      "sequence": 0,
      "content": "Let me calculate this using the calculator tool.",
      "timestamp": 1699999999000
    },
    {
      "type": "message",
      "sequence": 1,
      "content": "I'll use the calculator to solve this.",
      "timestamp": 1699999999200
    },
    {
      "type": "tool_call",
      "sequence": 2,
      "tool_call_id": "call_1",
      "tool_name": "calculator",
      "arguments": { "expression": "2+2" },
      "timestamp": 1699999999300
    },
    {
      "type": "tool_result",
      "sequence": 3,
      "tool_call_id": "call_1",
      "result": { "answer": 4 },
      "is_error": false,
      "duration_ms": 50,
      "timestamp": 1699999999350
    },
    {
      "type": "reasoning",
      "sequence": 4,
      "content": "The calculator returned 4, which is correct.",
      "timestamp": 1699999999400
    },
    {
      "type": "message",
      "sequence": 5,
      "content": "The answer is 4.",
      "timestamp": 1699999999600
    }
  ],
  
  "created_at": 1699999999000,
  "completed_at": 1699999999700,
  "duration_ms": 700,
  "tokens_used": {
    "prompt_tokens": 45,
    "completion_tokens": 28,
    "reasoning_tokens": 15
  },
  
  "incomplete": false
}
```

**Advantages of this format:**
- Natural execution order (sequence 0→1→2→3→4→5)
- Simple iteration for frontend/ML
- Single timestamp per item (when it started streaming)
- Tool calls/results integrated in sequence

---

## Frontend Usage

### Fetching History (Simple Query)

```typescript
// API endpoint: GET /conversations/:id/messages?limit=10
const response = await fetch(`/api/conversations/${convId}/messages?limit=10`);
const messages = await response.json();

// Backend query (MongoDB):
// db.messages.find({ conversation_id: convId })
//            .sort({ created_at: 1 })
//            .limit(10)

// Render
messages.forEach(msg => {
  msg.content_items.forEach(item => {
    switch(item.type) {
      case 'reasoning':
        renderReasoning(item.content, item.timestamp);
        break;
      case 'message':
        renderMessage(item.content, item.timestamp);
        break;
      case 'tool_call':
        renderToolCall(item.tool_name, item.arguments);
        break;
      case 'tool_result':
        renderToolResult(item.result, item.is_error);
        break;
    }
  });
});
```

**Simplicity:**
- No complex merging/ordering logic
- Simple loop: messages → items → render
- Timestamps preserved for "thought for X seconds" features

### Streaming Display (Real-time)

```typescript
eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch(data.type) {
    case 'reasoning':
      appendToReasoningBox(data.content);
      break;
    case 'message':
      appendToMessageBox(data.content);
      break;
    case 'tool_call':
      showToolIndicator(data.tool_name);
      break;
    case 'tool_result':
      hideToolIndicator();
      break;
  }
};
```

---

## Configuration

### GraphConfig

```rust
pub struct GraphConfig {
    pub max_iterations: usize,
    pub execution_timeout: Duration,
    pub enable_cancellation: bool,
    pub emit_node_events: bool,  // NodeEnter/NodeExit
}
```

**emit_node_events:**
- `false` (default): Cleaner stream, less noise
- `true`: Debugging, observability

---

## Design Decisions

### 1. NodeEnter/NodeExit: Configurable
**Decision:** Emit only if `emit_node_events = true`  
**Rationale:** Reduces payload in production, enables debugging when needed

### 2. Timestamps: Selective
**Decision:** Only in InitStream, ToolCall, NodeEnter  
**Rationale:** Reduces payload size, client can add timestamps if needed

### 3. Flat JSON Format
**Decision:** `{"type":"reasoning","content":"..."}` (no nested "data")  
**Rationale:** Simpler, smaller payload, cleaner frontend code

### 4. Multiple Blocks
**Decision:** List of reasoning/message blocks, not single concatenated string  
**Rationale:** Preserves execution sequence, enables duration tracking per block

### 5. Hybrid Persistence
**Decision:** Accumulate in memory, save once at end, save partial on cancel  
**Rationale:** Efficient (1 write), robust (handles cancellation), simple

---

## Edge Cases

### 1. Cancellation Mid-Stream
- Accumulator saves partial blocks
- `incomplete = true` flag in DB
- Client sees partial response

### 2. Reasoning Without Message
- Valid: LLM can reason and go straight to tool
- message_blocks can be empty

### 3. Multiple Tool Calls
- LLM can call multiple tools in parallel
- Each has unique tool_call_id
- Results correlated by ID

### 4. Content Order Flexibility
- Valid orders: reasoning→message, message→reasoning, reasoning→reasoning
- Generic finalization logic handles all cases
- No assumptions about order

### 5. Server Crash
- If crash before EndStream → data lost
- Trade-off accepted (rare occurrence)
- Alternative: Periodic checkpointing (adds complexity)

---

## Query Performance

### MongoDB Indexes

```javascript
// Primary index: fetch messages for a conversation
db.messages.createIndex({ conversation_id: 1, created_at: -1 });

// Secondary: lookup by run_id (debugging)
db.messages.createIndex({ run_id: 1 });
```

### Common Queries

**Fetch last N messages:**
```rust
db.collection("messages")
    .find(doc! { "conversation_id": conv_id })
    .sort(doc! { "created_at": -1 })
    .limit(10)
```

**Count messages in conversation:**
```rust
db.collection("messages")
    .count_documents(doc! { "conversation_id": conv_id })
```

**All queries are simple (no aggregations, no unpacking).**

---

## Fine-Tuning Integration

Flat list format is ML-friendly:

```python
# Convert to training format
for msg in messages:
    for item in msg['content_items']:
        training_example = {
            'role': item['type'],
            'content': item.get('content') or json.dumps(item),
            'timestamp': item['timestamp']
        }
        training_data.append(training_example)
```

Direct iteration, no restructuring needed.

---

## Next Steps

- Implement Graph execution with MessageAccumulator
- Create MongoDB indexes for conversation_id queries
- Define context_policy for history retrieval
- Implement cancellation token handling

