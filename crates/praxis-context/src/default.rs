use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use tiktoken_rs::cl100k_base;
use chrono::Utc;

use praxis_llm::{ChatClient, Message, Content};
use praxis_persist::{PersistenceClient, DBMessage};
use crate::strategy::{ContextStrategy, ContextWindow};
use crate::templates::{DEFAULT_SYSTEM_PROMPT_TEMPLATE, DEFAULT_SUMMARIZATION_PROMPT};

pub struct DefaultContextStrategy {
    max_tokens: usize,
    llm_client: Arc<dyn ChatClient>,
    system_prompt_template: String,
    summarization_template: String,
}

impl DefaultContextStrategy {
    pub fn new(
        max_tokens: usize,
        llm_client: Arc<dyn ChatClient>,
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
        llm_client: Arc<dyn ChatClient>,
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
    
    /// Build conversation text from messages
    fn build_conversation_text(messages: &[DBMessage]) -> String {
        messages.iter()
            .map(|m| {
                let role = match m.role {
                    praxis_persist::MessageRole::User => "User",
                    praxis_persist::MessageRole::Assistant => "Assistant",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Generate summary of old messages
    async fn generate_summary(&self, messages: &[DBMessage], previous_summary: Option<&str>) -> Result<String> {
        let conversation = Self::build_conversation_text(messages);
        
        let previous_summary_text = previous_summary.unwrap_or("Não temos resumo ainda.");
        
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
        
        let response = self.llm_client.chat(request).await?;
        
        let summary = response.content.unwrap_or_else(|| "Summary generation failed".to_string());
        
        Ok(summary)
    }
    
    /// Build system prompt.
    fn build_system_prompt(&self, summary: Option<&str>) -> String {
        let summary_text = summary.unwrap_or("Não temos resumo ainda.");
        self.system_prompt_template.replace("<summary>", summary_text)
    }
}

#[async_trait]
impl ContextStrategy for DefaultContextStrategy {
    async fn get_context_window(
        &self,
        thread_id: &str,
        persist_client: Arc<dyn PersistenceClient>,
    ) -> Result<ContextWindow> {
        // 1. Get thread
        let thread = persist_client.get_thread(thread_id).await?
            .ok_or_else(|| anyhow::anyhow!("Thread {} not found - should be created before sending messages", thread_id))?;
        
        // 2. Fetch messages after last_summary_update
        let messages_to_evaluate = persist_client
            .get_messages_after(thread_id, thread.last_summary_update)
            .await?;
        
        let existing_summary = thread.summary.as_ref().map(|s| s.text.as_str());
        if messages_to_evaluate.is_empty() {
            return Ok(ContextWindow {
                system_prompt: self.build_system_prompt(existing_summary),
                messages: vec![],
            });
        }
        
        // 3. Count tokens of CURRENT WINDOW
        let current_window_tokens = self.count_tokens(&messages_to_evaluate)?;
        
        // 4. If current window exceeds max_tokens, spawn async summary generation
        if current_window_tokens > self.max_tokens {
            // Clone everything needed for fire-and-forget task
            let messages_clone = messages_to_evaluate.clone();
            let previous_summary = existing_summary.map(|s| s.to_string());
            let persist_client_clone = Arc::clone(&persist_client);
            let thread_id_owned = thread_id.to_string();
            
            // Clone strategy fields to recreate context in async task
            let strategy = Self {
                max_tokens: self.max_tokens,
                llm_client: self.llm_client.clone(),
                system_prompt_template: self.system_prompt_template.clone(),
                summarization_template: self.summarization_template.clone(),
            };
            
            tokio::spawn(async move {
                if let Ok(summary_text) = strategy
                    .generate_summary(&messages_clone, previous_summary.as_deref())
                    .await {
                        let summary_time = Utc::now();
                        let _ = persist_client_clone.save_thread_summary(
                            &thread_id_owned,
                            summary_text,
                            summary_time
                        ).await;
                }
            });
        }
        
        // 6. Build system prompt with existing summary (if any)
        let system_prompt = self.build_system_prompt(existing_summary);
        
        // 7. Convert DBMessage → praxis_llm::Message
        let llm_messages = messages_to_evaluate
            .into_iter()
            .filter_map(|msg| msg.try_into().ok())
            .collect();
        
        Ok(ContextWindow {
            system_prompt,
            messages: llm_messages,
        })
    }
}

