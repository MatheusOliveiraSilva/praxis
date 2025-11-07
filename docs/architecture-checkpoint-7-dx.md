# Architecture Checkpoint 7: Developer Experience Layer (praxis-dx)

**Status**: 📋 Ready for Implementation  
**Date**: 2025-11-07  
**Phase**: Developer Tools & Ergonomics

---

## Overview

The **praxis-dx** crate provides developer experience tools to make Praxis easy to use, configure, and deploy. It includes a CLI tool, configuration file support, and ergonomic builder patterns.

### Goals

1. **Easy onboarding**: `praxis init` scaffolds new projects in seconds
2. **Config-driven**: TOML files for declarative agent configuration
3. **Type-safe builders**: Ergonomic Rust APIs with compile-time validation
4. **Hot reload**: Local development with file watching
5. **Modular middleware**: Pluggable logging, metrics, rate limiting

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Developer Workflow                      │
│                                                             │
│  1. praxis init my-agent     → scaffolds project           │
│  2. Edit praxis.toml          → configure agent            │
│  3. praxis dev                → runs with hot-reload       │
│  4. praxis config validate    → checks configuration       │
│  5. praxis deploy             → builds production image    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      praxis-dx (CLI)                        │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Commands                                            │  │
│  │  - init       → scaffold new project                 │  │
│  │  - dev        → run with hot-reload                  │  │
│  │  - config     → validate/show config                 │  │
│  │  - mcp        → manage MCP servers                   │  │
│  │  - deploy     → build Docker image                   │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Config Loader (praxis.toml)                         │  │
│  │  - Parse TOML                                        │  │
│  │  - Environment variable substitution                 │  │
│  │  - Validation                                        │  │
│  │  - Defaults                                          │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  AgentBuilder (Programmatic API)                     │  │
│  │  - Fluent API                                        │  │
│  │  - Type-safe construction                            │  │
│  │  - Config file integration                           │  │
│  └──────────────────────────────────────────────────────┘  │
│                              │                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Middleware Registry (Hooks)                         │  │
│  │  - Logging middleware                                │  │
│  │  - Metrics middleware (future)                       │  │
│  │  - Rate limiting (future)                            │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌─────────────────────────────────────────────────────────────┐
│              praxis-api + praxis-graph + praxis-mcp         │
│                                                             │
│  - Runtime components                                       │
│  - Configured via praxis-dx                                 │
└─────────────────────────────────────────────────────────────┘
```

---

## CLI Tool

### Command Structure

```bash
praxis [OPTIONS] <COMMAND>

Commands:
  init        Initialize a new Praxis project
  dev         Run agent in development mode with hot-reload
  config      Validate and display configuration
  mcp         Manage MCP servers
  deploy      Build production Docker image
  help        Print this message or the help of the given subcommand(s)

Options:
  -c, --config <FILE>    Config file path [default: praxis.toml]
  -v, --verbose          Verbose logging
  -h, --help             Print help
  -V, --version          Print version
```

---

### 1. `praxis init`

Scaffold a new Praxis project with sensible defaults.

```bash
praxis init my-weather-agent
```

**Generated Structure:**
```
my-weather-agent/
├── Cargo.toml
├── praxis.toml              # Agent configuration
├── src/
│   ├── main.rs              # Entry point using praxis-dx
│   └── lib.rs               # Optional custom logic
├── .env.example             # Environment variable template
├── Dockerfile               # Production container
└── README.md                # Quick start guide
```

**Generated `praxis.toml`:**
```toml
[agent]
name = "my-weather-agent"
description = "AI agent for weather queries"

[llm]
provider = "openai"
model = "gpt-4"
temperature = 0.7
max_tokens = 4096
api_key = "${OPENAI_API_KEY}"  # Environment variable substitution

[mcp]
# Comma-separated MCP server URLs
servers = "http://localhost:8000/mcp"

[system]
prompt = """
You are a helpful weather assistant.
Use the available tools to answer user questions about weather.
"""

[server]
host = "0.0.0.0"
port = 3000

[database]
uri = "${MONGODB_URI}"
database = "praxis"

[execution]
max_iterations = 10
request_timeout_secs = 30

