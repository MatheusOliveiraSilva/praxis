// Configuration layer for provider-agnostic LLM client creation
// This module provides a factory pattern for creating LLM clients from configuration

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Type of LLM provider
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    OpenAI,
    #[serde(rename = "azure_openai")]
    AzureOpenAI,
}

impl Default for ProviderType {
    fn default() -> Self {
        ProviderType::OpenAI
    }
}

/// Configuration for OpenAI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    /// Base URL for OpenAI API (optional, defaults to https://api.openai.com/v1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

impl OpenAIConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: None,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }
}

/// Configuration for Azure OpenAI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    pub api_key: String,
    pub endpoint: String,
    pub deployment_name: String,
    pub api_version: String,
}

impl AzureConfig {
    pub fn new(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        deployment_name: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            endpoint: endpoint.into(),
            deployment_name: deployment_name.into(),
            api_version: api_version.into(),
        }
    }
}

/// Provider-specific configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProviderDetails {
    OpenAI(OpenAIConfig),
    #[serde(rename = "azure_openai")]
    AzureOpenAI(AzureConfig),
}

/// Complete provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(flatten)]
    pub details: ProviderDetails,
}

impl ProviderConfig {
    /// Create OpenAI provider config
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self {
            details: ProviderDetails::OpenAI(OpenAIConfig::new(api_key)),
        }
    }

    /// Create Azure OpenAI provider config
    pub fn azure_openai(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        deployment_name: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Self {
        Self {
            details: ProviderDetails::AzureOpenAI(AzureConfig::new(
                api_key,
                endpoint,
                deployment_name,
                api_version,
            )),
        }
    }

    /// Get the provider type
    pub fn provider_type(&self) -> ProviderType {
        match self.details {
            ProviderDetails::OpenAI(_) => ProviderType::OpenAI,
            ProviderDetails::AzureOpenAI(_) => ProviderType::AzureOpenAI,
        }
    }
}

/// Factory for creating LLM clients from configuration
pub struct ClientFactory;

impl ClientFactory {
    /// Create an LLM client from provider configuration
    pub fn create_client(config: ProviderConfig) -> Result<Arc<dyn crate::traits::LLMClient>> {
        match config.details {
            ProviderDetails::OpenAI(openai_config) => {
                let client = crate::openai::OpenAIClient::new(openai_config.api_key)?;
                Ok(Arc::new(client))
            }
            ProviderDetails::AzureOpenAI(azure_config) => {
                let client = crate::azure_openai::AzureOpenAIClient::builder()
                    .api_key(azure_config.api_key)
                    .endpoint(azure_config.endpoint)
                    .api_version(azure_config.api_version)
                    .build()?;
                Ok(Arc::new(client))
            }
        }
    }

    /// Create a chat client from provider configuration
    pub fn create_chat_client(
        config: ProviderConfig,
    ) -> Result<Arc<dyn crate::traits::ChatClient>> {
        match config.details {
            ProviderDetails::OpenAI(openai_config) => {
                let client = crate::openai::OpenAIClient::new(openai_config.api_key)?;
                Ok(Arc::new(client))
            }
            ProviderDetails::AzureOpenAI(azure_config) => {
                let client = crate::azure_openai::AzureOpenAIClient::builder()
                    .api_key(azure_config.api_key)
                    .endpoint(azure_config.endpoint)
                    .deployment_name(azure_config.deployment_name)
                    .api_version(azure_config.api_version)
                    .build()?;
                Ok(Arc::new(client))
            }
        }
    }

    /// Create a reasoning client from provider configuration
    pub fn create_reasoning_client(
        config: ProviderConfig,
    ) -> Result<Arc<dyn crate::traits::ReasoningClient>> {
        match config.details {
            ProviderDetails::OpenAI(openai_config) => {
                let client = crate::openai::OpenAIClient::new(openai_config.api_key)?;
                Ok(Arc::new(client))
            }
            ProviderDetails::AzureOpenAI(azure_config) => {
                let client = crate::azure_openai::AzureOpenAIClient::builder()
                    .api_key(azure_config.api_key)
                    .endpoint(azure_config.endpoint)
                    .deployment_name(azure_config.deployment_name)
                    .api_version(azure_config.api_version)
                    .build()?;
                Ok(Arc::new(client))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_config() {
        let config = ProviderConfig::openai("test-key");
        assert_eq!(config.provider_type(), ProviderType::OpenAI);
    }

    #[test]
    fn test_azure_config() {
        let config = ProviderConfig::azure_openai(
            "test-key",
            "https://my-resource.openai.azure.com",
            "gpt-4-deployment",
            "2024-02-15-preview",
        );

        assert_eq!(config.provider_type(), ProviderType::AzureOpenAI);
    }

    #[test]
    fn test_azure_endpoint() {
        let azure_config = AzureConfig::new(
            "test-key",
            "https://my-resource.openai.azure.com",
            "gpt-4-deployment",
            "2024-02-15-preview",
        );
        assert_eq!(azure_config.endpoint, "https://my-resource.openai.azure.com");
        assert_eq!(azure_config.deployment_name, "gpt-4-deployment");
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = ProviderConfig::azure_openai(
            "test-key",
            "https://my-resource.openai.azure.com",
            "gpt-4-deployment",
            "2024-02-15-preview",
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.provider_type(), deserialized.provider_type());
    }
}
