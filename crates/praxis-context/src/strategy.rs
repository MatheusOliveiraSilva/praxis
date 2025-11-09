use std::sync::Arc;
use anyhow::Result;
use praxis_llm::Message;
use async_trait::async_trait;
use praxis_persist::PersistenceClient;

/// Result of context retrieval
#[derive(Debug, Clone)]
pub struct ContextWindow {
    pub system_prompt: String,
    pub messages: Vec<Message>,
}

/// Strategy for building context window from conversation history
#[async_trait]
pub trait ContextStrategy: Send + Sync {
    /// Get context window for a conversation
    async fn get_context_window(
        &self,
        thread_id: &str,
        persist_client: Arc<dyn PersistenceClient>,
    ) -> Result<ContextWindow>;
}

