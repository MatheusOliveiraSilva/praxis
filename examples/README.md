# Praxis Examples

This directory contains complete example applications demonstrating how to use the Praxis framework.

## Contents

### `praxis-api/`
Full-featured REST API example built with Axum that demonstrates:
- SSE (Server-Sent Events) streaming
- Thread and message persistence with MongoDB
- MCP multi-server integration
- Health checks and metrics
- Production-ready middleware (CORS, logging, timeouts)

See [praxis-api/README.md](praxis-api/README.md) for details.

### `agent_ui/`
Next.js web interface for interacting with the Praxis API:
- Real-time streaming chat interface
- Thread management
- Modern React UI with TypeScript

See [agent_ui/README.md](agent_ui/README.md) for details.

### `mcp_servers/`
Example MCP tool servers:
- `weather/` - Weather information server using FastMCP

### `scripts/`
Helper scripts for running the full stack:
- `start-all.sh` - Start API + UI + dependencies (MongoDB + MCP)
- `stop-all.sh` - Stop all services

## 🚀 Quick Start

```bash
# From the examples directory
cd scripts
./start-all.sh
```

Then open http://localhost:3000 in your browser.

## Learn More

These examples showcase best practices for building production-ready AI agent backends with Praxis. Study them to understand:

- How to compose the framework crates (`praxis-types`, `praxis-graph`, `praxis-llm`, `praxis-mcp`, `praxis-persist`)
- How to structure an async Rust web service
- How to handle streaming responses with SSE
- How to integrate MCP servers
- How to persist conversation state

For framework documentation, see the main [README](../README.md).

