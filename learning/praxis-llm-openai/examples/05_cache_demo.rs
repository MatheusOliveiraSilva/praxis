/// Example: HTTP Response Cache
/// 
/// Demonstrates how to use the built-in response cache to avoid
/// redundant API calls for identical requests.

use praxis_llm_openai::{OpenAIClient, ChatOptions, Message};
use std::time::{Duration, Instant};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set");
    
    println!("🔧 Demo: HTTP Response Cache\n");
    
    // Create client with 60-second cache TTL
    let client = OpenAIClient::with_cache(api_key, Some(Duration::from_secs(60)))?;
    
    let messages = vec![
        Message::system("You are a helpful assistant."),
        Message::human("What is 2+2?"),
    ];
    
    let options = ChatOptions::new()
        .temperature(0.7)
        .max_tokens(50);
    
    // First call - will hit API
    println!("📡 First call (API hit)...");
    let start = Instant::now();
    let response1 = client.chat_completion("gpt-4", messages.clone(), options.clone()).await?;
    let duration1 = start.elapsed();
    
    println!("✅ Response: {}", response1.content().unwrap_or("No content"));
    println!("⏱️  Duration: {:?}\n", duration1);
    
    // Second call - should hit cache
    println!("📡 Second call (cache hit)...");
    let start = Instant::now();
    let response2 = client.chat_completion("gpt-4", messages.clone(), options.clone()).await?;
    let duration2 = start.elapsed();
    
    println!("✅ Response: {}", response2.content().unwrap_or("No content"));
    println!("⏱️  Duration: {:?}", duration2);
    println!("🚀 Speedup: {:.2}x faster!\n", duration1.as_secs_f64() / duration2.as_secs_f64());
    
    // Cache statistics
    if let Some(stats) = client.cache_stats() {
        println!("📊 Cache Stats:");
        println!("   Total entries: {}", stats.total_entries);
        println!("   Active entries: {}", stats.active_entries);
        println!("   Expired entries: {}", stats.expired_entries);
    }
    
    // Different request - will hit API again
    println!("\n📡 Different request (API hit)...");
    let messages2 = vec![
        Message::system("You are a helpful assistant."),
        Message::human("What is 3+3?"),  // Different question
    ];
    
    let start = Instant::now();
    let response3 = client.chat_completion("gpt-4", messages2, options.clone()).await?;
    let duration3 = start.elapsed();
    
    println!("✅ Response: {}", response3.content().unwrap_or("No content"));
    println!("⏱️  Duration: {:?}\n", duration3);
    
    // Cache statistics updated
    if let Some(stats) = client.cache_stats() {
        println!("📊 Updated Cache Stats:");
        println!("   Total entries: {}", stats.total_entries);
        println!("   Active entries: {}", stats.active_entries);
    }
    
    // Cleanup expired entries
    println!("\n🧹 Cleaning up cache...");
    client.cleanup_cache();
    
    // Clear cache entirely
    println!("🗑️  Clearing cache...");
    client.clear_cache();
    
    if let Some(stats) = client.cache_stats() {
        println!("📊 Cache after clear:");
        println!("   Total entries: {}", stats.total_entries);
    }
    
    Ok(())
}
