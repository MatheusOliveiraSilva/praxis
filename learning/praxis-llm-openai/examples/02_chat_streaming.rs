use anyhow::Result;
use praxis_llm_openai::{LLMClient, OpenAIClient, ChatRequest, Message, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key)?;

    let request = ChatRequest::new(
        "gpt-5",
        vec![Message::human("Write a short poem about coding.")]
    );

    println!("Streaming response:");
    
    let mut stream = client.chat_completion_stream(request).await?;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::Message { content } => {
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

