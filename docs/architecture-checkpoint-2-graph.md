# Architecture Checkpoint 2: Graph Orchestration

**Status**: ✅ Finalized  
**Date**: 2025-11-01  
**Phase**: Conceptual exploration

---

## Graph Definition

The **Graph** is the orchestrator that manages the execution flow of Nodes, maintains shared state, and coordinates streaming communication.

### Responsibilities

1. **Manage execution loop** (LLM → Router → Tool → Router → LLM → END)
2. **Maintain shared state** (GraphState) across Node executions
3. **Coordinate communication** via bounded channels (events streaming)
4. **Handle errors gracefully** (tool failures vs app failures)
5. **Enforce limits** (timeout, max iterations, cancellation)

### NOT Responsible For

- Executing business logic (that's Nodes' job)
- Deciding routing logic (that's Router's job)
- Persisting state (that's Backend/DB's job)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                    Graph                        │
│                                                 │
│  ┌─────────────┐      ┌──────────────┐        │
│  │    State    │      │   Nodes      │        │
│  │             │      │  - LLMNode   │        │
│  │ - messages  │      │  - ToolNode  │        │
│  │ - config    │      └──────────────┘        │
│  │ - variables │                               │
│  └─────────────┘      ┌──────────────┐        │
│                       │   Router     │        │
│  ┌─────────────┐      │              │        │
│  │ LLMClient   │      │ next(state) │        │
│  │ (Arc)       │      │  → NextNode │        │
│  └─────────────┐      └──────────────┘        │
│                                                 │
│  ┌─────────────┐                               │
│  │  EventTx    │                               │
│  │ (bounded)   │                               │
│  └─────────────┘                               │
└─────────────────────────────────────────────────┘
         │
         │ StreamEvents (real-time)
         ↓
   ┌──────────────┐
   │  Gateway     │ → SSE → Client
   └──────────────┘
```

---

## Contract

```rust
struct Graph {
    nodes: HashMap<NodeType, Box<dyn Node>>,
    router: Box<dyn Router>,
    config: GraphConfig,
    llm_client: Arc<dyn LLMClient>,  // Shared, reusable across requests
    tool_executor: Arc<dyn ToolExecutor>,
}

impl Graph {
    fn spawn_run(&self, input: GraphInput) -> Receiver<StreamEvent>;
}
```

### Inputs (GraphInput)

```rust
struct GraphInput {
    conversation_id: String,      // To fetch history from DB
    last_message: Message,        // New message from client
    llm_config: LLMConfig,        // Model, temperature, reasoning effort, etc
    context_policy: ContextPolicy, // How many messages to fetch (k msgs, N tokens)
}
```

### Process

**Non-blocking execution (Option B - Streaming Real-Time):**

1. **Fetch history** from DB using `conversation_id` + `context_policy`
2. **Build GraphState** (history + last_message + config)
3. **Create bounded channel** for events (capacity: 1000)
4. **Spawn async task** to execute loop in background
5. **Return event_rx immediately** (caller doesn't wait)

**Why non-blocking?**
- ✅ Low latency: Client gets first response instantly
- ✅ Real-time streaming: Events arrive as they happen
- ✅ Cancellation-friendly: Client closes connection → task stops
- ✅ Scalable: Server doesn't block on slow LLM calls

### Execution Loop (inside spawned task)

```
1. current_node = LLM_NODE
2. iteration = 0
3. emit InitStream event

4. LOOP:
   a. Check guardrails (max_iterations, timeout, cancellation token)
   
   b. Execute: current_node.execute(&mut state, event_tx).await
   
   c. Handle result:
      - Ok(()) → continue
      - Err(node_error) → emit Error event, BREAK
      
   d. Call router: next_node = router.next(&state, current_node_type)
   
   e. Check next:
      - NextNode::End → BREAK
      - NextNode::Tool → current_node = TOOL_NODE
      - NextNode::LLM → current_node = LLM_NODE
      
   f. iteration += 1
   
   g. CONTINUE LOOP

