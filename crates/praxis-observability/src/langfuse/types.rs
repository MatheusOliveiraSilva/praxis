use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request body for creating a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceBody {
    pub id: String,
    pub name: Option<String>,
    pub user_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub tags: Option<Vec<String>>,
    pub timestamp: Option<String>,
}

/// Request body for creating a span
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpanBody {
    pub id: String,
    pub trace_id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub level: Option<String>,
    pub status_message: Option<String>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
}

/// Request body for creating a generation (LLM call)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationBody {
    pub id: String,
    pub trace_id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub model: String,
    pub model_parameters: Option<HashMap<String, serde_json::Value>>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub level: Option<String>,
    pub status_message: Option<String>,
    pub usage: Option<UsageInfo>,
}

/// Token usage information for LLM calls
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageInfo {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

/// Request body for updating a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceUpdateBody {
    pub id: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub tags: Option<Vec<String>>,
    pub output: Option<serde_json::Value>,
}

/// Batch ingestion request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionBatch {
    pub batch: Vec<IngestionEvent>,
}

/// Individual ingestion event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestionEvent {
    pub id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub body: serde_json::Value,
}

/// Generic API response
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse {
    pub id: Option<String>,
    pub message: Option<String>,
}

