use async_trait::async_trait;
use anyhow::Result;
use crate::types::NodeObservation;

/// Core trait for observability backends
/// 
/// Implementations provide tracing and monitoring for AI agent executions.
/// All methods are async and use fire-and-forget pattern to avoid blocking.
#[async_trait]
pub trait Observer: Send + Sync {
    /// Initialize a new trace for a graph execution run
    /// 
    /// # Arguments
    /// * `run_id` - Unique identifier for this execution run
    /// * `conversation_id` - Thread/conversation identifier
    /// 
    /// # Returns
    /// Result indicating whether trace initialization succeeded
    async fn trace_start(
        &self,
        run_id: String,
        conversation_id: String,
    ) -> Result<()>;

    /// Record an LLM node execution
    /// 
    /// # Arguments
    /// * `observation` - Captured input/output data from LLM execution
    /// 
    /// # Returns
    /// Result indicating whether tracing succeeded
    async fn trace_llm_node(
        &self,
        observation: NodeObservation,
    ) -> Result<()>;

    /// Record a tool node execution
    /// 
    /// # Arguments
    /// * `observation` - Captured input/output data from tool execution
    /// 
    /// # Returns
    /// Result indicating whether tracing succeeded
    async fn trace_tool_node(
        &self,
        observation: NodeObservation,
    ) -> Result<()>;

    /// Finalize the trace after graph execution completes
    /// 
    /// # Arguments
    /// * `run_id` - The run identifier from trace_start
    /// * `status` - Execution status (e.g., "success", "error")
    /// * `total_duration_ms` - Total execution time in milliseconds
    /// 
    /// # Returns
    /// Result indicating whether trace finalization succeeded
    async fn trace_end(
        &self,
        run_id: String,
        status: String,
        total_duration_ms: u64,
    ) -> Result<()>;
}

