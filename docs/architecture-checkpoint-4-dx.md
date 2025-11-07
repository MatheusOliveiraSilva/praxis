# Architecture Checkpoint 4: Developer Experience & High-Level API

**Status**: ✅ Finalized  
**Date**: 2025-11-07  
**Phase**: Strategic Planning

---

## Vision Statement

**Praxis should be THE backend framework for high-performance AI agents in Rust.**

Like how:
- **React** became THE UI framework
- **FastAPI** became THE Python web framework  
- **LangGraph** became THE Python agent framework

**Praxis should be THE Rust agent framework.**

### User Story

> "I need a chatbot with MCP tools? I pull Praxis agent, plug MCP server, connect to my API + database, and DONE."
>
> "I need another AI agent for task Y with different MCP server? I plug it in..."
>
> "I need to scale to millions of users? Praxis handles it."

---

## Current State Analysis

### ✅ What's EXCELLENT (Core Runtime)

**1. Technical Foundation**
- Node → Graph → Router architecture is robust
- Designed for millions of concurrent users
- Stateless, async, bounded channels for scalability
- Excellent separation of concerns
- StreamEvents well-modeled for real-time

**2. Scalability Properties**
- Horizontal scaling via stateless design
- Async I/O with Tokio
- Backpressure via bounded channels
- Resource control (timeouts, limits, cancellation)

**3. MCP Integration Planned**
- ToolExecutor abstraction ready
- MCP adapter on roadmap

**Verdict:** 🏆 **World-class runtime foundation**

---

### ❌ What's MISSING (Developer Experience)

#### 1. High-Level API (Critical!)

**Current state (too low-level):**
```rust
let graph = Graph::new(
    nodes: HashMap::new(),
    router: Box::new(SimpleRouter),
    config: GraphConfig { 
        max_iterations: 50,
        execution_timeout: Duration::from_secs(300),
        enable_cancellation: true,
        emit_node_events: false,
    },
    llm_client: Arc::new(OpenAIClient::new(...)),
    tool_executor: Arc::new(ToolExecutor::new(...)),
);
```

**Desired state (developer joy):**
```rust
let agent = Agent::builder()
    .with_name("customer_support_bot")
    .with_llm(OpenAI::gpt4())
    .with_reasoning_effort(ReasoningEffort::High)
    .with_mcp_server("filesystem", "npx -y @modelcontextprotocol/server-filesystem /tmp")
    .with_mcp_server("brave-search", "npx -y @modelcontextprotocol/server-brave-search")
    .with_local_tool("calculator", calculator_fn)
    .with_database(mongodb_client)
    .build()?;

// Using it is trivial
let stream = agent.chat("What files are in /tmp?").await?;
```

**Gap:** Need `Agent` + `AgentBuilder` abstraction layer

---

#### 2. Pluggability System

**Missing:**
- Plugin/extension system
- Hot-reload of tools
- Shared MCP server registry
- Middleware for intercepting execution (logs, auth, metrics)

**Current:** Everything must be configured manually via low-level APIs

---

#### 3. Quick-Start Path

**What LangGraph has:**
```python
from langgraph import Agent
agent = Agent.from_config("config.yaml")
```

**What Praxis needs:**
```rust
let agent = Agent::from_config("praxis.toml")?;
```

Or even simpler:
```rust
let agent = Agent::quick_start()
    .with_mcp("filesystem")
    .build()?;
```

---

#### 4. Config File Support

**Missing:** Declarative configuration for agents

**Desired:**
```toml
[agent]
name = "customer_support"
model = "gpt-4"
reasoning_effort = "high"

[[mcp_servers]]
name = "filesystem"
command = "npx -y @modelcontextprotocol/server-filesystem"
args = ["/tmp"]

[[mcp_servers]]
name = "brave-search"
command = "npx -y @modelcontextprotocol/server-brave-search"
env = { BRAVE_API_KEY = "${BRAVE_API_KEY}" }

[database]
uri = "mongodb://localhost:27017"
name = "praxis"
```

---

## Proposed Architecture: Three-Layer System

