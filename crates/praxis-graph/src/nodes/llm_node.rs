use crate::node::{EventSender, Node, NodeType};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use praxis_llm::{ChatOptions, ChatRequest, LLMClient, Message, ToolChoice};
use praxis_mcp::MCPToolExecutor;
use praxis_types::GraphState;
use std::sync::Arc;

pub struct LLMNode {
    client: Arc<dyn LLMClient>,
    mcp_executor: Arc<MCPToolExecutor>,
}

impl LLMNode {
    pub fn new(client: Arc<dyn LLMClient>, mcp_executor: Arc<MCPToolExecutor>) -> Self {
        Self { 
            client,
            mcp_executor,
        }
    }

    /// Convert praxis_llm::StreamEvent to praxis_types::StreamEvent
    fn convert_event(event: praxis_llm::StreamEvent) -> praxis_types::StreamEvent {
        match event {
            praxis_llm::StreamEvent::Reasoning { content } => {
                praxis_types::StreamEvent::Reasoning { content }
            }
            praxis_llm::StreamEvent::Message { content } => {
                praxis_types::StreamEvent::Message { content }
            }
            praxis_llm::StreamEvent::ToolCall {
                index,
                id,
                name,
                arguments,
            } => praxis_types::StreamEvent::ToolCall {
                index,
                id,
                name,
                arguments,
            },
            praxis_llm::StreamEvent::Done { finish_reason } => {
                praxis_types::StreamEvent::Done { finish_reason }
            }
        }
    }
}

#[async_trait]
impl Node for LLMNode {
    async fn execute(&self, state: &mut GraphState, event_tx: EventSender) -> Result<()> {
        // Get tools from all connected MCP servers
        let tools = self.mcp_executor.get_llm_tools().await?;
        
        // Build chat request from state
        let options = ChatOptions::new()
            .temperature(state.llm_config.temperature.unwrap_or(0.7))
            .max_tokens(state.llm_config.max_tokens.unwrap_or(4096))
            .tools(tools)
            .tool_choice(ToolChoice::auto());

        let request = ChatRequest::new(state.llm_config.model.clone(), state.messages.clone())
            .with_options(options);

        // Call LLM with streaming
        let mut stream = self.client.chat_completion_stream(request).await?;

        // Track tool calls as they stream in
        let mut accumulated_tool_calls: Vec<praxis_llm::ToolCall> = Vec::new();
        let mut tool_call_buffers: std::collections::HashMap<u32, (Option<String>, Option<String>, String)> = std::collections::HashMap::new();

        // Forward events and accumulate tool calls
        while let Some(event_result) = stream.next().await {
            let llm_event = event_result?;

            // Convert and forward to client
            let graph_event = Self::convert_event(llm_event.clone());
            event_tx.send(graph_event).await?;

            // Accumulate tool calls for state
            if let praxis_llm::StreamEvent::ToolCall { index, id, name, arguments } = llm_event {
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

        // Add assistant message to state
        let assistant_message = if accumulated_tool_calls.is_empty() {
            Message::AI {
                content: None, // Content was streamed, not accumulated here
                tool_calls: None,
                name: None,
            }
        } else {
            Message::AI {
                content: None,
                tool_calls: Some(accumulated_tool_calls),
                name: None,
            }
        };

        state.add_message(assistant_message);

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::LLM
    }
}

