# praxis-llm

Provider-agnostic LLM client library with support for Chat Completions and Responses API (reasoning).

## Features

- Chat Completions API
- Responses API with reasoning support
- Streaming support for both APIs
- History reconstruction for conversation management
- Provider-agnostic traits for easy extensibility
- Support for OpenAI and Azure OpenAI
- Factory pattern for dynamic provider selection

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

### Azure OpenAI

```rust
use praxis_llm::{AzureOpenAIClient, ChatClient, ChatRequest, Message};

let client = AzureOpenAIClient::builder()
    .api_key(api_key)
    .resource_name("my-resource")
    .deployment_name("gpt-4-deployment")
    .api_version("2024-02-15-preview")
    .build()?;

let request = ChatRequest::new("gpt-4", vec![
    Message::human("What is the capital of France?")
]);

let response = client.chat(request).await?;
println!("{}", response.content.unwrap_or_default());
```

### Factory Pattern

```rust
use praxis_llm::{ClientFactory, ProviderConfig};

// Create client from config (useful for dynamic provider selection)
let config = ProviderConfig::azure_openai(
    api_key,
    "my-resource",
    "gpt-4-deployment",
    "2024-02-15-preview"
);

let client = ClientFactory::create_client(config)?;
let response = client.chat(request).await?;
```

## Examples

See the `examples/` directory for complete working examples:

**OpenAI:**
- `01_chat.rs` - Basic chat completion
- `02_chat_streaming.rs` - Streaming chat
- `03_reasoning.rs` - Responses API with reasoning
- `04_reasoning_streaming.rs` - Streaming with reasoning

**Azure OpenAI:**
- `06_azure_chat.rs` - Basic Azure chat completion
- `07_azure_streaming.rs` - Azure streaming chat
- `08_azure_reasoning.rs` - Azure reasoning API
- `09_factory_pattern.rs` - Factory pattern for provider selection

Run examples:
```bash
# OpenAI
export OPENAI_API_KEY=your-key
cargo run --example 01_chat

# Azure OpenAI
export AZURE_OPENAI_API_KEY=your-key
export AZURE_OPENAI_RESOURCE_NAME=my-resource
export AZURE_OPENAI_DEPLOYMENT_NAME=gpt-4-deployment
cargo run --example 06_azure_chat
```

## Azure OpenAI Configuration

Azure OpenAI uses a different endpoint structure than OpenAI:

- **URL Pattern**: `https://{resource}.openai.azure.com/openai/deployments/{deployment}/...`
- **Authentication**: `api-key` header instead of `Authorization: Bearer`
- **Model Selection**: Specified via deployment name in URL, not in request body
- **API Version**: Required as query parameter

### Environment Variables

For Azure OpenAI, set these environment variables:

```bash
AZURE_OPENAI_API_KEY=your-azure-api-key
AZURE_OPENAI_RESOURCE_NAME=your-resource-name
AZURE_OPENAI_DEPLOYMENT_NAME=your-deployment-name
AZURE_OPENAI_API_VERSION=2024-02-15-preview  # Optional, defaults to latest
```

### Deployment Names

Azure OpenAI requires you to create deployments of models in your resource. The deployment name is different from the model name:

- **Model**: `gpt-4`, `gpt-4o`, `o1`, etc. (OpenAI model names)
- **Deployment**: Your custom deployment name (e.g., `my-gpt4-deployment`)

The client uses the deployment name in the URL and the model name for internal logic (e.g., determining if it's a reasoning model).

## License

MIT