[logging]
level = "info"
format = "json"  # "json" or "pretty"
```

**Generated `main.rs`:**
```rust
use praxis_dx::{AgentBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Load config from praxis.toml
    let agent = AgentBuilder::from_config("praxis.toml")?
        .build()
        .await?;
    
    // Start HTTP server
    agent.serve().await?;
    
    Ok(())
}
```

**Options:**
```bash
praxis init [OPTIONS] <NAME>

Arguments:
  <NAME>  Project name

Options:
  --template <TEMPLATE>  Use template [default: basic] [possible: basic, advanced, minimal]
  --no-git               Don't initialize git repository
  -h, --help             Print help
```

---

### 2. `praxis dev`

Run agent in development mode with hot-reload and pretty logging.

```bash
praxis dev
```

**Behavior:**
- Loads `praxis.toml` from current directory
- Watches for file changes (`.toml`, `.rs`)
- Auto-restarts on changes
- Pretty-printed logs (not JSON)
- Opens browser to health check endpoint

**Options:**
```bash
praxis dev [OPTIONS]

Options:
  -c, --config <FILE>    Config file [default: praxis.toml]
  -p, --port <PORT>      Override server port
  --no-watch             Disable hot-reload
  -h, --help             Print help
```

**Example Output:**
```
🚀 Praxis Development Server
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Agent:      my-weather-agent
 Config:     praxis.toml
 Server:     http://localhost:3000
 Health:     http://localhost:3000/v1/health
 Hot-reload: enabled
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[INFO] Connecting to MCP servers...
[INFO]   ✓ http://localhost:8000/mcp (2 tools)
[INFO] Database connection established
[INFO] Server listening on 0.0.0.0:3000

Watching for changes...
```

---

### 3. `praxis config`

Validate and display configuration.

```bash
praxis config validate
praxis config show
```

**`praxis config validate`:**
```bash
$ praxis config validate

✓ Configuration is valid

Agent: my-weather-agent
LLM: openai/gpt-4
MCP Servers: 1
  - http://localhost:8000/mcp
Database: mongodb://localhost:27017/praxis
```

**`praxis config show`:**
```bash
$ praxis config show

# Resolved configuration (with env vars substituted)
{
  "agent": {
    "name": "my-weather-agent",
    "description": "AI agent for weather queries"
  },
  "llm": {
    "provider": "openai",
    "model": "gpt-4",
    "temperature": 0.7,
    "max_tokens": 4096,
    "api_key": "sk-***************" # Redacted
  },
  ...
}
```

**Options:**
```bash
praxis config <COMMAND>

Commands:
  validate    Validate configuration file
  show        Display resolved configuration
  help        Print this message

Options:
  -c, --config <FILE>    Config file [default: praxis.toml]
  --no-redact            Don't redact sensitive values
  -h, --help             Print help
```

---

### 4. `praxis mcp`

Manage MCP servers.

```bash
praxis mcp list
praxis mcp add <URL>
praxis mcp remove <URL>
praxis mcp test <URL>
```

**`praxis mcp list`:**
```bash
$ praxis mcp list

MCP Servers (from praxis.toml):
  - http://localhost:8000/mcp
  - http://localhost:8001/mcp
```

**`praxis mcp add`:**
```bash
$ praxis mcp add http://localhost:8002/mcp

✓ Added MCP server: http://localhost:8002/mcp
Updated praxis.toml
```

**`praxis mcp test`:**
```bash
$ praxis mcp test http://localhost:8000/mcp

Testing connection to http://localhost:8000/mcp...
✓ Connected successfully

Available tools:
  - get_forecast (Get weather forecast for a location)
  - get_alerts (Get weather alerts for a location)

Total: 2 tools
```

---

### 5. `praxis deploy`

Build production-ready Docker image.

```bash
praxis deploy
```

**Behavior:**
- Validates configuration
- Builds optimized release binary
- Creates Docker image
- Tags with version from `Cargo.toml`

**Options:**
```bash
praxis deploy [OPTIONS]

Options:
  --tag <TAG>           Docker image tag [default: latest]
  --platform <ARCH>     Target platform [default: linux/amd64]
  --push                Push to registry after build
  --registry <URL>      Docker registry URL
  -h, --help            Print help
```

**Example:**
```bash
$ praxis deploy --tag v1.0.0 --push --registry ghcr.io/myorg

