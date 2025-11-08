use anyhow::Result;
use mongodb::bson::oid::ObjectId;
use praxis_persist::{
    PersistClient, ThreadMetadata, Message, MessageRole, MessageType,
};
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("MongoDB Connection Test");
    println!("=======================");
    println!();
    
    // Get MongoDB connection details
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string());
    let mongodb_database = std::env::var("MONGODB_DATABASE")
        .unwrap_or_else(|_| "praxis".to_string());
    
    // Connect to MongoDB
    print!("Connecting to {}... ", mongodb_uri);
    let client = PersistClient::new(&mongodb_uri, &mongodb_database).await?;
    println!("✓");
    println!("✓ Database: {}", mongodb_database);
    println!();
    
    // Create a test thread
    print!("Creating test thread... ");
    let test_user_id = format!("test_user_{}", chrono::Utc::now().timestamp());
    let metadata = ThreadMetadata {
        title: Some("Test Thread".to_string()),
        tags: vec!["test".to_string()],
    };
    let thread = client.threads().create_thread(test_user_id.clone(), metadata).await?;
    println!("✓");
    println!("✓ Created thread: {}", thread.id);
    println!();
    
    // Insert test messages
    print!("Inserting test messages... ");
    let messages = vec![
        Message {
            id: ObjectId::new(),
            thread_id: thread.id,
            user_id: test_user_id.clone(),
            role: MessageRole::User,
            message_type: MessageType::Message,
            content: "Hello, what's the weather?".to_string(),
            created_at: Utc::now(),
            duration_ms: None,
        },
        Message {
            id: ObjectId::new(),
            thread_id: thread.id,
            user_id: test_user_id.clone(),
            role: MessageRole::Assistant,
            message_type: MessageType::Reasoning,
            content: "I need to check the weather forecast...".to_string(),
            created_at: Utc::now(),
            duration_ms: Some(500),
        },
        Message {
            id: ObjectId::new(),
            thread_id: thread.id,
            user_id: test_user_id.clone(),
            role: MessageRole::Assistant,
            message_type: MessageType::ToolCall,
            content: "get_forecast".to_string(),
            created_at: Utc::now(),
            duration_ms: None,
        },
        Message {
            id: ObjectId::new(),
            thread_id: thread.id,
            user_id: test_user_id.clone(),
            role: MessageRole::Assistant,
            message_type: MessageType::ToolResult,
            content: "Sunny, 72°F".to_string(),
            created_at: Utc::now(),
            duration_ms: Some(250),
        },
        Message {
            id: ObjectId::new(),
            thread_id: thread.id,
            user_id: test_user_id.clone(),
            role: MessageRole::Assistant,
            message_type: MessageType::Message,
            content: "The weather is sunny with a temperature of 72°F.".to_string(),
            created_at: Utc::now(),
            duration_ms: Some(1200),
        },
    ];
    
    client.messages().save_messages(messages).await?;
    println!("✓");
    println!("✓ Inserted 5 messages");
    println!();
    
    // Test query by user_id (verify index usage)
    print!("Testing query by user_id... ");
    let user_threads = client.threads().list_threads(&test_user_id, 10).await?;
    assert_eq!(user_threads.len(), 1, "Expected 1 thread");
    println!("✓");
    println!("✓ Query by user_id: found {} thread", user_threads.len());
    println!();
    
    // Test query by thread_id (verify index usage)
    print!("Testing query by thread_id... ");
    let thread_messages = client.messages().get_messages(thread.id).await?;
    assert_eq!(thread_messages.len(), 5, "Expected 5 messages");
    println!("✓");
    println!("✓ Query by thread_id: found {} messages", thread_messages.len());
    println!();
    
    // Test pagination
    print!("Testing pagination... ");
    let paginated = client.messages().get_messages_paginated(thread.id, 2, None).await?;
    assert_eq!(paginated.len(), 2, "Expected 2 messages");
    println!("✓");
    println!("✓ Pagination test: passed");
    println!();
    
    // Test count
    print!("Testing message count... ");
    let count = client.messages().count_messages(thread.id).await?;
    assert_eq!(count, 5, "Expected 5 messages");
    println!("✓");
    println!("✓ Count test: {} messages", count);
    println!();
    
    // Test context manager
    print!("Testing context manager... ");
    let (context_messages, system_prompt) = client.context().get_context_window(thread.id).await?;
    assert!(!context_messages.is_empty(), "Expected messages in context");
    assert!(!system_prompt.is_empty(), "Expected system prompt");
    println!("✓");
    println!("✓ Context window: {} messages", context_messages.len());
    println!("✓ System prompt length: {} chars", system_prompt.len());
    println!();
    
    // Note: Cleanup is intentionally skipped for manual verification
    // Users can inspect the data in MongoDB Compass or mongosh
    println!("=======================");
    println!("All tests passed!");
    println!("=======================");
    println!();
    println!("Test data created:");
    println!("  User ID: {}", test_user_id);
    println!("  Thread ID: {}", thread.id);
    println!();
    println!("To inspect data:");
    println!("  docker exec -it praxis-mongo mongosh -u admin -p password123 praxis");
    println!("  db.threads.find({{}})");
    println!("  db.messages.find({{}})");
    println!();
    println!("To clean up test data:");
    println!("  db.threads.deleteMany({{ user_id: '{}' }})", test_user_id);
    println!("  db.messages.deleteMany({{ user_id: '{}' }})", test_user_id);
    
    Ok(())
}

