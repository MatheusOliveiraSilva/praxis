use std::sync::Arc;
use praxis::{LLMClient, MCPToolExecutor, PersistenceClient, ContextStrategy, Graph};
use crate::config::Config;

/// Shared application state passed to all handlers
/// 
/// All resources are wrapped in Arc for efficient sharing across async tasks.
/// The Graph is stateless and created once at startup for optimal performance.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub persist: Arc<dyn PersistenceClient>,
    pub context_strategy: Arc<dyn ContextStrategy>,
    pub llm_client: Arc<dyn LLMClient>,
    pub mcp_executor: Arc<MCPToolExecutor>,
    pub graph: Arc<Graph>,
}

impl AppState {
    pub fn new(
        config: Config,
        persist: Arc<dyn PersistenceClient>,
        context_strategy: Arc<dyn ContextStrategy>,
        llm_client: Arc<dyn LLMClient>,
        mcp_executor: Arc<MCPToolExecutor>,
        graph: Graph,
    ) -> Self {
        Self {
            config: Arc::new(config),
            persist,
            context_strategy,
            llm_client,
            mcp_executor,
            graph: Arc::new(graph),
        }
    }
}