Building production image...
  ✓ Cargo build --release completed (45s)
  ✓ Docker build completed (12s)
  ✓ Tagged: ghcr.io/myorg/my-weather-agent:v1.0.0
  ✓ Pushed to registry

Image: ghcr.io/myorg/my-weather-agent:v1.0.0
Size: 58MB
```

---

## Configuration File (praxis.toml)

### Full Schema

```toml
# ============================================
# Agent Configuration
# ============================================
[agent]
name = "my-agent"
description = "Agent description"
version = "1.0.0"

# ============================================
# LLM Provider
# ============================================
[llm]
provider = "openai"  # "openai", "azure", "anthropic" (future)
model = "gpt-4"
temperature = 0.7
max_tokens = 4096
api_key = "${OPENAI_API_KEY}"

# Optional: Azure OpenAI
# [llm.azure]
# endpoint = "${AZURE_OPENAI_ENDPOINT}"
# deployment = "gpt-4-deployment"

# ============================================
# MCP Servers
# ============================================
[mcp]
# Comma-separated URLs (parsed at startup)
servers = "http://localhost:8000/mcp,http://localhost:8001/mcp"

# Optional: Per-server configuration
# [[mcp.server]]
# url = "http://localhost:8000/mcp"
# name = "weather"
# timeout_secs = 10

# ============================================
# System Prompt
# ============================================
[system]
prompt = """
You are a helpful AI assistant.
Use the available tools to answer user questions.
Be concise and accurate.
"""

# ============================================
# HTTP Server
# ============================================
[server]
host = "0.0.0.0"
port = 3000

# Optional: TLS
# [server.tls]
# cert_path = "/path/to/cert.pem"
# key_path = "/path/to/key.pem"

# ============================================
# Database
# ============================================
[database]
uri = "${MONGODB_URI}"
database = "praxis"

# Optional: Connection pool
# [database.pool]
# min_connections = 5
# max_connections = 20

# ============================================
# Execution
# ============================================
[execution]
max_iterations = 10
request_timeout_secs = 30

# ============================================
# Context Management
# ============================================
[context]
# Max tokens before summarization kicks in
max_tokens = 8000

# Summarization prompt template
summary_prompt = """
Summarize the following conversation concisely:
{conversation}
"""

# ============================================
# Logging
# ============================================
[logging]
level = "info"  # "trace", "debug", "info", "warn", "error"
format = "json"  # "json" or "pretty"

# Optional: File output
# [logging.file]
# path = "/var/log/praxis/app.log"
# rotation = "daily"  # "daily", "hourly", "size"

# ============================================
# CORS
# ============================================
[cors]
origins = ["http://localhost:3000", "https://app.example.com"]
# allow_all = false  # Set to true for development

# ============================================
# Middleware (modular hooks)
# ============================================
[middleware]
# Enable/disable middleware
logging = true
# metrics = false  # Future: Prometheus metrics
# rate_limiting = false  # Future: rate limits

# ============================================
# Development
# ============================================
[dev]
hot_reload = true
pretty_logs = true
```

### Environment Variable Substitution

Variables in format `${VAR_NAME}` are replaced at load time:

```toml
[llm]
api_key = "${OPENAI_API_KEY}"

[database]
uri = "${MONGODB_URI:-mongodb://localhost:27017}"  # With default
```

### Validation Rules

- `agent.name`: Required, alphanumeric + hyphens
- `llm.model`: Required, valid model name
- `llm.temperature`: 0.0 - 2.0
- `llm.max_tokens`: > 0
- `mcp.servers`: Valid URLs or empty
- `server.port`: 1-65535
- `execution.max_iterations`: > 0

---

## AgentBuilder API

Programmatic API for building agents without config files.

### Basic Usage

```rust
use praxis_dx::AgentBuilder;

let agent = AgentBuilder::new("my-agent")
    .llm("gpt-4")
    .temperature(0.7)
    .max_tokens(4096)
    .mcp_servers("http://localhost:8000/mcp,http://localhost:8001/mcp")
    .system_prompt("You are a helpful assistant.")
    .mongodb("mongodb://localhost:27017", "praxis")
    .build()
    .await?;

// Start server
agent.serve().await?;
```

### Advanced Usage

```rust
use praxis_dx::{AgentBuilder, LLMProvider, MiddlewareConfig};

