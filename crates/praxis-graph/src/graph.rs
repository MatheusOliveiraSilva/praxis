use crate::node::{Node, NodeType};
use crate::nodes::{LLMNode, ToolNode};
use crate::router::{NextNode, Router, SimpleRouter};
use crate::builder::PersistenceConfig;
use anyhow::Result;
use praxis_llm::LLMClient;
use praxis_mcp::MCPToolExecutor;
use crate::types::{GraphConfig, GraphInput, GraphState, StreamEvent};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

/// Context for persistence operations
pub struct PersistenceContext {
    pub thread_id: String,
    pub user_id: String,
}

pub struct Graph {
    llm_client: Arc<dyn LLMClient>,
    mcp_executor: Arc<MCPToolExecutor>,
    config: GraphConfig,
    persistence: Option<Arc<PersistenceConfig>>,
}

impl Graph {
    pub fn new(
        llm_client: Arc<dyn LLMClient>,
        mcp_executor: Arc<MCPToolExecutor>,
        config: GraphConfig,
    ) -> Self {
        Self {
            llm_client,
            mcp_executor,
            config,
            persistence: None,
        }
    }
    
    pub(crate) fn new_with_persistence(
        llm_client: Arc<dyn LLMClient>,
        mcp_executor: Arc<MCPToolExecutor>,
        config: GraphConfig,
        persistence: Option<PersistenceConfig>,
    ) -> Self {
        Self {
            llm_client,
            mcp_executor,
            config,
            persistence: persistence.map(Arc::new),
        }
    }
    
    /// Create a builder for fluent construction
    pub fn builder() -> crate::builder::GraphBuilder {
        crate::builder::GraphBuilder::new()
    }

    /// Spawn execution in background, return event receiver
    pub fn spawn_run(
        &self,
        input: GraphInput,
        persistence_ctx: Option<PersistenceContext>,
    ) -> mpsc::Receiver<StreamEvent> {
        let (tx, rx) = mpsc::channel(1000);

        // Clone what we need for the spawned task
        let llm_client = Arc::clone(&self.llm_client);
        let mcp_executor = Arc::clone(&self.mcp_executor);
        let config = self.config.clone();
        let persistence = self.persistence.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::execute_loop(
                input,
                tx.clone(),
                llm_client,
                mcp_executor,
                config,
                persistence,
                persistence_ctx,
            ).await {
                let _ = tx
                    .send(StreamEvent::Error {
                        message: e.to_string(),
                        node_id: None,
                    })
                    .await;
            }
        });

        rx
    }

    async fn execute_loop(
        input: GraphInput,
        event_tx: mpsc::Sender<StreamEvent>,
        llm_client: Arc<dyn LLMClient>,
        mcp_executor: Arc<MCPToolExecutor>,
        config: GraphConfig,
        persistence: Option<Arc<PersistenceConfig>>,
        ctx: Option<PersistenceContext>,
    ) -> Result<()> {
        let start_time = Instant::now();

        // Build initial state
        let mut state = GraphState::from_input(input);

        // Create Observer if persistence enabled
        let mut accumulator: Option<praxis_persist::EventAccumulator<StreamEvent>> = match (&persistence, &ctx) {
            (Some(_), Some(c)) => Some(praxis_persist::EventAccumulator::new(
                c.thread_id.clone(),
                c.user_id.clone(),
            )),
            _ => None,
        };

        // Emit init event
        let init_event = StreamEvent::InitStream {
            run_id: state.run_id.clone(),
            conversation_id: state.conversation_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        event_tx.send(init_event.clone()).await?;

        // Create nodes
        let llm_node = LLMNode::new(llm_client, mcp_executor.clone());
        let tool_node = ToolNode::new(mcp_executor);
        let router = SimpleRouter;

        let mut current_node = NodeType::LLM;
        let mut iteration = 0;

        loop {
            // Guardrail: max iterations
            if iteration >= config.max_iterations {
                let error_event = StreamEvent::Error {
                    message: format!("Max iterations ({}) reached", config.max_iterations),
                    node_id: None,
                };
                event_tx.send(error_event.clone()).await?;
                break;
            }

            // Execute current node (this emits events via event_tx)
            match current_node {
                NodeType::LLM => {
                    llm_node.execute(&mut state, event_tx.clone()).await?;
                }
                NodeType::Tool => {
                    tool_node.execute(&mut state, event_tx.clone()).await?;
                }
            }

            // Route to next node
            let next = router.next(&state, current_node);

            match next {
                NextNode::End => break,
                NextNode::LLM => current_node = NodeType::LLM,
                NextNode::Tool => current_node = NodeType::Tool,
            }

            iteration += 1;
        }

        // Emit end event
        let total_duration = start_time.elapsed().as_millis() as u64;
        let end_event = StreamEvent::EndStream {
            status: "success".to_string(),
            total_duration_ms: total_duration,
        };
        event_tx.send(end_event.clone()).await?;
        
        // Observer pattern: check for final transition
        if let Some(ref mut acc) = accumulator {
            if let Some(completed_msg) = acc.push_and_check_transition(&end_event) {
                if let Some(ref p) = persistence {
                    p.client.save_message(completed_msg).await?;
                }
            }
        }

        // Finalize any remaining buffer
        if let Some(mut acc) = accumulator {
            if let Some(final_msg) = acc.finalize() {
                if let Some(ref p) = persistence {
                    p.client.save_message(final_msg).await?;
                }
            }
        }

        Ok(())
    }
}

