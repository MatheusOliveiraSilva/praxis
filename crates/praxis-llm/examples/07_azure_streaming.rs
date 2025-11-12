use anyhow::Result;
use futures::StreamExt;
use praxis_llm::{AzureOpenAIClient, ChatClient, ChatRequest, Message, StreamEvent};

#[tokio::main]
async fn main() -> Result<()> {
    // Load Azure OpenAI configuration from environment variables
    let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
    let resource_name = std::env::var("AZURE_OPENAI_RESOURCE_NAME")?;
    let deployment_name = std::env::var("AZURE_OPENAI_DEPLOYMENT_NAME")?;
    let api_version = std::env::var("AZURE_OPENAI_API_VERSION")
        .unwrap_or_else(|_| "2024-02-15-preview".to_string());

    println!("Azure OpenAI Streaming Chat Example");
    println!("====================================\n");
    println!("Resource: {}", resource_name);
    println!("Deployment: {}", deployment_name);
    println!("API Version: {}\n", api_version);

    // Create Azure OpenAI client using builder pattern
    let client = AzureOpenAIClient::builder()
        .api_key(api_key)
        .resource_name(resource_name)
        .deployment_name(deployment_name)
        .api_version(api_version)
        .build()?;

    // Create a chat request for streaming
    let request = ChatRequest::new(
        "gpt-4",
        vec![Message::human("Write a short poem about artificial intelligence in exactly 4 lines.")],
    );

    println!("Streaming response:\n");
    println!("---");

    // Get streaming response
    let mut stream = client.chat_stream(request).await?;

    // Process stream events
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::Message { content } => {
                print!("{}", content);
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            StreamEvent::Reasoning { content } => {
                println!("\n[Reasoning: {}]", content);
            }
            StreamEvent::ToolCall { index, id, name, arguments } => {
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
            }
        }
    }

    println!("\nStream complete!");

    Ok(())
}

