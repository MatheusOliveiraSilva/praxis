use anyhow::Result;
use praxis_llm::{ChatClient, OpenAIClient, ChatRequest, Message};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key)?;

    let request = ChatRequest::new(
        "gpt-4o",
        vec![Message::human("What is the capital of France?")]
    );

    let response = client.chat(request).await?;

    println!("Response: {}", response.content.unwrap_or_default());
    
    if let Some(usage) = response.usage {
        println!("Tokens used: {}", usage.total_tokens);
    }

    Ok(())
}

