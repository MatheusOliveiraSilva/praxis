use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Azure,
    Anthropic,
}

impl Default for Provider {
    fn default() -> Self {
        Provider::OpenAI
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub max_iterations: usize,
    pub execution_timeout: Duration,
    pub enable_cancellation: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            execution_timeout: Duration::from_secs(300),
            enable_cancellation: true,
        }
    }
}

impl GraphConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.execution_timeout = timeout;
        self
    }

    pub fn with_cancellation(mut self, enabled: bool) -> Self {
        self.enable_cancellation = enabled;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub model: String,
    #[serde(default)]
    pub provider: Provider,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

impl LLMConfig {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            provider: Provider::default(),
            temperature: None,
            max_tokens: None,
            reasoning_effort: None,
        }
    }

    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.provider = provider;
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn with_reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(effort.into());
        self
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            provider: Provider::default(),
            temperature: Some(1.0),
            max_tokens: Some(4096),
            reasoning_effort: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextPolicy {
    LastK { k: usize },
    AllMessages,
}

impl Default for ContextPolicy {
    fn default() -> Self {
        Self::LastK { k: 10 }
    }
}

