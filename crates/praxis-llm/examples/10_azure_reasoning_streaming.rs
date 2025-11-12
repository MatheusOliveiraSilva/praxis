use anyhow::Result;
use futures::StreamExt;
use praxis_llm::{
    AzureOpenAIClient, Message, ReasoningClient, ReasoningConfig, ResponseRequest, StreamEvent,
};

#[tokio::main]
async fn main() -> Result<()> {
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

    // Process stream events
    while let Some(event) = stream.next().await {
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

    Ok(())
}

