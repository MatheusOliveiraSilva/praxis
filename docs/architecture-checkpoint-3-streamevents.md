# Checkpoint 3: StreamEvent & Persistence

**Date**: 2025-11-09  
**Status**: Implemented

---

## Summary

Defined the event model for real-time streaming and persistence strategy. Key decisions: unified `StreamEvent` in `praxis-graph`, generic `EventAccumulator` with trait-based extraction, incremental saving on type transitions.

---

## StreamEvent Structure

All events unified in `praxis-graph::StreamEvent`:

```rust
pub enum StreamEvent {
    InitStream { run_id, conversation_id, timestamp },
    Reasoning { content },
    Message { content },
    ToolCall { index, id, name, arguments },
    ToolResult { tool_call_id, result, is_error, duration_ms },
    Done { finish_reason },
    Error { message, node_id },
    EndStream { status, total_duration_ms },
}
```

**Key Points:**
- Flat JSON serialization (`#[serde(tag = "type")]`)
- Automatic conversion from `praxis_llm::StreamEvent` via `From` trait
- Used throughout Graph execution and streaming

---

## Persistence Strategy

### Incremental Saving

`EventAccumulator<E: StreamEventExtractor>` accumulates events and saves on type transitions:

- **Reasoning → Message**: Save reasoning block
- **Message → ToolCall**: Save message block
- **ToolCall → Message**: Save tool call

**Benefits:**
- Saves complete logical units
- Reduces database writes
- Handles cancellation gracefully

### Generic Design

```rust
pub trait StreamEventExtractor {
    fn is_reasoning(&self) -> bool;
    fn is_message(&self) -> bool;
    fn is_tool_call(&self) -> bool;
    // ... content extraction methods
}
```

Allows `praxis-persist` to work with any event type without circular dependencies.

---

## Current Implementation

- ✅ `StreamEvent` in `praxis-graph/src/types/events.rs`
- ✅ `EventAccumulator` in `praxis-persist/src/accumulator.rs`
- ✅ `StreamEventExtractor` trait for flexibility
- ✅ Incremental saving on type transitions

---

**See also:** [architecture.md](./architecture.md) for complete system overview
