use anyhow::Result;
use praxis_mcp::MCPClient;

/// Example: Connect to MCP server via HTTP
/// 
/// This demonstrates connecting to a remote MCP server over HTTP,
/// which is the recommended approach for production deployments.
/// 
/// # Running this example
/// 
/// 1. First, start an MCP server with HTTP transport:
///    ```bash
///    # Python example (would need HTTP server implementation)
///    python3 mcp_servers/weather_server_http.py
///    
///    # Or use a pre-built MCP server with HTTP
///    # (many official MCP servers support HTTP via --transport http)
///    ```
/// 
/// 2. Then run this client:
///    ```bash
///    cargo run --example http_client
///    ```

#[tokio::main]
async fn main() -> Result<()> {
    println!("MCP HTTP Client Example\n");

    // Connect to MCP server via HTTP
    println!("Connecting to MCP server at http://localhost:3000/mcp...");
    
    let client = MCPClient::new_http(
        "weather",
        "http://localhost:3000/mcp"
    ).await?;
    
    println!("Connected!\n");

    // List available tools
    println!("Available tools:");
    let tools = client.list_tools().await?;
    
    for tool in &tools {
        println!("  - {} - {:?}", tool.name, tool.description);
    }

    // Call a tool
    if !tools.is_empty() {
        println!("\nCalling tool: {}", tools[0].name);
        
        let result = client.call_tool(
            &tools[0].name,
            serde_json::json!({
                "location": "San Francisco, CA"
            })
        ).await?;
        
        println!("Result: {:?}", result);
    }

    println!("\nDone!");
    Ok(())
}

