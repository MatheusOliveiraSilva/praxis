//! AI agent with MCP tool integration
//!
//! This example shows how to create an AI agent that can use
//! external tools via the Model Context Protocol (MCP).
//!
//! # Setup
//!
//! 1. Start an MCP server (e.g., weather service):
//!    ```bash
//!    cd mcp_servers/weather
//!    uv run weather.py
//!    ```
//!
//! 2. Run this example:
//!    ```bash
//!    export OPENAI_API_KEY=sk-...
//!    export MCP_SERVERS=http://localhost:8000/mcp
//!    cargo run --example with_mcp_tools
//!    ```

use praxis::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🤖 Praxis Agent with MCP Tools Example\n");

    // Get configuration
    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY not set");
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let mcp_servers = std::env::var("MCP_SERVERS")
        .expect("MCP_SERVERS not set (comma-separated URLs)");

    // Create agent with MCP servers
    println!("📦 Building agent with MCP servers...");
    let agent = AgentBuilder::new()
        .mongodb(&mongodb_uri, "praxis_example")
        .openai_key(&openai_key)
        .mcp_servers(&mcp_servers)
        .build()
        .await?;

    println!("✅ Agent ready with MCP tools!\n");

    // List available tools
    println!("🔧 Available tools:");
    let tools = agent.mcp_executor().list_all_tools().await?;
    for (server, tool_list) in tools {
        println!("  Server: {}", server);
        for tool in tool_list {
            println!("    - {} ({})", tool.name, tool.description);
        }
    }
    println!();

    // Example queries that use tools
    let queries = vec![
        "What's the weather in San Francisco?",
        "Can you get the forecast for Tokyo?",
    ];

    for (i, query) in queries.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Query {}: {}", i + 1, query);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let response = agent.chat(query).await?;

        println!("Response: {}\n", response);
    }

    println!("✨ Done!");

    Ok(())
}
