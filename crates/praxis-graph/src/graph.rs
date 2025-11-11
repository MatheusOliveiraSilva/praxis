use crate::node::{Node, NodeType};
use crate::nodes::{LLMNode, ToolNode};
use crate::router::{NextNode, Router, SimpleRouter};
use crate::builder::PersistenceConfig;
use praxis_llm::ReasoningClient;
#[cfg(feature = "observability")]
use crate::builder::ObserverConfig;
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
    reasoning_client: Option<Arc<dyn praxis_llm::ReasoningClient>>,
    mcp_executor: Arc<MCPToolExecutor>,
    config: GraphConfig,
    persistence: Option<Arc<PersistenceConfig>>,
    #[cfg(feature = "observability")]
    observer: Option<Arc<ObserverConfig>>,
}

impl Graph {
    pub fn new(
        llm_client: Arc<dyn LLMClient>,
        mcp_executor: Arc<MCPToolExecutor>,
        config: GraphConfig,
    ) -> Self {
        Self {
            llm_client,
            reasoning_client: None,
            mcp_executor,
            config,
            persistence: None,
            #[cfg(feature = "observability")]
            observer: None,
        }
    }
    
    pub(crate) fn new_with_config(
        llm_client: Arc<dyn LLMClient>,
        reasoning_client: Option<Arc<dyn praxis_llm::ReasoningClient>>,
        mcp_executor: Arc<MCPToolExecutor>,
        config: GraphConfig,
        persistence: Option<PersistenceConfig>,
        #[cfg(feature = "observability")]
        observer: Option<ObserverConfig>,
    ) -> Self {
        Self {
            llm_client,
            reasoning_client,
            mcp_executor,
            config,
            persistence: persistence.map(Arc::new),
            #[cfg(feature = "observability")]
            observer: observer.map(Arc::new),
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
        let reasoning_client = self.reasoning_client.clone();
        let mcp_executor = Arc::clone(&self.mcp_executor);
        let config = self.config.clone();
        let persistence = self.persistence.clone();
        #[cfg(feature = "observability")]
        let observer = self.observer.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::execute_loop(
                input,
                tx.clone(),
                llm_client,
                reasoning_client,
                mcp_executor,
                config,
                persistence,
                #[cfg(feature = "observability")]
                observer,
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
        reasoning_client: Option<Arc<dyn ReasoningClient>>,
        mcp_executor: Arc<MCPToolExecutor>,
        config: GraphConfig,
        persistence: Option<Arc<PersistenceConfig>>,
        #[cfg(feature = "observability")]
        observer: Option<Arc<ObserverConfig>>,
        ctx: Option<PersistenceContext>,
    ) -> Result<()> {
        let start_time = Instant::now();

        // Build initial state
        let mut state = GraphState::from_input(input);

        // Initialize tracing if observer is configured
        #[cfg(feature = "observability")]
        if let Some(ref obs) = observer {
            let obs_clone = Arc::clone(&obs.observer);
            let run_id = state.run_id.clone();
            let conversation_id = state.conversation_id.clone();
            tokio::spawn(async move {
                if let Err(e) = obs_clone.trace_start(run_id, conversation_id).await {
                    tracing::error!("Failed to start trace: {}", e);
                }
            });
        }

        // Emit init event
        let init_event = StreamEvent::InitStream {
            run_id: state.run_id.clone(),
            conversation_id: state.conversation_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        event_tx.send(init_event.clone()).await?;

        // Create nodes
        let mut llm_node = LLMNode::new(llm_client.clone(), mcp_executor.clone());
        
        if let Some(reasoning_client) = reasoning_client.clone() {
            llm_node = llm_node.with_reasoning_client(reasoning_client);
        }
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

            let node_start = Instant::now();
            
            // Store state snapshot before execution for observation
            let messages_before = state.messages.len();

            // Execute current node (this emits events via event_tx)
            match current_node {
                NodeType::LLM => {
                    llm_node.execute(&mut state, event_tx.clone()).await?;
                }
                NodeType::Tool => {
                    tool_node.execute(&mut state, event_tx.clone()).await?;
                }
            }

            let node_duration = node_start.elapsed().as_millis() as u64;

            // After node execution: persistence + observability (fire-and-forget)
            Self::handle_post_node_execution(
                &state,
                current_node,
                node_start,
                node_duration,
                messages_before,
                &persistence,
                #[cfg(feature = "observability")]
                &observer,
                &ctx,
            ).await;

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
        
        // Finalize tracing
        #[cfg(feature = "observability")]
        if let Some(ref obs) = observer {
            let obs_clone = Arc::clone(&obs.observer);
            let run_id = state.run_id.clone();
            tokio::spawn(async move {
                if let Err(e) = obs_clone.trace_end(run_id, "success".to_string(), total_duration).await {
                    tracing::error!("Failed to end trace: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Handle post-node execution: persistence and observability
    async fn handle_post_node_execution(
        state: &GraphState,
        node_type: NodeType,
        node_start: Instant,
        #[allow(unused_variables)]
        node_duration: u64,
        messages_before: usize,
        persistence: &Option<Arc<PersistenceConfig>>,
        #[cfg(feature = "observability")]
        observer: &Option<Arc<ObserverConfig>>,
        ctx: &Option<PersistenceContext>,
    ) {
        // Extract messages added by this node
        let new_messages = if state.messages.len() > messages_before {
            &state.messages[messages_before..]
        } else {
            &[]
        };

        // Persistence: save messages
        // For LLM nodes, use structured outputs if available; otherwise fallback to messages
        if let (Some(persist), Some(context)) = (persistence, ctx) {
            if node_type == NodeType::LLM && state.last_outputs.is_some() {
                // New approach: Save structured outputs (reasoning + message separately)
                if let Some(outputs) = &state.last_outputs {
                    for output in outputs {
                        let db_message = Self::convert_output_to_db(
                            output,
                            &context.thread_id,
                            &context.user_id,
                        );
                        
                        if let Some(db_msg) = db_message {
                            let client = Arc::clone(&persist.client);
                            tokio::spawn(async move {
                                if let Err(e) = client.save_message(db_msg).await {
                                    tracing::error!("Failed to save output to database: {}", e);
                                }
                            });
                        }
                    }
                }
            } else {
                // Fallback: Save messages directly (for Tool nodes or old LLM nodes)
                for msg in new_messages {
                    let db_message = Self::convert_message_to_db(
                        msg,
                        &context.thread_id,
                        &context.user_id,
                        node_type,
                    );
                    
                    if let Some(db_msg) = db_message {
                        let client = Arc::clone(&persist.client);
                        tokio::spawn(async move {
                            if let Err(e) = client.save_message(db_msg).await {
                                tracing::error!("Failed to save message: {}", e);
                            }
                        });
                    }
                }
            }
        }

        // Observability: send observation
        #[cfg(feature = "observability")]
        if let Some(obs) = observer {
            let observation = Self::create_observation(
                state,
                node_type,
                node_start,
                node_duration,
                new_messages,
            );

            if let Some(obs_data) = observation {
                let obs_clone = Arc::clone(&obs.observer);
                tokio::spawn(async move {
                    let result = match obs_data.node_type.as_str() {
                        "llm" => obs_clone.trace_llm_node(obs_data).await,
                        "tool" => obs_clone.trace_tool_node(obs_data).await,
                        _ => Ok(()),
                    };
                    
                    if let Err(e) = result {
                        tracing::error!("Failed to trace node execution: {}", e);
                    }
                });
            }
        }
    }

    /// Convert GraphOutput to DBMessage
    fn convert_output_to_db(
        output: &crate::types::GraphOutput,
        thread_id: &str,
        user_id: &str,
    ) -> Option<praxis_persist::DBMessage> {
        use crate::types::GraphOutput;
        use praxis_persist::{MessageRole, MessageType};

        match output {
            GraphOutput::Reasoning { id, content } => {
                Some(praxis_persist::DBMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    thread_id: thread_id.to_string(),
                    user_id: user_id.to_string(),
                    role: MessageRole::Assistant,
                    message_type: MessageType::Reasoning,
                    content: content.clone(),
                    tool_call_id: None,
                    tool_name: None,
                    arguments: None,
                    reasoning_id: Some(id.clone()),
                    created_at: chrono::Utc::now(),
                    duration_ms: None,
                })
            }
            GraphOutput::Message { id, content, tool_calls } => {
                if let Some(calls) = tool_calls {
                    // Save first tool call (expand to handle all in production)
                    if let Some(first_call) = calls.first() {
                        Some(praxis_persist::DBMessage {
                            id: uuid::Uuid::new_v4().to_string(),
                            thread_id: thread_id.to_string(),
                            user_id: user_id.to_string(),
                            role: MessageRole::Assistant,
                            message_type: MessageType::ToolCall,
                            content: String::new(),
                            tool_call_id: Some(first_call.id.clone()),
                            tool_name: Some(first_call.function.name.clone()),
                            arguments: serde_json::from_str(&first_call.function.arguments).ok(),
                            reasoning_id: Some(id.clone()),
                            created_at: chrono::Utc::now(),
                            duration_ms: None,
                        })
                    } else {
                        None
                    }
                } else if !content.is_empty() {
                    Some(praxis_persist::DBMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        thread_id: thread_id.to_string(),
                        user_id: user_id.to_string(),
                        role: MessageRole::Assistant,
                        message_type: MessageType::Message,
                        content: content.clone(),
                        tool_call_id: None,
                        tool_name: None,
                        arguments: None,
                        reasoning_id: Some(id.clone()),
                        created_at: chrono::Utc::now(),
                        duration_ms: None,
                    })
                } else {
                    None
                }
            }
        }
    }
    
    /// Convert praxis-llm Message to praxis-persist DBMessage
    fn convert_message_to_db(
        msg: &praxis_llm::Message,
        thread_id: &str,
        user_id: &str,
        _node_type: NodeType,
    ) -> Option<praxis_persist::DBMessage> {
        use praxis_llm::Message;
        use praxis_persist::{MessageRole, MessageType};

        match msg {
            Message::AI { content, tool_calls, .. } => {
                if let Some(calls) = tool_calls {
                    // Save tool calls as separate messages
                    // For simplicity, we'll create a message for the first tool call
                    // In production, you might want to handle all tool calls
                    if let Some(first_call) = calls.first() {
                        Some(praxis_persist::DBMessage {
                            id: uuid::Uuid::new_v4().to_string(),
                            thread_id: thread_id.to_string(),
                            user_id: user_id.to_string(),
                            role: MessageRole::Assistant,
                            message_type: MessageType::ToolCall,
                            content: String::new(),
                            tool_call_id: Some(first_call.id.clone()),
                            tool_name: Some(first_call.function.name.clone()),
                            arguments: serde_json::from_str(&first_call.function.arguments).ok(),
                            reasoning_id: None,
                            created_at: chrono::Utc::now(),
                            duration_ms: None,
                        })
                    } else {
                        None
                    }
                } else if let Some(content) = content {
                    Some(praxis_persist::DBMessage {
                        id: uuid::Uuid::new_v4().to_string(),
                        thread_id: thread_id.to_string(),
                        user_id: user_id.to_string(),
                        role: MessageRole::Assistant,
                        message_type: MessageType::Message,
                        content: content.as_text().unwrap_or("").to_string(),
                        tool_call_id: None,
                        tool_name: None,
                        arguments: None,
                        reasoning_id: None,
                        created_at: chrono::Utc::now(),
                        duration_ms: None,
                    })
                } else {
                    None
                }
            }
            Message::Tool { tool_call_id, content } => {
                Some(praxis_persist::DBMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    thread_id: thread_id.to_string(),
                    user_id: user_id.to_string(),
                    role: MessageRole::Assistant,
                    message_type: MessageType::ToolResult,
                    content: content.as_text().unwrap_or("").to_string(),
                    tool_call_id: Some(tool_call_id.clone()),
                    tool_name: None,
                    arguments: None,
                    reasoning_id: None,
                    created_at: chrono::Utc::now(),
                    duration_ms: None,
                })
            }
            _ => None,
            }
        }

    /// Create observation data for tracing
    #[cfg(feature = "observability")]
    fn create_observation(
        state: &GraphState,
        node_type: NodeType,
        _node_start: Instant,
        node_duration: u64,
        new_messages: &[praxis_llm::Message],
    ) -> Option<praxis_observability::NodeObservation> {
        use praxis_observability::{NodeObservation, NodeObservationData, NodeOutput, LangfuseMessage, ToolCallInfo, ToolResultInfo};
        use crate::types::GraphOutput;

        let span_id = uuid::Uuid::new_v4().to_string();
        let started_at = chrono::Utc::now() - chrono::Duration::milliseconds(node_duration as i64);

        match node_type {
            NodeType::LLM => {
                let input_count = state.messages.len() - new_messages.len();
                
                tracing::info!(
                    "LLM observation - total messages: {}, input_count: {}, new_messages: {}",
                    state.messages.len(),
                    input_count,
                    new_messages.len()
                );
                
                let input_messages: Vec<LangfuseMessage> = state.messages[..input_count]
                    .iter()
                    .filter_map(Self::convert_to_langfuse_message)
                    .collect();

                // Use structured outputs if available
                let outputs = if let Some(ref last_outputs) = state.last_outputs {
                    last_outputs.iter().map(|output| {
                        match output {
                            GraphOutput::Reasoning { id, content } => {
                                NodeOutput::Reasoning {
                                    id: id.clone(),
                                    content: content.clone(),
                                }
                            }
                            GraphOutput::Message { id, content, tool_calls } => {
                                if tool_calls.is_some() {
                                    NodeOutput::ToolCalls {
                                        calls: tool_calls.as_ref().unwrap().iter().map(|call| {
                                            ToolCallInfo {
                                                id: call.id.clone(),
                                                name: call.function.name.clone(),
                                                arguments: serde_json::from_str(&call.function.arguments)
                                                    .unwrap_or(serde_json::json!({})),
                                            }
                                        }).collect(),
                                    }
                                } else {
                                    NodeOutput::Message {
                                        id: id.clone(),
                                        content: content.clone(),
                                    }
                                }
                            }
                        }
                    }).collect()
                } else {
                    // Fallback: convert from new_messages
                    vec![]
                };

                if outputs.is_empty() {
                    tracing::warn!("No outputs available for LLM observation");
                    return None;
                }

                tracing::info!(
                    "Created LLM observation: input_messages={}, outputs={}",
                    input_messages.len(),
                    outputs.len()
                );

                Some(NodeObservation {
                    span_id,
                    run_id: state.run_id.clone(),
                    conversation_id: state.conversation_id.clone(),
                    node_type: "llm".to_string(),
                    started_at,
                    duration_ms: node_duration,
                    data: NodeObservationData::Llm {
                        input_messages,
                        outputs,
                        model: state.llm_config.model.clone(),
                        usage: None,
                    },
                    metadata: std::collections::HashMap::new(),
                })
            }
            NodeType::Tool => {
                // Extract tool calls from previous AI message
                let tool_calls: Vec<ToolCallInfo> = state.messages
                    .iter()
                    .rev()
                    .find_map(|msg| match msg {
                        praxis_llm::Message::AI { tool_calls: Some(calls), .. } => {
                            Some(calls.iter().map(|call| ToolCallInfo {
                                id: call.id.clone(),
                                name: call.function.name.clone(),
                                arguments: serde_json::from_str(&call.function.arguments)
                                    .unwrap_or(serde_json::json!({})),
                            }).collect())
                        }
                        _ => None,
                    })?;

                // Extract tool results from new messages
                let tool_results: Vec<ToolResultInfo> = new_messages
                    .iter()
                    .filter_map(|msg| match msg {
                        praxis_llm::Message::Tool { tool_call_id, content } => {
                            Some(ToolResultInfo {
                                tool_call_id: tool_call_id.clone(),
                                tool_name: "unknown".to_string(), // TODO: track tool name
                                result: content.as_text().unwrap_or("").to_string(),
                                is_error: false,
                                duration_ms: 0, // TODO: track individual tool duration
                            })
                        }
                        _ => None,
                    })
                    .collect();

                tracing::debug!(
                    "Creating Tool observation: tool_calls_count={}, tool_results_count={}",
                    tool_calls.len(),
                    tool_results.len()
                );

                Some(NodeObservation {
                    span_id,
                    run_id: state.run_id.clone(),
                    conversation_id: state.conversation_id.clone(),
                    node_type: "tool".to_string(),
                    started_at,
                    duration_ms: node_duration,
                    data: NodeObservationData::Tool {
                        tool_calls,
                        tool_results,
                    },
                    metadata: std::collections::HashMap::new(),
                })
            }
        }
    }

    /// Convert praxis-llm Message to Langfuse format
    #[cfg(feature = "observability")]
    fn convert_to_langfuse_message(msg: &praxis_llm::Message) -> Option<praxis_observability::LangfuseMessage> {
        use praxis_observability::{LangfuseMessage, ToolCallInfo};

        match msg {
            praxis_llm::Message::System { content, .. } => Some(LangfuseMessage {
                role: "system".to_string(),
                content: content.as_text().unwrap_or("").to_string(),
                name: None,
                tool_call_id: None,
                tool_calls: None,
            }),
            praxis_llm::Message::Human { content, .. } => Some(LangfuseMessage {
                role: "user".to_string(),
                content: content.as_text().unwrap_or("").to_string(),
                name: None,
                tool_call_id: None,
                tool_calls: None,
            }),
            praxis_llm::Message::AI { content, tool_calls, .. } => {
                let tool_calls_converted = tool_calls.as_ref().map(|calls| {
                    calls.iter().map(|call| ToolCallInfo {
                        id: call.id.clone(),
                        name: call.function.name.clone(),
                        arguments: serde_json::from_str(&call.function.arguments)
                            .unwrap_or(serde_json::json!({})),
                    }).collect()
                });

                Some(LangfuseMessage {
                    role: "assistant".to_string(),
                    content: content.as_ref()
                        .and_then(|c| c.as_text())
                        .unwrap_or("")
                        .to_string(),
                    name: None,
                    tool_call_id: None,
                    tool_calls: tool_calls_converted,
                })
            }
            praxis_llm::Message::Tool { tool_call_id, content } => Some(LangfuseMessage {
                role: "tool".to_string(),
                content: content.as_text().unwrap_or("").to_string(),
                name: None,
                tool_call_id: Some(tool_call_id.clone()),
                tool_calls: None,
            }),
        }
    }
}