let agent = AgentBuilder::new("advanced-agent")
    // LLM configuration
    .llm_provider(LLMProvider::OpenAI {
        model: "gpt-4".to_string(),
        api_key: env::var("OPENAI_API_KEY")?,
        temperature: Some(0.7),
        max_tokens: Some(4096),
    })
    
    // MCP servers (parsed from comma-separated string)
    .mcp_servers("http://localhost:8000/mcp,http://localhost:8001/mcp")
    
    // System prompt
    .system_prompt_file("prompts/system.txt")?
    
    // Database
    .mongodb("mongodb://localhost:27017", "praxis")
    
    // Server configuration
    .bind("0.0.0.0", 3000)
    
    // Execution limits
    .max_iterations(10)
    .request_timeout(Duration::from_secs(30))
    
    // Middleware (modular hooks)
    .middleware(MiddlewareConfig {
        logging: true,
        metrics: false,
        rate_limiting: false,
    })
    
    // CORS
    .cors_origins(vec!["http://localhost:3000"])
    
    // Build
    .build()
    .await?;
```

### From Config File

```rust
use praxis_dx::AgentBuilder;

// Load from praxis.toml
let agent = AgentBuilder::from_config("praxis.toml")?
    .build()
    .await?;

// Override specific values
let agent = AgentBuilder::from_config("praxis.toml")?
    .port(8080)  // Override port
    .llm("gpt-4-turbo")  // Override model
    .build()
    .await?;
```

### Builder Pattern

```rust
pub struct AgentBuilder {
    name: String,
    llm_config: Option<LLMConfig>,
    mcp_servers: Option<String>,  // Comma-separated
    system_prompt: Option<String>,
    db_config: Option<DatabaseConfig>,
    server_config: ServerConfig,
    execution_config: ExecutionConfig,
    middleware_config: MiddlewareConfig,
    cors_origins: Vec<String>,
}

impl AgentBuilder {
    pub fn new(name: impl Into<String>) -> Self { ... }
    
    pub fn from_config(path: impl AsRef<Path>) -> Result<Self> { ... }
    
    // LLM
    pub fn llm(mut self, model: impl Into<String>) -> Self { ... }
    pub fn temperature(mut self, temp: f32) -> Self { ... }
    pub fn max_tokens(mut self, tokens: u32) -> Self { ... }
    
    // MCP (parses comma-separated string)
    pub fn mcp_servers(mut self, servers: impl Into<String>) -> Self { ... }
    
    // System prompt
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self { ... }
    pub fn system_prompt_file(mut self, path: impl AsRef<Path>) -> Result<Self> { ... }
    
    // Database
    pub fn mongodb(mut self, uri: impl Into<String>, db: impl Into<String>) -> Self { ... }
    
    // Server
    pub fn bind(mut self, host: impl Into<String>, port: u16) -> Self { ... }
    pub fn port(mut self, port: u16) -> Self { ... }
    
    // Execution
    pub fn max_iterations(mut self, max: usize) -> Self { ... }
    pub fn request_timeout(mut self, timeout: Duration) -> Self { ... }
    
    // Middleware
    pub fn middleware(mut self, config: MiddlewareConfig) -> Self { ... }
    
    // CORS
    pub fn cors_origins(mut self, origins: Vec<String>) -> Self { ... }
    
    // Build (async - connects to DB, MCP servers)
    pub async fn build(self) -> Result<Agent> { ... }
}
```

---

## Middleware System

Modular middleware hooks for extensibility.

### Current Middleware

#### 1. Logging Middleware

Already implemented via `tower-http` TraceLayer.

```rust
// Enabled by default in praxis.toml
[middleware]
logging = true
```

Logs:
- Request start/end
- Request ID
- Duration
- Status code
- Error details

---

### Future Middleware (Modular Hooks)

#### 2. Metrics Middleware (Future)

Export Prometheus metrics.

```rust
// Enable in praxis.toml
[middleware]
metrics = true

[middleware.metrics]
endpoint = "/metrics"
```

**Metrics:**
- `praxis_requests_total{method, endpoint, status}`
- `praxis_request_duration_seconds{method, endpoint}`
- `praxis_graph_execution_duration_seconds`
- `praxis_llm_tokens_used{model, type}`
- `praxis_tool_calls_total{tool_name, status}`

---

#### 3. Rate Limiting Middleware (Future)

Per-user or per-API-key rate limits.

```rust
// Enable in praxis.toml
[middleware]
rate_limiting = true

