#[cfg(feature = "mongodb")]
use mongodb::{Client, bson::oid::ObjectId};
#[cfg(feature = "mongodb")]
use async_trait::async_trait;
#[cfg(feature = "mongodb")]
use chrono::{DateTime, Utc};

#[cfg(feature = "mongodb")]
use crate::trait_client::PersistenceClient;
#[cfg(feature = "mongodb")]
use crate::models::{DBMessage, Thread, ThreadMetadata, ThreadSummary};
#[cfg(feature = "mongodb")]
use crate::dbs::mongo::models::MongoMessage;
#[cfg(feature = "mongodb")]
use crate::dbs::mongo::repositories::{MongoMessageRepository, MongoThreadRepository};
#[cfg(feature = "mongodb")]
use crate::error::{Result, PersistError};

#[cfg(feature = "mongodb")]
pub struct MongoPersistenceClient {
    message_repo: MongoMessageRepository,
    thread_repo: MongoThreadRepository,
}

#[cfg(feature = "mongodb")]
impl MongoPersistenceClient {
    /// Connect to MongoDB and create client
    pub async fn connect(mongodb_uri: &str, database: &str) -> Result<Self> {
        let client = Client::with_uri_str(mongodb_uri)
            .await
            .map_err(|e| PersistError::Connection(e.to_string()))?;
        
        let message_repo = MongoMessageRepository::new(&client, database);
        let thread_repo = MongoThreadRepository::new(&client, database);
        
        Ok(Self {
            message_repo,
            thread_repo,
        })
    }
}

#[cfg(feature = "mongodb")]
#[async_trait]
impl PersistenceClient for MongoPersistenceClient {
    async fn save_message(&self, message: DBMessage) -> Result<()> {
        let mongo_message: MongoMessage = message.into();
        self.message_repo.save_message(mongo_message).await?;
        Ok(())
    }
    
    async fn get_messages(&self, thread_id: &str) -> Result<Vec<DBMessage>> {
        let object_id = ObjectId::parse_str(thread_id)
            .map_err(|e| PersistError::InvalidObjectId(e.to_string()))?;
        
        let mongo_messages = self.message_repo.get_messages(object_id).await?;
        let db_messages = mongo_messages.into_iter().map(|m| m.into()).collect();
        Ok(db_messages)
    }
    
    async fn get_messages_after(
        &self,
        thread_id: &str,
        after: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<DBMessage>> {
        let object_id = ObjectId::parse_str(thread_id)
            .map_err(|e| PersistError::InvalidObjectId(e.to_string()))?;
        
        let mongo_messages = self.message_repo.get_messages_after(object_id, after).await?;
        let db_messages = mongo_messages.into_iter().map(|m| m.into()).collect();
        Ok(db_messages)
    }
    
    async fn create_thread(&self, user_id: &str, metadata: ThreadMetadata) -> Result<Thread> {
        let mongo_thread = self.thread_repo.create_thread(user_id.to_string(), metadata).await?;
        Ok(mongo_thread.into())
    }
    
    async fn get_thread(&self, thread_id: &str) -> Result<Option<Thread>> {
        let object_id = ObjectId::parse_str(thread_id)
            .map_err(|e| PersistError::InvalidObjectId(e.to_string()))?;
        
        let mongo_thread = self.thread_repo.get_thread(object_id).await?;
        Ok(mongo_thread.map(|t| t.into()))
    }
    
    async fn save_thread_summary(
        &self,
        thread_id: &str,
        summary: String,
        generated_at: DateTime<Utc>,
    ) -> Result<()> {
        let object_id = ObjectId::parse_str(thread_id)
            .map_err(|e| PersistError::InvalidObjectId(e.to_string()))?;
        
        let thread_summary = ThreadSummary {
            text: summary,
            generated_at,
            total_tokens_before_summary: 0, // TODO: calculate this properly
            messages_count: 0, // TODO: calculate this properly
        };
        
        self.thread_repo.update_summary(object_id, thread_summary).await?;
        Ok(())
    }
    
    async fn delete_thread(&self, thread_id: &str, user_id: &str) -> Result<()> {
        let object_id = ObjectId::parse_str(thread_id)
            .map_err(|e| PersistError::InvalidObjectId(e.to_string()))?;
        
        self.thread_repo.delete_thread(object_id, user_id).await?;
        Ok(())
    }
    
    async fn list_threads(
        &self,
        user_id: &str,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<Thread>> {
        let mongo_threads = self.thread_repo.list_threads(user_id, limit, skip).await?;
        let threads = mongo_threads.into_iter().map(|t| t.into()).collect();
        Ok(threads)
    }
}

