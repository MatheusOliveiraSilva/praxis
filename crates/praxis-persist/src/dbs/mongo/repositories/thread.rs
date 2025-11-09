#[cfg(feature = "mongodb")]
use mongodb::{Client, Collection, bson::doc, bson::oid::ObjectId};
#[cfg(feature = "mongodb")]
use futures::TryStreamExt;
#[cfg(feature = "mongodb")]
use chrono::Utc;

#[cfg(feature = "mongodb")]
use crate::dbs::mongo::models::MongoThread;
#[cfg(feature = "mongodb")]
use crate::models::{ThreadMetadata, ThreadSummary};
#[cfg(feature = "mongodb")]
use crate::error::Result;

#[cfg(feature = "mongodb")]
#[derive(Clone)]
pub struct MongoThreadRepository {
    collection: Collection<MongoThread>,
}

#[cfg(feature = "mongodb")]
impl MongoThreadRepository {
    pub fn new(client: &Client, db_name: &str) -> Self {
        let collection = client.database(db_name).collection("threads");
        Self { collection }
    }
    
    /// Create a new thread
    pub async fn create_thread(
        &self,
        user_id: String,
        metadata: ThreadMetadata,
    ) -> Result<MongoThread> {
        let now = Utc::now();
        let thread = MongoThread {
            id: ObjectId::new(),
            user_id,
            created_at: now,
            updated_at: now,
            metadata,
            last_summary_update: now,
            summary: None,
        };
        
        self.collection.insert_one(&thread).await?;
        Ok(thread)
    }
    
    /// Get thread by ID
    pub async fn get_thread(&self, thread_id: ObjectId) -> Result<Option<MongoThread>> {
        let filter = doc! { "_id": thread_id };
        Ok(self.collection.find_one(filter).await?)
    }
    
    /// List threads for a user
    pub async fn list_threads(
        &self,
        user_id: &str,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<MongoThread>> {
        let filter = doc! { "user_id": user_id };
        let mut find_opts = self.collection
            .find(filter)
            .sort(doc! { "updated_at": -1 });
        
        if let Some(limit) = limit {
            find_opts = find_opts.limit(limit);
        }
        if let Some(skip) = skip {
            find_opts = find_opts.skip(skip.try_into().unwrap_or(0));
        }
        
        let threads = find_opts
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
        let now = bson::DateTime::now();
        let filter = doc! { "_id": thread_id };
        let update = doc! {
            "$set": {
                "summary": bson::to_bson(&summary)?,
                "last_summary_update": now,
                "updated_at": now
            }
        };
        
        self.collection.update_one(filter, update).await?;
        Ok(())
    }
    
    /// Delete thread
    pub async fn delete_thread(&self, thread_id: ObjectId, user_id: &str) -> Result<()> {
        let filter = doc! { "_id": thread_id, "user_id": user_id };
        self.collection.delete_one(filter).await?;
        Ok(())
    }
}

