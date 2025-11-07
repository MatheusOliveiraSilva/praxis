# Architecture Checkpoint 5: MCP-Native Design

**Date:** 2025-11-07  
**Status:** ✅ Architecture Implemented, SDK Integration in Progress  
**Related:** [MCP Migration Guide](../MCP_MIGRATION.md)

---

## Context

### The Problem

Initial implementation had tools hardcoded in the application:

```rust
// ❌ Tools implemented in application code
pub struct MockToolExecutor;

impl ToolExecutor for MockToolExecutor {
    async fn execute(&self, tool_name: &str, args: &str) -> Result<String> {
        match tool_name {
            "calculator" => Ok("42"),
            "weather" => Ok("sunny"),
            // ... more hardcoded tools
        }
    }
}
```

**Issues:**
1. ❌ Tools coupled to application code
2. ❌ Need to recompile/redeploy to change tools
3. ❌ Can't leverage existing MCP ecosystem
4. ❌ Violates separation of concerns
5. ❌ Hard to test tools independently

### The Insight

**Tools should come from MCP servers, not application code.**

Praxis should be a **MCP client** that orchestrates agents, not a tool implementation platform.

---

## Design Decision

### Core Principle

> **Praxis is an AI agent runtime, not a tool library.**  
> Tools are provided by external MCP servers that follow the Model Context Protocol.

### Architecture Change

**Before:**
```
┌────────────────────────┐
│   Praxis Agent         │
│                        │
│  ┌─────────────────┐  │
│  │  Graph          │  │
│  │  - LLMNode      │  │
│  │  - ToolNode     │  │
│  └─────────────────┘  │
│          ↓             │
│  ┌─────────────────┐  │
│  │ MockToolExecutor│  │ ❌ Tools hardcoded
│  │  calculator()   │  │
│  │  weather()      │  │
│  └─────────────────┘  │
└────────────────────────┘
```

**After:**
```
┌──────────────────────────────────────────────┐
│         Praxis Agent (Rust)                  │
│                                              │
│  ┌────────────────────────────────────────┐ │
│  │   praxis-graph (Core Runtime)          │ │
│  │   - LLMNode                             │ │
│  │   - ToolNode                            │ │
│  │   - Graph Orchestrator                  │ │
│  └────────────────────────────────────────┘ │
│                    ↓                         │
│  ┌────────────────────────────────────────┐ │
│  │   praxis-mcp (MCP Client) 🆕           │ │
│  │   - MCPClient                           │ │
│  │   - MCPToolExecutor                     │ │
│  └────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
            ↓         ↓         ↓
  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
  │ MCP Server   │ │ MCP Server   │ │ MCP Server   │
  │  (Python)    │ │ (TypeScript) │ │   (Rust)     │
  │              │ │              │ │              │
  │  weather     │ │ filesystem   │ │   custom     │
  │  forecast    │ │ file_read    │ │   tools      │
  └──────────────┘ └──────────────┘ └──────────────┘
```

---

## Implementation

### New Crate: `praxis-mcp`

**Location:** `crates/praxis-mcp/`

**Purpose:** Native MCP client integration for Praxis

**Components:**

#### 1. MCPClient

Manages connection to a single MCP server.

```rust
pub struct MCPClient {
    server_name: String,
    // Connection to MCP server via rmcp SDK
}

impl MCPClient {
    /// Connect to an MCP server
    pub async fn new(
        server_name: impl Into<String>,
        command: impl AsRef<str>,
        args: Vec<&str>,
    ) -> Result<Self>;

    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>>;

    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: Value) 
        -> Result<Vec<ToolResponse>>;
}
```

**Example:**
```rust
// Connect to Python weather server
let client = MCPClient::new(
    "weather",
    "python3",
    vec!["weather_server.py"]
).await?;

// List tools
let tools = client.list_tools().await?;
// => ["get_weather", "get_forecast"]

// Call tool
let result = client.call_tool(
    "get_weather",
    json!({"location": "San Francisco"})
).await?;
```

#### 2. MCPToolExecutor

Manages multiple MCP servers and routes tool calls.

```rust
pub struct MCPToolExecutor {
    clients: Arc<RwLock<HashMap<String, Arc<MCPClient>>>>,
}

impl MCPToolExecutor {
    pub fn new() -> Self;
    
    /// Add an MCP server
    pub async fn add_server(&self, client: MCPClient) -> Result<()>;
    
    /// List all tools from all servers
    pub async fn list_all_tools(&self) 
        -> Result<Vec<(String, Vec<ToolInfo>)>>;
}
```

