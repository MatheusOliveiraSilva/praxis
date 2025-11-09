# Praxis Documentation

Welcome to the Praxis documentation! This directory contains architectural decisions, design documents, and planning materials.

---

## 📚 Quick Start

**New to Praxis?** Start with the [main crate README](../../crates/praxis/README.md) for a quick introduction.

**Want to understand the architecture?** Read [architecture.md](./architecture.md) for the complete system overview.

---

## 📚 Documentation Index

### Core Documents

**[plan.md](./plan.md)** - Project philosophy and learning approach
- Project goals and principles
- Incremental, reflective learning methodology

**[architecture.md](./architecture.md)** - Complete system architecture
- System overview and design decisions
- Core components (Graph, Node, Router, StreamEvent)
- Data flow and scalability properties
- **Start here for full understanding**

---

## 🔄 Architecture Evolution

These documents show how the architecture evolved:

### ✅ [Checkpoint 1: Node Abstraction](./architecture-checkpoint-1-node.md)
Basic unit of computation - Node contract and GraphState structure.

### ✅ [Checkpoint 2: Graph Orchestration](./architecture-checkpoint-2-graph.md)
Execution flow, Router pattern, error handling, and guardrails.

### ✅ [Checkpoint 3: StreamEvent & Persistence](./architecture-checkpoint-3-streamevents.md)
Real-time streaming, event model, and persistence strategy.

### ✅ [Checkpoint 4: Developer Experience](./architecture-checkpoint-4-dx.md)
High-level API design and developer experience improvements.

### ✅ [Checkpoint 5: MCP Integration](./architecture-checkpoint-5-mcp.md)
Model Context Protocol integration and tool execution.

### ✅ [Checkpoint 6: API Layer](./architecture-checkpoint-6-api.md)
REST API wrapper and HTTP integration.

### ✅ [Checkpoint 7: DX Enhancements](./architecture-checkpoint-7-dx.md)
Developer experience improvements and ergonomics.

### ✅ [Checkpoint 8: Persistence Layer](./architecture-checkpoint-8-persistence.md)
Persistence architecture, context management, and incremental saving.

---

## 🏗️ Current Architecture

Praxis consists of focused crates:

- **`praxis`**: Main crate that re-exports everything
- **`praxis-graph`**: React agent orchestrator with graph execution
- **`praxis-llm`**: Provider-agnostic LLM client (OpenAI, Azure)
- **`praxis-mcp`**: Model Context Protocol client and executor
- **`praxis-persist`**: Persistence layer with MongoDB support
- **`praxis-context`**: Context management and summarization

**Key Design Decisions:**
- Types live in `praxis-graph` (no separate `praxis-types` crate)
- Generic `EventAccumulator` with `StreamEventExtractor` trait for flexibility
- Separation of persistence (data access) and context management (logic)
- Zero-copy streaming optimizations

---

## 🚀 Current Status

- ✅ **Core runtime complete** (Graph, Node, Router, StreamEvent)
- ✅ **LLM integration** (OpenAI, Azure with streaming)
- ✅ **MCP integration** (Tool execution)
- ✅ **Persistence layer** (MongoDB with incremental saving)
- ✅ **Context management** (Automatic summarization)
- ✅ **Main crate** (`praxis` aggregates all crates)

---

**Last updated:** 2025-11-09  
**Current focus:** Production readiness and documentation
