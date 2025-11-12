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
    .endpoint("https://my-resource.openai.azure.com/openai/deployments/gpt-4-deployment")
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
    "https://my-resource.openai.azure.com/openai/deployments/gpt-4-deployment",
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
- `03_reasoning.rs` - Responses API with reasoning (o1 models)
- `04_reasoning_streaming.rs` - Streaming with reasoning
- `05_tool_use.rs` - Function calling and tools

**Azure OpenAI:**
- `06_azure_chat.rs` - Basic Azure chat completion
- `07_azure_streaming.rs` - Azure streaming chat
- `09_factory_pattern.rs` - Factory pattern for provider selection

**Note:** Azure OpenAI currently does not support the Responses API (reasoning models like o1-preview/o1-mini). For reasoning capabilities, use the OpenAI provider directly.

Run examples:
```bash
# OpenAI
export OPENAI_API_KEY=your-key
cargo run --example 01_chat

# Azure OpenAI (Chat Completions only)
export AZURE_OPENAI_API_KEY=your-key
export AZURE_OPENAI_ENDPOINT=https://my-resource.openai.azure.com
export AZURE_OPENAI_API_VERSION=2024-02-15-preview  # Optional
cargo run --example 06_azure_chat
```

## Azure OpenAI Configuration

Azure OpenAI uses a different endpoint structure than OpenAI and has some limitations:

- **URL Pattern**: `https://{resource}.openai.azure.com/openai/deployments/{deployment}/...`
- **Authentication**: `api-key` header instead of `Authorization: Bearer`
- **Model Selection**: Specified via deployment name (passed as model parameter in requests)
- **API Version**: Required as query parameter
- **⚠️ Limitations**: 
  - Does not support the Responses API (`/responses` endpoint)
  - Reasoning models (o1-preview, o1-mini) are not available
  - Only Chat Completions API is supported

### Environment Variables

For Azure OpenAI, set these environment variables:

```bash
AZURE_OPENAI_API_KEY=your-azure-api-key
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com
AZURE_OPENAI_API_VERSION=2024-02-15-preview  # Optional
```

### Endpoint Format

The Azure endpoint is the base URL of your Azure OpenAI resource:

```
https://{resource-name}.openai.azure.com
```

Example:
```
https://my-openai-resource.openai.azure.com
```

The deployment name is specified per-request via the model parameter, and the client constructs the full URL internally:
```
https://{resource}.openai.azure.com/openai/deployments/{deployment}/chat/completions?api-version={version}
```

This design allows you to use different deployments with the same client instance:

```rust
let client = AzureOpenAIClient::builder()
    .api_key(api_key)
    .endpoint("https://my-resource.openai.azure.com")
    .api_version("2024-02-15-preview")
    .build()?;

// Use different deployments for different models
let gpt4_request = ChatRequest::new("gpt-4-deployment", messages);
let gpt35_request = ChatRequest::new("gpt-35-turbo-deployment", messages);
```

## License

MIT