5. emit EndStream event
6. close channel
```

---

## Routing Logic

Handled by **Router** (separate from Graph):

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

### Simple Router Implementation

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

**Flow example:**
```
LLM (no tool_calls) → END
LLM (has tool_calls) → Tool → LLM (no tool_calls) → END
LLM (has tool_calls) → Tool → LLM (has tool_calls) → Tool → LLM → END
```

**Design rationale:**
- ✅ **Separation of concerns**: Router decides, Graph executes
- ✅ **Testable**: Can unit test routing logic independently
- ✅ **Extensible**: Later can add conditional routing, parallel execution, etc

---

## Error Handling

### Tool Failure (Resilient)

When a tool fails **inside ToolNode**:

```rust
match execute_tool(&tool_call).await {
    Ok(result) => {
        // Success: normal tool_result
        state.add_tool_result(ToolResult {
            tool_call_id: tool_call.id,
            content: result,
            is_error: false,
        });
    }
    Err(tool_error) => {
        // Tool FAILED, but Node continues
        // Create error result for LLM to see and handle
        state.add_tool_result(ToolResult {
            tool_call_id: tool_call.id,
            content: format!("Tool failed: {}", tool_error),
            is_error: true,
        });
        emit(StreamEvent::ToolResult { error: Some(...) });
    }
}
// ToolNode returns Ok(()) even if tools failed
```

**Why?**
- LLM can see error and try fallback strategy
- Graceful degradation instead of total failure
- User sees what went wrong

### Node Failure (Fatal)

When a Node itself fails (app-level error):

```rust
match node.execute(&mut state, event_tx).await {
    Ok(()) => { /* continue */ }
    Err(node_error) => {
        // App-level error: stop execution
        emit(StreamEvent::Error {
            message: node_error.to_string(),
            node_id: current_node.id(),
        });
        break;  // Stop loop
    }
}
```

**Examples of app-level errors:**
- LLM service completely down
- State corruption
- Timeout exceeded
- Cancellation requested

### Error Type Summary

| Error Type | Behavior | Example |
|------------|----------|---------|
| **Tool failure** | Create error tool_result, continue | API timeout, tool not found |
| **Node failure** | Stop execution, emit Error event | LLM service down, state corruption |
| **Guardrail limit** | Stop execution, emit Error event | max_iterations reached, timeout |

**Pattern**: **Resilient Command** — Commands are resilient to internal failures but fail explicitly on structural issues.

---

## Guardrails & Limits

```rust
struct GraphConfig {
    max_iterations: usize,       // Prevent infinite loops (e.g., 50)
    execution_timeout: Duration,  // Total timeout (e.g., 5 minutes)
    enable_cancellation: bool,    // Allow mid-execution cancellation
}
```

### Why each guardrail?

**max_iterations:**
- Prevents: LLM → Tool → LLM → Tool... infinitely
- Trigger: After N iterations, emit Error and stop
- Typical value: 50 iterations (very generous for normal flows)

**execution_timeout:**
- Prevents: Hung LLM calls, very slow tools
- Trigger: After T seconds, cancel task and emit Error
- Typical value: 5 minutes (most requests should finish in < 30s)

**enable_cancellation:**
- Prevents: Wasted resources on abandoned requests
- Trigger: Client closes SSE connection → task stops
- Implementation: Use tokio::select! with cancellation token

---

## State Management

### Stateless Graph (Chosen Approach)

Each `Graph.spawn_run()` call:
- Creates new execution context
- Fetches fresh history from DB
- Executes independently
- Doesn't maintain state between requests

**Why stateless?**
- ✅ **Horizontal scale**: Any server can handle any request
- ✅ **Simple**: No memory management, no cache invalidation
- ✅ **Robust**: Crash doesn't lose state (everything from DB)
- ✅ **Consistent**: Always has latest data from DB

**Trade-off:** Extra DB query per request, but acceptable for:
- DB queries are fast (< 10ms for indexed lookups)
- Can optimize with read replicas
- Gain massive scalability benefits

### No Cross-Request Cache (Decision)

We will **NOT** cache history between user requests because:
- ❌ Complex to sync with dynamic context policies
- ❌ Cache invalidation is hard (different servers, concurrent requests)
- ❌ Stale data risk (user sent message on another device)
- ✅ DB is fast enough for fresh queries

**Alternative considered:** LRU cache with short TTL (1 min)
**Decision:** Keep it simple initially, optimize later if needed

---

## LLM Client Management

### Arc Pattern (Simple, Chosen)

```rust
// Create at server startup (once)
let llm_client = Arc::new(OpenAIClient::new(...));

