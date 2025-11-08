use anyhow::Result;
use mongodb::bson::oid::ObjectId;
use praxis_persist::{
    PersistClient, ThreadMetadata, Message, MessageRole, MessageType,
};
use praxis_llm::OpenAIClient;
use chrono::Utc;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Praxis Persistence - Simple Example");
    println!("====================================\n");
    
    // 1. Connect to MongoDB with builder API
    println!("1. Connecting to MongoDB...");
    let mongodb_uri = "mongodb://admin:password123@localhost:27017";
    let mongodb_database = "praxis";
    
    // Get API key for LLM
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY required for summarization");
    
    let llm_client = Arc::new(OpenAIClient::new(api_key)?);
    
    let client = PersistClient::builder()
        .mongodb_uri(mongodb_uri)
        .database(mongodb_database)
        .max_tokens(30_000)
        .llm_client(llm_client)
        .build()
        .await?;
    
    println!("   ✓ Connected!\n");
    
    // 2. Create a thread
    println!("2. Creating a new conversation thread...");
    let metadata = ThreadMetadata {
        title: Some("Simple Example Chat".to_string()),
        tags: vec!["example".to_string(), "demo".to_string()],
    };
    
    let thread = client
        .threads()
        .create_thread("user_123".to_string(), metadata)
        .await?;
    
    println!("   ✓ Thread created: {}", thread.id);
    println!("   User: {}", thread.user_id);
    println!("   Title: {:?}\n", thread.metadata.title);
    
    // 3. Add a user message
    println!("3. Adding user message...");
    let user_message = Message {
        id: ObjectId::new(),
        thread_id: thread.id,
        user_id: "user_123".to_string(),
        role: MessageRole::User,
        message_type: MessageType::Message,
        content: "Hello! Can you help me with Rust?".to_string(),
        created_at: Utc::now(),
        duration_ms: None,
    };
    
    client.messages().save_message(user_message).await?;
    println!("   ✓ User message saved\n");
    
    // 4. Add assistant messages (reasoning + response)
    println!("4. Adding assistant messages...");
    let assistant_messages = vec![
        Message {
            id: ObjectId::new(),
            thread_id: thread.id,
            user_id: "user_123".to_string(),
            role: MessageRole::Assistant,
            message_type: MessageType::Reasoning,
            content: "The user is asking about Rust. I should provide helpful information.".to_string(),
            created_at: Utc::now(),
            duration_ms: Some(300),
        },
        Message {
            id: ObjectId::new(),
            thread_id: thread.id,
            user_id: "user_123".to_string(),
            role: MessageRole::Assistant,
            message_type: MessageType::Message,
            content: "Of course! Rust is a systems programming language focused on safety, speed, and concurrency. What would you like to know?".to_string(),
            created_at: Utc::now(),
            duration_ms: Some(1200),
        },
    ];
    
    client.messages().save_messages(assistant_messages).await?;
    println!("   ✓ Assistant messages saved\n");
    
    // 5. Retrieve all messages
    println!("5. Retrieving conversation history...");
    let messages = client.messages().get_messages(thread.id).await?;
    println!("   ✓ Found {} messages:\n", messages.len());
    
    for (i, msg) in messages.iter().enumerate() {
        println!("   [{}] {:?} - {:?}", i + 1, msg.role, msg.message_type);
        println!("       {}", msg.content);
        if let Some(duration) = msg.duration_ms {
            println!("       (took {}ms)", duration);
        }
        println!();
    }
    
    // 6. Get context window (with auto-summarization if needed)
    println!("6. Getting context window...");
    let (context_messages, system_prompt) = client
        .context()
        .get_context_window(thread.id)
        .await?;
    
    println!("   ✓ Context window ready");
    println!("   Messages in context: {}", context_messages.len());
    println!("   System prompt: {}\n", system_prompt);
    
    // 7. List user's threads
    println!("7. Listing user's threads...");
    let user_threads = client.threads().list_threads("user_123", 10).await?;
    println!("   ✓ Found {} thread(s):\n", user_threads.len());
    
    for thread in user_threads {
        println!("   - {} ({})", thread.id, thread.metadata.title.unwrap_or_default());
        println!("     Created: {}", thread.created_at);
        println!("     Tags: {:?}\n", thread.metadata.tags);
    }
    
    println!("====================================");
    println!("Example completed successfully!");
    println!("====================================\n");
    println!("Thread ID: {}", thread.id);
    println!("\nTo view in MongoDB:");
    println!("  docker exec -it praxis-mongo mongosh -u admin -p password123 praxis");
    println!("  db.threads.findOne({{ _id: ObjectId('{}') }})", thread.id);
    println!("  db.messages.find({{ thread_id: ObjectId('{}') }})", thread.id);
    
    Ok(())
}

