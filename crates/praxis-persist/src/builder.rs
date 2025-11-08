use std::sync::Arc;
use std::path::Path;
use praxis_llm::LLMClient;

use crate::{PersistClient, templates::DEFAULT_SYSTEM_PROMPT_TEMPLATE};
use crate::error::{Result, PersistError};

pub struct PersistClientBuilder {
    mongodb_uri: Option<String>,
    database: Option<String>,
    max_tokens: usize,
    llm_client: Option<Arc<dyn LLMClient>>,
    system_prompt_template: String,
}

impl PersistClientBuilder {
    pub fn new() -> Self {
        Self {
            mongodb_uri: None,
            database: None,
            max_tokens: 30_000,
            llm_client: None,
            system_prompt_template: DEFAULT_SYSTEM_PROMPT_TEMPLATE.to_string(),
        }
    }
    
    pub fn mongodb_uri(mut self, uri: impl Into<String>) -> Self {
        self.mongodb_uri = Some(uri.into());
        self
    }
    
    pub fn database(mut self, db: impl Into<String>) -> Self {
        self.database = Some(db.into());
        self
    }
    
    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = tokens;
        self
    }
    
    pub fn llm_client(mut self, client: Arc<dyn LLMClient>) -> Self {
        self.llm_client = Some(client);
        self
    }
    
    pub fn system_prompt_template(mut self, template: impl Into<String>) -> Self {
        self.system_prompt_template = template.into();
        self
    }
    
    pub fn system_prompt_template_file(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let template = std::fs::read_to_string(path)
            .map_err(|e| PersistError::Internal(format!("Failed to read template file: {}", e)))?;
        self.system_prompt_template = template;
        Ok(self)
    }
    
    pub async fn build(self) -> Result<PersistClient> {
        let mongodb_uri = self.mongodb_uri
            .ok_or_else(|| PersistError::Internal("mongodb_uri is required".to_string()))?;
        let database = self.database
            .ok_or_else(|| PersistError::Internal("database is required".to_string()))?;
        let llm_client = self.llm_client
            .ok_or_else(|| PersistError::Internal("llm_client is required".to_string()))?;
        
        PersistClient::new_with_config(
            mongodb_uri,
            database,
            self.max_tokens,
            llm_client,
            self.system_prompt_template,
        ).await
    }
}

impl Default for PersistClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

