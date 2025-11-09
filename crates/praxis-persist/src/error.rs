use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistError {
    #[cfg(feature = "mongodb")]
    #[error("Database error: {0}")]
    Database(#[from] mongodb::error::Error),
    
    #[cfg(feature = "mongodb")]
    #[error("BSON serialization error: {0}")]
    BsonSerialization(#[from] bson::ser::Error),
    
    #[cfg(feature = "mongodb")]
    #[error("BSON deserialization error: {0}")]
    BsonDeserialization(#[from] bson::de::Error),
    
    #[error("Thread not found: {0}")]
    ThreadNotFound(String),
    
    #[error("Message not found: {0}")]
    MessageNotFound(String),
    
    #[error("Invalid object ID: {0}")]
    InvalidObjectId(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

// Allow conversion from anyhow::Error
impl From<anyhow::Error> for PersistError {
    fn from(err: anyhow::Error) -> Self {
        PersistError::Other(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PersistError>;

