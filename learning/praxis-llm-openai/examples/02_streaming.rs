use praxis_llm_openai::{OpenAIClient, ChatOptions, Message};
use anyhow::Result;
use futures::StreamExt;
use std::env;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");
    
    let client = OpenAIClient::new(api_key)?;
    
    println!("🌊 Praxis LLM - OpenAI Streaming Example\n");
    
    // Build conversation
    let messages = vec![
        Message::system("You are a helpful assistant that speaks concisely."),
        Message::human("Write a short poem about Rust programming language (3-4 lines)."),
    ];
    
    println!("📤 Sending message (streaming)...\n");
    println!("💬 Assistant:");
    
    // Get stream
    let mut stream = client
        .chat_completion_stream(
            "gpt-4o-mini",
            messages,
            ChatOptions::new(),
        )
        .await?;
    
    let mut content_buffer = String::new();
    
    // Process stream chunks
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // Print content delta
                if let Some(content) = chunk.content() {
                    print!("{}", content);
                    io::stdout().flush()?; // Flush to show immediately
                    content_buffer.push_str(content);
                }
                
                // Check if done
                if chunk.is_done() {
                    println!("\n");
                    println!("✅ Stream complete!");
                    println!("📊 Total characters: {}", content_buffer.len());
                    break;
                }
            }
            Err(e) => {
                eprintln!("\n❌ Error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
