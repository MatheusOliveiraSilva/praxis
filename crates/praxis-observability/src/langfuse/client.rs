use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use std::time::Duration;

use super::types::{
    GenerationBody, IngestionBatch, SpanBody, TraceBody,
};

/// HTTP client for Langfuse API
/// 
/// Handles authentication, request formatting, and communication with Langfuse.
/// All methods use async fire-and-forget pattern for non-blocking operation.
pub struct LangfuseClient {
    client: Client,
    host: String,
    public_key: String,
    secret_key: String,
}

impl LangfuseClient {
    /// Create a new Langfuse client
    /// 
    /// # Arguments
    /// * `public_key` - Langfuse public API key
    /// * `secret_key` - Langfuse secret API key
    /// * `host` - Langfuse host URL (e.g., "https://cloud.langfuse.com")
    pub fn new(public_key: String, secret_key: String, host: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            host: host.trim_end_matches('/').to_string(),
            public_key,
            secret_key,
        })
    }

    /// Create a new trace
    pub async fn create_trace(&self, body: TraceBody) -> Result<()> {
        let url = format!("{}/api/public/traces", self.host);
        
        let response = self
            .client
            .post(&url)
            .basic_auth(&self.public_key, Some(&self.secret_key))
            .json(&body)
            .send()
            .await
            .context("Failed to send create trace request")?;

        self.handle_response(response).await
    }

    /// Create a new span
    pub async fn create_span(&self, body: SpanBody) -> Result<()> {
        let url = format!("{}/api/public/spans", self.host);
        
        let response = self
            .client
            .post(&url)
            .basic_auth(&self.public_key, Some(&self.secret_key))
            .json(&body)
            .send()
            .await
            .context("Failed to send create span request")?;

        self.handle_response(response).await
    }

    /// Create a new generation (LLM call)
    pub async fn create_generation(&self, body: GenerationBody) -> Result<()> {
        let url = format!("{}/api/public/generations", self.host);
        
        let response = self
            .client
            .post(&url)
            .basic_auth(&self.public_key, Some(&self.secret_key))
            .json(&body)
            .send()
            .await
            .context("Failed to send create generation request")?;

        self.handle_response(response).await
    }

    /// Update a trace with final status
    pub async fn update_trace(&self, body: TraceBody) -> Result<()> {
        let url = format!("{}/api/public/traces", self.host);
        
        let response = self
            .client
            .post(&url)
            .basic_auth(&self.public_key, Some(&self.secret_key))
            .json(&body)
            .send()
            .await
            .context("Failed to send update trace request")?;

        self.handle_response(response).await
    }

    /// Send batch ingestion request
    /// 
    /// More efficient for multiple events at once
    pub async fn ingest_batch(&self, batch: IngestionBatch) -> Result<()> {
        let url = format!("{}/api/public/ingestion", self.host);
        
        let response = self
            .client
            .post(&url)
            .basic_auth(&self.public_key, Some(&self.secret_key))
            .json(&batch)
            .send()
            .await
            .context("Failed to send batch ingestion request")?;

        self.handle_response(response).await
    }

    /// Handle API response
    async fn handle_response(&self, response: reqwest::Response) -> Result<()> {
        let status = response.status();
        
        if status.is_success() || status == StatusCode::ACCEPTED {
            tracing::debug!("Langfuse API request successful: {}", status);
            Ok(())
        } else {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            
            tracing::error!(
                "Langfuse API request failed: status={}, body={}",
                status,
                body
            );
            
            anyhow::bail!("Langfuse API error: {} - {}", status, body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LangfuseClient::new(
            "pk-test".to_string(),
            "sk-test".to_string(),
            "https://cloud.langfuse.com".to_string(),
        );
        
        assert!(client.is_ok());
    }
}

