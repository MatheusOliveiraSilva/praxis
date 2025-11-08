use mongodb::{Client, Collection, bson::doc};
use mongodb::bson::oid::ObjectId;
use futures::TryStreamExt;
use chrono::Utc;

use crate::models::{Thread, ThreadMetadata, ThreadSummary};
use crate::error::Result;

pub struct ThreadRepository {
    collection: Collection<Thread>,
}

impl ThreadRepository {
    pub fn new(client: &Client, db_name: &str) -> Self {
        let collection = client.database(db_name).collection("threads");
        Self { collection }
    }
    
    /// Create a new thread
    pub async fn create_thread(
        &self,
        user_id: String,
        metadata: ThreadMetadata,
    ) -> Result<Thread> {
        let thread = Thread {
            id: ObjectId::new(),
            user_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata,
            summary: None,
        };
        
        self.collection.insert_one(&thread).await?;
        Ok(thread)
    }
    
    /// Get thread by ID
    pub async fn get_thread(&self, thread_id: ObjectId) -> Result<Option<Thread>> {
        let filter = doc! { "_id": thread_id };
        Ok(self.collection.find_one(filter).await?)
    }
    
    /// List threads for a user
    pub async fn list_threads(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<Thread>> {
        let filter = doc! { "user_id": user_id };
        let threads = self.collection
            .find(filter)
            .sort(doc! { "updated_at": -1 })
            .limit(limit)
            .await?
            .try_collect()
            .await?;
        Ok(threads)
    }
    
    /// Update thread summary
    pub async fn update_summary(
        &self,
        thread_id: ObjectId,
        summary: ThreadSummary,
    ) -> Result<()> {
        let filter = doc! { "_id": thread_id };
        let update = doc! {
            "$set": {
                "summary": bson::to_bson(&summary)?,
                "updated_at": bson::DateTime::now()
            }
        };
        
        self.collection.update_one(filter, update).await?;
        Ok(())
    }
    
    /// Touch thread (update updated_at)
    pub async fn touch_thread(&self, thread_id: ObjectId) -> Result<()> {
        let filter = doc! { "_id": thread_id };
        let update = doc! { "$set": { "updated_at": bson::DateTime::now() } };
        self.collection.update_one(filter, update).await?;
        Ok(())
    }
}

