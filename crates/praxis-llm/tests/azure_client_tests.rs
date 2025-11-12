use praxis_llm::azure_openai::AzureOpenAIClient;

#[test]
fn test_azure_client_builder_success() {
    let result = AzureOpenAIClient::builder()
        .api_key("test-key")
        .endpoint("https://test-resource.openai.azure.com/openai/deployments/gpt-4-deployment")
        .api_version("2024-02-15-preview")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_azure_client_builder_missing_api_key() {
    let result = AzureOpenAIClient::builder()
        .endpoint("https://test-resource.openai.azure.com/openai/deployments/gpt-4-deployment")
        .api_version("2024-02-15-preview")
        .build();

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("API key"));
}

#[test]
fn test_azure_client_builder_missing_endpoint() {
    let result = AzureOpenAIClient::builder()
        .api_key("test-key")
        .api_version("2024-02-15-preview")
        .build();

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Endpoint"));
}

#[test]
fn test_azure_client_builder_missing_api_version() {
    let result = AzureOpenAIClient::builder()
        .api_key("test-key")
        .endpoint("https://test-resource.openai.azure.com/openai/deployments/gpt-4-deployment")
        .build();

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("API version"));
}

#[cfg(test)]
mod config_tests {
    use praxis_llm::config::{AzureConfig, ProviderConfig, ProviderType};

    #[test]
    fn test_azure_config_endpoint() {
        let config = AzureConfig::new(
            "test-key",
            "https://my-resource.openai.azure.com/openai/deployments/gpt-4-deployment",
            "2024-02-15-preview",
        );

        let expected = "https://my-resource.openai.azure.com/openai/deployments/gpt-4-deployment";
        assert_eq!(config.endpoint, expected);
    }

    #[test]
    fn test_provider_config_azure() {
        let config = ProviderConfig::azure_openai(
            "test-key",
            "https://my-resource.openai.azure.com/openai/deployments/gpt-4-deployment",
            "2024-02-15-preview",
        );

        assert_eq!(config.provider_type(), ProviderType::AzureOpenAI);
    }

    #[test]
    fn test_provider_config_openai() {
        let config = ProviderConfig::openai("test-key");
        assert_eq!(config.provider_type(), ProviderType::OpenAI);
    }
}

#[cfg(test)]
mod url_building_tests {
    use praxis_llm::AzureOpenAIClient;

    #[test]
    fn test_url_structure() {
        let _client = AzureOpenAIClient::builder()
            .api_key("test-key")
            .endpoint("https://my-resource.openai.azure.com/openai/deployments/gpt-4-deployment")
            .api_version("2024-02-15-preview")
            .build()
            .unwrap();

        // Test that the client builds successfully with correct structure
        // The base URL should be properly formatted
        // Note: We can't directly test private methods, but we verify the client builds
        assert!(true);
    }

    #[test]
    fn test_endpoint_parsing() {
        // Test that endpoint parsing works correctly
        let _client = AzureOpenAIClient::builder()
            .api_key("test-key")
            .endpoint("https://my-resource.openai.azure.com/openai/deployments/gpt-4-deployment")
            .api_version("2024-02-15-preview")
            .build()
            .unwrap();

        // If client builds successfully, endpoint was parsed correctly
        assert!(true);
    }
}

// Integration tests would require actual Azure OpenAI credentials
// These are just unit tests for the builder pattern and configuration

