pub mod observer;
pub mod types;

#[cfg(feature = "langfuse")]
pub mod langfuse;

// Re-export main types
pub use observer::Observer;
pub use types::{
    NodeObservation, NodeObservationData, NodeOutput, LangfuseMessage, TraceContext, 
    ToolCallInfo, ToolResultInfo,
};

// Re-export TokenUsage from praxis-llm to avoid duplication
pub use praxis_llm::TokenUsage;

#[cfg(feature = "langfuse")]
pub use langfuse::observer::LangfuseObserver;