```
┌─────────────────────────────────────────────────────────┐
│         DEVELOPER EXPERIENCE LAYER (New!)               │
│                                                         │
│  Crates:                                                │
│  - praxis-agent      (Agent, AgentBuilder)              │
│  - praxis-registry   (MCPRegistry, ToolRegistry)        │
│  - praxis-middleware (Logging, Auth, Retry, Metrics)    │
│  - praxis-templates  (RAG, Code, Support presets)       │
│                                                         │
│  Features:                                              │
│  - Fluent builder API                                   │
│  - Config file support (praxis.toml)                    │
│  - Agent templates                                      │
│  - Middleware system                                    │
│  - Quick-start helpers                                  │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│         CORE RUNTIME LAYER (Existing)                   │
│                                                         │
│  Crates:                                                │
│  - praxis-types      (GraphState, StreamEvent, etc)     │
│  - praxis-graph      (Graph, Node, Router)              │
│                                                         │
│  Architecture:                                          │
│  - Node abstraction (LLMNode, ToolNode)                 │
│  - Graph orchestrator (execution loop)                  │
│  - Router (flow decision)                               │
│  - StreamEvent model                                    │
│  - MessageAccumulator                                   │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│         INTEGRATION LAYER (Partial)                     │
│                                                         │
│  Crates:                                                │
│  - praxis-llm        (OpenAI, Azure, Anthropic clients) │
│  - praxis-mcp        (MCP protocol adapter)             │
│  - praxis-tools      (Local tool execution)             │
│  - praxis-db         (MongoDB persistence)              │
│  - praxis-gateway    (HTTP/SSE server)                  │
└─────────────────────────────────────────────────────────┘
```

---

## New Crate: `praxis-agent`

High-level API that wraps the core runtime with developer-friendly interface.

### Structure

```rust
// praxis-agent/src/lib.rs

/// High-level Agent abstraction
pub struct Agent {
    graph: Arc<Graph>,
    config: AgentConfig,
    tool_registry: Arc<ToolRegistry>,
    mcp_registry: Arc<MCPRegistry>,
}

/// Fluent builder for Agent
pub struct AgentBuilder {
    name: String,
    llm_config: LLMConfig,
    tools: Vec<Tool>,
    mcp_servers: Vec<MCPServerConfig>,
    middleware: Vec<Box<dyn Middleware>>,
    database: Option<DatabaseConfig>,
    graph_config: GraphConfig,
}

impl AgentBuilder {
    pub fn new(name: impl Into<String>) -> Self { ... }
    
    pub fn with_llm(mut self, llm: LLMConfig) -> Self {
        self.llm_config = llm;
        self
    }
    
    pub fn with_mcp_server(mut self, name: &str, command: &str) -> Self {
        self.mcp_servers.push(MCPServerConfig {
            name: name.to_string(),
            command: command.to_string(),
            ..Default::default()
        });
        self
    }
    
    pub fn with_local_tool<F>(mut self, name: &str, func: F) -> Self 
    where 
        F: Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value>> + Send + Sync + 'static
    {
        self.tools.push(Tool::new(name, func));
        self
    }
    
    pub fn with_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware.push(Box::new(middleware));
        self
    }
    
    pub fn with_database(mut self, config: DatabaseConfig) -> Self {
        self.database = Some(config);
        self
    }
    
    pub async fn build(self) -> Result<Agent> {
        // 1. Initialize LLM client
        let llm_client = self.llm_config.create_client()?;
        
        // 2. Initialize tool registry
        let tool_registry = ToolRegistry::new();
        for tool in self.tools {
            tool_registry.register(tool);
        }
        
        // 3. Initialize MCP registry
        let mcp_registry = MCPRegistry::new();
        for mcp_config in self.mcp_servers {
            mcp_registry.register(mcp_config).await?;
        }
        
        // 4. Create tool executor
        let tool_executor = ToolExecutor::new(
            Arc::clone(&tool_registry),
            Arc::clone(&mcp_registry),
        );
        
        // 5. Build Graph
        let graph = Graph::new(
            nodes: build_default_nodes(),
            router: Box::new(SimpleRouter::new()),
            config: self.graph_config,
            llm_client,
            tool_executor,
        );
        
        // 6. Wrap middleware
        let graph = wrap_with_middleware(graph, self.middleware);
        
        Ok(Agent {
            graph: Arc::new(graph),
            config: AgentConfig { name: self.name },
            tool_registry,
            mcp_registry,
        })
    }
}

impl Agent {
    /// Start building a new agent
    pub fn builder() -> AgentBuilder {
        AgentBuilder::default()
    }
    
    /// Quick-start with sensible defaults
    pub fn quick_start() -> AgentBuilder {
        AgentBuilder::new("quick_agent")
            .with_llm(LLMConfig::default_openai())
            .with_graph_config(GraphConfig::default())
    }
    
    /// Load agent from config file
    pub async fn from_config(path: impl AsRef<Path>) -> Result<Self> {
        let config = AgentConfigFile::load(path)?;
        config.to_builder().build().await
    }
    
    /// Load agent from TOML string
    pub async fn from_toml(toml: &str) -> Result<Self> {
        let config: AgentConfigFile = toml::from_str(toml)?;
        config.to_builder().build().await
    }
    
    /// Simple chat interface (no conversation history)
    pub async fn chat(&self, message: &str) -> Result<EventStream> {
        let conversation_id = Uuid::new_v4().to_string();
        self.chat_with_context(message, &conversation_id).await
    }
    
    /// Chat with conversation context (fetches history from DB)
    pub async fn chat_with_context(
        &self,
        message: &str,
        conversation_id: &str
    ) -> Result<EventStream> {
        let input = GraphInput {
            conversation_id: conversation_id.to_string(),
            last_message: Message::user(message),
            llm_config: self.config.llm_config.clone(),
            context_policy: ContextPolicy::default(),
        };
        
        let event_rx = self.graph.spawn_run(input);
        Ok(EventStream::new(event_rx))
    }
    
    /// Get tool registry (for inspection)
    pub fn tools(&self) -> &ToolRegistry {
        &self.tool_registry
    }
    
    /// Get MCP registry (for inspection)
    pub fn mcp_servers(&self) -> &MCPRegistry {
        &self.mcp_registry
    }
}
```

