use praxis_llm_openai::{OpenAIClient, ChatOptions, Message};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Get API key from environment
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");
    
    // Create client
    let client = OpenAIClient::new(api_key)?;
    
    println!("🤖 Praxis LLM - OpenAI Simple Chat Example\n");
    
    // Build conversation
    let messages = vec![
        Message::system("You are a helpful assistant that speaks concisely."),
        Message::human("What is the capital of France? Answer in one sentence."),
    ];
    
    println!("📤 Sending messages:");
    for msg in &messages {
        println!("  - {}: {:?}", msg.role(), msg);
    }
    println!();
    
    // Send request
    println!("⏳ Waiting for response...\n");
    
    let response = client
        .chat_completion(
            "gpt-4o-mini", // Cheaper model for testing
            messages,
            ChatOptions::new(),
        )
        .await?;
    
    // Print response
    println!("✅ Response received!\n");
    println!("📊 Metadata:");
    println!("  - ID: {}", response.id);
    println!("  - Model: {}", response.model);
    println!("  - Tokens: {} prompt + {} completion = {} total",
        response.usage.prompt_tokens,
        response.usage.completion_tokens,
        response.usage.total_tokens
    );
    println!();
    
    println!("💬 Assistant:");
    if let Some(content) = response.content() {
        println!("{}", content);
    } else {
        println!("(no content)");
    }
    println!();
    
    // Convert to our Message type
    if let Some(msg) = response.to_message() {
        println!("🔄 Converted to Message:");
        println!("{:#?}", msg);
    }
    
    Ok(())
}
