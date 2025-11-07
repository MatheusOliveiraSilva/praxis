# Praxis Documentation

Welcome to the Praxis documentation! This directory contains all architectural decisions, design documents, and planning materials for the project.

---

## 📚 Documentation Index

### Core Documents

**[plan.md](./plan.md)** - The guiding philosophy and learning journey
- Project goals and principles
- Learning approach (incremental, reflective)
- Working methodology with AI assistance

**[architecture.md](./architecture.md)** - Consolidated architecture document
- Complete system overview
- All core components (Node, Graph, Router, StreamEvent)
- Data flow and design decisions
- Scalability properties
- **Start here for full system understanding**

---

## 🔄 Architecture Evolution (Checkpoints)

These documents show how the architecture evolved over time. Read them in order for context, or jump to specific topics:

### ✅ [Checkpoint 1: Node Abstraction](./architecture-checkpoint-1-node.md)
**Focus:** Basic unit of computation

- What is a Node?
- Node contract and responsibilities
- GraphState structure
- Event channels (bounded)
- Behavior clarifications

**Key insight:** Nodes execute, they don't decide flow.

---

### ✅ [Checkpoint 2: Graph Orchestration](./architecture-checkpoint-2-graph.md)
**Focus:** Execution flow and coordination

- Graph definition and responsibilities
- Execution loop design
- Router pattern (separation of routing logic)
- Error handling (tool failures vs app failures)
- Guardrails (timeouts, max iterations, cancellation)
- Stateless design for horizontal scaling

**Key insight:** Graph orchestrates, Router decides, Nodes execute.

---

### ✅ [Checkpoint 3: StreamEvent & Persistence](./architecture-checkpoint-3-streamevents.md)
**Focus:** Real-time streaming and data model

- StreamEvent structure (all event types)
- Serialization for SSE (flat JSON)
- MessageAccumulator (generic finalization logic)
- ContentItem flat list (not grouped blocks)
- Hybrid persistence strategy (accumulate → save once)
- MongoDB schema design

**Key insight:** Flat list of content items simplifies everything (frontend, DB, ML).

---

### ✅ [Checkpoint 4: Developer Experience & High-Level API](./architecture-checkpoint-4-dx.md) 🆕
**Focus:** Making Praxis easy to use and widely adopted

- Vision: THE framework for Rust AI agents
- Gap analysis (what's missing for great DX)
- Three-layer architecture (DX, Core, Integration)
- `praxis-agent` crate (Agent, AgentBuilder)
- `praxis-registry` crate (MCPRegistry, ToolRegistry)
- Config file support (praxis.toml)
- Agent templates (RAG, Code, Support)
- Middleware system (extensibility)
- Updated roadmap with DX phase

**Key insight:** Technical excellence + Developer joy = Adoption.

---

## 🎯 Quick Navigation

**For understanding the system:**
- Start with [architecture.md](./architecture.md) for complete overview
- Read checkpoints 1-3 for deep understanding of runtime
- Read checkpoint 4 for understanding of developer experience

**For implementation:**
- Follow the phases in [architecture.md Next Steps](./architecture.md#next-steps)
- Checkpoint 4 has detailed API designs for DX layer

**For philosophy and learning:**
- Read [plan.md](./plan.md) to understand the project's approach

---

## 📊 Visual Summary

```
┌─────────────────────────────────────────┐
│    DEVELOPER EXPERIENCE LAYER           │  ← Checkpoint 4
│    (Agent, Builder, Registry,           │
│     Templates, Middleware)              │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│    CORE RUNTIME LAYER                   │  ← Checkpoints 1-3
│    (Node, Graph, Router,                │
│     StreamEvent, MessageAccumulator)    │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│    INTEGRATION LAYER                    │  ← Phase 2-3
│    (LLM, MCP, Tools, DB, Gateway)       │
└─────────────────────────────────────────┘
```

---

## 🚀 Current Status

- ✅ **Architecture design complete** (Checkpoints 1-4)
- ✅ **DX strategy defined** (Checkpoint 4)
- 🚧 **Implementation in progress** (praxis-llm crate)
- 📋 **Next:** Implement praxis-types, praxis-graph, then praxis-agent

---

## 💡 Contributing to Docs

When adding new architectural decisions:
1. For major changes, create a new checkpoint document
2. Update `architecture.md` with consolidated changes
3. Add links in this README
4. Update version and date in main documents

---

**Last updated:** 2025-11-07  
**Current focus:** Checkpoint 4 - Developer Experience Layer


