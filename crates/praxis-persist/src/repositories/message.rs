use mongodb::{Client, Collection, bson::doc};
use mongodb::bson::oid::ObjectId;
use futures::TryStreamExt;
use chrono::{DateTime, Utc};

use crate::models::Message;
use crate::error::Result;

#[derive(Clone)]
pub struct MessageRepository {
    collection: Collection<Message>,
}

impl MessageRepository {
    pub fn new(client: &Client, db_name: &str) -> Self {
        let collection = client.database(db_name).collection("messages");
        Self { collection }
    }
    
    /// Save a single message
    pub async fn save_message(&self, message: Message) -> Result<ObjectId> {
        self.collection.insert_one(&message).await?;
        Ok(message.id)
    }
    
    /// Save multiple messages (batch)
    pub async fn save_messages(&self, messages: Vec<Message>) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }
        self.collection.insert_many(messages).await?;
        Ok(())
    }
    
    /// Get all messages for a thread
    pub async fn get_messages(&self, thread_id: ObjectId) -> Result<Vec<Message>> {
        let filter = doc! { "thread_id": thread_id };
        let messages = self.collection
            .find(filter)
            .sort(doc! { "created_at": 1 })
            .await?
            .try_collect()
            .await?;
        Ok(messages)
    }
    
    /// Get messages with pagination
    pub async fn get_messages_paginated(
        &self,
        thread_id: ObjectId,
        limit: i64,
        before: Option<ObjectId>,
    ) -> Result<Vec<Message>> {
        let mut filter = doc! { "thread_id": thread_id };
        if let Some(before_id) = before {
            filter.insert("_id", doc! { "$lt": before_id });
        }
        
        let mut messages: Vec<Message> = self.collection
            .find(filter)
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?
            .try_collect()
            .await?;
        messages.reverse(); // Return in chronological order
        Ok(messages)
    }
    
    /// Count messages in a thread
    pub async fn count_messages(&self, thread_id: ObjectId) -> Result<u64> {
        let filter = doc! { "thread_id": thread_id };
        Ok(self.collection.count_documents(filter).await?)
    }
    
    /// Get messages created after a specific timestamp
    pub async fn get_messages_after(
        &self,
        thread_id: ObjectId,
        after: DateTime<Utc>,
    ) -> Result<Vec<Message>> {
        let filter = doc! {
            "thread_id": thread_id,
            "created_at": { "$gt": bson::DateTime::from_millis(after.timestamp_millis()) }
        };
        
        let messages = self.collection
            .find(filter)
            .sort(doc! { "created_at": 1 })
            .await?
            .try_collect()
            .await?;
        
        Ok(messages)
    }
}

