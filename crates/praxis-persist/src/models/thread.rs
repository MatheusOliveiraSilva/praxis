use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_summary_update: DateTime<Utc>,
    pub metadata: ThreadMetadata,
    pub summary: Option<ThreadSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadMetadata {
    pub title: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSummary {
    pub text: String,
    pub generated_at: DateTime<Utc>,
    pub total_tokens_before_summary: usize,
    pub messages_count: usize,
}

