use std::sync::Arc;
use anyhow::{Result, anyhow};

use praxis_llm::LLMClient;
use praxis_mcp::MCPToolExecutor;
use crate::types::GraphConfig;

use crate::graph::Graph;

/// Configuration for optional persistence
pub struct PersistenceConfig {
    pub client: Arc<dyn praxis_persist::PersistenceClient>,
}

/// Builder for constructing a Graph with optional components
pub struct GraphBuilder {
    llm_client: Option<Arc<dyn LLMClient>>,
    mcp_executor: Option<Arc<MCPToolExecutor>>,
    config: GraphConfig,
    persistence_config: Option<PersistenceConfig>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            llm_client: None,
            mcp_executor: None,
            config: GraphConfig::default(),
            persistence_config: None,
        }
    }
    
    /// Set the LLM client
    pub fn llm_client(mut self, client: Arc<dyn LLMClient>) -> Self {
        self.llm_client = Some(client);
        self
    }
    
    /// Set the MCP tool executor
    pub fn mcp_executor(mut self, executor: Arc<MCPToolExecutor>) -> Self {
        self.mcp_executor = Some(executor);
        self
    }
    
    /// Set the graph configuration
    pub fn config(mut self, config: GraphConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Enable persistence with a PersistenceClient
    pub fn with_persistence(mut self, client: Arc<dyn praxis_persist::PersistenceClient>) -> Self {
        self.persistence_config = Some(PersistenceConfig { client });
        self
    }
    
    /// Build the Graph
    pub fn build(self) -> Result<Graph> {
        let llm_client = self.llm_client
            .ok_or_else(|| anyhow!("LLM client is required"))?;
        let mcp_executor = self.mcp_executor
            .ok_or_else(|| anyhow!("MCP executor is required"))?;
        
        Ok(Graph::new_with_persistence(
            llm_client,
            mcp_executor,
            self.config,
            self.persistence_config,
        ))
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

