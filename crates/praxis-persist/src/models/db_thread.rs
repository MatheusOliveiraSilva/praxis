use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Database-agnostic thread model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: ThreadMetadata,
    pub last_summary_update: DateTime<Utc>,
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

