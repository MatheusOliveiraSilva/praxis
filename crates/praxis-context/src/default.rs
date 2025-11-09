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
        // 1. Get thread and determine which messages to use
        let thread = persist_client.get_thread(thread_id).await?;
        
        let (existing_summary, messages_to_evaluate) = 
            if let Some(summary) = thread.as_ref().and_then(|t| t.summary.as_ref()) {
                // Has summary - get only messages after last summary
                let recent_msgs = persist_client.get_messages_after(thread_id, summary.generated_at).await?;
                (Some(summary.text.as_str()), recent_msgs)
            } else {
                // No summary (or no thread) - get all messages
                let all_msgs = persist_client.get_messages(thread_id).await?;
                (None, all_msgs)
            };
        
        if messages_to_evaluate.is_empty() {
            return Ok(ContextWindow {
                system_prompt: self.build_system_prompt(existing_summary),
                messages: vec![],
            });
        }
        
        // 2. Count tokens of CURRENT WINDOW (not all messages)
        let current_window_tokens = self.count_tokens(&messages_to_evaluate)?;
        
        // 3. If current window exceeds max_tokens, spawn async summary generation
        if current_window_tokens > self.max_tokens {
            // Clone everything needed for fire-and-forget task
            let messages_clone = messages_to_evaluate.clone();
            let previous_summary = existing_summary.map(|s| s.to_string());
            let llm_client = self.llm_client.clone();
            let summarization_template = self.summarization_template.clone();
            let persist_client_clone = Arc::clone(&persist_client);
            let thread_id_owned = thread_id.to_string();
            
            // Fire and forget - spawn task to generate and save new summary
            tokio::spawn(async move {
                // Build conversation text
                let conversation = messages_clone.iter()
                    .map(|m| {
                        let role = match m.role {
                            praxis_persist::MessageRole::User => "User",
                            praxis_persist::MessageRole::Assistant => "Assistant",
                        };
                        format!("{}: {}", role, m.content)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                
                // Build summary prompt
                let previous_text = previous_summary.as_deref().unwrap_or("Não temos resumo ainda.");
                let summary_prompt = summarization_template
                    .replace("<previous_summary>", previous_text)
                    .replace("<conversation>", &conversation);
                
                // Generate summary
                let request = praxis_llm::ChatRequest::new(
                    "gpt-4o-mini".to_string(),
                    vec![Message::Human {
                        content: Content::text(summary_prompt),
                        name: None,
                    }],
                );
                
                // Call LLM and save summary
                if let Ok(response) = llm_client.chat_completion(request).await {
                    if let Some(summary_text) = response.content {
                        let summary_time = Utc::now();
                        // Save to database (fire and forget - ignore errors)
                        let _ = persist_client_clone.save_thread_summary(
                            &thread_id_owned,
                            summary_text,
                            summary_time
                        ).await;
                    }
                }
            });
        }
        
        // 4. Build system prompt with existing summary (if any)
        let system_prompt = self.build_system_prompt(existing_summary);
        
        // 5. Convert DBMessage → praxis_llm::Message
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

