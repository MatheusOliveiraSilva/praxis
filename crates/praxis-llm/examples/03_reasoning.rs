use anyhow::Result;
use praxis_llm::{LLMClient, OpenAIClient, ResponseRequest, Message, ReasoningConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key)?;

    let request = ResponseRequest::new(
        "gpt-5",
        vec![Message::human("Solve this problem: If a train travels 120 km in 2 hours, what is its average speed? And after that make a dynamic programming problem that can be solved in O(n) time and space complexity.")]
    ).with_reasoning(ReasoningConfig::medium());

    let response = client.response(request).await?;

    if let Some(reasoning) = &response.reasoning {
        println!("Reasoning:\n{}\n", reasoning);
    }

    if let Some(message) = &response.message {
        println!("Response: {}", message);
    }

    if let Some(usage) = &response.usage {
        println!("Tokens - Total: {}, Reasoning: {}",
            usage.total_tokens,
            usage.reasoning_tokens.unwrap_or(0)
        );
    }

    Ok(())
}