---

## New Crate: `praxis-registry`

Centralized registries for tools and MCP servers.

### MCPRegistry

```rust
// praxis-registry/src/mcp.rs

#[derive(Debug, Clone)]
pub struct MCPServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

pub struct MCPRegistry {
    servers: Arc<RwLock<HashMap<String, MCPServerHandle>>>,
}

impl MCPRegistry {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Global shared registry
    pub fn global() -> &'static MCPRegistry {
        static INSTANCE: OnceLock<MCPRegistry> = OnceLock::new();
        INSTANCE.get_or_init(|| MCPRegistry::new())
    }
    
    /// Register MCP server (spawns subprocess)
    pub async fn register(&self, config: MCPServerConfig) -> Result<()> {
        let handle = MCPServerHandle::spawn(config.clone()).await?;
        let mut servers = self.servers.write().await;
        servers.insert(config.name, handle);
        Ok(())
    }
    
    /// Get server by name
    pub async fn get(&self, name: &str) -> Option<MCPServerHandle> {
        let servers = self.servers.read().await;
        servers.get(name).cloned()
    }
    
    /// List all registered servers
    pub async fn list(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.keys().cloned().collect()
    }
    
    /// Unregister server (kills subprocess)
    pub async fn unregister(&self, name: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(handle) = servers.remove(name) {
            handle.shutdown().await?;
        }
        Ok(())
    }
}
```

### ToolRegistry

```rust
// praxis-registry/src/tools.rs

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register local tool
    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name().to_string(), Arc::new(tool));
    }
    
    /// Register function as tool
    pub async fn register_fn<F>(&self, name: &str, description: &str, func: F)
    where
        F: Fn(serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value>> + Send + Sync + 'static
    {
        let tool = FunctionTool::new(name, description, func);
        self.register(tool).await;
    }
    
    /// Get tool by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }
    
    /// List all tools
    pub async fn list(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }
}
```

---

## New Feature: Config File Support

### TOML Configuration

