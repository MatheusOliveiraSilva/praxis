use async_channel::{bounded, Sender, Receiver};
use tokio::time::{sleep, Duration};

/// Simulates a "Node" producing events
async fn producer(tx: Sender<String>, name: &str, count: usize, delay_ms: u64) {
    println!("[{}] Starting to produce {} events...", name, count);
    
    for i in 1..=count {
        let event = format!("{} - Event #{}", name, i);
        
        // Try to send event (will WAIT if channel is full - backpressure!)
        match tx.send(event.clone()).await {
            Ok(_) => {
                println!("[{}] ✓ Sent: {}", name, event);
            }
            Err(e) => {
                println!("[{}] ✗ Failed to send: {}", name, e);
            }
        }
        
        
        // Simulate work (generating tokens from LLM, for example)
        sleep(Duration::from_millis(delay_ms)).await;
    }
    
    println!("[{}] Finished producing events", name);
}

/// Simulates a "Gateway" consuming events and sending to client
async fn consumer(rx: Receiver<String>, delay_ms: u64) {
    println!("[Consumer] Starting to consume events...\n");
    
    let mut count = 0;
    while let Ok(event) = rx.recv().await {
        count += 1;
        println!("[Consumer] 📩 Received: {}", event);
        
        // Simulate slow client (slow network, for example)
        sleep(Duration::from_millis(delay_ms)).await;
    }
    
    println!("\n[Consumer] Finished consuming {} events", count);
}

#[tokio::main]
async fn main() {
    println!("=== Bounded Channel Example ===\n");
    
    // Create a bounded channel with capacity of 5
    // This means only 5 messages can be "in flight" at once
    let capacity = 5;
    let (tx, rx) = bounded::<String>(capacity);
    
    
    println!("Channel capacity: {}\n", capacity);
    println!("Scenario: Fast producer (100ms) vs Slow consumer (300ms)");
    println!("Expected behavior: Producer will BLOCK when channel is full (backpressure)\n");
    println!("---\n");
    
    // Clone sender for multiple producers
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    
    // Spawn producer tasks (fast - 100ms between events)
    let producer1 = tokio::spawn(async move {
        producer(tx1, "Producer-1", 10, 100).await;
    });
    
    let producer2 = tokio::spawn(async move {
        producer(tx2, "Producer-2", 10, 100).await;
    });
    
    // Drop original sender so channel closes when all senders are done
    drop(tx);
    
    // Spawn consumer task (slow - 300ms per event)
    let consumer_task = tokio::spawn(async move {
        consumer(rx, 300).await;
    });
    
    // Wait for all tasks
    let _ = tokio::join!(producer1, producer2, consumer_task);
    
    println!("\n=== Example Complete ===");
    println!("\nKey Observations:");
    println!("1. Producers were FASTER than consumer (100ms vs 300ms)");
    println!("2. Channel has capacity of 5, so after 5 events, producers WAIT");
    println!("3. This is BACKPRESSURE: prevents memory overflow");
    println!("4. In Praxis: protects server from slow clients consuming SSE");
}

