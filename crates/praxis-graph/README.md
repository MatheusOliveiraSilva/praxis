# Praxis Graph - Core Runtime

The core runtime for Praxis AI agents, implementing the Graph orchestrator with Node abstraction and React agent pattern.

## Features

- **Node Abstraction**: LLMNode, ToolNode - composable units of computation
- **Router**: SimpleRouter implementing React pattern (LLM → Tool → LLM → END)
- **Event Streaming**: Real-time token-by-token streaming via bounded channels
- **Tool Execution**: Mock tools (calculator, weather, search) with graceful error handling
- **Guardrails**: Max iterations, timeouts, cancellation support

## Architecture

```
Graph Orchestrator
    ↓
LLMNode → Router → ToolNode → Router → LLMNode → END
    ↓                ↓
  Events          Events
    ↓                ↓
  Client          Client
```

## Quick Start

### Prerequisites

```bash
export OPENAI_API_KEY=your_openai_api_key_here
```

### Run Interactive Demo

```bash
cargo run --example react_loop
```

This will start an interactive CLI where you can chat with the React agent.

### Example Interactions

**Simple Question (no tools):**
```
You: What is the capital of France?
Assistant: The capital of France is Paris.
```

**Using Tools:**
```
You: What's the weather like in San Francisco?
Assistant:
💭 Reasoning: I should use the get_weather tool...
🔧 Calling tool: get_weather ({"location": "San Francisco, CA"})
✓ Tool result (112ms): {"temperature": 22, "condition": "sunny", "location": "San Francisco"}
The weather in San Francisco is currently sunny with a temperature of 22°C.
```

**Calculator Example:**
```
You: What's 1234 multiplied by 5678?
Assistant:
💭 Reasoning: I need to use the calculator tool...
🔧 Calling tool: calculator ({"expression": "1234 * 5678"})
✓ Tool result (105ms): {"result": 42, "expression": "1234 * 5678"}
The result of 1234 multiplied by 5678 is 7,006,652.
```

## Testing

### Build all crates

```bash
cargo build --all
```

### Run tests

```bash
cargo test --all
```

### Check lints

```bash
cargo clippy --all
```

## Code Structure

```
src/
├── node.rs              # Node trait, NodeType enum
├── router.rs            # Router trait, SimpleRouter
├── graph.rs             # Graph orchestrator
├── tools.rs             # ToolExecutor trait, MockToolExecutor
└── nodes/
    ├── llm_node.rs      # LLM interaction node
    └── tool_node.rs     # Tool execution node
```

## Next Steps

- [ ] Add real MCP integration (replace MockToolExecutor)
- [ ] Add persistence layer (MongoDB)
- [ ] Add more sophisticated routers (conditional, parallel)
- [ ] Add observability (metrics, tracing)
- [ ] Performance benchmarks

## See Also

- [Architecture Documentation](../../docs/architecture.md)
- [Plan & Philosophy](../../docs/plan.md)
- [praxis-types](../praxis-types) - Shared type definitions
- [praxis-llm](../praxis-llm) - LLM client implementations

