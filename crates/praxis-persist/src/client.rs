use mongodb::Client;

use crate::repositories::{ThreadRepository, MessageRepository};
use crate::context::ContextManager;
use crate::error::{Result, PersistError};

pub struct PersistClient {
    thread_repo: ThreadRepository,
    message_repo: MessageRepository,
    context_manager: ContextManager,
}

impl PersistClient {
    pub async fn new(mongodb_uri: &str, db_name: &str) -> Result<Self> {
        let client = Client::with_uri_str(mongodb_uri)
            .await
            .map_err(|e| PersistError::Connection(e.to_string()))?;
        
        let thread_repo = ThreadRepository::new(&client, db_name);
        let message_repo = MessageRepository::new(&client, db_name);
        
        // Default max_tokens for context window (8000 tokens)
        let context_manager = ContextManager::new(
            ThreadRepository::new(&client, db_name),
            MessageRepository::new(&client, db_name),
            8000,
        );
        
        Ok(Self {
            thread_repo,
            message_repo,
            context_manager,
        })
    }
    
    pub fn threads(&self) -> &ThreadRepository {
        &self.thread_repo
    }
    
    pub fn messages(&self) -> &MessageRepository {
        &self.message_repo
    }
    
    pub fn context(&self) -> &ContextManager {
        &self.context_manager
    }
}

