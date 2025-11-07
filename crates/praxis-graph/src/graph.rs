use crate::node::{Node, NodeType};
use crate::nodes::{LLMNode, ToolNode};
use crate::router::{NextNode, Router, SimpleRouter};
use anyhow::Result;
use praxis_llm::LLMClient;
use praxis_mcp::MCPToolExecutor;
use praxis_types::{GraphConfig, GraphInput, GraphState, StreamEvent};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

pub struct Graph {
    llm_client: Arc<dyn LLMClient>,
    mcp_executor: Arc<MCPToolExecutor>,
    config: GraphConfig,
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
        }
    }

    /// Spawn execution in background, return event receiver
    pub fn spawn_run(&self, input: GraphInput) -> mpsc::Receiver<StreamEvent> {
        let (tx, rx) = mpsc::channel(1000);

        // Clone what we need for the spawned task
        let llm_client = Arc::clone(&self.llm_client);
        let mcp_executor = Arc::clone(&self.mcp_executor);
        let config = self.config.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::execute_loop(input, tx.clone(), llm_client, mcp_executor, config).await {
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
    ) -> Result<()> {
        let start_time = Instant::now();

        // Build initial state
        let mut state = GraphState::from_input(input);

        // Emit init event
        event_tx
            .send(StreamEvent::InitStream {
                run_id: state.run_id.clone(),
                conversation_id: state.conversation_id.clone(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            })
            .await?;

        // Create nodes
        let llm_node = LLMNode::new(llm_client, mcp_executor.clone());
        let tool_node = ToolNode::new(mcp_executor);
        let router = SimpleRouter;

        let mut current_node = NodeType::LLM;
        let mut iteration = 0;

        loop {
            // Guardrail: max iterations
            if iteration >= config.max_iterations {
                event_tx
                    .send(StreamEvent::Error {
                        message: format!("Max iterations ({}) reached", config.max_iterations),
                        node_id: None,
                    })
                    .await?;
                break;
            }

            // Execute current node
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
        event_tx
            .send(StreamEvent::EndStream {
                status: "success".to_string(),
                total_duration_ms: total_duration,
            })
            .await?;

        // TODO: Persistence layer
        // After execution, save state.messages to MongoDB
        // Use AssistantMessage from praxis-llm::history

        Ok(())
    }
}

