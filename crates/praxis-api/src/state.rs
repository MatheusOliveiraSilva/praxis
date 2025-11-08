use std::sync::Arc;
use praxis_llm::LLMClient;
use praxis_mcp::MCPToolExecutor;
use praxis_persist::PersistClient;
use crate::config::Config;

/// Shared application state passed to all handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub persist: Arc<PersistClient>,
    pub llm_client: Arc<dyn LLMClient>,
    pub mcp_executor: Arc<MCPToolExecutor>,
}

impl AppState {
    pub fn new(
        config: Config,
        persist: PersistClient,
        llm_client: Arc<dyn LLMClient>,
        mcp_executor: MCPToolExecutor,
    ) -> Self {
        Self {
            config: Arc::new(config),
            persist: Arc::new(persist),
            llm_client,
            mcp_executor: Arc::new(mcp_executor),
        }
    }
}