[middleware.rate_limiting]
requests_per_minute = 60
burst = 10
```

**Implementation:**
- Token bucket algorithm
- Redis-backed (distributed)
- Configurable per endpoint

---

### Middleware Registration

```rust
pub trait Middleware: Send + Sync {
    fn name(&self) -> &str;
    
    async fn before_request(&self, req: &Request) -> Result<()>;
    
    async fn after_request(&self, req: &Request, res: &Response) -> Result<()>;
    
    async fn on_error(&self, req: &Request, error: &ApiError) -> Result<()>;
}

pub struct MiddlewareRegistry {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl MiddlewareRegistry {
    pub fn new() -> Self { ... }
    
    pub fn register(&mut self, middleware: Box<dyn Middleware>) { ... }
    
    pub async fn run_before(&self, req: &Request) -> Result<()> { ... }
    
    pub async fn run_after(&self, req: &Request, res: &Response) -> Result<()> { ... }
}
```

**Usage in AgentBuilder:**
```rust
let agent = AgentBuilder::new("my-agent")
    .middleware(MiddlewareConfig {
        logging: true,
        metrics: true,
        rate_limiting: false,
    })
    .build()
    .await?;
```

---

## Crate Structure

```
praxis-dx/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API
│   ├── main.rs                 # CLI binary
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── init.rs             # praxis init
│   │   ├── dev.rs              # praxis dev
│   │   ├── config.rs           # praxis config
│   │   ├── mcp.rs              # praxis mcp
│   │   └── deploy.rs           # praxis deploy
│   ├── config/
│   │   ├── mod.rs
│   │   ├── loader.rs           # TOML parsing
│   │   ├── validation.rs       # Config validation
│   │   ├── env_vars.rs         # ${VAR} substitution
│   │   └── types.rs            # Config structs
│   ├── builder/
│   │   ├── mod.rs
│   │   └── agent_builder.rs    # AgentBuilder API
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── registry.rs         # Middleware system
│   │   ├── logging.rs          # Logging middleware
│   │   └── hooks.rs            # Trait definition
│   ├── templates/
│   │   ├── mod.rs
│   │   ├── basic.rs            # Basic template
│   │   └── advanced.rs         # Advanced template
│   └── error.rs                # DX-specific errors
└── templates/
    ├── basic/                   # Embedded template files
    │   ├── Cargo.toml
    │   ├── praxis.toml
    │   ├── main.rs
    │   └── Dockerfile
    └── advanced/
        └── ...
```

---

## Dependencies

```toml
[dependencies]
# Praxis core
praxis-types = { path = "../praxis-types" }
praxis-api = { path = "../praxis-api" }
praxis-graph = { path = "../praxis-graph" }
praxis-llm = { path = "../praxis-llm" }
praxis-mcp = { path = "../praxis-mcp" }
praxis-persist = { path = "../praxis-persist" }

# CLI framework
clap = { version = "4.4", features = ["derive", "env"] }

# Config parsing
toml = "0.8"
serde = { workspace = true }
serde_json = { workspace = true }

# File watching (for hot-reload)
notify = "6.1"

# Async runtime
tokio = { workspace = true, features = ["full"] }

# Error handling
anyhow = { workspace = true }
thiserror = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Template engine
handlebars = "5.1"

# Utils
dotenv = "0.15"
```

---

## Implementation Phases

### Phase 1: Config Loader ✓
- [ ] TOML parsing with serde
- [ ] Environment variable substitution
- [ ] Validation rules
- [ ] Error messages
- [ ] Default values

### Phase 2: CLI Basic Commands ✓
- [ ] `praxis init` - scaffold project
- [ ] `praxis config validate` - validate config
- [ ] `praxis config show` - display config
- [ ] Template embedding
- [ ] Clap argument parsing

### Phase 3: AgentBuilder ✓
- [ ] Builder pattern implementation
- [ ] `from_config()` integration
- [ ] Fluent API for all settings
- [ ] MCP server string parsing
- [ ] Async build with validation

### Phase 4: Development Mode ✓
- [ ] `praxis dev` command
- [ ] File watching with `notify`
- [ ] Auto-restart on changes
- [ ] Pretty logs for dev
- [ ] Health check browser opening

### Phase 5: MCP Management ✓
- [ ] `praxis mcp list`
- [ ] `praxis mcp add` - update praxis.toml
- [ ] `praxis mcp remove`
- [ ] `praxis mcp test` - connection test

### Phase 6: Deployment ✓
- [ ] `praxis deploy` command
- [ ] Docker image building
- [ ] Multi-platform support
- [ ] Registry push
- [ ] Size optimization

### Phase 7: Middleware System ✓
- [ ] Middleware trait definition
- [ ] Registry implementation
- [ ] Logging middleware integration
- [ ] Hooks for metrics (stub)
- [ ] Hooks for rate limiting (stub)

---

## Testing Strategy

### Unit Tests
- Config parsing with various TOML files
- Environment variable substitution
- Validation rules
- Template rendering

### Integration Tests
```rust
#[tokio::test]
async fn test_agent_builder_from_config() {
    let config_path = "tests/fixtures/valid-config.toml";
    
    let agent = AgentBuilder::from_config(config_path)
        .unwrap()
        .build()
        .await
        .unwrap();
    
    assert_eq!(agent.name(), "test-agent");
}

#[test]
fn test_config_validation() {
    let invalid_config = r#"
        [agent]
        name = ""  # Invalid: empty name
    "#;
    
    let result = Config::from_str(invalid_config);
    assert!(result.is_err());
}
```

### CLI Tests
```bash
# Test init command
$ praxis init test-agent
$ cd test-agent
$ cargo check  # Should compile

# Test config validation
$ praxis config validate  # Should pass

# Test MCP commands
$ praxis mcp list
```

---

## Examples

### Example 1: Quick Start

```rust
use praxis_dx::AgentBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::from_config("praxis.toml")?
        .build()
        .await?;
    
