use crate::node::{EventSender, Node, NodeType};
use anyhow::Result;
use async_trait::async_trait;
use praxis_mcp::{MCPToolExecutor, ToolResponse};
use praxis_types::{GraphState, StreamEvent};
use std::sync::Arc;
use std::time::Instant;

pub struct ToolNode {
    mcp_executor: Arc<MCPToolExecutor>,
}

impl ToolNode {
    pub fn new(mcp_executor: Arc<MCPToolExecutor>) -> Self {
        Self { mcp_executor }
    }
}

#[async_trait]
impl Node for ToolNode {
    async fn execute(&self, state: &mut GraphState, event_tx: EventSender) -> Result<()> {
        // Get pending tool calls from state
        let tool_calls = state.get_pending_tool_calls();

        if tool_calls.is_empty() {
            return Ok(());
        }

        // Execute each tool call
        for tool_call in tool_calls {
            let start = Instant::now();

            // Parse arguments from string to Value
            let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;
            
            match self
                .mcp_executor
                .execute_tool(&tool_call.function.name, args)
                .await
            {
                Ok(responses) => {
                    // Join all responses into a single result string
                    let result = ToolResponse::join_responses(&responses);
                    
                    // Success: emit result event
                    event_tx
                        .send(StreamEvent::ToolResult {
                            tool_call_id: tool_call.id.clone(),
                            result: result.clone(),
                            is_error: false,
                            duration_ms: start.elapsed().as_millis() as u64,
                        })
                        .await?;

                    // Add tool result to state
                    state.add_tool_result(tool_call.id, result);
                }
                Err(e) => {
                    // Tool failed (resilient) - emit error result
                    let error_msg = format!("Tool execution failed: {}", e);

                    event_tx
                        .send(StreamEvent::ToolResult {
                            tool_call_id: tool_call.id.clone(),
                            result: error_msg.clone(),
                            is_error: true,
                            duration_ms: start.elapsed().as_millis() as u64,
                        })
                        .await?;

                    // Add error result to state so LLM can see it
                    state.add_tool_result(tool_call.id, error_msg);
                }
            }
        }

        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::Tool
    }
}

