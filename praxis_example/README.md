# Praxis Example

This is a reference implementation demonstrating how to use the Praxis framework.

## Prerequisites

- Docker (for MongoDB)
- Rust 1.75+

## Quick Start

### 1. Start MongoDB

```bash
cd praxis_example
./scripts/setup-mongo.sh
```

This script will:
- Start MongoDB in a Docker container (if not already running)
- Wait for MongoDB to be ready
- Create necessary indexes
- Print the connection string

### 2. Test MongoDB Connection

```bash
# From project root
cargo run --bin test-mongo
```

This will:
- Connect to MongoDB
- Create a test thread
- Insert test messages
- Query data to verify indexes
- Test pagination and counting
- Test context manager
- Print success confirmation

### 3. Stop MongoDB

```bash
cd praxis_example
./scripts/stop-mongo.sh
```

Or to also remove data:

```bash
./scripts/stop-mongo.sh --clean
```

## MongoDB Connection

The MongoDB instance runs on:
- **Host**: localhost
- **Port**: 27017
- **Username**: admin
- **Password**: password123
- **Database**: praxis

**Connection URI**: `mongodb://admin:password123@localhost:27017`

## Environment Variables

Copy `.env.example` to `.env` and configure:

```bash
MONGODB_URI=mongodb://admin:password123@localhost:27017
MONGODB_DATABASE=praxis
OPENAI_API_KEY=your_api_key_here  # Required for LLM summarization
```

**Note**: The OpenAI API key is required for the automatic summarization feature. When the conversation exceeds the configured token limit (default: 30,000 tokens), the system will automatically generate a summary of previous messages using GPT-4o-mini.

## Collections

### threads
Stores conversation threads with metadata and optional summaries.

**Indexes**:
- `{ user_id: 1, created_at: -1 }`
- `{ _id: 1, user_id: 1 }`

### messages
Stores all messages in threads (user, assistant, reasoning, tool calls, tool results).

**Indexes**:
- `{ user_id: 1, thread_id: 1, created_at: 1 }`
- `{ thread_id: 1, created_at: 1 }`
- `{ user_id: 1, created_at: -1 }`

## Testing

Run the validation binary to ensure everything is working:

```bash
cargo run --bin test-mongo
```

Expected output:
```
MongoDB Connection Test
=======================
✓ Connected to mongodb://localhost:27017
✓ Database: praxis
✓ Created thread: 507f1f77bcf86cd799439011
✓ Inserted 5 messages
✓ Query by user_id: found 1 thread
✓ Query by thread_id: found 5 messages
✓ Pagination test: passed
✓ Count test: 5 messages
✓ Context window: 5 messages
✓ System prompt length: 44 chars

All tests passed!
```

## Inspecting Data

### Using mongosh CLI

```bash
# Connect to MongoDB
docker exec -it praxis-mongo mongosh -u admin -p password123 praxis

# List collections
show collections

# Query threads
db.threads.find()

# Query messages
db.messages.find()

# Check indexes
db.threads.getIndexes()
db.messages.getIndexes()
```

### Using MongoDB Compass (GUI)

1. Download from https://www.mongodb.com/products/compass
2. Connect using: `mongodb://admin:password123@localhost:27017`
3. Navigate to `praxis` database

## Troubleshooting

### MongoDB won't start

Check if another process is using port 27017:
```bash
lsof -i :27017
```

### Container already exists

Remove the old container:
```bash
docker rm -f praxis-mongo
./scripts/setup-mongo.sh
```

### Network issues

Make sure Docker is running:
```bash
docker info
```

## Usage Example

### Basic Usage with Builder Pattern

```rust
use praxis_persist::PersistClient;
use praxis_llm::OpenAIClient;
use std::sync::Arc;

// Create LLM client for summarization
let llm_client = Arc::new(OpenAIClient::new(api_key)?);

// Build PersistClient with custom configuration
let client = PersistClient::builder()
    .mongodb_uri("mongodb://admin:password123@localhost:27017")
    .database("praxis")
    .max_tokens(30_000)  // Token limit before summarization
    .llm_client(llm_client)
    .build()
    .await?;

// Create a thread
let thread = client.threads()
    .create_thread("user_123".to_string(), metadata)
    .await?;

// Save messages
client.messages()
    .save_message(message)
    .await?;

// Get context window (with automatic summarization if needed)
let (messages, system_prompt) = client.context()
    .get_context_window(thread.id)
    .await?;
```

### Customizing System Prompt Template

```rust
// Using custom template string
let client = PersistClient::builder()
    .mongodb_uri(uri)
    .database("praxis")
    .llm_client(llm_client)
    .system_prompt_template("Custom prompt with <summary> placeholder")
    .build()
    .await?;

// Or loading from file
let client = PersistClient::builder()
    .mongodb_uri(uri)
    .database("praxis")
    .llm_client(llm_client)
    .system_prompt_template_file("prompts/system.txt")?
    .build()
    .await?;
```

### How Async Summarization Works

1. When `get_context_window()` is called, the system checks token count
2. If tokens exceed `max_tokens`, an async task is spawned to generate a summary
3. The current request returns immediately with existing data
4. Summary generation happens in the background using GPT-4o-mini
5. The summary is saved to MongoDB with a timestamp (`last_summary_update`)
6. Future calls use the summary and only load messages after `last_summary_update`
7. If the window grows again, the process repeats (incremental summarization)

This ensures:
- **No blocking**: Users never wait for summarization
- **Scalability**: Conversations can be arbitrarily long
- **Cost efficiency**: Only new messages are summarized each time
- **Context preservation**: Summaries include previous summary content

## Next Steps

This example serves as the foundation for:
1. **praxis-api**: REST API with SSE streaming
2. **praxis-dx**: CLI tools and configuration management
3. Full agent implementation with persistence

## Key Features

### Builder Pattern API
- Fluent, type-safe configuration
- Required fields enforced at compile time
- Optional fields with sensible defaults

### Async Background Summarization
- Non-blocking: context retrieval never waits for summarization
- Incremental: only new messages are summarized
- Configurable token limits

### Template System
- Embedded default templates
- Customizable via string or file
- `<summary>` placeholder replacement

## License

MIT

