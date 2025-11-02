use praxis_llm_openai::{OpenAIClient, ChatOptions, Message, ToolChoice};
use praxis_llm_openai::mcp::{MCPClient, mcp_tools_to_openai};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔗 Praxis LLM - OpenAI + MCP Integration Example\n");
    
    // Get configuration from environment
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");
    
    let mcp_command = env::var("MCP_COMMAND")
        .unwrap_or_else(|_| "npx".to_string());
    
    let mcp_args_str = env::var("MCP_ARGS")
        .unwrap_or_else(|_| "-y @modelcontextprotocol/server-everything".to_string());
    
    let mcp_args: Vec<&str> = mcp_args_str.split_whitespace().collect();
    
    println!("📋 Configuration:");
    println!("  MCP Command: {} {}", mcp_command, mcp_args.join(" "));
    println!();
    
    // Connect to MCP server
    println!("🔌 Connecting to MCP server...");
    let mut mcp_client = MCPClient::connect(&mcp_command, &mcp_args)?;
    println!("✅ Connected!\n");
    
    // List available tools
    println!("📦 Fetching available tools...");
    let mcp_tools = mcp_client.list_tools()?;
    println!("✅ Found {} tools:\n", mcp_tools.len());
    
    for tool in &mcp_tools {
        println!("  🔧 {}", tool.name);
        if let Some(desc) = &tool.description {
            println!("     {}", desc);
        }
    }
    println!();
    
    // Convert MCP tools to OpenAI format
    let openai_tools = mcp_tools_to_openai(&mcp_tools);
    
    // Create OpenAI client
    let openai_client = OpenAIClient::new(api_key)?;
    
    // Build conversation that requires tool use
    let tool_name = mcp_tools.first()
        .map(|t| t.name.as_str())
        .unwrap_or("unknown");
    
    let messages = vec![
        Message::system(format!(
            "You are a helpful assistant with access to tools. \
            You have access to a tool called '{}'. Use it when appropriate.",
            tool_name
        )),
        Message::human(format!(
            "I want you to use the '{}' tool. Call it with appropriate arguments based on its schema.",
            tool_name
        )),
    ];
    
    println!("💬 Sending message to OpenAI with MCP tools...\n");
    
    // First API call - expect tool call
    let response = openai_client
        .chat_completion(
            "gpt-4o-mini",
            messages.clone(),
            ChatOptions::new()
                .tools(openai_tools)
                .tool_choice(ToolChoice::required()), // Force tool usage
        )
        .await?;
    
    println!("✅ Response received!\n");
    
    // Check if model wants to use tool
    if let Some(tool_calls) = response.tool_calls() {
        println!("🔧 Model requested {} tool call(s):\n", tool_calls.len());
        
        for tool_call in tool_calls {
            println!("Tool Call:");
            println!("  ID: {}", tool_call.id);
            println!("  Function: {}", tool_call.function.name);
            println!("  Arguments: {}\n", tool_call.function.arguments);
            
            // Execute via MCP
            println!("⚙️  Executing via MCP...");
            
            let arguments = tool_call.arguments_value()?;
            
            match mcp_client.call_tool(&tool_call.function.name, arguments) {
                Ok(result) => {
                    println!("✅ Tool execution successful!");
                    println!("📄 Result: {}\n", result);
                    
                    // Build conversation with tool result
                    let mut conversation = messages.clone();
                    conversation.push(response.to_message().unwrap());
                    conversation.push(Message::tool_result(
                        tool_call.id.clone(),
                        result.as_str().unwrap_or("").to_string(),
                    ));
                    
                    // Second API call - model sees tool result
                    println!("📤 Sending tool result back to OpenAI...\n");
                    
                    let final_response = openai_client
                        .chat_completion(
                            "gpt-4o-mini",
                            conversation,
                            ChatOptions::new(),
                        )
                        .await?;
                    
                    println!("✅ Final response:\n");
                    println!("💬 Assistant: {}\n", final_response.content().unwrap_or("(no content)"));
                    
                    println!("📊 Token usage:");
                    println!("  - First call: {} tokens", response.usage.total_tokens);
                    println!("  - Second call: {} tokens", final_response.usage.total_tokens);
                    println!("  - Total: {} tokens", 
                        response.usage.total_tokens + final_response.usage.total_tokens);
                }
                Err(e) => {
                    eprintln!("❌ Tool execution failed: {}", e);
                }
            }
        }
    } else {
        println!("⚠️  Model didn't use tools (unexpected with tool_choice=required)");
        if let Some(content) = response.content() {
            println!("Response: {}", content);
        }
    }
    
    Ok(())
}
