use mongodb::bson::oid::ObjectId;
use chrono::Utc;

use crate::models::{Message, ThreadSummary};
use crate::repositories::{ThreadRepository, MessageRepository};
use crate::error::{Result, PersistError};

pub struct ContextManager {
    thread_repo: ThreadRepository,
    message_repo: MessageRepository,
    max_tokens: usize,
}

impl ContextManager {
    pub fn new(
        thread_repo: ThreadRepository,
        message_repo: MessageRepository,
        max_tokens: usize,
    ) -> Self {
        Self {
            thread_repo,
            message_repo,
            max_tokens,
        }
    }
    
    /// Get context window for a thread
    /// Returns: (messages, system_prompt_with_summary)
    pub async fn get_context_window(
        &self,
        thread_id: ObjectId,
    ) -> Result<(Vec<Message>, String)> {
        // 1. Get thread (check for existing summary)
        let thread = self.thread_repo.get_thread(thread_id).await?
            .ok_or_else(|| PersistError::ThreadNotFound(thread_id.to_string()))?;
        
        // 2. Get all messages
        let messages = self.message_repo.get_messages(thread_id).await?;
        
        // 3. Count total tokens
        let total_tokens = self.count_tokens(&messages)?;
        
        // 4. If under limit, return all messages
        if total_tokens <= self.max_tokens {
            let system_prompt = self.build_system_prompt(thread.summary);
            return Ok((messages, system_prompt));
        }
        
        // 5. Over limit - trigger summarization if not already done
        let summary = if thread.summary.is_none() {
            // Generate summary
            let summary = self.generate_summary(&messages).await?;
            self.thread_repo.update_summary(thread_id, summary.clone()).await?;
            summary
        } else {
            thread.summary.unwrap()
        };
        
        // 6. Keep only recent messages after summary
        let messages_after_summary = self.get_messages_after_summary(&messages, &summary);
        
        // 7. Build system prompt with summary
        let system_prompt = self.build_system_prompt(Some(summary));
        
        Ok((messages_after_summary, system_prompt))
    }
    
    /// Count tokens in messages (stub: 1 token ≈ 4 characters)
    fn count_tokens(&self, messages: &[Message]) -> Result<usize> {
        let total_chars: usize = messages.iter()
            .map(|m| m.content.len())
            .sum();
        Ok(total_chars / 4)
    }
    
    /// Generate summary of messages (stub implementation without LLM)
    async fn generate_summary(&self, messages: &[Message]) -> Result<ThreadSummary> {
        // Build conversation text
        let conversation = messages.iter()
            .map(|m| {
                let role_str = match m.role {
                    crate::models::MessageRole::User => "User",
                    crate::models::MessageRole::Assistant => "Assistant",
                };
                format!("{}: {}", role_str, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        // For now, create a simple summary without calling LLM
        // In production, this would call the LLM to generate a real summary
        let summary_text = if conversation.len() > 500 {
            format!("{}... [conversation continues]", &conversation[..500])
        } else {
            conversation
        };
        
        let total_tokens = self.count_tokens(messages)?;
        
        Ok(ThreadSummary {
            text: summary_text,
            generated_at: Utc::now(),
            total_tokens_before_summary: total_tokens,
            messages_count: messages.len(),
        })
    }
    
    /// Get messages after summary was generated
    fn get_messages_after_summary(
        &self,
        messages: &[Message],
        summary: &ThreadSummary,
    ) -> Vec<Message> {
        messages.iter()
            .filter(|m| m.created_at > summary.generated_at)
            .cloned()
            .collect()
    }
    
    /// Build system prompt with optional summary
    fn build_system_prompt(&self, summary: Option<ThreadSummary>) -> String {
        let base_prompt = "You are a helpful AI assistant.";
        
        if let Some(summary) = summary {
            format!(
                "{}\n\nConversation summary (previous messages):\n{}",
                base_prompt,
                summary.text
            )
        } else {
            base_prompt.to_string()
        }
    }
}