```toml
# praxis.toml

[agent]
name = "customer_support_bot"
model = "gpt-4"
reasoning_effort = "high"
max_iterations = 50
timeout_seconds = 300

[[mcp_servers]]
name = "filesystem"
command = "npx -y @modelcontextprotocol/server-filesystem"
args = ["/tmp"]

[[mcp_servers]]
name = "brave-search"
command = "npx -y @modelcontextprotocol/server-brave-search"
env = { BRAVE_API_KEY = "${BRAVE_API_KEY}" }

[[local_tools]]
name = "calculator"
type = "builtin"

[[local_tools]]
name = "custom_api"
type = "http"
url = "https://api.example.com/tool"
method = "POST"
headers = { Authorization = "Bearer ${API_KEY}" }

[database]
type = "mongodb"
uri = "mongodb://localhost:27017"
database = "praxis"

[http]
port = 8080
host = "0.0.0.0"
enable_cors = true
```

### Loading Config

```rust
// praxis-agent/src/config.rs

#[derive(Debug, Deserialize)]
pub struct AgentConfigFile {
    pub agent: AgentSection,
    pub mcp_servers: Vec<MCPServerSection>,
    pub local_tools: Vec<LocalToolSection>,
    pub database: Option<DatabaseSection>,
    pub http: Option<HttpSection>,
}

impl AgentConfigFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn to_builder(self) -> AgentBuilder {
        let mut builder = Agent::builder()
            .with_name(self.agent.name);
        
        // Add LLM config
        builder = builder.with_llm(LLMConfig {
            model: self.agent.model,
            reasoning_effort: self.agent.reasoning_effort,
            ..Default::default()
        });
        
        // Add MCP servers
        for mcp in self.mcp_servers {
            builder = builder.with_mcp_server(&mcp.name, &mcp.command);
        }
        
        // Add local tools
        for tool in self.local_tools {
            match tool.type_.as_str() {
                "builtin" => {
                    builder = builder.with_builtin_tool(&tool.name);
                }
                "http" => {
                    builder = builder.with_http_tool(&tool.name, tool.url.unwrap());
                }
                _ => {}
            }
        }
        
        // Add database
        if let Some(db) = self.database {
            builder = builder.with_database(db.into());
        }
        
        builder
    }
}
```

---

## New Feature: Agent Templates

Pre-configured agent setups for common use cases.

```rust
// praxis-templates/src/lib.rs

pub enum AgentTemplate {
    RAG,
    CodeAssistant,
    CustomerSupport,
    DataAnalyst,
    WebSearch,
}

impl AgentTemplate {
    pub fn builder(self) -> AgentBuilder {
        match self {
            Self::RAG => Self::rag_template(),
            Self::CodeAssistant => Self::code_template(),
            Self::CustomerSupport => Self::support_template(),
            Self::DataAnalyst => Self::analyst_template(),
            Self::WebSearch => Self::search_template(),
        }
    }
    
    fn rag_template() -> AgentBuilder {
        Agent::builder()
            .with_name("rag_assistant")
            .with_llm(OpenAI::gpt4())
            .with_reasoning_effort(ReasoningEffort::Medium)
            .with_local_tool("vector_search", vector_search_tool())
            .with_local_tool("document_retriever", doc_retriever_tool())
    }
    
    fn code_template() -> AgentBuilder {
        Agent::builder()
            .with_name("code_assistant")
            .with_llm(OpenAI::gpt4())
            .with_reasoning_effort(ReasoningEffort::High)
            .with_mcp_server("filesystem", "npx -y @modelcontextprotocol/server-filesystem .")
            .with_mcp_server("github", "npx -y @modelcontextprotocol/server-github")
    }
    
    fn support_template() -> AgentBuilder {
        Agent::builder()
            .with_name("customer_support")
            .with_llm(OpenAI::gpt4())
            .with_reasoning_effort(ReasoningEffort::Low)
            .with_local_tool("knowledge_base", kb_search_tool())
            .with_local_tool("create_ticket", ticket_tool())
            .with_local_tool("escalate", escalation_tool())
    }
}

// Usage:
let agent = AgentTemplate::CodeAssistant
    .builder()
    .with_custom_config(...)
    .build()
    .await?;
```

---

## New Feature: Middleware System

Extensibility via middleware pattern.

