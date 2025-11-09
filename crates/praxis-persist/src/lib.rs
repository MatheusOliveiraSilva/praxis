// Core modules
pub mod models;
pub mod error;
pub mod trait_client;
pub mod accumulator;
pub mod dbs;
pub mod templates;

// Public exports
pub use trait_client::PersistenceClient;
pub use accumulator::EventAccumulator;
pub use models::{DBMessage, MessageRole, MessageType, Thread, ThreadMetadata, ThreadSummary};
pub use error::{PersistError, Result};
pub use templates::{DEFAULT_SYSTEM_PROMPT_TEMPLATE, DEFAULT_SUMMARIZATION_PROMPT};
