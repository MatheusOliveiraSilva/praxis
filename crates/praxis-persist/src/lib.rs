// Core modules
mod models;
mod error;
mod trait_client;
mod accumulator;

#[cfg(feature = "mongodb")]
mod dbs;

// Public exports
pub use trait_client::PersistenceClient;
pub use accumulator::EventAccumulator;
pub use models::{DBMessage, MessageRole, MessageType, Thread, ThreadMetadata, ThreadSummary};
pub use error::{PersistError, Result};

#[cfg(feature = "mongodb")]
pub use dbs::mongo::MongoPersistenceClient;
