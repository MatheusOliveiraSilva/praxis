# Praxis LLM - OpenAI Implementation

✅ **Complete!** Learning project implementing OpenAI chat completions from scratch in Rust.

## Features Implemented

- ✅ **Chat completions** (system, user, assistant, tool messages)
- ✅ **Streaming** (SSE parsing, token-by-token)
- ✅ **Function calling / Tools**
- ✅ **MCP (Model Context Protocol) integration** via stdio
- ✅ **Type-safe message abstractions** (extensible for multimodal)
- ✅ **HTTP direct** (no SDK, reqwest + rustls only)

## Quick Start

### Prerequisites

```bash
# Set OpenAI API key
export OPENAI_API_KEY=sk-...
```

### Run Examples

```bash
# 1. Simple chat (non-streaming)
cargo run --example 01_simple_chat

# 2. Streaming (token-by-token)
cargo run --example 02_streaming

# 3. Tools/Function calling (calculator example)
cargo run --example 03_tools

# 4. MCP integration (connect to MCP server, use its tools)
cargo run --example 04_mcp_integration
```

## Example Outputs

### Example 1: Simple Chat

```
🤖 Praxis LLM - OpenAI Simple Chat Example

📤 Sending messages:
  - system: Message::System { ... }
  - user: Message::Human { ... }

⏳ Waiting for response...

✅ Response received!

📊 Metadata:
  - ID: chatcmpl-...
  - Model: gpt-4o-mini
  - Tokens: 25 prompt + 12 completion = 37 total

💬 Assistant:
The capital of France is Paris.
```

### Example 2: Streaming

```
🌊 Praxis LLM - OpenAI Streaming Example

📤 Sending message (streaming)...

💬 Assistant:
Rust is strong and fast,
Safe concurrency at last,
Memory guaranteed.

✅ Stream complete!
📊 Total characters: 87
```

### Example 3: Tools

```
🔧 Praxis LLM - OpenAI Tools Example

📋 Available tools:
  - calculator: Evaluates a mathematical expression

📤 Sending message with tools available...

✅ Response received!

🔧 Model requested 1 tool call(s):

Tool Call ID: call_abc123
Function: calculator
Arguments: {"expression":"156 * 47"}
Expression to evaluate: 156 * 47
Result: 7332

📤 Sending tool result back to model...

✅ Final response:

💬 Assistant: The result of 156 multiplied by 47 is 7,332.

📊 Total tokens used:
  - First call: 95 tokens
  - Second call: 48 tokens
  - Grand total: 143 tokens
```

### Example 4: MCP Integration

```
🔗 Praxis LLM - OpenAI + MCP Integration Example

📋 Configuration:
  MCP Command: npx -y @modelcontextprotocol/server-everything

🔌 Connecting to MCP server...
✅ Connected!

📦 Fetching available tools...
✅ Found 3 tools:

  🔧 echo
     Echoes back the input
  🔧 add
     Adds two numbers
  🔧 get_time
     Gets current time

💬 Sending message to OpenAI with MCP tools...

✅ Response received!

🔧 Model requested 1 tool call(s):

Tool Call:
  ID: call_xyz789
  Function: echo
  Arguments: {"message":"Hello from MCP!"}

⚙️  Executing via MCP...
✅ Tool execution successful!
📄 Result: "Hello from MCP!"

📤 Sending tool result back to OpenAI...

✅ Final response:

💬 Assistant: I successfully used the echo tool, and it returned: "Hello from MCP!"

📊 Token usage:
  - First call: 120 tokens
  - Second call: 55 tokens
  - Total: 175 tokens
```

## Architecture

### Message Types (Praxis format - provider-agnostic)

```rust
// High-level message types
Message::System { content }
Message::Human { content }
Message::AI { content, tool_calls }
Message::Tool { tool_call_id, content }

// Convenience constructors
Message::system("You are a helpful assistant")
Message::human("What's 2+2?")
Message::ai("The answer is 4.")
Message::tool_result("call_123", "4")
```

### Content Types (Extensible for multimodal)

```rust
Content::Text(String)

// Future (prepared but not implemented):
// Content::Parts(Vec<ContentPart>)
// ContentPart::Text { text }
// ContentPart::ImageUrl { url, detail }
```

### OpenAI Client

```rust
// Non-streaming
let response = client
    .chat_completion(
        "gpt-4o-mini",
        messages,
        ChatOptions::new()
            .temperature(0.7)
            .max_tokens(100)
            .tools(tools)
            .tool_choice(ToolChoice::auto()),
    )
    .await?;

// Streaming
let mut stream = client
    .chat_completion_stream(
        "gpt-4o-mini",
        messages,
        ChatOptions::new(),
    )
    .await?;

while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk?.content() {
        print!("{}", content);
    }
}
```

