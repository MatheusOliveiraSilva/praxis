pub mod observer;
pub mod types;

#[cfg(feature = "langfuse")]
pub mod langfuse;

// Re-export main types
pub use observer::Observer;
pub use types::{
    NodeObservation, NodeObservationData, LangfuseMessage, TraceContext, 
    ToolCallInfo, ToolResultInfo, TokenUsage,
};

#[cfg(feature = "langfuse")]
pub use langfuse::observer::LangfuseObserver;

