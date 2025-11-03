# praxis-llm

Provider-agnostic LLM client library with support for Chat Completions and Responses API (reasoning).

## Features

- Chat Completions API
- Responses API with reasoning support
- Streaming support for both APIs
- History reconstruction for conversation management
- Provider-agnostic traits for easy extensibility

## Installation

```toml
[dependencies]
praxis-llm = "0.1"
```

## Usage

### Chat Completions

```rust
use praxis_llm::{LLMClient, OpenAIClient, ChatRequest, Message};

let client = OpenAIClient::new(api_key)?;
let request = ChatRequest::new("gpt-4o", vec![
    Message::human("What is the capital of France?")
]);

let response = client.chat_completion(request).await?;
println!("{}", response.content.unwrap_or_default());
```

### Streaming

```rust
use praxis_llm::{LLMClient, OpenAIClient, ChatRequest, Message, StreamEvent};
use futures::StreamExt;

let client = OpenAIClient::new(api_key)?;
let request = ChatRequest::new("gpt-4o", vec![
    Message::human("Write a poem about coding.")
]);

let mut stream = client.chat_completion_stream(request).await?;

while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::Message { content } => print!("{}", content),
        _ => {}
    }
}
```

### Reasoning (Responses API)

```rust
use praxis_llm::{LLMClient, OpenAIClient, ResponseRequest, Message, ReasoningConfig};

let client = OpenAIClient::new(api_key)?;
let request = ResponseRequest::new("gpt-5", vec![
    Message::human("Explain quantum entanglement.")
]).with_reasoning(ReasoningConfig::medium());

let response = client.response(request).await?;

if let Some(reasoning) = response.reasoning {
    println!("Reasoning: {}", reasoning);
}
if let Some(message) = response.message {
    println!("Response: {}", message);
}
```

## Examples

See the `examples/` directory for complete working examples:

- `01_chat.rs` - Basic chat completion
- `02_chat_streaming.rs` - Streaming chat
- `03_reasoning.rs` - Responses API with reasoning
- `04_reasoning_streaming.rs` - Streaming with reasoning

Run examples:
```bash
cargo run --example 01_chat
```

## License

MIT