// Share across requests (cheap clone, just increments ref count)
let graph = Graph::new(Arc::clone(&llm_client));
```

**Benefits:**
- ✅ Shared ownership across async tasks
- ✅ Thread-safe (Arc is Send + Sync)
- ✅ Connection reuse (TCP connections stay open)
- ✅ Memory efficient (one instance, many users)

**Alternative considered:** LLM Pool (round-robin, multiple clients)
**Decision:** Start simple with single Arc, add pool later if needed

---

## Streaming Communication

### Bounded Channel for Events

```rust
// Create channel with capacity
let (event_tx, event_rx) = bounded::<StreamEvent>(1000);

// Node produces events
event_tx.send(StreamEvent::Reasoning { ... }).await;

// Gateway consumes and streams via SSE
while let Ok(event) = event_rx.recv().await {
    send_sse_to_client(event);
}
```

**Why bounded (capacity = 1000)?**
- **Backpressure**: If client is slow, Node waits automatically
- **Memory control**: Prevents unbounded queue growth
- **Scalability**: Protects server from slow/malicious clients

**See**: `learning/channels-example/` for executable demonstration

---

## GraphState Structure

```rust
struct GraphState {
    // Immutable config
    llm_config: LLMConfig,
    conversation_id: String,
    run_id: String,  // Unique per execution (for tracing)
    
    // Context (built from DB query)
    messages: Vec<Message>,  // [history from DB] + [last_message from client]
    
    // Dynamic variables (mutable by Nodes)
    variables: HashMap<String, Value>,  // Custom data Nodes can store/read
}

impl GraphState {
    fn last_message(&self) -> &Message;
    fn last_message_has_tool_calls(&self) -> bool;
    fn add_message(&mut self, msg: Message);
    fn add_tool_result(&mut self, result: ToolResult);
}
```

---

## Design Patterns Applied

1. **Command Pattern**: Nodes are executable commands
2. **Strategy Pattern**: Router decides flow strategy
3. **Observer Pattern**: Nodes emit events, Gateway observes
4. **Reactor Pattern**: Event loop with async I/O
5. **Orchestrator Pattern**: Graph coordinates Node execution

---

## Scalability Properties

| Property | Implementation | Benefit |
|----------|----------------|---------|
| **Stateless** | No state between requests | Horizontal scale |
| **Non-blocking** | Spawn background tasks | Server handles concurrent requests |
| **Backpressure** | Bounded channels | Memory control |
| **Shared clients** | Arc<LLMClient> | Connection reuse |
| **Timeouts** | execution_timeout | Prevents hung requests |
| **Cancellation** | Tokio cancellation tokens | Saves resources |

**Target**: Millions of concurrent users with horizontal scaling

---

## Example Flow

```
Client sends:
  conversation_id: "conv_123"
  last_message: "What's 2+2 using calculator?"
  llm_config: { model: "gpt-4", reasoning: "high" }

Backend:
  1. Fetch history from DB (conversation_id = "conv_123")
  2. Apply context_policy (last 10 messages)
  3. Build GraphState.messages = [history] + [new message]
  4. Create bounded channel (1000 capacity)
  5. Spawn task → run execution loop
  6. Return event_rx to Gateway
  7. Gateway streams events via SSE to client

Execution loop:
  1. emit: InitStream
  2. LLMNode.execute() → calls LLM with context
     emit: Reasoning chunks ("Let me calculate...")
     emit: Message chunks ("I'll use the calculator")
     emit: ToolCall { tool: "calculator", args: {expr: "2+2"} }
  3. Router.next() → sees tool_calls → NextNode::Tool
  4. ToolNode.execute() → executes calculator(2+2) = 4
     emit: ToolResult { result: "4" }
  5. Router.next() → always LLM after tools
  6. LLMNode.execute() → calls LLM with tool_result in context
     emit: Reasoning chunks ("The result is 4")
     emit: Message chunks ("The answer is 4")
  7. Router.next() → no tool_calls → NextNode::End
  8. emit: EndStream
  9. close channel

Client sees (via SSE):
  data: {"type":"init_stream"}
  data: {"type":"reasoning","content":"Let me calculate..."}
  data: {"type":"message","content":"I'll use the calculator"}
  data: {"type":"tool_call","tool":"calculator","args":{"expr":"2+2"}}
  data: {"type":"tool_result","result":"4"}
  data: {"type":"reasoning","content":"The result is 4"}
  data: {"type":"message","content":"The answer is 4"}
  data: {"type":"end_stream"}
```

---

## Next Steps

- Define StreamEvent structure (event types, fields, serialization)
- Implement actual Rust code for Graph
- Create LLMClient trait and mock implementation
- Create ToolExecutor trait and local tools
- Build Gateway with SSE endpoint

