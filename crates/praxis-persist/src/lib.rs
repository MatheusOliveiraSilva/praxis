pub mod models;
pub mod repositories;
pub mod context;
pub mod client;
pub mod error;
pub mod builder;
pub mod templates;

pub use models::{Thread, ThreadMetadata, ThreadSummary, Message, MessageRole, MessageType};
pub use repositories::{ThreadRepository, MessageRepository};
pub use context::ContextManager;
pub use client::PersistClient;
pub use error::PersistError;
pub use builder::PersistClientBuilder;
pub use templates::{DEFAULT_SYSTEM_PROMPT_TEMPLATE, DEFAULT_SUMMARIZATION_PROMPT};

