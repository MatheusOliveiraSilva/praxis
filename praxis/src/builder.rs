//! High-level builder API for creating AI agents

use crate::{Graph, GraphConfig, LLMConfig, OpenAIClient, MCPToolExecutor, PersistClient};
use anyhow::{Context, Result};
use std::sync::Arc;

/// High-level builder for creating AI agents
///
/// # Example
///
/// ```rust,no_run
/// use praxis::prelude::*;
///
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// let agent = AgentBuilder::new()
///     .mongodb("mongodb://localhost:27017", "praxis")
///     .openai_key("sk-...")
///     .mcp_servers("http://localhost:8000/mcp")
///     .max_tokens(30_000)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct AgentBuilder {
    // MongoDB
    mongodb_uri: Option<String>,
    database: Option<String>,
    
    // LLM
    openai_key: Option<String>,
    model: String,
    temperature: f32,
    
    // MCP
    mcp_servers: Option<String>,
    
    // Context
    max_tokens: usize,
    
    // Graph config
    graph_config: GraphConfig,
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentBuilder {
    /// Create a new agent builder with sensible defaults
    pub fn new() -> Self {
        Self {
            mongodb_uri: None,
            database: None,
            openai_key: None,
            model: "gpt-4o".to_string(),
            temperature: 0.7,
            mcp_servers: None,
            max_tokens: 30_000,
            graph_config: GraphConfig::default(),
        }
    }
    
    /// Set MongoDB connection (required)
    ///
    /// # Example
    /// ```rust,no_run
    /// # use praxis::prelude::*;
    /// let builder = AgentBuilder::new()
    ///     .mongodb("mongodb://localhost:27017", "praxis");
    /// ```
    pub fn mongodb(mut self, uri: impl Into<String>, database: impl Into<String>) -> Self {
        self.mongodb_uri = Some(uri.into());
        self.database = Some(database.into());
        self
    }
    
    /// Set OpenAI API key (required)
    ///
    /// # Example
    /// ```rust,no_run
    /// # use praxis::prelude::*;
    /// let builder = AgentBuilder::new()
    ///     .openai_key("sk-...");
    /// ```
    pub fn openai_key(mut self, key: impl Into<String>) -> Self {
        self.openai_key = Some(key.into());
        self
    }
    
    /// Set LLM model (default: gpt-4o)
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
    
    /// Set temperature (default: 0.7)
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
    
    /// Set MCP servers (comma-separated URLs)
    ///
    /// # Example
    /// ```rust,no_run
    /// # use praxis::prelude::*;
    /// let builder = AgentBuilder::new()
    ///     .mcp_servers("http://localhost:8000/mcp,http://localhost:8001/mcp");
    /// ```
    pub fn mcp_servers(mut self, servers: impl Into<String>) -> Self {
        self.mcp_servers = Some(servers.into());
        self
    }
    
    /// Set max tokens before summarization (default: 30,000)
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }
    
    /// Set graph configuration
    pub fn graph_config(mut self, config: GraphConfig) -> Self {
        self.graph_config = config;
        self
    }
    
    /// Build the agent
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - MongoDB URI or database is not set
    /// - OpenAI API key is not set
    /// - MongoDB connection fails
    /// - MCP server connection fails
    pub async fn build(self) -> Result<Agent> {
        // Validate required fields
        let mongodb_uri = self.mongodb_uri
            .context("MongoDB URI is required. Call .mongodb(uri, database)")?;
        let database = self.database
            .context("Database name is required")?;
        let openai_key = self.openai_key
            .context("OpenAI API key is required. Call .openai_key(key)")?;
        
        // Create LLM client
        let llm_client = Arc::new(OpenAIClient::new(openai_key, self.model));
        
        // Create persist client
        let persist_client = PersistClient::builder()
            .mongodb_uri(&mongodb_uri)
            .database(&database)
            .max_tokens(self.max_tokens)
            .llm_client(Arc::clone(&llm_client))
            .build()
            .await
            .context("Failed to create persist client")?;
        
        // Create MCP executor
        let mcp_executor = if let Some(servers) = self.mcp_servers {
            let executor = MCPToolExecutor::new();
            
            // Parse comma-separated servers
            for server_url in servers.split(',').map(str::trim) {
                if !server_url.is_empty() {
                    let client = crate::mcp::MCPClient::connect_http(server_url)
                        .await
                        .with_context(|| format!("Failed to connect to MCP server: {}", server_url))?;
                    executor.add_server(client).await
                        .context("Failed to add MCP server")?;
                }
            }
            
            Arc::new(executor)
        } else {
            Arc::new(MCPToolExecutor::new())
        };
        
        // Create graph
        let graph = Graph::new(
            llm_client,
            Arc::clone(&mcp_executor),
            self.graph_config,
        );
        
        Ok(Agent {
            graph: Arc::new(graph),
            persist: Arc::new(persist_client),
            mcp_executor,
        })
    }
}

/// A configured AI agent ready to process conversations
pub struct Agent {
    graph: Arc<Graph>,
    persist: Arc<PersistClient>,
    mcp_executor: Arc<MCPToolExecutor>,
}

impl Agent {
    /// Simple chat interface (creates new thread)
    ///
    /// # Example
    /// ```rust,no_run
    /// # use praxis::prelude::*;
    /// # async fn example(agent: Agent) -> Result<()> {
    /// let response = agent.chat("What is 2+2?").await?;
    /// println!("{}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chat(&self, message: impl AsRef<str>) -> Result<String> {
        // Create a new thread
        let thread = self.persist.threads()
            .create_thread(
                "default_user".to_string(),
                Default::default(),
            )
            .await?;
        
        // Get context and run
        self.chat_in_thread(thread.id, message).await
    }
    
    /// Chat in an existing thread
    pub async fn chat_in_thread(
        &self,
        thread_id: mongodb::bson::oid::ObjectId,
        message: impl AsRef<str>,
    ) -> Result<String> {
        // Implementation would integrate with Graph execution
        // This is a simplified version
        todo!("Implement chat_in_thread with full Graph execution")
    }
    
    /// Stream chat responses (returns async stream of events)
    pub async fn chat_stream(
        &self,
        message: impl AsRef<str>,
    ) -> Result<impl futures::Stream<Item = Result<crate::StreamEvent>>> {
        // Implementation would return the Graph's event receiver as a stream
        todo!("Implement chat_stream")
    }
    
    /// Get the underlying Graph for advanced usage
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
    
    /// Get the persist client
    pub fn persist(&self) -> &PersistClient {
        &self.persist
    }
    
    /// Get the MCP executor
    pub fn mcp_executor(&self) -> &MCPToolExecutor {
        &self.mcp_executor
    }
}
