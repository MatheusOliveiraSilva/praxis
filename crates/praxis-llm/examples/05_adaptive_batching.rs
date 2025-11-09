use anyhow::Result;
use praxis_llm::{ReasoningClient, OpenAIClient, ResponseRequest, Message, ReasoningConfig, StreamEvent};
use praxis_llm::buffer_utils::AdaptiveEventBatcher;
use futures::StreamExt;
use tokio::time::{Instant, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let client = OpenAIClient::new(api_key)?;

    let request = ResponseRequest::new(
        "gpt-5",
        vec![Message::human("Solve this problem: If a train travels 120 km in 2 hours, what is its average speed? And after that make a dynamic programming problem that can be solved in O(n) time and space complexity.")]
    ).with_reasoning(ReasoningConfig::medium());

    println!("Streaming with adaptive batching (window adjusts based on latency):\n");
    
    let mut stream = client.reason_stream(request).await?;
    
    // Adaptive batcher: base 50ms, min 20ms, max 200ms
    let mut batcher = AdaptiveEventBatcher::new(50, 20, 200);
    let mut reasoning_displayed = false;
    let start_time = Instant::now();

    loop {
        tokio::select! {
            // Receive events from stream
            event_result = stream.next() => {
                match event_result {
                    Some(Ok(event)) => {
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
                    
                    // Simulate network latency (in production, measure actual send time)
                    // For demo, we'll use a mock latency based on batch size
                    let mock_latency = Duration::from_millis(
                        (batch.len() as u64 * 2).min(100) // Mock: 2ms per event, max 100ms
                    );
                    
                    // Record latency for adaptive adjustment
                    batcher.record_latency(mock_latency);
                    
                    display_batch(&batch, &mut reasoning_displayed)?;
                    
                    // Show adaptive stats periodically
                    let stats = batcher.stats();
                    if stats.total_batches % 5 == 0 {
                        println!("\n[ADAPTIVE] Window: {}ms | Latency: {:.1}ms | Events/batch: {:.1}\n",
                            stats.current_window_ms,
                            stats.avg_latency_ms,
                            stats.avg_events_per_batch
                        );
                    }
                }
            }
        }
    }

    let elapsed = start_time.elapsed();
    let stats = batcher.stats();
    
    println!("\n\n=== Final Stats ===");
    println!("Total events: {}", stats.total_events);
    println!("Total batches: {}", stats.total_batches);
    println!("Avg events/batch: {:.1}", stats.avg_events_per_batch);
    println!("Final window: {}ms", stats.current_window_ms);
    println!("Avg latency: {:.1}ms", stats.avg_latency_ms);
    println!("Reduction: {}% fewer network calls", 
        if stats.total_events > 0 { 
            100 - (stats.total_batches * 100 / stats.total_events) 
        } else { 
            0 
        }
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

