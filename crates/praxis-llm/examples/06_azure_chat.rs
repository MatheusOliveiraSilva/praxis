use anyhow::Result;
use praxis_llm::{AzureOpenAIClient, ChatClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<()> {
    // Load Azure OpenAI configuration from environment variables
    let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
    let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")?;
    let api_version = std::env::var("AZURE_OPENAI_API_VERSION")
        .unwrap_or_else(|_| "2024-02-15-preview".to_string());

    println!("Azure OpenAI Chat Completion Example");
    println!("=====================================\n");
    println!("Endpoint: {}", endpoint);
    println!("API Version: {}\n", api_version);

    // Create Azure OpenAI client using builder pattern
    let client = AzureOpenAIClient::builder()
        .api_key(api_key)
        .endpoint(endpoint)
        .api_version(api_version)
        .build()?;

    // Create a simple chat request
    // The deployment name is passed via the model parameter
    let deployment_name = "gpt-4-deployment"; // Your Azure deployment name
    let request = ChatRequest::new(
        deployment_name,
        vec![Message::human("What is the capital of France?")],
    );
    
    println!("Using deployment: {}\n", deployment_name);

    println!("Sending request...\n");

    // Get response
    let response = client.chat(request).await?;

    // Print the response
    println!("Response: {}", response.content.unwrap_or_default());

    if let Some(usage) = response.usage {
        println!("\nToken Usage:");
        println!("  Input tokens: {}", usage.input_tokens);
        println!("  Output tokens: {}", usage.output_tokens);
        println!("  Total tokens: {}", usage.total_tokens);
    }

    if let Some(finish_reason) = response.finish_reason {
        println!("Finish reason: {}", finish_reason);
    }

    Ok(())
}
