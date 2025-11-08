use mongodb::Client;
use std::sync::Arc;
use praxis_llm::LLMClient;

use crate::repositories::{ThreadRepository, MessageRepository};
use crate::context::ContextManager;
use crate::error::{Result, PersistError};
use crate::builder::PersistClientBuilder;

pub struct PersistClient {
    thread_repo: ThreadRepository,
    message_repo: MessageRepository,
    context_manager: ContextManager,
}

impl PersistClient {
    /// Create builder for fluent API
    pub fn builder() -> PersistClientBuilder {
        PersistClientBuilder::new()
    }
    
    /// Internal constructor (called by builder)
    pub(crate) async fn new_with_config(
        mongodb_uri: String,
        database: String,
        max_tokens: usize,
        llm_client: Arc<dyn LLMClient>,
        system_prompt_template: String,
    ) -> Result<Self> {
        let client = Client::with_uri_str(&mongodb_uri)
            .await
            .map_err(|e| PersistError::Connection(e.to_string()))?;
        
        let thread_repo = ThreadRepository::new(&client, &database);
        let message_repo = MessageRepository::new(&client, &database);
        
        let context_manager = ContextManager::new(
            ThreadRepository::new(&client, &database),
            MessageRepository::new(&client, &database),
            max_tokens,
            llm_client,
            system_prompt_template,
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

