#[cfg(feature = "mongodb")]
use mongodb::{Client, Collection, bson, bson::doc, bson::oid::ObjectId};
#[cfg(feature = "mongodb")]
use futures::TryStreamExt;

#[cfg(feature = "mongodb")]
use crate::dbs::mongo::models::MongoMessage;
#[cfg(feature = "mongodb")]
use crate::error::Result;

#[cfg(feature = "mongodb")]
#[derive(Clone)]
pub struct MongoMessageRepository {
    collection: Collection<MongoMessage>,
}

#[cfg(feature = "mongodb")]
impl MongoMessageRepository {
    pub fn new(client: &Client, db_name: &str) -> Self {
        let collection = client.database(db_name).collection("messages");
        Self { collection }
    }
    
    /// Save a single message
    pub async fn save_message(&self, message: MongoMessage) -> Result<ObjectId> {
        self.collection.insert_one(&message).await?;
        Ok(message.id)
    }
    
    /// Get all messages for a thread
    pub async fn get_messages(&self, thread_id: ObjectId) -> Result<Vec<MongoMessage>> {
        let filter = doc! { "thread_id": thread_id };
        let messages = self.collection
            .find(filter)
            .sort(doc! { "created_at": 1 })
            .await?
            .try_collect()
            .await?;
        Ok(messages)
    }
    
    /// Get messages after a certain timestamp
    pub async fn get_messages_after(
        &self,
        thread_id: ObjectId,
        after: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<MongoMessage>> {
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

