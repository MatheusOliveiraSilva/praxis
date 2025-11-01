# Architecture Checkpoint 1: Node Abstraction

**Status**: ✅ Finalized  
**Date**: 2025-11-01  
**Phase**: Conceptual exploration

---

## Node Definition

A **Node** is the basic unit of computation in Praxis. It represents a single step in the agent execution flow.

### Responsibilities

1. **Execute specific logic** (call LLM, execute tools, make decisions)
2. **Emit events** during execution for real-time streaming
3. **Modify shared state** (add LLM response, tool results, etc)
4. **Handle errors gracefully** and propagate them

### NOT Responsible For

- Deciding next node (Router's responsibility)
- Managing execution flow (Graph's responsibility)
- Persisting state (Backend's responsibility)

---

## Contract

```rust
trait Node {
    async fn execute(
        &self, 
        state: &mut GraphState, 
        event_tx: EventSender
    ) -> Result<()>;
}
```

### Inputs

**state: &mut GraphState**
- Immutable config: `llm_config`, `conversation_id`, `run_id`
- Mutable context: `messages` (history from DB + latest msg), `variables`

**event_tx: EventSender**
- Bounded channel to emit `StreamEvent` during execution
- Backpressure mechanism for scalability

### Outputs

**Result<()>**
- `Ok(())`: Node executed successfully
- `Err(error)`: Node failed (propagate to Graph)

**Side-effects:**
- Modifies `state` (adds responses, tool results)
- Emits events via `event_tx`

---

## Design Patterns

- **Command Pattern**: Each Node is an executable command
- **Observer Pattern**: Nodes emit events, external observers consume
- **Single Responsibility**: Node executes, doesn't decide flow

---

## Node Types

### LLMNode
- Calls LLM with current state
- Streams reasoning/message tokens as events
- Adds LLM response to state.messages

### ToolNode
- Reads tool_calls from last message
- Executes tools (local or MCP)
- Adds tool_results to state
- If tool fails → creates error tool_result (not app failure)

---

## GraphState Structure

```rust
struct GraphState {
    // Config (immutable during execution)
    llm_config: LLMConfig,
    conversation_id: String,
    run_id: String,
    
    // Context (built by backend - Option A)
    // Backend fetches history from DB based on context_policy
    messages: Vec<Message>,
    
    // Dynamic variables (mutable by Nodes)
    variables: HashMap<String, Value>,
}
```

### Input Assembly (Option A)

Client sends:
- `last_message`: The user's new message
- `conversation_id`: To fetch history from DB
- `llm_config`: Model settings, reasoning effort, etc
- `context_policy`: How many messages to retrieve (k messages, N tokens, etc)

Backend:
1. Queries DB for conversation history
2. Applies context_policy
3. Builds `GraphState.messages` = [history + last_message]
4. Executes Graph

**Rationale**: Scalable, centralized control, optimizes context/cost

---

## Event Channel (Bounded)

**What is it?**
- Communication pipe between producer (Node) and consumer (Gateway)
- **Bounded** = has maximum capacity (e.g., 1000 events)
- Provides **backpressure**: if full, producer waits

**Why bounded?**
- Controls memory usage
- Prevents system crash if consumer is slow
- Essential for scaling to millions of users

**Example:**
```rust
let (event_tx, event_rx) = bounded(1000);

// Node sends events
event_tx.send(StreamEvent::Reasoning { content: "..." }).await;

// Gateway receives and streams via SSE
while let Ok(event) = event_rx.recv().await {
    send_sse_to_client(event);
}
```

---

## Node Equivalent to Python's `__call__`

In Python (LangGraph), nodes are classes with `__call__`:
```python
class MyNode:
    def __call__(self, state):
        # execute logic
```

In Rust, we use **Trait with execute method**:
```rust
trait Node {
    async fn execute(&self, state: &mut GraphState, ...) -> Result<()>;
}

// Usage
llm_node.execute(&mut state, event_tx).await?;
```

**Why not implement `Fn` trait?**
- Async functions don't work well with `Fn`/`FnOnce`
- Trait approach is more idiomatic and flexible in Rust

---

## Behavior Clarifications

### State Mutation
Nodes CAN and SHOULD modify state:
- **LLMNode**: Appends LLM response to `state.messages`
- **ToolNode**: Appends tool_results to `state.messages`

### Error Handling
- **App-level error**: Node returns `Err` → Graph stops
- **Tool failure**: Creates error tool_result, continues execution (LLM sees error and can fallback)

### Routing
- Node does NOT decide next node
- Graph calls Router after each Node execution
- Router reads state and determines next step

---

## Routing Logic (handled by Router, not Node)

```
After LLM Node:
  - If last message has tool_calls → ToolNode
  - Else → END

After Tool Node:
  - Always → LLMNode
```

---

## Scalability Considerations

1. **Nodes are stateless**: No internal state, everything in GraphState
2. **Async execution**: Non-blocking I/O for LLM/tool calls
3. **Bounded channels**: Memory control via backpressure
4. **Error isolation**: Node failure doesn't crash system

---

## Next Steps

- Define Graph orchestration (loop, routing, limits)
- Define StreamEvent structure (event types, fields)
- Design LLM and Tool abstractions
- Create actual Rust implementation