### MCP Integration

```rust
// Connect to MCP server (stdio)
let mut mcp_client = MCPClient::connect("npx", &[
    "-y",
    "@modelcontextprotocol/server-everything"
])?;

// List available tools
let mcp_tools = mcp_client.list_tools()?;

// Convert to OpenAI format
let openai_tools = mcp_tools_to_openai(&mcp_tools);

// Send to OpenAI with tools
let response = openai_client
    .chat_completion("gpt-4o-mini", messages, 
        ChatOptions::new().tools(openai_tools))
    .await?;

// Execute tool via MCP
if let Some(tool_call) = response.tool_calls()?.first() {
    let args = tool_call.arguments_value()?;
    let result = mcp_client.call_tool(&tool_call.function.name, args)?;
}
```

## Design Decisions

### 1. No SDK Dependencies

- **What**: HTTP calls via `reqwest`, JSON with `serde_json`
- **Why**: Learn how APIs work, full control, no bloat
- **Trade-off**: More code, but educational

### 2. Type-Safe Message Enum

- **What**: Rust enum for message types (not HashMap/JSON)
- **Why**: Compile-time safety, IDE autocomplete, clear semantics
- **Trade-off**: Conversion logic needed (worth it!)

### 3. Extensible Content

- **What**: `Content` enum supports `Text` now, `Parts` for multimodal later
- **Why**: Prepared for images/audio without breaking changes
- **Trade-off**: Slightly more complex, but future-proof

### 4. MCP via Stdio

- **What**: JSON-RPC over stdin/stdout (process spawning)
- **Why**: Most common MCP transport, simple to implement
- **Trade-off**: HTTP/SSE MCP not supported yet (can add later)

### 5. Streaming with async-stream

- **What**: SSE parsing with buffer + line-by-line processing
- **Why**: Handles chunked HTTP responses correctly
- **Trade-off**: Manual parsing vs SDK (but we learn!)

## Code Structure

```
praxis-llm-openai/
├── src/
│   ├── types/
│   │   ├── message.rs       # Message enum
│   │   ├── content.rs       # Content enum
│   │   └── tool.rs          # Tool, ToolCall, ToolChoice
│   ├── client.rs            # OpenAIClient (HTTP direct)
│   ├── streaming.rs         # SSE parser
│   └── mcp/
│       ├── client.rs        # MCPClient (JSON-RPC stdio)
│       └── adapter.rs       # MCP → OpenAI conversion
├── examples/
│   ├── 01_simple_chat.rs    # Basic request/response
│   ├── 02_streaming.rs      # Token-by-token streaming
│   ├── 03_tools.rs          # Function calling
│   └── 04_mcp_integration.rs# MCP + OpenAI
└── Cargo.toml
```

## What Was Learned

1. **OpenAI API internals**: Request/response format, streaming SSE, tool calling
2. **Async Rust**: tokio, async-trait, Stream, Pin<Box<...>>
3. **Type design**: Enums for message types, extensibility patterns
4. **Error handling**: anyhow, Result propagation, context
5. **JSON-RPC**: MCP protocol, stdio communication
6. **Streaming**: SSE parsing, buffering, line-by-line processing

## Next Steps

### For Praxis Project

This code is ready to be adapted into `praxis-llm` crate:

1. **Extract to crate**: Move to `praxis-llm/` in main project
2. **Add providers**: Anthropic, Azure, Bedrock (same trait)
3. **Unified trait**: `LLMClient` trait all providers implement
4. **Builder pattern**: Easy instantiation for any provider
5. **Integration**: Use in Praxis Graph/Node execution

### Extensions (Future)

- ⬜ Multimodal: Add image support (Vision API)
- ⬜ MCP HTTP/SSE: Support other transports
- ⬜ Reasoning tokens: Parse o1-style extended_reasoning
- ⬜ Batch API: For async workloads
- ⬜ Fine-tuning: Training data generation helpers

## Testing with Your MCP Server

```bash
# Default: Uses MCP example server
cargo run --example 04_mcp_integration

# Custom MCP server
MCP_COMMAND="node" \
MCP_ARGS="/path/to/your/mcp-server.js" \
cargo run --example 04_mcp_integration

# Or with npx
MCP_COMMAND="npx" \
MCP_ARGS="-y your-mcp-package" \
cargo run --example 04_mcp_integration
```

## References

- [OpenAI Chat Completions API](https://platform.openai.com/docs/api-reference/chat)
- [Model Context Protocol (MCP)](https://modelcontextprotocol.io/)
- [OpenAI Function Calling Guide](https://platform.openai.com/docs/guides/function-calling)
- [Server-Sent Events (SSE)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events)

---

**Status**: ✅ Complete and ready to use!  
**License**: MIT (educational project)  
**Author**: Praxis Learning Journey