```rust
// praxis-middleware/src/lib.rs

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn before_execute(&self, ctx: &mut Context) -> Result<()>;
    async fn after_execute(&self, ctx: &mut Context, result: &ExecutionResult) -> Result<()>;
}

pub struct Context {
    pub conversation_id: String,
    pub run_id: String,
    pub start_time: Instant,
    pub user_metadata: HashMap<String, Value>,
}

// Example: Logging middleware
pub struct LoggingMiddleware {
    logger: slog::Logger,
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before_execute(&self, ctx: &mut Context) -> Result<()> {
        info!(self.logger, "Execution started";
            "run_id" => &ctx.run_id,
            "conversation_id" => &ctx.conversation_id
        );
        Ok(())
    }
    
    async fn after_execute(&self, ctx: &mut Context, result: &ExecutionResult) -> Result<()> {
        let duration = ctx.start_time.elapsed();
        info!(self.logger, "Execution completed";
            "run_id" => &ctx.run_id,
            "duration_ms" => duration.as_millis(),
            "status" => format!("{:?}", result.status)
        );
        Ok(())
    }
}

// Example: Retry middleware
pub struct RetryMiddleware {
    max_attempts: usize,
    backoff: Duration,
}

// Example: Auth middleware
pub struct AuthMiddleware {
    validator: Arc<dyn TokenValidator>,
}

// Example: Rate limit middleware
pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter>,
}

// Example: Metrics middleware
pub struct MetricsMiddleware {
    registry: prometheus::Registry,
}

// Usage:
let agent = Agent::builder()
    .with_middleware(LoggingMiddleware::new())
    .with_middleware(RetryMiddleware::with_max_attempts(3))
    .with_middleware(AuthMiddleware::new(validator))
    .with_middleware(MetricsMiddleware::new())
    .build()?;
```

---

## Updated Roadmap

### Phase 1: Core Runtime (2-3 weeks) ✅ ALREADY PLANNED
- [x] Node abstraction (design complete)
- [x] Graph orchestration (design complete)
- [x] StreamEvent model (design complete)
- [ ] Implement `praxis-types` crate
- [ ] Implement `praxis-graph` crate
- [ ] Implement `praxis-llm` crate (basic LLMClient trait)

### Phase 2: Integration (2-3 weeks) ✅ ALREADY PLANNED
- [ ] LLMClient real implementation (OpenAI, Azure)
- [ ] Basic ToolExecutor
- [ ] MCP protocol adapter
- [ ] MongoDB persistence layer

### Phase 3: **Developer Experience (3-4 weeks) 🆕 NEW!**
- [ ] **`praxis-agent` crate**
  - Agent struct (high-level wrapper)
  - AgentBuilder with fluent API
  - EventStream wrapper
- [ ] **`praxis-registry` crate**
  - MCPRegistry (spawn/manage MCP servers)
  - ToolRegistry (local tools + MCP unified)
- [ ] **Config file support**
  - AgentConfigFile (TOML deserialization)
  - Environment variable substitution
  - Validation & error messages
- [ ] **`praxis-templates` crate**
  - RAG assistant template
  - Code assistant template
  - Customer support template
- [ ] **`praxis-middleware` crate**
  - Middleware trait
  - LoggingMiddleware
  - RetryMiddleware
  - MetricsMiddleware

### Phase 4: Gateway & Examples (2-3 weeks)
- [ ] `praxis-gateway` crate (HTTP/SSE server)
- [ ] Example: Simple chatbot (10 lines)
- [ ] Example: RAG agent with vector search
- [ ] Example: Code assistant with GitHub MCP
- [ ] Example: Multi-agent system
- [ ] Comprehensive documentation

### Phase 5: Ecosystem & Polish (ongoing)
- [ ] CLI tool (`praxis-cli`)
  - `praxis new <name>` - scaffold new agent
  - `praxis run` - run agent from config
  - `praxis mcp list` - list MCP servers
  - `praxis mcp add <name>` - add MCP server
- [ ] Public MCP registry (curated list)
- [ ] Observability dashboard
- [ ] Performance benchmarks
- [ ] VS Code extension (optional)

---

## Design Principles

### 1. Developer Joy is Essential

**Goal:** Time to first working agent < 5 minutes

