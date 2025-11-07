use anyhow::Result;
use praxis_mcp::MCPClient;
use serde_json::json;

/// Simple example: Connect to MCP server and call a tool
/// 
/// # Setup
/// 
/// 1. Start your MCP server:
///    ```bash
///    cd mcp_servers/weather
///    uv run weather.py
///    ```
/// 
/// 2. Run this example:
///    ```bash
///    cargo run --example simple_mcp_agent
///    ```

#[tokio::main]
async fn main() -> Result<()> {
    println!("Simple Praxis MCP Agent\n");

    // ============================================
    // 1. Connect to MCP Server via HTTP
    // ============================================
    println!("Connecting to MCP server at http://localhost:8000/mcp...");
    
    let client = MCPClient::new_http(
        "weather",
        "http://localhost:8000/mcp"
    ).await?;
    
    println!("Connected!\n");

    // ============================================
    // 2. List Available Tools
    // ============================================
    println!("Available tools:");
    let tools = client.list_tools().await?;
    
    for tool in &tools {
        println!("  • {}", tool.name);
        if let Some(desc) = &tool.description {
            println!("    {}", desc);
        }
    }
    println!();

    // ============================================
    // 3. Call a Tool
    // ============================================
    println!("Calling tool: get_alerts for California...\n");
    
    let result = client.call_tool(
        "get_alerts",
        json!({
            "state": "CA"
        })
    ).await?;
    
    println!("Result:");
    for response in result {
        println!("{}", response.to_string());
    }
    
    println!("\n---\n");
    
    // ============================================
    // 4. Call Another Tool
    // ============================================
    println!("Calling tool: get_forecast for San Francisco...\n");
    
    let result = client.call_tool(
        "get_forecast",
        json!({
            "latitude": 37.7749,
            "longitude": -122.4194
        })
    ).await?;
    
    println!("Result:");
    for response in result {
        println!("{}", response.to_string());
    }

    println!("\nDone!");
    Ok(())
}

