use crate::node::NodeType;
use praxis_types::GraphState;

/// Decides which node to execute next based on current state
pub trait Router: Send + Sync {
    fn next(&self, state: &GraphState, current: NodeType) -> NextNode;
}

#[derive(Debug, Clone, PartialEq)]
pub enum NextNode {
    LLM,
    Tool,
    End,
}

/// Simple router implementing React agent pattern:
/// LLM -> Tool (if tool_calls present) -> LLM -> END
pub struct SimpleRouter;

impl Router for SimpleRouter {
    fn next(&self, state: &GraphState, current: NodeType) -> NextNode {
        match current {
            NodeType::LLM => {
                // Check if last message has tool calls
                if state.has_pending_tool_calls() {
                    NextNode::Tool
                } else {
                    NextNode::End
                }
            }
            NodeType::Tool => {
                // Always return to LLM after executing tools
                NextNode::LLM
            }
        }
    }
}

