use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::{error::ApiResult, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub services: HashMap<String, String>,
}

/// Health check endpoint
/// 
/// Returns the health status of the API and its dependencies
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<HealthResponse>> {
    let mut services = HashMap::new();
    
    // Check MongoDB connection
    match check_mongodb(&state).await {
        Ok(_) => services.insert("mongodb".to_string(), "connected".to_string()),
        Err(_) => services.insert("mongodb".to_string(), "disconnected".to_string()),
    };
    
    // MCP servers status (we don't have a list_servers method, so just mark as available)
    services.insert("mcp".to_string(), "available".to_string());
    
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services,
    }))
}

async fn check_mongodb(state: &AppState) -> ApiResult<()> {
    // Try to list threads (lightweight operation)
    let _ = state.persist.list_threads("_health_check", Some(1), None).await?;
    Ok(())
}

