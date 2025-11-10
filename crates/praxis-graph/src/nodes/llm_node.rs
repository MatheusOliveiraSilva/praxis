use crate::node::{EventSender, Node, NodeType};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use praxis_llm::{ChatClient, ChatOptions, ChatRequest, Message, ToolChoice};
use praxis_mcp::MCPToolExecutor;
use crate::types::GraphState;
use std::sync::Arc;

pub struct LLMNode {
    client: Arc<dyn ChatClient>,
    mcp_executor: Arc<MCPToolExecutor>,
}

impl LLMNode {
    pub fn new(client: Arc<dyn ChatClient>, mcp_executor: Arc<MCPToolExecutor>) -> Self {
        Self { 
            client,
            mcp_executor,
        }
    }

    /// Convert praxis_llm::StreamEvent to Graph StreamEvent
    /// Uses automatic From trait conversion
    fn convert_event(event: praxis_llm::StreamEvent) -> crate::types::StreamEvent {
        event.into()
    }
}

#[async_trait]
impl Node for LLMNode {
    async fn execute(&self, state: &mut GraphState, event_tx: EventSender) -> Result<()> {
        // Get tools from all connected MCP servers
        let tools = self.mcp_executor.get_llm_tools().await?;
        
        // Build chat request from state
        let options = ChatOptions::new()
            .tools(tools)
            .tool_choice(ToolChoice::auto());

        let request = ChatRequest::new(state.llm_config.model.clone(), state.messages.clone())
            .with_options(options);

        // Call LLM with streaming
        let mut stream = self.client.chat_stream(request).await?;

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

