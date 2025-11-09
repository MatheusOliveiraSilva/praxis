use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::models::{DBMessage, Thread, ThreadMetadata};
use crate::error::Result;

/// Trait for database persistence operations
/// 
/// Implementations provide database-specific CRUD operations
#[async_trait]
pub trait PersistenceClient: Send + Sync {
    /// Save a single message to the database
    async fn save_message(&self, message: DBMessage) -> Result<()>;
    
    /// Get all messages for a thread
    async fn get_messages(&self, thread_id: &str) -> Result<Vec<DBMessage>>;
    
    /// Get messages after a certain timestamp (for context window after summary)
    async fn get_messages_after(
        &self,
        thread_id: &str,
        after: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<DBMessage>>;
    
    /// Create a new thread
    async fn create_thread(&self, user_id: &str, metadata: ThreadMetadata) -> Result<Thread>;
    
    /// Get a thread by ID
    async fn get_thread(&self, thread_id: &str) -> Result<Option<Thread>>;
    
    /// Save a thread summary
    async fn save_thread_summary(
        &self,
        thread_id: &str,
        summary: String,
        generated_at: DateTime<Utc>,
    ) -> Result<()>;
    
    /// Delete a thread
    async fn delete_thread(&self, thread_id: &str, user_id: &str) -> Result<()>;
    
    /// List threads for a user
    async fn list_threads(
        &self,
        user_id: &str,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Thread>>;
}

