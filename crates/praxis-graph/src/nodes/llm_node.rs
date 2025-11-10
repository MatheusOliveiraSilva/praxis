use crate::node::{EventSender, Node, NodeType};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use praxis_llm::{ChatClient, ReasoningClient, ChatOptions, ChatRequest, ResponseRequest, ReasoningConfig, Message, ToolChoice};
use praxis_mcp::MCPToolExecutor;
use crate::types::GraphState;
use std::sync::Arc;

pub struct LLMNode {
    client: Arc<dyn ChatClient>,
    reasoning_client: Option<Arc<dyn ReasoningClient>>,
    mcp_executor: Arc<MCPToolExecutor>,
}

impl LLMNode {
    pub fn new(client: Arc<dyn ChatClient>, mcp_executor: Arc<MCPToolExecutor>) -> Self {
        let reasoning_client = None; // We'll set this from client if it implements both traits
        Self { 
            client,
            reasoning_client,
            mcp_executor,
        }
    }

    pub fn with_reasoning_client(mut self, reasoning_client: Arc<dyn ReasoningClient>) -> Self {
        self.reasoning_client = Some(reasoning_client);
        self
    }

    /// Convert praxis_llm::StreamEvent to Graph StreamEvent
    /// Uses automatic From trait conversion
    fn convert_event(event: praxis_llm::StreamEvent) -> crate::types::StreamEvent {
        event.into()
    }

    /// Check if model should use Reasoning API
    fn is_reasoning_model(model: &str) -> bool {
        model.starts_with("gpt-5") || model.starts_with("o")
    }
}

#[async_trait]
impl Node for LLMNode {
    async fn execute(&self, state: &mut GraphState, event_tx: EventSender) -> Result<()> {
        let model = &state.llm_config.model;
        
        // Check if we should use Reasoning API (for gpt-5, o1 models)
        let use_reasoning_api = Self::is_reasoning_model(model) && self.reasoning_client.is_some();
        
        tracing::info!(
            "LLM_NODE: Executing with model={}, use_reasoning_api={}",
            model,
            use_reasoning_api
        );

        // Call appropriate API based on model type
        let mut stream = if use_reasoning_api {
            // Use Reasoning API for gpt-5/o1 models (streams reasoning separately)
            let reasoning_config = state.llm_config.reasoning_effort
                .as_ref()
                .map(|effort| match effort.as_str() {
                    "low" => ReasoningConfig::low(),
                    "high" => ReasoningConfig::high(),
                    _ => ReasoningConfig::medium(),
                });

            let request = ResponseRequest::new(model.clone(), state.messages.clone());
            let request = if let Some(config) = reasoning_config {
                request.with_reasoning(config)
            } else {
                request
            };

            self.reasoning_client.as_ref().unwrap().reason_stream(request).await?
        } else {
            // Use Chat API for regular models
            let tools = self.mcp_executor.get_llm_tools().await?;
            
            let mut options = ChatOptions::new()
                .tools(tools)
                .tool_choice(ToolChoice::auto());
            
            if let Some(temp) = state.llm_config.temperature {
                options = options.temperature(temp);
            }
            if let Some(max_tokens) = state.llm_config.max_tokens {
                options = options.max_tokens(max_tokens);
            }

            let request = ChatRequest::new(model.clone(), state.messages.clone())
                .with_options(options);

            self.client.chat_stream(request).await?
        };

        // Track tool calls and text content as they stream in
        let mut accumulated_tool_calls: Vec<praxis_llm::ToolCall> = Vec::new();
        let mut tool_call_buffers: std::collections::HashMap<u32, (Option<String>, Option<String>, String)> = std::collections::HashMap::new();
        let mut accumulated_text_content = String::new();

        // Forward events and accumulate content
        while let Some(event_result) = stream.next().await {
            let llm_event = event_result?;

            // Convert and forward to client
            let graph_event = Self::convert_event(llm_event.clone());
            event_tx.send(graph_event).await?;

            // Accumulate based on event type
            match llm_event {
                praxis_llm::StreamEvent::Message { content } | 
                praxis_llm::StreamEvent::Reasoning { content } => {
                    // Accumulate text content (from both Message and Reasoning events)
                    accumulated_text_content.push_str(&content);
                }
                praxis_llm::StreamEvent::ToolCall { index, id, name, arguments } => {
                    // Accumulate tool calls
                let entry = tool_call_buffers.entry(index).or_insert((None, None, String::new()));
                
                if let Some(id) = id {
                    entry.0 = Some(id);
                }
                if let Some(name) = name {
                    entry.1 = Some(name);
                }
                if let Some(args) = arguments {
                    entry.2.push_str(&args);
                }
                }
                _ => {} // Ignore other events (Done, Error, etc.)
            }
        }

        // Build final tool calls from accumulated data
        for (_, (id, name, arguments)) in tool_call_buffers {
            if let (Some(id), Some(name)) = (id, name) {
                accumulated_tool_calls.push(praxis_llm::ToolCall {
                    id,
                    tool_type: "function".to_string(),
                    function: praxis_llm::types::FunctionCall {
                        name,
                        arguments,
                    },
                });
            }
        }

        // Add assistant message to state with accumulated content
        let content = if !accumulated_text_content.is_empty() {
            tracing::debug!(
                "LLM node completed - accumulated text content length: {}",
                accumulated_text_content.len()
            );
            Some(praxis_llm::Content::Text(accumulated_text_content))
        } else {
            tracing::debug!("LLM node completed - no text content accumulated");
            None
        };

        let tool_calls = if !accumulated_tool_calls.is_empty() {
            tracing::debug!(
                "LLM node completed - accumulated {} tool calls",
                accumulated_tool_calls.len()
            );
            Some(accumulated_tool_calls)
        } else {
            None
        };

        let assistant_message = Message::AI {
            content,
            tool_calls,
                name: None,
        };

        state.add_message(assistant_message);

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::LLM
    }
}

