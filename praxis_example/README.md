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
```

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

## Next Steps

This example serves as the foundation for:
1. **praxis-api**: REST API with SSE streaming
2. **praxis-dx**: CLI tools and configuration management
3. Full agent implementation with persistence

## License

MIT

