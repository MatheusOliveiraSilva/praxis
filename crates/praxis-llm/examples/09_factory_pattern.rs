use anyhow::Result;
use praxis_llm::{ChatClient, ChatRequest, ClientFactory, Message, ProviderConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Provider Factory Pattern Example");
    println!("=================================\n");

    // Example 1: Create OpenAI client using factory
    println!("Example 1: OpenAI via Factory");
    println!("------------------------------");
    
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        let config = ProviderConfig::openai(api_key);
        let client: Arc<dyn ChatClient> = ClientFactory::create_chat_client(config)?;

        let request = ChatRequest::new("gpt-4o", vec![Message::human("Say hello!")]);
        let response = client.chat(request).await?;
        println!("Response: {}\n", response.content.unwrap_or_default());
    } else {
        println!("Skipped (OPENAI_API_KEY not set)\n");
    }

    // Example 2: Create Azure OpenAI client using factory
    println!("Example 2: Azure OpenAI via Factory");
    println!("------------------------------------");
    
    let azure_key = std::env::var("AZURE_OPENAI_API_KEY");
    let azure_endpoint = std::env::var("AZURE_OPENAI_ENDPOINT");
    
    if let (Ok(api_key), Ok(endpoint)) = (azure_key, azure_endpoint) {
        
        let config = ProviderConfig::azure_openai(
            api_key,
            endpoint,
            "2024-02-15-preview",
        );
        
        let client: Arc<dyn ChatClient> = ClientFactory::create_chat_client(config)?;

        // Deployment name is passed via model parameter
        let deployment_name = "gpt-4-deployment"; // Your Azure deployment name
        let request = ChatRequest::new(deployment_name, vec![Message::human("Say hello!")]);
        let response = client.chat(request).await?;
        println!("Response: {}\n", response.content.unwrap_or_default());
    } else {
        println!("Skipped (Azure environment variables not set)\n");
    }

    // Example 3: Dynamic provider selection based on config
    println!("Example 3: Dynamic Provider Selection (Chat Only)");
    println!("--------------------------------------------------");
    
    // In a real application, this could come from a config file or database
    let provider_type = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    
    println!("Selected provider: {}", provider_type);
    
    let (config, model) = match provider_type.as_str() {
        "azure" | "azure_openai" => {
            let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
            let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")?;
            let config = ProviderConfig::azure_openai(api_key, endpoint, "2024-02-15-preview");
            // For Azure, model is the deployment name
            (config, "gpt-4-deployment".to_string())
        }
        _ => {
            let api_key = std::env::var("OPENAI_API_KEY")?;
            let config = ProviderConfig::openai(api_key);
            // For OpenAI, model is the model name
            (config, "gpt-4".to_string())
        }
    };
    
    // Note: Using create_chat_client for compatibility with both providers
    // Azure OpenAI doesn't support reasoning models, so we use ChatClient
    let client = ClientFactory::create_chat_client(config)?;
    
    let request = ChatRequest::new(model, vec![Message::human("What is 2+2?")]);
    let response = client.chat(request).await?;
    println!("Response: {}", response.content.unwrap_or_default());

    println!("\n✓ Factory pattern allows easy provider switching!");

    Ok(())
}

