use anyhow::Result;
use praxis_llm::{ReasoningClient, OpenAIClient, ResponseRequest, Message, ReasoningConfig, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key)?;

    let request = ResponseRequest::new(
        "gpt-5",
        vec![Message::human("Solve this problem: If a train travels 120 km in 2 hours, what is its average speed? And after that make a dynamic programming problem that can be solved in O(n) time and space complexity.")]
    ).with_reasoning(ReasoningConfig::medium());

    println!("Streaming response with reasoning:\n");
    
    let mut stream = client.reason_stream(request).await?;
    let mut reasoning_displayed = false;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::Reasoning { content } => {
                if !reasoning_displayed {
                    println!("[REASONING]");
                    reasoning_displayed = true;
                }
                print!("{}", content);
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            StreamEvent::Message { content } => {
                if reasoning_displayed {
                    println!("\n\n[RESPONSE]");
                    reasoning_displayed = false;
                }
                print!("{}", content);
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            StreamEvent::Done { .. } => {
                println!("\n\nDone.");
            }
            _ => {}
        }
    }

    Ok(())
}

