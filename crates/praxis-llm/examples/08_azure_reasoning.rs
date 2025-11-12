use anyhow::Result;
use praxis_llm::{
    AzureOpenAIClient, Message, ReasoningClient, ReasoningConfig, ResponseRequest,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load Azure OpenAI configuration from environment variables
    let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
    let resource_name = std::env::var("AZURE_OPENAI_RESOURCE_NAME")?;
    let deployment_name = std::env::var("AZURE_OPENAI_DEPLOYMENT_NAME")?;
    let api_version = std::env::var("AZURE_OPENAI_API_VERSION")
        .unwrap_or_else(|_| "2024-02-15-preview".to_string());

    println!("Azure OpenAI Reasoning Example");
    println!("===============================\n");
    println!("Resource: {}", resource_name);
    println!("Deployment: {}", deployment_name);
    println!("API Version: {}\n", api_version);
    println!("Note: This requires a deployment with o1 or gpt-5 model support\n");

    // Create Azure OpenAI client using builder pattern
    let client = AzureOpenAIClient::builder()
        .api_key(api_key)
        .resource_name(resource_name)
        .deployment_name(deployment_name)
        .api_version(api_version)
        .build()?;

    // Create a reasoning request with medium effort
    let request = ResponseRequest::new(
        "o1", // Use o1 model for reasoning
        vec![Message::human(
            "Explain the concept of quantum entanglement in simple terms.",
        )],
    )
    .with_reasoning(ReasoningConfig::medium());

    println!("Sending reasoning request...\n");

    // Get response with reasoning
    let response = client.reason(request).await?;

    // Print reasoning (if available)
    if let Some(reasoning) = response.reasoning {
        println!("Reasoning:");
        println!("----------");
        println!("{}\n", reasoning);
    }

    // Print the final message
    if let Some(message) = response.message {
        println!("Response:");
        println!("---------");
        println!("{}", message);
    }

    // Print usage statistics
    if let Some(usage) = response.usage {
        println!("\nToken Usage:");
        println!("  Input tokens: {}", usage.input_tokens);
        println!("  Output tokens: {}", usage.output_tokens);
        println!("  Total tokens: {}", usage.total_tokens);
        if let Some(reasoning_tokens) = usage.reasoning_tokens {
            println!("  Reasoning tokens: {}", reasoning_tokens);
        }
    }

    if let Some(status) = response.status {
        println!("Status: {}", status);
    }

    Ok(())
}

