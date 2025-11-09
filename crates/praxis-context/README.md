# Praxis Context

Context management and summarization for AI agent conversations.

## Features

- Token counting with tiktoken
- Automatic summarization
- Context window strategies
- Template-based system prompts

## Usage

```rust
use praxis_context::{DefaultContextStrategy, ContextStrategy};
use praxis_persist::PersistenceClient;
use std::sync::Arc;

let strategy = DefaultContextStrategy::new(
    llm_client,
    max_tokens: 8000,
);

let context = strategy.get_context_window(
    thread_id,
    persist_client,
).await?;
```

