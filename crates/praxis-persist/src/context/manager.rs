use mongodb::bson::oid::ObjectId;
use chrono::Utc;
use std::sync::Arc;
use praxis_llm::LLMClient;

use crate::models::{Message, ThreadSummary, MessageRole, MessageType};
use crate::repositories::{ThreadRepository, MessageRepository};
use crate::error::{Result, PersistError};
use crate::templates::DEFAULT_SUMMARIZATION_PROMPT;

pub struct ContextManager {
    thread_repo: ThreadRepository,
    message_repo: MessageRepository,
    max_tokens: usize,
    llm_client: Arc<dyn LLMClient>,
    system_prompt_template: String,
}

impl ContextManager {
    pub fn new(
        thread_repo: ThreadRepository,
        message_repo: MessageRepository,
        max_tokens: usize,
        llm_client: Arc<dyn LLMClient>,
        system_prompt_template: String,
    ) -> Self {
        Self {
            thread_repo,
            message_repo,
            max_tokens,
            llm_client,
            system_prompt_template,
        }
    }
    
    /// Clone Arc-wrapped fields for async task
    fn clone_for_async(&self) -> Self {
        Self {
            thread_repo: self.thread_repo.clone(),
            message_repo: self.message_repo.clone(),
            max_tokens: self.max_tokens,
            llm_client: Arc::clone(&self.llm_client),
            system_prompt_template: self.system_prompt_template.clone(),
        }
    }
    
    /// Get context window for a thread
    /// Returns: (messages, system_prompt_with_summary)
    pub async fn get_context_window(
        &self,
        thread_id: ObjectId,
    ) -> Result<(Vec<Message>, String)> {
        // 1. Get thread
        let thread = self.thread_repo.get_thread(thread_id).await?
            .ok_or_else(|| PersistError::ThreadNotFound(thread_id.to_string()))?;
        
        // 2. Get messages after last_summary_update
        let messages = self.message_repo
            .get_messages_after(thread_id, thread.last_summary_update)
            .await?;
        
        // 3. Count tokens
        let total_tokens = self.count_tokens(&messages)?;
        
        // 4. Check if summarization needed
        if total_tokens > self.max_tokens {
            // Trigger async summarization (fire-and-forget)
            let context = self.clone_for_async();
            let previous_summary = thread.summary.clone();
            let messages_clone = messages.clone();
            
            tokio::spawn(async move {
                if let Err(e) = context.generate_and_save_summary(
                    thread_id,
                    messages_clone,
                    previous_summary,
                ).await {
                    tracing::error!(
                        "Failed to generate summary for thread {}: {}",
                        thread_id,
                        e
                    );
                }
            });
            
            // Return immediately with current data
            let system_prompt = self.build_system_prompt(thread.summary.as_ref());
            return Ok((messages, system_prompt));
        }
        
        // 5. Under limit, return normally
        let system_prompt = self.build_system_prompt(thread.summary.as_ref());
        Ok((messages, system_prompt))
    }
    
    /// Count tokens in messages (stub: 1 token ≈ 4 characters)
    fn count_tokens(&self, messages: &[Message]) -> Result<usize> {
        let total_chars: usize = messages.iter()
            .map(|m| m.content.len())
            .sum();
        Ok(total_chars / 4)
    }
    
    /// Generate summary using real LLM
    async fn generate_summary(
        &self,
        messages: &[Message],
        previous_summary: Option<&str>,
    ) -> Result<ThreadSummary> {
        use praxis_llm::{ChatRequest, Message as LLMMessage, Content};
        
        // 1. Build system message from template
        let previous_summary_text = previous_summary.unwrap_or("None");
        let system_prompt = DEFAULT_SUMMARIZATION_PROMPT
            .replace("<previous_summary>", previous_summary_text);
        
        let mut llm_messages = vec![
            LLMMessage::System {
                content: Content::text(system_prompt),
                name: None,
            }
        ];
        
        // 2. Convert DB messages to LLM messages (only Message type, skip tool calls)
        for msg in messages.iter().filter(|m| matches!(m.message_type, MessageType::Message)) {
            let llm_msg = match msg.role {
                MessageRole::User => LLMMessage::Human {
                    content: Content::text(&msg.content),
                    name: None,
                },
                MessageRole::Assistant => LLMMessage::AI {
                    content: Some(Content::text(&msg.content)),
                    tool_calls: None,
                    name: None,
                },
            };
            llm_messages.push(llm_msg);
        }
        
        // 3. Call LLM with simple request
        let request = ChatRequest::new(
            "gpt-4o-mini".to_string(),  // Use cheaper model for summarization
            llm_messages,
        );
        
        let response = self.llm_client.chat_completion(request).await?;
        
        // 4. Extract summary text
        let summary_text = response.content
            .ok_or_else(|| PersistError::Internal("LLM returned no content for summary".to_string()))?;
        
        // 5. Create summary metadata
        let total_tokens = self.count_tokens(messages)?;
        
        Ok(ThreadSummary {
            text: summary_text,
            generated_at: Utc::now(),
            total_tokens_before_summary: total_tokens,
            messages_count: messages.len(),
        })
    }
    
    /// Generate summary and save to database
    async fn generate_and_save_summary(
        &self,
        thread_id: ObjectId,
        messages: Vec<Message>,
        previous_summary: Option<ThreadSummary>,
    ) -> Result<()> {
        // 1. Generate summary
        let previous_text = previous_summary.as_ref().map(|s| s.text.as_str());
        let summary = self.generate_summary(&messages, previous_text).await?;
        
        // 2. Save to database
        self.thread_repo.update_summary(
            thread_id,
            summary,
            Utc::now(),  // New last_summary_update timestamp
        ).await?;
        
        tracing::info!("Summary generated for thread {}", thread_id);
        Ok(())
    }
    
    /// Build system prompt with optional summary
    fn build_system_prompt(&self, summary: Option<&ThreadSummary>) -> String {
        let summary_text = if let Some(summary) = summary {
            &summary.text
        } else {
            "No summary available."
        };
        
        // Replace <summary> placeholder in template
        self.system_prompt_template.replace("<summary>", summary_text)
    }
}