**Example:**
```rust
let executor = MCPToolExecutor::new();

// Add multiple servers
executor.add_server(weather_client).await?;
executor.add_server(fs_client).await?;
executor.add_server(github_client).await?;

// All tools available automatically
let all_tools = executor.list_all_tools().await?;
```

### Example MCP Server (Python)

**Location:** `mcp_servers/weather_server.py`

A complete MCP server in ~150 lines of Python:

```python
#!/usr/bin/env python3
from mcp_server import MCPServer

server = MCPServer("weather")

@server.tool(
    name="get_weather",
    description="Get current weather",
    schema={...}
)
async def get_weather(args):
    location = args["location"]
    return json.dumps({
        "location": location,
        "temperature": 22,
        "condition": "sunny"
    })

server.run()  # stdio JSON-RPC
```

### Deprecation Strategy

**File:** `crates/praxis-graph/src/tools.rs`

```rust
#[deprecated(
    since = "0.2.0",
    note = "Use praxis_mcp::MCPClient and MCPToolExecutor instead. \
            Tools should come from MCP servers."
)]
pub trait ToolExecutor: Send + Sync {
    async fn execute(&self, tool_name: &str, arguments: &str) -> Result<String>;
    fn list_tools(&self) -> Vec<String>;
}

#[deprecated(
    since = "0.2.0",
    note = "Use real MCP servers instead. \
            See praxis-mcp crate and mcp_servers/ directory."
)]
pub struct MockToolExecutor;
```

**Result:** Code still compiles but emits warnings, guiding developers to migrate.

---

## Benefits

### 1. Separation of Concerns

| Layer | Responsibility |
|-------|----------------|
| **Praxis** | Agent runtime, orchestration, streaming |
| **MCP Servers** | Business logic, tool implementation |

### 2. Language Agnostic

Tools can be written in **any language**:
- Python (easiest for data science/ML)
- TypeScript (web APIs, Node ecosystem)
- Rust (performance-critical tools)
- Go, Java, etc.

### 3. Hot Reload

Restart MCP server → new tools available  
**No need to recompile/redeploy Praxis!**

### 4. Ecosystem Access

Leverage the entire MCP ecosystem:
- Official MCP servers (filesystem, GitHub, Slack, etc.)
- Community servers
- Custom internal tools

### 5. Fault Isolation

MCP server crashes → doesn't bring down agent  
Agent can reconnect or use different server

### 6. Testability

Test tools independently:
```bash
# Test MCP server directly
python test_weather_server.py

# Test Praxis with mock MCP responses
cargo test --package praxis-mcp
```

---

## Migration Path

### Phase 1: ✅ Architecture (COMPLETE)

- [x] Create `praxis-mcp` crate
- [x] Implement `MCPClient` (stub)
- [x] Implement `MCPToolExecutor`
- [x] Deprecate `MockToolExecutor`
- [x] Create example Python MCP server

### Phase 2: 🚧 SDK Integration (IN PROGRESS)

- [ ] Complete rmcp SDK integration
- [ ] Handle stdio/HTTP/WebSocket transports
- [ ] Add connection pooling
- [ ] Add error recovery

### Phase 3: 📋 Graph Integration (NEXT)

- [ ] Make `ToolNode` use `MCPToolExecutor`
- [ ] Make `LLMNode` fetch tool schemas from MCP
- [ ] Update examples to use MCP
- [ ] Remove `MockToolExecutor`

### Phase 4: 📋 DX Enhancements (FUTURE)

- [ ] `MCPRegistry` for server discovery
- [ ] Config file support (`praxis.toml`)
- [ ] MCP server templates
- [ ] CLI tools for server management

---

## API Examples

### Basic Usage

```rust
use praxis_mcp::{MCPClient, MCPToolExecutor};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup
    let executor = MCPToolExecutor::new();
    
    // Add servers
    let weather = MCPClient::new(
        "weather",
        "python3",
        vec!["servers/weather.py"]
    ).await?;
    executor.add_server(weather).await?;
    
    let fs = MCPClient::new(
        "filesystem",
        "npx",
        vec!["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    ).await?;
    executor.add_server(fs).await?;
    
    // Use in agent
    let graph = Graph::new(llm_client, executor, config);
    let result = graph.run(input).await?;
    
    Ok(())
}
```

### With Config File (Future)

```toml
# praxis.toml
[mcp.weather]
command = "python3"
args = ["servers/weather.py"]

[mcp.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]

[mcp.github]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
env = { GITHUB_TOKEN = "${GITHUB_TOKEN}" }
```

