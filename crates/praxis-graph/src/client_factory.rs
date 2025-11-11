use crate::types::{LLMConfig, Provider};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use praxis_llm::{LLMClient, ReasoningClient};

/// Factory Pattern: Centralized logic for client creation and configuration
/// 
/// This factory encapsulates the logic of determining which LLM client to use
/// based on model configuration, abstracting provider-specific details from the graph.
pub struct ClientFactory;

impl ClientFactory {
    /// Check if a model supports reasoning capabilities
    /// 
    /// Reasoning models (gpt-5, o1-*) require special handling and use the Responses API
    pub fn supports_reasoning(model: &str) -> bool {
        model.starts_with("gpt-5") || model.starts_with("o1")
    }
    
    /// Validate that the given LLM configuration is supported
    pub fn validate_config(config: &LLMConfig) -> Result<()> {
        match config.provider {
            Provider::OpenAI => Ok(()),
            Provider::Azure => {
                Err(anyhow!("Azure provider not yet implemented. Use Provider::OpenAI for now."))
            }
            Provider::Anthropic => {
                Err(anyhow!("Anthropic provider not yet implemented. Use Provider::OpenAI for now."))
            }
        }
    }
    
    /// Determine if the given client supports reasoning based on the model
    /// 
    /// This is a runtime check to see if we should attempt to use the Reasoning API
    pub fn should_use_reasoning_api(
        config: &LLMConfig,
        reasoning_client: &Option<Arc<dyn ReasoningClient>>,
    ) -> bool {
        Self::supports_reasoning(&config.model) && reasoning_client.is_some()
    }
    
    /// Future: Create an LLM client from configuration
    /// 
    /// Currently, clients are created at the application level and passed to the graph.
    /// This method is reserved for future use when we might want to create clients
    /// dynamically at runtime.
    #[allow(dead_code)]
    pub fn create_client(_config: &LLMConfig, _api_key: &str) -> Result<Arc<dyn LLMClient>> {
        // Future implementation
        Err(anyhow!("Dynamic client creation not yet implemented. Create clients at application level and pass to GraphBuilder."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LLMConfig;
    
    #[test]
    fn test_supports_reasoning() {
        assert!(ClientFactory::supports_reasoning("gpt-5"));
        assert!(ClientFactory::supports_reasoning("gpt-5-turbo"));
        assert!(ClientFactory::supports_reasoning("o1-preview"));
        assert!(ClientFactory::supports_reasoning("o1-mini"));
        
        assert!(!ClientFactory::supports_reasoning("gpt-4o"));
        assert!(!ClientFactory::supports_reasoning("gpt-4o-mini"));
        assert!(!ClientFactory::supports_reasoning("gpt-3.5-turbo"));
    }
    
    #[test]
    fn test_validate_config() {
        let openai_config = LLMConfig::new("gpt-4o").with_provider(Provider::OpenAI);
        assert!(ClientFactory::validate_config(&openai_config).is_ok());
        
        let azure_config = LLMConfig::new("gpt-4o").with_provider(Provider::Azure);
        assert!(ClientFactory::validate_config(&azure_config).is_err());
        
        let anthropic_config = LLMConfig::new("claude-3").with_provider(Provider::Anthropic);
        assert!(ClientFactory::validate_config(&anthropic_config).is_err());
    }
}

