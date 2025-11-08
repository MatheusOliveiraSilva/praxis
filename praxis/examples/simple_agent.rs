//! Simple AI agent example using the Praxis framework
//!
//! This example shows how to create a basic AI agent with just
//! MongoDB persistence and OpenAI integration.
//!
//! # Usage
//!
//! ```bash
//! # Set environment variables
//! export OPENAI_API_KEY=sk-...
//! export MONGODB_URI=mongodb://localhost:27017
//!
//! # Run the example
//! cargo run --example simple_agent
//! ```

use praxis::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🤖 Praxis Simple Agent Example\n");

    // Get configuration from environment
    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY not set");
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    // Create agent with builder pattern
    println!("📦 Building agent...");
    let agent = AgentBuilder::new()
        .mongodb(&mongodb_uri, "praxis_example")
        .openai_key(&openai_key)
        .model("gpt-4o")
        .temperature(0.7)
        .max_tokens(30_000)
        .build()
        .await?;

    println!("✅ Agent ready!\n");

    // Example conversations
    let queries = vec![
        "What is 2+2?",
        "Explain Rust ownership in one sentence.",
        "What is the capital of Brazil?",
    ];

    for (i, query) in queries.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Query {}: {}", i + 1, query);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let response = agent.chat(query).await?;

        println!("Response: {}\n", response);
    }

    println!("✨ Done!");

    Ok(())
}