```rust
// This should work out of the box:
use praxis::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::quick_start()
        .with_mcp("filesystem")
        .build()
        .await?;
    
    let mut stream = agent.chat("What files are in the current directory?").await?;
    
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::Message { content } => print!("{}", content),
            _ => {}
        }
    }
    
    Ok(())
}
```

### 2. Progressive Disclosure

- **Simple:** Quick-start API for beginners
- **Intermediate:** Builder API with common options
- **Advanced:** Direct Graph API for full control

Users can graduate through levels as they need more control.

### 3. Sensible Defaults

- Model: GPT-4 (or best available)
- Reasoning: Medium effort
- Timeout: 5 minutes
- Max iterations: 50
- Database: In-memory (for development)

### 4. Documentation Driven by Use Cases

**Good docs:**
- ✅ "How to build a RAG chatbot in 5 minutes"
- ✅ "How to plug any MCP server"
- ✅ "How to scale to 1M users"
- ✅ "How to add custom tools"

**Less priority:**
- ⚠️ "Node trait architecture explanation" (advanced docs)
- ⚠️ "Graph execution internals" (advanced docs)

---

## Success Metrics

### Adoption Metrics
- **Time to first agent:** < 5 minutes
- **Lines of code for simple agent:** < 15 lines
- **Documentation clarity:** Non-Rust devs can use it
- **GitHub stars:** Aim for 1K+ in first 6 months

### Technical Metrics
- **Performance:** < 100ms overhead vs direct LLM call
- **Scalability:** Handle 10K concurrent agents per server
- **Reliability:** 99.9% uptime
- **Memory:** < 10MB per active agent

### Community Metrics
- **Examples:** 20+ example agents
- **MCP servers:** 50+ in registry
- **Contributors:** 20+ active contributors
- **Blog posts:** 10+ community tutorials

---

## Comparison: LangGraph vs Praxis

| Feature | LangGraph (Python) | Praxis (Rust) |
|---------|-------------------|---------------|
| **Performance** | ⚠️ Python overhead | ✅ Native Rust speed |
| **Concurrency** | ⚠️ GIL limitations | ✅ Tokio async (no GIL) |
| **Memory** | ⚠️ High (Python) | ✅ Low (Rust ownership) |
| **Type Safety** | ⚠️ Runtime errors | ✅ Compile-time checks |
| **Developer UX** | ✅ Simple API | 🎯 **Working on it!** |
| **MCP Support** | ✅ Built-in | 🎯 **In roadmap** |
| **Streaming** | ✅ SSE support | ✅ SSE + real-time events |
| **Scalability** | ⚠️ Limited (Python) | ✅ Millions of users |
| **Ecosystem** | ✅ Mature | 🚧 Building |

**Praxis advantage:** Performance + Scalability  
**LangGraph advantage:** Ecosystem maturity  
**Praxis goal:** Match LangGraph's DX, exceed in performance

---

## Next Steps

### Immediate Actions

1. **Review this checkpoint** with team/community
2. **Prioritize Phase 3** (Developer Experience layer)
3. **Start with `praxis-agent` crate** (highest impact)
4. **Create first example** that showcases simplicity

### Questions to Answer

1. Should we support YAML config in addition to TOML?
2. Should templates be in separate crate or built into `praxis-agent`?
3. Do we need a plugin system beyond middleware?
4. Should there be a visual editor (like LangGraph Studio)?

---

## Reflection: Framework Success Formula

**Successful frameworks have:**

1. ✅ **Solid technical foundation** ← Praxis HAS this
2. ✅ **Developer joy** ← Praxis NEEDS this (Phase 3)
3. ✅ **Great documentation** ← Praxis NEEDS this (Phase 4)
4. ✅ **Active community** ← Praxis will BUILD this (ongoing)

**Examples:**
- **React:** Fast (Virtual DOM) + Joy (JSX, hooks)
- **FastAPI:** Fast (async) + Joy (decorators, auto docs)
- **LangGraph:** OK perf + Joy (visual editor, simple API)

**Praxis formula:**
- **Fastest** (Rust, async, stateless) ← Already achieved
- **Joyful** (builder, config files, templates) ← Phase 3 target
- **Scalable** (millions of users) ← Architecture supports it
- **Observable** (streaming events, metrics) ← Built-in

---

**End of Checkpoint 4: Developer Experience & High-Level API**


