use anyhow::Result;
use praxis_llm::{ReasoningClient, OpenAIClient, ResponseRequest, Message, ReasoningConfig, StreamEvent, EventBatcher};
use futures::StreamExt;
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key)?;

    let request = ResponseRequest::new(
        "gpt-5",
        vec![Message::human("Solve this problem: If a train travels 120 km in 2 hours, what is its average speed? And after that make a dynamic programming problem that can be solved in O(n) time and space complexity.")]
    ).with_reasoning(ReasoningConfig::medium());

    println!("Streaming response with reasoning (batched for network efficiency):\n");
    
    let mut stream = client.reason_stream(request).await?;
    let mut batcher = EventBatcher::new(50); // 50ms batching window
    let mut reasoning_displayed = false;
    let start_time = Instant::now();
    let mut total_events = 0;
    let mut total_batches = 0;

    loop {
        tokio::select! {
            // Receive events from stream
            event_result = stream.next() => {
                match event_result {
                    Some(Ok(event)) => {
                        total_events += 1;
                        batcher.push(event);
                    }
                    Some(Err(e)) => {
                        eprintln!("Stream error: {}", e);
                        break;
                    }
                    None => {
                        // Stream ended, flush remaining events
                        if !batcher.is_empty() {
                            let batch = batcher.take();
                            total_batches += 1;
                            display_batch(&batch, &mut reasoning_displayed)?;
                        }
                        break;
                    }
                }
            }
            
            // Flush batch when timer expires
            _ = batcher.ticker().tick() => {
                if !batcher.is_empty() {
                    let batch = batcher.take();
                    total_batches += 1;
                    display_batch(&batch, &mut reasoning_displayed)?;
                }
            }
        }
    }

    let elapsed = start_time.elapsed();
    println!("\n\nDone.");
    println!("Stats: {} events in {} batches ({}% reduction in network calls)", 
        total_events, 
        total_batches,
        if total_events > 0 { 100 - (total_batches * 100 / total_events) } else { 0 }
    );
    println!("Time elapsed: {:.2}s", elapsed.as_secs_f64());

    Ok(())
}

fn display_batch(batch: &[StreamEvent], reasoning_displayed: &mut bool) -> Result<()> {
    for event in batch {
        match event {
            StreamEvent::Reasoning { content } => {
                if !*reasoning_displayed {
                    println!("[REASONING]");
                    *reasoning_displayed = true;
                }
                print!("{}", content);
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            StreamEvent::Message { content } => {
                if *reasoning_displayed {
                    println!("\n\n[RESPONSE]");
                    *reasoning_displayed = false;
                }
                print!("{}", content);
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            StreamEvent::Done { .. } => {
                // Done event handled in main loop
            }
            _ => {}
        }
    }
    Ok(())
}