```rust
// Load from config
let config = PraxisConfig::load("praxis.toml")?;
let executor = config.build_mcp_executor().await?;
```

---

## MCP Server Ecosystem

### Official Servers

Available via npx:

```bash
# Filesystem operations
npx -y @modelcontextprotocol/server-filesystem /path

# Brave Search
npx -y @modelcontextprotocol/server-brave-search

# GitHub
npx -y @modelcontextprotocol/server-github

# Google Drive
npx -y @modelcontextprotocol/server-gdrive

# Slack
npx -y @modelcontextprotocol/server-slack
```

### Custom Servers

Create in minutes:

**Python:**
```python
from mcp.server import Server

app = Server("my-tools")

@app.list_tools()
async def list_tools():
    return [Tool(name="my_tool", ...)]

@app.call_tool()
async def call_tool(name, args):
    return [TextContent(text="result")]
```

**Rust (using rmcp):**
```rust
use rmcp::*;

#[tokio::main]
async fn main() {
    let server = MyServer::new()
        .serve((stdin(), stdout()))
        .await
        .unwrap();
}
```

---

## Security Considerations

### Process Isolation

Each MCP server runs in its own process:
- ✅ Server crash doesn't affect agent
- ✅ Resource limits per server
- ✅ Can use Docker/containers for isolation

### Capability Model

MCP servers declare capabilities:
```json
{
  "capabilities": {
    "tools": {},
    "resources": { "subscribe": true },
    "prompts": {}
  }
}
```

Praxis only exposes what it needs.

### Authentication

Future: MCP servers can require authentication:
```rust
let client = MCPClient::new("secure-tools", "python3", args)
    .with_auth(AuthToken::from_env("MCP_TOKEN"))
    .await?;
```

---

## Performance Considerations

### Process Spawning

**Concern:** Spawning processes is slow  
**Mitigation:** 
- Keep MCP servers running (don't spawn per request)
- Connection pooling
- Optional HTTP transport for remote servers

### IPC Overhead

**Concern:** stdio/JSON-RPC overhead  
**Mitigation:**
- Batch tool calls when possible
- Use HTTP/WebSocket for high-throughput scenarios
- MCP protocol is designed for efficiency

### Benchmarks (TODO)

Measure:
- Time to spawn MCP server
- Latency per tool call
- Throughput (calls/sec)
- Memory overhead

---

## Future Enhancements

### 1. MCP Server Registry

Central registry of available servers:

```rust
let registry = MCPRegistry::new()
    .discover_from_env()     // $MCP_SERVERS_PATH
    .discover_from_config()  // praxis.toml
    .discover_from_npm();    // npx packages

let executor = registry.build_executor().await?;
```

### 2. Tool Decorators (Low Priority)

Python-style decorators for inline tools:

```rust
#[tool(name = "calculator", description = "Math")]
async fn calculator(expr: String) -> Result<f64> {
    // implementation
}
```

Internally, this spawns a mini MCP server.

### 3. Remote MCP Servers

Connect to MCP servers over HTTP/WebSocket:

```rust
let client = MCPClient::connect_http(
    "https://api.example.com/mcp"
).await?;
```

### 4. MCP Server Marketplace

Discover and install MCP servers:

```bash
praxis mcp search weather
praxis mcp install @modelcontextprotocol/server-brave-search
praxis mcp list
```

---

## References

- [MCP Specification](https://modelcontextprotocol.io)
- [Rust MCP SDK (rmcp)](https://github.com/modelcontextprotocol/rust-sdk)
- [Official MCP Servers](https://github.com/modelcontextprotocol/servers)
- [Migration Guide](../MCP_MIGRATION.md)
- [praxis-mcp README](../crates/praxis-mcp/README.md)

---

## Summary

**Key Changes:**

1. ✅ Created `praxis-mcp` crate for MCP client functionality
2. ✅ Implemented `MCPClient` and `MCPToolExecutor`
3. ✅ Deprecated `MockToolExecutor` with clear migration path
4. ✅ Created example Python MCP server
5. ✅ Documented architecture and benefits

**Philosophy:**

> Tools should come from MCP servers, not application code.  
> Praxis is an agent runtime, not a tool library.

**Status:**

- Architecture: ✅ Complete
- SDK Integration: 🚧 In Progress
- Graph Integration: 📋 Next
- DX Layer: 📋 Future

**Next Steps:**

1. Complete rmcp SDK integration in `MCPClient`
2. Refactor `ToolNode` to use `MCPToolExecutor`
3. Update examples and documentation
4. Remove deprecated code in v1.0

