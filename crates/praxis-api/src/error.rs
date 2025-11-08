use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Thread not found: {0}")]
    ThreadNotFound(String),
    
    #[error("Message not found: {0}")]
    MessageNotFound(String),
    
    #[error("Invalid request: {0}")]
    BadRequest(String),
    
    #[error("Database error: {0}")]
    Database(#[from] mongodb::error::Error),
    
    #[error("BSON error: {0}")]
    Bson(#[from] bson::de::Error),
    
    #[error("Persistence error: {0}")]
    Persist(#[from] praxis_persist::PersistError),
    
    #[error("Graph execution error: {0}")]
    Graph(#[from] anyhow::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::ThreadNotFound(_) | ApiError::MessageNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            ApiError::BadRequest(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            ApiError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            ApiError::Persist(ref e) => {
                tracing::error!("Persistence error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Storage error".to_string())
            }
            ApiError::Graph(ref e) => {
                tracing::error!("Graph error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Processing error".to_string())
            }
            ApiError::Config(ref msg) => {
                tracing::error!("Config error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error".to_string())
            }
            ApiError::Bson(_) | ApiError::Internal => {
                tracing::error!("Internal error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };
        
        let body = Json(json!({
            "error": message
        }));
        
        (status, body).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

