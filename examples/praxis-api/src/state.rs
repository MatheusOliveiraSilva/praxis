use std::sync::Arc;
use praxis_llm::LLMClient;
use praxis_mcp::MCPToolExecutor;
use praxis_persist::PersistClient;
use praxis_graph::Graph;
use crate::config::Config;

/// Shared application state passed to all handlers
/// 
/// All resources are wrapped in Arc for efficient sharing across async tasks.
/// The Graph is stateless and created once at startup for optimal performance.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub persist: Arc<PersistClient>,
    pub llm_client: Arc<dyn LLMClient>,
    pub mcp_executor: Arc<MCPToolExecutor>,
    pub graph: Arc<Graph>,
}

impl AppState {
    pub fn new(
        config: Config,
        persist: PersistClient,
        llm_client: Arc<dyn LLMClient>,
        mcp_executor: Arc<MCPToolExecutor>,
        graph: Graph,
    ) -> Self {
        Self {
            config: Arc::new(config),
            persist: Arc::new(persist),
            llm_client,
            mcp_executor,
            graph: Arc::new(graph),
        }
    }
}

