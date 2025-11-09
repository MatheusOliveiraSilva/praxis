use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use tiktoken_rs::cl100k_base;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;

use praxis_llm::{LLMClient, Message, Content};
use praxis_persist::{PersistenceClient, DBMessage};
use crate::strategy::{ContextStrategy, ContextWindow};

pub struct DefaultContextStrategy {
    max_tokens: usize,
    llm_client: Arc<dyn LLMClient>,
    system_prompt_template: String,
    summarization_template: String,
}

impl DefaultContextStrategy {
    pub fn new(
        max_tokens: usize,
        llm_client: Arc<dyn LLMClient>,
    ) -> Self {
        Self {
            max_tokens,
            llm_client,
            system_prompt_template: praxis_persist::DEFAULT_SYSTEM_PROMPT_TEMPLATE.to_string(),
            summarization_template: praxis_persist::DEFAULT_SUMMARIZATION_PROMPT.to_string(),
        }
    }
    
    pub fn with_templates(
        max_tokens: usize,
        llm_client: Arc<dyn LLMClient>,
        system_prompt_template: String,
        summarization_template: String,
    ) -> Self {
        Self {
            max_tokens,
            llm_client,
            system_prompt_template,
            summarization_template,
        }
    }
    
    /// Count tokens in messages using tiktoken
    fn count_tokens(&self, messages: &[DBMessage]) -> Result<usize> {
        let bpe = cl100k_base().map_err(|e| anyhow::anyhow!("Tokenizer error: {}", e))?;
        
        let mut total_tokens = 0;
        for msg in messages {
            let tokens = bpe.encode_with_special_tokens(&msg.content);
            total_tokens += tokens.len();
        }
        
        Ok(total_tokens)
    }
    
    /// Generate summary of old messages
    async fn generate_summary(&self, messages: &[DBMessage], previous_summary: Option<&str>) -> Result<String> {
        // Build conversation text from messages
        let conversation = messages.iter()
            .map(|m| {
                let role = match m.role {
                    praxis_persist::MessageRole::User => "User",
                    praxis_persist::MessageRole::Assistant => "Assistant",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        // Build summary prompt from template
        let previous_summary_text = previous_summary
            .unwrap_or("Não temos resumo ainda.");
        
        let summary_prompt = self.summarization_template
            .replace("<previous_summary>", previous_summary_text)
            + &format!("\n\nConversation:\n{}", conversation);
        
        // Call LLM to generate summary (non-streaming)
        let request = praxis_llm::ChatRequest::new(
            "gpt-4o-mini".to_string(),
            vec![Message::Human {
                content: Content::text(summary_prompt),
                name: None,
            }],
        );
        
        // Use streaming and collect the response
        let mut stream = self.llm_client.chat_completion_stream(request).await?;
        let mut summary = String::new();
        
        while let Some(event) = stream.try_next().await? {
            if let praxis_llm::StreamEvent::Message { content, .. } = event {
                summary.push_str(&content);
            }
        }
        
        Ok(summary)
    }
    
    /// Filter messages after a certain timestamp
    fn filter_recent_messages(&self, messages: &[DBMessage], after: DateTime<Utc>) -> Vec<DBMessage> {
        messages.iter()
            .filter(|m| m.created_at > after)
            .cloned()
            .collect()
    }
    
    /// Build system prompt with summary
    fn build_prompt_with_summary(&self, summary: String) -> String {
        self.system_prompt_template.replace("<summary>", &summary)
    }
    
    /// Default system prompt without summary
    fn build_prompt_without_summary(&self) -> String {
        self.system_prompt_template.replace("<summary>", "Não temos resumo ainda.")
    }
}

#[async_trait]
impl ContextStrategy for DefaultContextStrategy {
    async fn get_context_window(
        &self,
        thread_id: &str,
        persist_client: &dyn PersistenceClient,
    ) -> Result<ContextWindow> {
        // 1. Get all DBMessages from persistence
        let db_messages = persist_client.get_messages(thread_id).await?;
        
        // If no messages, return empty context
        if db_messages.is_empty() {
            return Ok(ContextWindow {
                system_prompt: self.build_prompt_without_summary(),
                messages: vec![],
            });
        }
        
        // 2. Count tokens
        let total_tokens = self.count_tokens(&db_messages)?;
        
        // 3. If over limit, generate summary
        let (system_prompt, messages_to_use) = if total_tokens > self.max_tokens {
            // Need to summarize - get thread to see if we already have a summary
            let thread = persist_client.get_thread(thread_id).await?;
            
            let (summary_text, summary_time) = if let Some(thread) = thread {
                if let Some(summary) = &thread.summary {
                    // Use existing summary
                    (summary.text.clone(), summary.generated_at)
                } else {
                    // Generate new summary (no previous summary)
                    let summary_text = self.generate_summary(&db_messages, None).await?;
                    let summary_time = Utc::now();
                    
                    // Save summary to persistence
                    persist_client.save_thread_summary(thread_id, summary_text.clone(), summary_time).await?;
                    
                    (summary_text, summary_time)
                }
            } else {
                // No thread found, generate summary anyway
                let summary_text = self.generate_summary(&db_messages, None).await?;
                (summary_text, Utc::now())
            };
            
            let recent_messages = self.filter_recent_messages(&db_messages, summary_time);
            (self.build_prompt_with_summary(summary_text), recent_messages)
        } else {
            (self.build_prompt_without_summary(), db_messages)
        };
        
        // 4. Convert DBMessage → praxis_llm::Message
        let llm_messages = messages_to_use
            .into_iter()
            .filter_map(|msg| msg.try_into().ok())
            .collect();
        
        Ok(ContextWindow {
            system_prompt,
            messages: llm_messages,
        })
    }
}

