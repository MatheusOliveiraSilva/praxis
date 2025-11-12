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
    let azure_resource = std::env::var("AZURE_OPENAI_RESOURCE_NAME");
    let azure_deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT_NAME");
    
    if let (Ok(api_key), Ok(resource), Ok(deployment)) = 
        (azure_key, azure_resource, azure_deployment) {
        
        let config = ProviderConfig::azure_openai(
            api_key,
            resource,
            deployment,
            "2024-02-15-preview",
        );
        
        let client: Arc<dyn ChatClient> = ClientFactory::create_chat_client(config)?;

        let request = ChatRequest::new("gpt-4", vec![Message::human("Say hello!")]);
        let response = client.chat(request).await?;
        println!("Response: {}\n", response.content.unwrap_or_default());
    } else {
        println!("Skipped (Azure environment variables not set)\n");
    }

    // Example 3: Dynamic provider selection based on config
    println!("Example 3: Dynamic Provider Selection");
    println!("--------------------------------------");
    
    // In a real application, this could come from a config file or database
    let provider_type = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    
    println!("Selected provider: {}", provider_type);
    
    let config = match provider_type.as_str() {
        "azure" | "azure_openai" => {
            let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
            let resource = std::env::var("AZURE_OPENAI_RESOURCE_NAME")?;
            let deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT_NAME")?;
            ProviderConfig::azure_openai(api_key, resource, deployment, "2024-02-15-preview")
        }
        _ => {
            let api_key = std::env::var("OPENAI_API_KEY")?;
            ProviderConfig::openai(api_key)
        }
    };
    
    let client = ClientFactory::create_client(config)?;
    
    let request = ChatRequest::new("gpt-4", vec![Message::human("What is 2+2?")]);
    let response = client.chat(request).await?;
    println!("Response: {}", response.content.unwrap_or_default());

    println!("\n✓ Factory pattern allows easy provider switching!");

    Ok(())
}

