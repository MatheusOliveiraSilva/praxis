use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use tiktoken_rs::cl100k_base;
use chrono::Utc;

use praxis_llm::{LLMClient, Message, Content};
use praxis_persist::{PersistenceClient, DBMessage};
use crate::strategy::{ContextStrategy, ContextWindow};
use crate::templates::{DEFAULT_SYSTEM_PROMPT_TEMPLATE, DEFAULT_SUMMARIZATION_PROMPT};

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
            system_prompt_template: DEFAULT_SYSTEM_PROMPT_TEMPLATE.to_string(),
            summarization_template: DEFAULT_SUMMARIZATION_PROMPT.to_string(),
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
    
    /// Generate summary of old messages.
    async fn generate_summary(&self, messages: &[DBMessage], previous_summary: Option<&str>) -> Result<String> {
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
        
        let previous_summary_text = previous_summary
            .unwrap_or("Não temos resumo ainda.");
        
        let summary_prompt = self.summarization_template
            .replace("<previous_summary>", previous_summary_text)
            .replace("<conversation>", &conversation);
        
        let request = praxis_llm::ChatRequest::new(
            "gpt-4o-mini".to_string(),
            vec![Message::Human {
                content: Content::text(summary_prompt),
                name: None,
            }],
        );
        
        let response = self.llm_client.chat_completion(request).await?;
        
        let summary = response.content.unwrap_or_else(|| "Summary generation failed".to_string());
        
        Ok(summary)
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
        
        // 3. If over limit, use summary + recent messages
        let (system_prompt, messages_to_use) = if total_tokens > self.max_tokens {
            // Get thread to check if we have a summary
            let thread = persist_client.get_thread(thread_id).await?;
            
            let (summary_text, summary_time) = if let Some(thread) = &thread {
                if let Some(summary) = &thread.summary {
                    // Use existing summary - just get messages after summary
                    (summary.text.clone(), summary.generated_at)
                } else {
                    // Need to generate new summary (first time - blocking)
                    // In production, you might want to do this async with a job queue
                    let summary_text = self.generate_summary(&db_messages, None).await?;
                    let summary_time = Utc::now();
                    
                    // Save summary
                    persist_client.save_thread_summary(thread_id, summary_text.clone(), summary_time).await?;
                    
                    (summary_text, summary_time)
                }
            } else {
                // No thread found, use all messages for now
                return Ok(ContextWindow {
                    system_prompt: self.build_prompt_without_summary(),
                    messages: db_messages.into_iter().filter_map(|msg| msg.try_into().ok()).collect(),
                });
            };
            
            // Get only recent messages after summary (filter in DB!)
            let recent_messages = persist_client.get_messages_after(thread_id, summary_time).await?;
            
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

