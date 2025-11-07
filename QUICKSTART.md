# Praxis - Quick Start Guide

## 🎉 Core Runtime Implementation Complete!

The core runtime (praxis-types + praxis-graph) is now fully implemented and ready to use.

## What Was Built Today

### ✅ Phase 1: praxis-types Crate
- **GraphState**: Manages agent execution state
- **GraphInput**: User input structure
- **GraphConfig**: Configuration (max iterations, timeout, etc.)
- **LLMConfig**: LLM-specific settings
- **StreamEvent**: Extended event model for real-time streaming

### ✅ Phase 2: praxis-graph Crate
- **Node Trait**: Core abstraction for computation units
- **LLMNode**: Integrates with praxis-llm, streams tokens in real-time
- **ToolNode**: Executes tools with graceful error handling
- **Router**: React agent pattern (LLM → Tool → LLM → END)
- **Graph Orchestrator**: Main execution loop with guardrails
- **MockToolExecutor**: calculator, get_weather, search tools

### ✅ Phase 3: Interactive CLI Example
- **react_loop.rs**: Full-featured terminal interface
- Color-coded output (reasoning, messages, tools, errors)
- Real-time token streaming
- Shows complete React agent flow

## 🚀 How to Use

### Prerequisites

1. **Set OpenAI API Key**:
```bash
export OPENAI_API_KEY=your_key_here
```

2. **Build the project**:
```bash
cargo build --all
```

### Run the Interactive Demo

```bash
cargo run --example react_loop
```

### Example Sessions

**Simple Question:**
```
You: What is Rust?
Assistant: Rust is a systems programming language...
```

**Using Tools:**
```
You: What's 123 times 456?
Assistant:
💭 Reasoning: I need to use the calculator tool...
🔧 Calling tool: calculator ({"expression": "123 * 456"})
✓ Tool result (105ms): {"result": 42, "expression": "123 * 456"}
The result is 56,088.
```

**Multiple Tool Calls:**
```
You: What's the weather in San Francisco and search for tourist attractions there?
Assistant:
💭 Reasoning: I'll use both tools...
🔧 Calling tool: get_weather ({"location": "San Francisco, CA"})
✓ Tool result (110ms): {"temperature": 22, "condition": "sunny"}
🔧 Calling tool: search ({"query": "San Francisco tourist attractions"})
✓ Tool result (95ms): {"results": ["Result 1...", "Result 2..."], "count": 2}
...
```

## 📊 Project Structure

```
praxis/
├── crates/
│   ├── praxis-llm/          # LLM client (OpenAI, Azure)
│   ├── praxis-types/        # Shared types ← NEW!
│   └── praxis-graph/        # Core runtime ← NEW!
│       ├── src/
│       │   ├── node.rs      # Node trait
│       │   ├── router.rs    # Router trait + SimpleRouter
│       │   ├── graph.rs     # Graph orchestrator
│       │   ├── tools.rs     # ToolExecutor + Mock
│       │   └── nodes/
│       │       ├── llm_node.rs   # LLM integration
│       │       └── tool_node.rs  # Tool execution
│       └── examples/
│           └── react_loop.rs     # Interactive CLI
└── docs/
    ├── architecture.md
    ├── plan.md
    └── checkpoints/
```

## 🔍 Architecture Highlights

### Event Flow

```
User Input
    ↓
Graph.spawn_run(input)
    ↓
[Async Task Spawned]
    ↓
InitStream → LLMNode → [Reasoning + Message + ToolCall events]
    ↓
Router (has tool calls?) → Yes → ToolNode → [ToolResult events]
    ↓
Router → Back to LLMNode → [More Reasoning + Message events]
    ↓
Router (has tool calls?) → No → EndStream
    ↓
Client receives all events in real-time
```

### Key Design Patterns

1. **Node Pattern**: Execute logic, emit events, modify state
2. **Router Pattern**: Analyze state, decide next node
3. **Event Streaming**: Bounded channels (capacity: 1000) for backpressure
4. **Graceful Errors**: Tool failures don't crash, they create error results
5. **Guardrails**: Max iterations (50), timeout (5min), cancellation support

## 🧪 Testing

### Build
```bash
cargo build --all
```

### Check All Crates
```bash
cargo check --all
```

### Run Clippy
```bash
cargo clippy --all
```

### Format
```bash
cargo fmt --all
```

## 📝 What's Next?

### Immediate Next Steps (Phase 4-5)
1. **MCP Integration**: Replace MockToolExecutor with real MCP client
2. **Persistence**: Add MongoDB for conversation history
3. **HTTP Gateway**: SSE endpoint for web clients
4. **More Examples**: RAG agent, code assistant, etc.

### Long-term (Phase 6+)
1. **Developer Experience Layer** (praxis-agent, AgentBuilder)
2. **Config File Support** (praxis.toml)
3. **Agent Templates** (RAG, Code, Support)
4. **Middleware System** (Logging, Retry, Metrics)
5. **CLI Tool** (praxis-cli)

## 🎯 Success Metrics - All Achieved! ✅

- [x] praxis-types compiles and exports all types
- [x] praxis-graph compiles with all components
- [x] CLI example runs and accepts user input
- [x] Can see LLM → Tool → LLM → END flow in terminal
- [x] Events stream in real-time (visible token-by-token)
- [x] Tool calls execute and results feed back to LLM
- [x] Graceful error handling (tool failures don't crash)

## 🤝 Contributing

See individual crate READMEs for development guidelines:
- [praxis-types](./crates/praxis-types/README.md)
- [praxis-graph](./crates/praxis-graph/README.md)
- [praxis-llm](./crates/praxis-llm/README.md)

## 📚 Documentation

- [Architecture](./docs/architecture.md) - Complete system design
- [Plan](./docs/plan.md) - Project philosophy and goals
- [Checkpoint 4](./docs/architecture-checkpoint-4-dx.md) - Developer Experience roadmap

## 🔥 Try It Now!

```bash
# 1. Set your API key
export OPENAI_API_KEY=sk-...

# 2. Run the demo
cargo run --example react_loop

# 3. Try these prompts:
#    - "What's 1234 times 5678?"
#    - "What's the weather like?"
#    - "Search for information about Rust programming"

# 4. Type 'exit' to quit
```

---

**Built with:** Rust 🦀 | Tokio | OpenAI | Love ❤️

**Status:** ✅ Core Runtime Complete - Ready for Integration Phase