    agent.serve().await?;
    Ok(())
}
```

### Example 2: Programmatic Configuration

```rust
use praxis_dx::AgentBuilder;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::new("my-agent")
        .llm("gpt-4")
        .temperature(0.7)
        .mcp_servers(&env::var("MCP_SERVERS")?)
        .system_prompt("You are a helpful assistant.")
        .mongodb(&env::var("MONGODB_URI")?, "praxis")
        .bind("0.0.0.0", 3000)
        .build()
        .await?;
    
    agent.serve().await?;
    Ok(())
}
```

### Example 3: Custom Middleware

```rust
use praxis_dx::{AgentBuilder, Middleware, Request, Response, ApiError};
use async_trait::async_trait;

struct CustomLoggingMiddleware;

#[async_trait]
impl Middleware for CustomLoggingMiddleware {
    fn name(&self) -> &str {
        "custom_logging"
    }
    
    async fn before_request(&self, req: &Request) -> anyhow::Result<()> {
        println!("Custom log: {} {}", req.method(), req.uri());
        Ok(())
    }
    
    async fn after_request(&self, _req: &Request, res: &Response) -> anyhow::Result<()> {
        println!("Custom log: Response status {}", res.status());
        Ok(())
    }
    
    async fn on_error(&self, _req: &Request, error: &ApiError) -> anyhow::Result<()> {
        println!("Custom log: Error {}", error.message);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::from_config("praxis.toml")?
        // Add custom middleware hook (future API)
        // .add_middleware(Box::new(CustomLoggingMiddleware))
        .build()
        .await?;
    
    agent.serve().await?;
    Ok(())
}
```

---

## Next Steps

After praxis-dx is complete:
1. **praxis-persist** - MongoDB integration with context management
2. **Metrics middleware** - Prometheus export
3. **Rate limiting middleware** - Redis-backed limits
4. **Auth middleware** - JWT/API key validation
5. **Multi-tenancy** - Workspace/org support in config

---

## Questions to Address During Implementation

1. Should `praxis dev` watch Cargo.toml changes too?
2. Hot-reload strategy: restart process or reload config in-memory?
3. Template engine: embedded or downloadable from registry?
4. Config validation: fail-fast or collect all errors?
5. MCP server string format: allow JSON array as alternative?

---

**Ready to implement Phase 1 when you are!** 🚀

