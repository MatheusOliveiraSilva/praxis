use anyhow::Result;
use futures::StreamExt;
use praxis_llm::{
    AzureOpenAIClient, Message, ReasoningClient, ReasoningConfig, ResponseRequest, StreamEvent,
};
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a file to save raw chunks
    let mut raw_chunks_file = File::create("azure_streaming_raw_chunks.txt")?;
    writeln!(raw_chunks_file, "=== Azure OpenAI Streaming Raw Chunks ===\n")?;
    // Load Azure OpenAI configuration from environment variables
    let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
    let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")?;
    let api_version = std::env::var("AZURE_OPENAI_API_VERSION")
        .unwrap_or_else(|_| "2024-02-15-preview".to_string());

    println!("Azure OpenAI Reasoning + Streaming Example");
    println!("==========================================\n");
    println!("Endpoint: {}", endpoint);
    println!("API Version: {}\n", api_version);

    // Create Azure OpenAI client using builder pattern
    let client = AzureOpenAIClient::builder()
        .api_key(api_key)
        .endpoint(endpoint)
        .api_version(api_version)
        .build()?;

    // Create a reasoning request with streaming enabled
    let deployment_name = "gpt-5"; // Your Azure deployment name
    let request = ResponseRequest::new(
        deployment_name,
        vec![Message::human(
            "Explain the difference between machine learning and deep learning in simple terms, \
             highlighting key concepts and practical applications.",
        )],
    )
    .with_reasoning(ReasoningConfig::high()); // Use high reasoning effort

    println!("Using deployment: {}", deployment_name);
    println!("Reasoning effort: high\n");
    println!("Streaming reasoning response:\n");
    println!("---");

    // Get streaming response with reasoning
    let mut stream = client.reason_stream(request).await?;

    let mut reasoning_content = String::new();
    let mut message_content = String::new();
    let mut token_count = 0;
    let mut chunk_number = 0;

    // Process stream events
    while let Some(event) = stream.next().await {
        chunk_number += 1;
        
        // Save raw chunk to file
        writeln!(raw_chunks_file, "--- Chunk #{} ---", chunk_number)?;
        writeln!(raw_chunks_file, "{:#?}", event)?;
        writeln!(raw_chunks_file, "")?;
        raw_chunks_file.flush()?;
        
        match event? {
            StreamEvent::Reasoning { content } => {
                // Reasoning tokens are streamed first (internal thought process)
                reasoning_content.push_str(&content);
                print!("\r[Reasoning tokens: {}]", reasoning_content.len());
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            StreamEvent::Message { content } => {
                // Clear the reasoning counter line
                if !reasoning_content.is_empty() && token_count == 0 {
                    println!("\n\n[Reasoning complete - {} chars]\n", reasoning_content.len());
                    println!("Response:\n");
                }
                // Print message content as it streams
                print!("{}", content);
                std::io::Write::flush(&mut std::io::stdout())?;
                message_content.push_str(&content);
                token_count += 1;
            }
            StreamEvent::ToolCall {
                index,
                id,
                name,
                arguments,
            } => {
                println!(
                    "\n[Tool call #{}: {} - {} with args: {}]",
                    index,
                    id.unwrap_or_default(),
                    name.unwrap_or_default(),
                    arguments.unwrap_or_default()
                );
            }
            StreamEvent::Done { finish_reason } => {
                println!("\n---\n");
                if let Some(reason) = finish_reason {
                    println!("Finish reason: {}", reason);
                }
                println!("Total message tokens: {}", token_count);
                println!("Total chunks received: {}", chunk_number);
            }
        }
    }

    println!("\n✓ Reasoning + Streaming complete!");
    
    // Summary
    if !reasoning_content.is_empty() {
        println!("\nSummary:");
        println!("  Reasoning phase: {} chars of internal reasoning", reasoning_content.len());
        println!("  Response phase: {} tokens streamed", token_count);
    }
    
    // Close the raw chunks file
    writeln!(raw_chunks_file, "\n=== End of stream - Total chunks: {} ===", chunk_number)?;
    raw_chunks_file.flush()?;
    
    println!("\n📁 Raw chunks saved to: azure_streaming_raw_chunks.txt");
    println!("   Use this file to debug and understand the streaming format!");

    Ok(())
}

