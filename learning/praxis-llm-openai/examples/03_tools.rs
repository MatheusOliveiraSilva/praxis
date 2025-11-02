use praxis_llm_openai::{OpenAIClient, ChatOptions, Message, Tool, ToolChoice};
use anyhow::Result;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");
    
    let client = OpenAIClient::new(api_key)?;
    
    println!("🔧 Praxis LLM - OpenAI Tools Example\n");
    
    // Define calculator tool
    let calculator_tool = Tool::new(
        "calculator",
        "Evaluates a mathematical expression and returns the result",
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The mathematical expression to evaluate (e.g., '2 + 2', '10 * 5')"
                }
            },
            "required": ["expression"]
        }),
    );
    
    println!("📋 Available tools:");
    println!("  - calculator: {}", calculator_tool.function.description.as_ref().unwrap());
    println!();
    
    // Build conversation
    let messages = vec![
        Message::system("You are a helpful assistant with access to a calculator. Use it when you need to perform calculations."),
        Message::human("What is 156 multiplied by 47? Use the calculator tool."),
    ];
    
    println!("📤 Sending message with tools available...\n");
    
    // First API call - expect tool call
    let response = client
        .chat_completion(
            "gpt-4o-mini",
            messages.clone(),
            ChatOptions::new()
                .tools(vec![calculator_tool])
                .tool_choice(ToolChoice::auto()), // Let model decide
        )
        .await?;
    
    println!("✅ Response received!\n");
    
    // Check if model wants to use tool
    if let Some(tool_calls) = response.tool_calls() {
        println!("🔧 Model requested {} tool call(s):\n", tool_calls.len());
        
        for tool_call in tool_calls {
            println!("Tool Call ID: {}", tool_call.id);
            println!("Function: {}", tool_call.function.name);
            println!("Arguments: {}", tool_call.function.arguments);
            
            // Parse arguments
            let args: serde_json::Value = tool_call.parse_arguments()?;
            if let Some(expression) = args.get("expression").and_then(|v| v.as_str()) {
                println!("Expression to evaluate: {}", expression);
                
                // Simulate tool execution (in real scenario, execute actual tool)
                let result = evaluate_expression(expression);
                println!("Result: {}\n", result);
                
                // Build conversation with tool result
                let mut conversation = messages.clone();
                conversation.push(response.to_message().unwrap()); // Add assistant message with tool_call
                conversation.push(Message::tool_result(
                    tool_call.id.clone(),
                    result.to_string(),
                ));
                
                // Second API call - model sees tool result
                println!("📤 Sending tool result back to model...\n");
                
                let final_response = client
                    .chat_completion(
                        "gpt-4o-mini",
                        conversation,
                        ChatOptions::new(),
                    )
                    .await?;
                
                println!("✅ Final response:\n");
                println!("💬 Assistant: {}\n", final_response.content().unwrap());
                
                println!("📊 Total tokens used:");
                println!("  - First call: {} tokens", response.usage.total_tokens);
                println!("  - Second call: {} tokens", final_response.usage.total_tokens);
                println!("  - Grand total: {} tokens", 
                    response.usage.total_tokens + final_response.usage.total_tokens);
            }
        }
    } else {
        println!("⚠️  Model didn't use tools (unexpected)");
        if let Some(content) = response.content() {
            println!("Response: {}", content);
        }
    }
    
    Ok(())
}

/// Simulate calculator tool execution
fn evaluate_expression(expr: &str) -> f64 {
    // In real implementation, use a proper expression evaluator
    // For demo, handle simple cases
    match expr {
        "156 * 47" | "156*47" => 156.0 * 47.0,
        "2 + 2" | "2+2" => 4.0,
        _ => {
            // Fallback: try basic eval
            println!("  (using basic evaluator)");
            0.0 // In real scenario, use a library like meval
        }
    }
}
