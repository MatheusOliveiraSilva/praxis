use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistError {
    #[error("Database error: {0}")]
    Database(#[from] mongodb::error::Error),
    
    #[error("BSON serialization error: {0}")]
    BsonSerialization(#[from] bson::ser::Error),
    
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
}

pub type Result<T> = std::result::Result<T, PersistError>;

