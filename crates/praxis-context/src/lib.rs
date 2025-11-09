mod strategy;
mod default;
mod templates;

pub use strategy::{ContextStrategy, ContextWindow};
pub use default::DefaultContextStrategy;
pub use templates::{DEFAULT_SYSTEM_PROMPT_TEMPLATE, DEFAULT_SUMMARIZATION_PROMPT};
