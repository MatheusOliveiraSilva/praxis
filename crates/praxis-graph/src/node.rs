use anyhow::Result;
use async_trait::async_trait;
use crate::types::{GraphState, StreamEvent};
use tokio::sync::mpsc;

pub type EventSender = mpsc::Sender<StreamEvent>;

/// Core abstraction for a unit of computation in the graph
#[async_trait]
pub trait Node: Send + Sync {
    /// Execute the node's logic, potentially modifying state and emitting events
    async fn execute(&self, state: &mut GraphState, event_tx: EventSender) -> Result<()>;
    
    /// Return the type of this node
    fn node_type(&self) -> NodeType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    LLM,
    Tool,
}

