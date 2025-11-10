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

/// Configuration for optional observability
#[cfg(feature = "observability")]
pub struct ObserverConfig {
    pub observer: Arc<dyn praxis_observability::Observer>,
}

/// Builder for constructing a Graph with optional components
pub struct GraphBuilder {
    llm_client: Option<Arc<dyn LLMClient>>,
    mcp_executor: Option<Arc<MCPToolExecutor>>,
    config: GraphConfig,
    persistence_config: Option<PersistenceConfig>,
    #[cfg(feature = "observability")]
    observer_config: Option<ObserverConfig>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            llm_client: None,
            mcp_executor: None,
            config: GraphConfig::default(),
            persistence_config: None,
            #[cfg(feature = "observability")]
            observer_config: None,
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
    
    /// Enable observability with an Observer
    #[cfg(feature = "observability")]
    pub fn with_observer(mut self, observer: Arc<dyn praxis_observability::Observer>) -> Self {
        self.observer_config = Some(ObserverConfig { observer });
        self
    }
    
    /// Build the Graph
    pub fn build(self) -> Result<Graph> {
        let llm_client = self.llm_client
            .ok_or_else(|| anyhow!("LLM client is required"))?;
        let mcp_executor = self.mcp_executor
            .ok_or_else(|| anyhow!("MCP executor is required"))?;
        
        Ok(Graph::new_with_config(
            llm_client,
            mcp_executor,
            self.config,
            self.persistence_config,
            #[cfg(feature = "observability")]
            self.observer_config,
        ))
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

