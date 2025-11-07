# Praxis Architecture

**Version**: 1.1  
**Date**: 2025-11-07  
**Status**: Conceptual Design Complete + DX Layer Planned

---

## Architecture Evolution

This document is the consolidated architecture. For detailed evolution, see:

- ✅ [Checkpoint 1: Node Abstraction](./architecture-checkpoint-1-node.md)
- ✅ [Checkpoint 2: Graph Orchestration](./architecture-checkpoint-2-graph.md)
- ✅ [Checkpoint 3: StreamEvent & Persistence](./architecture-checkpoint-3-streamevents.md)
- ✅ [Checkpoint 4: Developer Experience & High-Level API](./architecture-checkpoint-4-dx.md)
- ⭐ [Checkpoint 5: MCP-Native Design](./architecture-checkpoint-5-mcp.md) **NEW!**

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture Diagram](#architecture-diagram)
3. [Core Components](#core-components)
   - [Node](#node)
   - [Graph](#graph)
   - [StreamEvent](#streamevent)
   - [Router](#router)
4. [Data Flow](#data-flow)
5. [Key Design Decisions](#key-design-decisions)
6. [Trade-offs](#trade-offs)
7. [Scalability Properties](#scalability-properties)
8. [Quick Reference](#quick-reference)

---

## System Overview

**Praxis** is a runtime for AI agents built in Rust, inspired by LangGraph, designed for **reflexão → decisão → ação** workflows with real-time streaming, tool execution (local and MCP), and horizontal scalability.

### Core Philosophy

> "Praxis" = **action guided by reason**

- **Learning-first**: Understand the "why" before coding
- **Scalable by design**: Async, stateless, backpressure-aware
- **Idiomatic Rust**: Traits, ownership, Send/Sync patterns
- **Observable**: Real-time event streaming for debugging and UX

### Architecture Principles

1. **Separation of Concerns**: Node executes, Router decides, Graph orchestrates
2. **Stateless Execution**: No state between requests (DB is source of truth)
3. **Non-Blocking I/O**: Async/await throughout, bounded channels for communication
4. **Graceful Degradation**: Tool failures don't crash execution
5. **Resource Control**: Timeouts, iteration limits, cancellation support

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                           CLIENT (Browser/App)                      │
│                                                                     │
│  UI Components:                                                     │
│  - Message input                                                    │
│  - Streaming display (reasoning + message)                         │
│  - Tool execution indicators                                        │
│  - EventSource (SSE connection)                                     │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  │ HTTP POST /chat
                                  │ { conversation_id, last_message, llm_config }
                                  ↓
┌─────────────────────────────────────────────────────────────────────┐
│                           GATEWAY (HTTP Server)                     │
│                                                                     │
│  - Receives client request                                          │
│  - Validates input                                                  │
│  - Calls Graph.spawn_run() → returns event_rx                      │
│  - Streams events via SSE to client                                 │
│  - Handles cancellation (client disconnect)                         │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  │ Graph.spawn_run(input)
                                  ↓
┌─────────────────────────────────────────────────────────────────────┐
│                              BACKEND                                │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  1. History Fetcher                                          │  │
│  │     - Query MongoDB for conversation history                 │  │
│  │     - Apply context_policy (last N msgs, token limit)        │  │
│  │     - Build initial messages list                            │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                  │                                  │
│                                  ↓                                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  2. Graph Orchestrator                                       │  │
│  │                                                              │  │
│  │  ┌────────────────────────────────────────────────────────┐ │  │
│  │  │  GraphState (shared mutable state)                     │ │  │
│  │  │  - llm_config (immutable)                              │ │  │
│  │  │  - conversation_id, run_id                             │ │  │
│  │  │  - messages: Vec<Message> (mutable)                    │ │  │
│  │  │  - variables: HashMap<String, Value>                   │ │  │
│  │  └────────────────────────────────────────────────────────┘ │  │
│  │                                                              │  │
│  │  ┌────────────────────────────────────────────────────────┐ │  │
│  │  │  Execution Loop (spawned async task)                   │ │  │
│  │  │                                                         │ │  │
│  │  │  current_node = LLM_NODE                               │ │  │
│  │  │  iteration = 0                                         │ │  │
│  │  │                                                         │ │  │
│  │  │  LOOP:                                                  │ │  │
│  │  │    1. Check guardrails (timeout, max_iter, cancel)     │ │  │
│  │  │    2. node.execute(&mut state, event_tx) → events      │ │  │
│  │  │    3. Handle errors (tool vs app failures)             │ │  │
│  │  │    4. router.next(&state) → NextNode                   │ │  │
│  │  │    5. If NextNode::End → BREAK                         │ │  │
│  │  │    6. Else → current_node = next, iteration++          │ │  │
│  │  │                                                         │ │  │
│  │  └────────────────────────────────────────────────────────┘ │  │
│  │                                                              │  │
│  │  Components:                                                 │  │
│  │  - Nodes: HashMap<NodeType, Box<dyn Node>>                  │  │
│  │  - Router: Box<dyn Router>                                   │  │
│  │  - LLMClient: Arc<dyn LLMClient> (shared)                    │  │
│  │  - ToolExecutor: Arc<dyn ToolExecutor>                       │  │
│  │  - Event channel: bounded(1000)                              │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│                                  │                                  │
│  ┌─────────────────────────┬────┴────┬─────────────────────────┐   │
│  │                         │         │                         │   │
│  ▼                         ▼         ▼                         ▼   │
│ ┌──────────┐         ┌──────────┐  ┌──────────┐       ┌──────────┐│
│ │ LLMNode  │         │ToolNode  │  │ Router   │       │Accumulator││
│ │          │         │          │  │          │       │           ││
│ │- Call LLM│         │- Execute │  │- Analyze │       │- Process  ││
│ │- Stream  │         │  tools   │  │  state   │       │  events   ││
│ │  tokens  │         │- Create  │  │- Decide  │       │- Build    ││
│ │- Emit    │         │  results │  │  next    │       │  content  ││
│ │  events  │         │          │  │  node    │       │  items    ││
│ └──────────┘         └──────────┘  └──────────┘       └──────────┘│
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  3. MessageAccumulator (in-memory)                           │  │
│  │     - Accumulates streaming events                           │  │
│  │     - Builds flat list of ContentItems                       │  │
│  │     - Finalizes on EndStream → AssistantMessage              │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                  │                                  │
│                                  ↓                                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  4. Persistence Layer                                        │  │
│  │     - Save AssistantMessage to MongoDB (once)                │  │
│  │     - Save partial message on cancellation                   │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ↓
┌─────────────────────────────────────────────────────────────────────┐
│                          MONGODB (Persistence)                      │
│                                                                     │
│  Collections:                                                       │
│  - conversations: { id, user_id, created_at, ... }                 │
│  - messages: {                                                      │
│      _id,                                                           │
│      conversation_id,                                               │
│      run_id,                                                        │
│      role: "user" | "assistant",                                    │
│      content_items: [                                               │
│        { type: "reasoning", content, sequence, timestamp },         │
│        { type: "message", content, sequence, timestamp },           │
│        { type: "tool_call", tool_name, arguments, ... },            │
│        { type: "tool_result", result, is_error, ... }               │
│      ],                                                             │
│      created_at,                                                    │
│      completed_at,                                                  │
│      tokens_used,                                                   │
│      incomplete: bool                                               │
│    }                                                                │
│                                                                     │
│  Indexes:                                                           │
│  - { conversation_id: 1, created_at: -1 }                          │
│  - { run_id: 1 }                                                    │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                    EXTERNAL SERVICES                                │
│                                                                     │
│  - LLM APIs (OpenAI, Azure, Anthropic)                              │
│  - Tool APIs (calculator, web search, MCP servers)                  │
└─────────────────────────────────────────────────────────────────────┘
```

### Request Flow Summary

```
Client → Gateway → Backend:
  1. Fetch history from DB (context_policy applied)
  2. Build GraphState (history + new message)
  3. Create bounded channel (capacity: 1000)
  4. Spawn async task (execution loop)
  5. Return event_rx immediately (non-blocking)

Gateway:
  6. Stream events to client via SSE

Execution Loop (spawned task):
  7. LLMNode.execute() → stream tokens → emit events
  8. Router.next() → analyze state → decide next node
  9. If tool_calls → ToolNode.execute() → results → LLMNode again
  10. If no tool_calls → NextNode::End
  11. MessageAccumulator.finalize() → AssistantMessage
  12. Save to MongoDB (single write)
  13. Close channel → Gateway closes SSE
```

---

## Core Components

### Node

**Definition**: The basic unit of computation. Executes specific logic and emits events.

#### Contract

```rust
trait Node {
    async fn execute(
        &self, 
        state: &mut GraphState, 
        event_tx: EventSender
    ) -> Result<()>;
}
```

#### Responsibilities

- ✅ Execute specific logic (call LLM, run tools)
- ✅ Emit events during execution
- ✅ Modify shared state (add responses, results)
- ✅ Handle errors gracefully

#### NOT Responsible For

- ❌ Deciding next node (Router's job)
- ❌ Managing execution flow (Graph's job)
- ❌ Persisting state (Backend's job)

#### Node Types

**LLMNode**
- Calls LLM with current state
- Streams reasoning/message tokens
- Adds LLM response to state.messages

**ToolNode**
- Reads tool_calls from last message
- Executes tools (local or MCP)
- Adds tool_results to state
- Tool failure → creates error tool_result (not app crash)

#### Design Patterns

- **Command Pattern**: Each Node is an executable command
- **Observer Pattern**: Nodes emit events, external observers consume
- **Single Responsibility**: Node executes, doesn't decide flow

---

### Graph

**Definition**: The orchestrator that manages Node execution, maintains state, and coordinates streaming.

#### Contract

```rust
struct Graph {
    nodes: HashMap<NodeType, Box<dyn Node>>,
    router: Box<dyn Router>,
    config: GraphConfig,
    llm_client: Arc<dyn LLMClient>,
    tool_executor: Arc<dyn ToolExecutor>,
}

impl Graph {
    fn spawn_run(&self, input: GraphInput) -> Receiver<StreamEvent>;
}
```

#### Responsibilities

- ✅ Manage execution loop (LLM → Router → Tool → Router → LLM → END)
- ✅ Maintain shared state (GraphState) across Nodes
- ✅ Coordinate communication via bounded channels
- ✅ Handle errors (tool failures vs app failures)
- ✅ Enforce limits (timeout, max iterations, cancellation)

#### NOT Responsible For

- ❌ Executing business logic (Nodes' job)
- ❌ Deciding routing logic (Router's job)
- ❌ Persisting state (Backend/DB's job)

#### Execution Loop

```
1. current_node = LLM_NODE
2. iteration = 0
3. emit InitStream

4. LOOP:
   a. Check guardrails (max_iterations, timeout, cancellation)
   b. node.execute(&mut state, event_tx).await
   c. Handle result (Ok → continue, Err → emit Error and BREAK)
   d. router.next(&state, current_node) → NextNode
   e. If NextNode::End → BREAK
   f. Else → current_node = next, iteration++

5. emit EndStream
6. close channel
```

#### Stateless Design

Each `Graph.spawn_run()` call:
- Creates new execution context
- Fetches fresh history from DB
- Executes independently
- Doesn't maintain state between requests

**Why?**
- ✅ Horizontal scale: Any server can handle any request
- ✅ Simple: No cache invalidation, no memory management
- ✅ Robust: Crash doesn't lose state (DB is source of truth)
- ✅ Consistent: Always latest data

**Trade-off:** Extra DB query per request (acceptable, queries are fast ~10ms)

#### Guardrails

```rust
struct GraphConfig {
    max_iterations: usize,       // Prevent infinite loops (e.g., 50)
    execution_timeout: Duration,  // Total timeout (e.g., 5 min)
    enable_cancellation: bool,    // Allow mid-execution cancellation
    emit_node_events: bool,       // NodeEnter/NodeExit for debugging
}
```

---

### StreamEvent

**Definition**: The event model transmitted via bounded channels, streamed to clients via SSE.

#### Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    InitStream { run_id, conversation_id, timestamp },
    Reasoning { content },
    Message { content },
    ToolCall { tool_call_id, tool_name, arguments, timestamp },
    ToolResult { tool_call_id, result, is_error, duration_ms },
    NodeEnter { node_id, node_type, timestamp },      // Optional (debug)
    NodeExit { node_id, duration_ms },                 // Optional (debug)
    Error { message, node_id, error_code },
    EndStream { status, total_duration_ms, tokens_used },
}
```

#### Serialization

Uses `#[serde(tag = "type")]` for flat JSON (no nested "data" field):

```json
{"type":"reasoning","content":"Thinking..."}
{"type":"message","content":"The answer is 42."}
{"type":"tool_call","tool_call_id":"call_1","tool_name":"calculator","arguments":{"expr":"2+2"},"timestamp":1699999999}
```

When sent via SSE, framework adds `data:` prefix:
```
data: {"type":"reasoning","content":"Thinking..."}

```

Browser EventSource automatically parses and strips `data:` prefix.

#### Event Types Summary

| Event | Purpose | When Emitted |
|-------|---------|--------------|
| **InitStream** | Mark execution start | First event after spawn |
| **Reasoning** | Internal LLM thoughts | During LLM streaming (token-by-token) |
| **Message** | Response to user | During LLM streaming (token-by-token) |
| **ToolCall** | LLM decided to use tool | After LLM generates tool_call |
| **ToolResult** | Tool execution result | After ToolNode executes tool |
| **NodeEnter/Exit** | Debugging/observability | If `emit_node_events = true` |
| **Error** | Fatal app-level error | LLM service down, timeout, state corruption |
| **EndStream** | Execution complete | Loop ends (Success/Error/Cancelled) |

---

### Router

**Definition**: The decision-maker that analyzes state and determines the next node.

#### Contract

```rust
trait Router {
    fn next(&self, state: &GraphState, current: NodeType) -> NextNode;
}

enum NextNode {
    LLM,
    Tool,
    End,
}
```

#### Simple Router Implementation

```rust
impl Router for SimpleRouter {
    fn next(&self, state: &GraphState, current: NodeType) -> NextNode {
        match current {
            NodeType::LLM => {
                // Check if last message has tool_calls
                if state.last_message_has_tool_calls() {
                    NextNode::Tool  // Execute tools
                } else {
                    NextNode::End  // Done
                }
            }
            NodeType::Tool => {
                // Always return to LLM after tools
                NextNode::LLM
            }
        }
    }
}
```

#### Flow Examples

```
Simple query (no tools):
  LLM (no tool_calls) → END

Tool usage:
  LLM (has tool_calls) → Tool → LLM (no tool_calls) → END

Multiple tools:
  LLM (has tool_calls) → Tool → LLM (has tool_calls) → Tool → LLM → END
```

#### Design Rationale

- ✅ **Separation of concerns**: Router decides, Graph executes
- ✅ **Testable**: Can unit test routing logic independently
- ✅ **Extensible**: Later can add conditional routing, parallel execution

---

## Data Flow

### 1. Client Request

```typescript
POST /chat
{
  "conversation_id": "conv_123",
  "last_message": {
    "role": "user",
    "content": "What's 2+2 using calculator?"
  },
  "llm_config": {
    "model": "gpt-4",
    "reasoning_effort": "high"
  },
  "context_policy": {
    "type": "last_k_messages",
    "k": 10
  }
}
```

### 2. Backend Processing

```rust
// 1. Fetch history from DB
let history = db.fetch_messages(conversation_id, context_policy).await?;

// 2. Build GraphState
let mut state = GraphState {
    llm_config,
    conversation_id,
    run_id: Uuid::new_v4().to_string(),
    messages: [history, vec![last_message]].concat(),
    variables: HashMap::new(),
};

// 3. Create bounded channel
let (event_tx, event_rx) = bounded::<StreamEvent>(1000);

// 4. Spawn async task
tokio::spawn(async move {
    // Execution loop here...
});

// 5. Return event_rx immediately (non-blocking)
event_rx
```

### 3. Execution Loop

```rust
let mut current_node = NodeType::LLM;
let mut iteration = 0;
let mut accumulator = MessageAccumulator::new(run_id, conversation_id, timestamp);

event_tx.send(StreamEvent::InitStream { ... }).await?;

loop {
    // Guardrails
    if iteration >= config.max_iterations {
        event_tx.send(StreamEvent::Error { 
            message: "Max iterations reached".into() 
        }).await?;
        break;
    }
    
    // Execute node
    let node = nodes.get(&current_node)?;
    match node.execute(&mut state, event_tx.clone()).await {
        Ok(()) => {},
        Err(e) => {
            event_tx.send(StreamEvent::Error { 
                message: e.to_string() 
            }).await?;
            break;
        }
    }
    
    // Accumulate events for persistence
    while let Ok(event) = event_rx.try_recv() {
        accumulator.process_event(&event, timestamp);
    }
    
    // Route
    let next = router.next(&state, current_node);
    match next {
        NextNode::End => break,
        NextNode::LLM => current_node = NodeType::LLM,
        NextNode::Tool => current_node = NodeType::Tool,
    }
    
    iteration += 1;
}

// Finalize and persist
let assistant_msg = accumulator.finalize(end_timestamp, tokens_used);
db.save_message(assistant_msg).await?;

event_tx.send(StreamEvent::EndStream { 
    status: StreamStatus::Success,
    total_duration_ms,
    tokens_used,
}).await?;
```

### 4. Client Streaming

```typescript
const eventSource = new EventSource('/chat');

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch(data.type) {
    case 'init_stream':
      console.log('Started:', data.run_id);
      break;
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
      if (data.is_error) showError(data.result);
      break;
    case 'end_stream':
      console.log('Done:', data.status);
      eventSource.close();
      break;
  }
};
```

---

## Key Design Decisions

### 1. Flat List for Content Items

**Decision**: Store reasoning, message, tool_calls, tool_results in a single flat list (not grouped blocks)

**Structure**:
```rust
pub struct AssistantMessage {
    pub content_items: Vec<ContentItem>,  // Ordered by sequence
    // ... other fields
}

pub enum ContentItem {
    Reasoning { sequence, content, timestamp },
    Message { sequence, content, timestamp },
    ToolCall { sequence, tool_call_id, tool_name, arguments, timestamp },
    ToolResult { sequence, tool_call_id, result, is_error, duration_ms, timestamp },
}
```

**Rationale**:
- ✅ Simple frontend rendering (iterate in order)
- ✅ Simple DB queries (sort by sequence/timestamp)
- ✅ Easy for ML fine-tuning (direct format)
- ✅ Natural ordering (sequence preserves execution flow)

**Alternative Rejected**: Grouped blocks (`reasoning_blocks[]`, `message_blocks[]`)
- ❌ Complex frontend (merge/ordering logic)
- ❌ Complex queries (unpacking, aggregations)

### 2. Stateless Graph

**Decision**: No state cached between requests, always fetch from DB

**Rationale**:
- ✅ Horizontal scale (any server, any request)
- ✅ No cache invalidation complexity
- ✅ Always consistent (latest data)
- ✅ Robust (crash doesn't lose state)

**Trade-off**: Extra DB query (~10ms) per request (acceptable)

### 3. Bounded Channels (Capacity: 1000)

**Decision**: Use bounded channels for event communication

**Rationale**:
- ✅ Backpressure: If client is slow, Node waits automatically
- ✅ Memory control: Prevents unbounded queue growth
- ✅ Scalability: Protects server from slow/malicious clients

**Alternative Rejected**: Unbounded channels
- ❌ Memory leaks on slow consumers
- ❌ No backpressure

### 4. Non-Blocking Execution (spawn_run)

**Decision**: Spawn async task for execution, return event_rx immediately

**Rationale**:
- ✅ Low latency: Client gets first response instantly
- ✅ Real-time streaming: Events arrive as they happen
- ✅ Cancellation-friendly: Client closes → task stops
- ✅ Scalable: Server doesn't block on slow LLM calls

**Alternative Rejected**: Blocking execution (wait for completion)
- ❌ High latency (wait for entire response)
- ❌ No real-time streaming
- ❌ Server blocked during LLM calls

### 5. Hybrid Persistence

**Decision**: Accumulate events in memory, save once at end (save partial on cancellation)

**Rationale**:
- ✅ Efficient: Single DB write (normal case)
- ✅ Robust: Cancellation saves partial state
- ✅ Simple: No periodic writes, no synchronization

**Trade-off**: Server crash before EndStream → data lost (rare, acceptable)

**Alternative Rejected**: Real-time persistence (write each event)
- ❌ High DB load (hundreds of writes per request)
- ❌ Complexity (synchronization, ordering)

### 6. Arc<LLMClient> (Shared)

**Decision**: Single LLMClient shared across requests via Arc

**Rationale**:
- ✅ Connection reuse (TCP connections stay open)
- ✅ Memory efficient (one instance, many users)
- ✅ Thread-safe (Arc is Send + Sync)

**Alternative Rejected**: LLMClient per request
- ❌ Connection overhead (TCP handshake each time)
- ❌ Memory waste

### 7. Tool Failures Don't Stop Execution

**Decision**: Tool failure creates error tool_result, execution continues

**Rationale**:
- ✅ Graceful degradation (LLM sees error, can try fallback)
- ✅ Better UX (user sees what went wrong)
- ✅ Resilient (one bad tool doesn't crash system)

**Error Types**:
- **Tool failure**: Create error tool_result, continue → LLM handles it
- **Node failure**: Stop execution, emit Error event → fatal

### 8. Router Decides Flow

**Decision**: Separate Router component for next node decision

**Rationale**:
- ✅ Separation of concerns (Router decides, Graph executes)
- ✅ Testable (unit test routing logic independently)
- ✅ Extensible (can add complex routing later)

**Alternative Rejected**: Graph decides inline
- ❌ Tight coupling
- ❌ Hard to test
- ❌ Hard to extend

---

## Trade-offs

### Latency vs Consistency

**Choice**: Fetch fresh data from DB each request  
**Trade-off**: +10ms latency for consistency and simplicity  
**Justification**: Horizontal scale benefits outweigh small latency cost

### Memory vs Resilience

**Choice**: Accumulate in memory, save once at end  
**Trade-off**: Risk losing data on server crash (rare)  
**Justification**: Massive reduction in DB load, simpler code

### Channel Capacity

**Choice**: Bounded channel with capacity 1000  
**Trade-off**: Very fast producers might slow down (backpressure)  
**Justification**: Protects system from unbounded memory growth

### Node Events (NodeEnter/NodeExit)

**Choice**: Configurable (off by default)  
**Trade-off**: Less observability in production  
**Justification**: Reduces payload, can enable for debugging

### Error Handling

**Choice**: Tool failures are resilient, Node failures are fatal  
**Trade-off**: Some complexity in distinguishing error types  
**Justification**: Better UX (graceful degradation vs total failure)

---

## Scalability Properties

### Horizontal Scaling

| Property | Implementation | Benefit |
|----------|----------------|---------|
| **Stateless** | No state between requests | Any server handles any request |
| **DB as source of truth** | Always fetch from MongoDB | No server affinity needed |
| **Shared clients** | Arc<LLMClient> | Connection pooling, memory efficient |
| **Load balancing** | Any server can handle any user | Simple round-robin LB |

### Resource Control

| Property | Implementation | Benefit |
|----------|----------------|---------|
| **Bounded channels** | Capacity: 1000 | Memory control via backpressure |
| **Timeouts** | execution_timeout | Prevents hung requests |
| **Max iterations** | max_iterations | Prevents infinite loops |
| **Cancellation** | Tokio cancellation tokens | Saves resources on client disconnect |

### Performance

| Property | Implementation | Benefit |
|----------|----------------|---------|
| **Async I/O** | Tokio runtime | Non-blocking, concurrent requests |
| **Non-blocking spawn** | spawn_run returns immediately | Low latency to first byte |
| **Streaming** | Token-by-token via SSE | Real-time UX |
| **Connection reuse** | Arc<LLMClient> | Fast LLM calls (no TCP handshake) |

### Target Scale

- **Concurrent users**: Millions (with horizontal scaling)
- **Requests per server**: Thousands (async, non-blocking)
- **DB queries per request**: 1 (fetch history) + 1 (save message)
- **Memory per request**: ~10KB (GraphState) + ~100KB (events buffer)

---

## Quick Reference

### Component Responsibilities

```
Node:
  ✅ Execute logic
  ✅ Emit events
  ✅ Modify state
  ❌ Don't decide flow

Graph:
  ✅ Orchestrate loop
  ✅ Manage state
  ✅ Enforce limits
  ❌ Don't execute business logic

Router:
  ✅ Analyze state
  ✅ Decide next node
  ❌ Don't execute nodes

Backend:
  ✅ Fetch history
  ✅ Build GraphState
  ✅ Persist messages
  ❌ Don't manage execution loop
```

### Execution Flow Cheat Sheet

```
Request → Gateway:
  1. Validate input
  2. Call Graph.spawn_run(input) → event_rx
  3. Stream events via SSE

Backend (spawned task):
  4. Fetch history from DB
  5. Build GraphState
  6. Loop: LLM → Router → (Tool)? → Router → LLM → END
  7. Accumulate events in memory
  8. Save message to DB (once)
  9. Close channel

Client:
  10. Receive events via EventSource
  11. Render streaming UI
  12. Close connection on EndStream
```

### Error Handling Rules

```
Tool Failure:
  → Create error tool_result
  → Continue execution
  → LLM sees error and handles

Node Failure:
  → Emit Error event
  → Stop execution
  → Client sees error

Guardrail Hit (timeout/max_iter):
  → Emit Error event
  → Stop execution
  → Save partial message with incomplete=true
```

### GraphState Structure

```rust
GraphState {
    // Immutable (config)
    llm_config: LLMConfig,
    conversation_id: String,
    run_id: String,
    
    // Mutable (context)
    messages: Vec<Message>,  // [history from DB] + [new message]
    variables: HashMap<String, Value>,
}
```

### StreamEvent Types

```
InitStream    → Execution start
Reasoning     → Internal LLM thoughts (streamed)
Message       → Response to user (streamed)
ToolCall      → LLM decided to use tool
ToolResult    → Tool execution result (+ is_error flag)
NodeEnter/Exit→ Debugging (optional)
Error         → Fatal app error
EndStream     → Execution complete (+ status + tokens)
```

### MongoDB Schema

```javascript
{
  _id: "msg_abc123",
  conversation_id: "conv_xyz",
  run_id: "run_789",
  role: "assistant",
  
  // Flat ordered list (key decision!)
  content_items: [
    { type: "reasoning", sequence: 0, content: "...", timestamp: ... },
    { type: "message", sequence: 1, content: "...", timestamp: ... },
    { type: "tool_call", sequence: 2, tool_call_id: "...", tool_name: "...", arguments: {...}, timestamp: ... },
    { type: "tool_result", sequence: 3, tool_call_id: "...", result: {...}, is_error: false, duration_ms: 50, timestamp: ... },
  ],
  
  created_at: 1699999999000,
  completed_at: 1699999999700,
  duration_ms: 700,
  tokens_used: { prompt_tokens: 45, completion_tokens: 28, reasoning_tokens: 15 },
  incomplete: false
}

// Indexes
{ conversation_id: 1, created_at: -1 }
{ run_id: 1 }
```

---

## Next Steps

### Checkpoints Completed
1. ✅ Checkpoint 1: Node abstraction (DONE)
2. ✅ Checkpoint 2: Graph orchestration (DONE)
3. ✅ Checkpoint 3: StreamEvent & Persistence (DONE)
4. ✅ Checkpoint 4: Developer Experience & High-Level API (DONE)
5. ✅ Architecture consolidation (DONE)

### Phase 1: Foundational Learning
- Study Rust async/concurrency (Chapter 16 + async book)
- Practice bounded channels (see `learning/channels-example/`)
- Understand Send/Sync traits

### Phase 2: Core Implementation (2-3 weeks)
- Create `praxis-types` crate (GraphState, StreamEvent, ContentItem)
- Implement Node trait + LLMNode/ToolNode
- Implement Graph + SimpleRouter
- Create MessageAccumulator

### Phase 3: Integration (2-3 weeks)
- Implement LLMClient (mock → real OpenAI/Azure)
- Implement ToolExecutor (local tools → MCP adapter)
- MongoDB persistence layer

### Phase 4: Developer Experience (3-4 weeks) 🆕
**See [Checkpoint 4](./architecture-checkpoint-4-dx.md) for details**
- Create `praxis-agent` crate (Agent, AgentBuilder)
- Create `praxis-registry` crate (MCPRegistry, ToolRegistry)
- Config file support (praxis.toml)
- Agent templates (RAG, Code, Support)
- Middleware system (Logging, Retry, Metrics)

### Phase 5: Gateway & Examples (2-3 weeks)
- Create Gateway with SSE endpoint
- Example: Simple chatbot (< 15 lines)
- Example: RAG agent with MCP
- Example: Code assistant
- Comprehensive documentation

### Phase 6: Refinement & Ecosystem (ongoing)
- CLI tool (`praxis-cli`)
- Add observability (tracing, metrics, dashboard)
- Write tests (unit + integration)
- Benchmark and optimize
- Public MCP registry
- Community building

---

**End of Architecture Document**
