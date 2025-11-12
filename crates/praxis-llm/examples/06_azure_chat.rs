use anyhow::Result;
use praxis_llm::{AzureOpenAIClient, ChatClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<()> {
    // Load Azure OpenAI configuration from environment variables
    let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
    let resource_name = std::env::var("AZURE_OPENAI_RESOURCE_NAME")?;
    let deployment_name = std::env::var("AZURE_OPENAI_DEPLOYMENT_NAME")?;
    let api_version = std::env::var("AZURE_OPENAI_API_VERSION")
        .unwrap_or_else(|_| "2024-02-15-preview".to_string());

    println!("Azure OpenAI Chat Completion Example");
    println!("=====================================\n");
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

    // Create a simple chat request
    let request = ChatRequest::new(
        "gpt-4", // Model name is used for internal logic, deployment is in URL
        vec![Message::human("What is the capital of France?")],
    );

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
