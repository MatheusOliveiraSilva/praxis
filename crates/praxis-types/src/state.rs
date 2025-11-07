use crate::config::{LLMConfig, ContextPolicy};
use praxis_llm::{Message, ToolCall};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphState {
    pub conversation_id: String,
    pub run_id: String,
    pub messages: Vec<Message>,
    pub llm_config: LLMConfig,
    pub variables: HashMap<String, serde_json::Value>,
}

impl GraphState {
    pub fn new(
        conversation_id: String,
        run_id: String,
        messages: Vec<Message>,
        llm_config: LLMConfig,
    ) -> Self {
        Self {
            conversation_id,
            run_id,
            messages,
            llm_config,
            variables: HashMap::new(),
        }
    }

    pub fn from_input(input: GraphInput) -> Self {
        let mut messages = Vec::new();
        
        // TODO: In a real implementation, we'd fetch history from DB here
        // For now, just use the last message
        messages.push(input.last_message);

        Self {
            conversation_id: input.conversation_id,
            run_id: uuid::Uuid::new_v4().to_string(),
            messages,
            llm_config: input.llm_config,
            variables: HashMap::new(),
        }
    }

    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn has_pending_tool_calls(&self) -> bool {
        if let Some(last_msg) = self.last_message() {
            match last_msg {
                Message::AI { tool_calls, .. } => tool_calls.is_some(),
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn get_pending_tool_calls(&self) -> Vec<ToolCall> {
        if let Some(last_msg) = self.last_message() {
            match last_msg {
                Message::AI { tool_calls: Some(calls), .. } => calls.clone(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        }
    }

    pub fn add_tool_result(&mut self, tool_call_id: String, result: String) {
        self.messages.push(Message::Tool {
            tool_call_id,
            content: praxis_llm::Content::text(result),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphInput {
    pub conversation_id: String,
    pub last_message: Message,
    pub llm_config: LLMConfig,
    pub context_policy: ContextPolicy,
}

impl GraphInput {
    pub fn new(
        conversation_id: impl Into<String>,
        last_message: Message,
        llm_config: LLMConfig,
    ) -> Self {
        Self {
            conversation_id: conversation_id.into(),
            last_message,
            llm_config,
            context_policy: ContextPolicy::default(),
        }
    }

    pub fn with_context_policy(mut self, policy: ContextPolicy) -> Self {
        self.context_policy = policy;
        self
    }
}

