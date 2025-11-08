# Praxis Agent UI

> ⚠️ **Disclaimer**: This is a **basic testing frontend** only. No responsibility is taken for visual bugs, UI inconsistencies, or production readiness. Use for development and testing purposes only.

Modern web interface for the Praxis AI Agent built with Next.js, TypeScript, and Tailwind CSS.

## 🎯 Purpose

This UI serves as a **minimal interface for testing and development** of the Praxis backend API. It demonstrates:
- Real-time message streaming via SSE
- Tool execution visualization
- Conversation management
- Event-driven architecture

## 🚀 Tech Stack

- **Next.js 14** (App Router)
- **TypeScript** (Strict mode)
- **Tailwind CSS** (Utility-first styling)
- **Server-Sent Events** (Real-time streaming)

## 📦 Installation

```bash
npm install
```

## 🏃 Quick Start

### Development
```bash
npm run dev
```
Opens on http://localhost:3000

### Production Build
```bash
npm run build
npm start
```

## 🛠️ Prerequisites

Before running the UI, ensure these services are running:

1. **Praxis API** (port 8000)
   ```bash
   cd ../crates/praxis-api
   cargo run --release --bin praxis-api
   ```

2. **MongoDB** (port 27017)
   ```bash
   cd ../praxis_example
   ./scripts/setup-mongo.sh
   ```

3. **MCP Weather Server** (port 8005)
   ```bash
   cd ../mcp_servers/weather
   PORT=8005 uv run python weather.py
   ```

## 🏗️ Architecture

### Design Patterns

- **Strategy Pattern**: Event handlers for different SSE event types
- **Singleton Pattern**: Handler registry for O(1) dispatch
- **Observer Pattern**: React hooks for state management
- **Memoization**: Optimized component re-renders

### Project Structure

```
agent_ui/
├── app/                      # Next.js App Router
│   ├── layout.tsx           # Root layout
│   ├── page.tsx             # Home page
│   └── globals.css          # Global styles
├── components/              # React components
│   ├── messages/            # Message renderers
│   │   ├── UserMessage.tsx
│   │   ├── AssistantMessage.tsx
│   │   ├── ReasoningMessage.tsx
│   │   └── MessageRenderer.tsx
│   ├── tools/               # Tool execution display
│   │   └── ToolCallRenderer.tsx
│   ├── ChatArea.tsx         # Main chat interface
│   └── Sidebar.tsx          # Thread list
├── hooks/                   # Custom React hooks
│   └── useChat.ts           # Chat state management
├── lib/                     # Business logic
│   ├── handlers/            # SSE event handlers
│   │   ├── base-handler.ts
│   │   ├── message-handler.ts
│   │   ├── reasoning-handler.ts
│   │   ├── tool-call-handler.ts
│   │   ├── tool-result-handler.ts
│   │   └── done-handler.ts
│   └── stream-processor.ts  # SSE parsing
└── types/                   # TypeScript types
    ├── index.ts             # Core types
    └── events.ts            # SSE event types
```

## 🎨 Features

### ✅ Implemented
- Real-time SSE streaming
- Message accumulation by type
- Tool call visualization with status
- Thread CRUD operations
- Auto-create thread on first message
- Chronological message ordering
- Typing indicators
- Error handling

### ⚠️ Known Limitations
- No markdown rendering
- No code syntax highlighting
- No file upload support
- No authentication
- Basic responsive design only
- Minimal accessibility features
- No offline support
- Limited error recovery

## 🔧 Configuration

The UI connects to:
- **API Base URL**: `http://localhost:8000`
- **User ID**: `test_user` (hardcoded for testing)

To change these, edit constants in:
- `hooks/useChat.ts`
- `components/ChatArea.tsx`

## 📡 API Integration

### Endpoints Used
```
GET    /threads                      # List threads
POST   /threads                      # Create thread
GET    /threads/:id/messages         # Load messages
POST   /threads/:id/messages         # Send message (SSE)
DELETE /threads/:id                  # Delete thread
```

### SSE Events Handled
```
event: info          # Stream start
event: message       # Assistant message chunks
event: reasoning     # Reasoning chunks (o1/o3 models)
event: tool_call     # Tool execution request
event: tool_result   # Tool execution result
event: done          # Stream completion
event: error         # Error during processing
```

## 🐛 Debugging

### Check Services
```bash
# API health
curl http://localhost:8000/health

# MongoDB
mongosh mongodb://admin:password123@localhost:27017

# MCP Server
curl http://localhost:8005/mcp
```

### Common Issues

**Port 3000 already in use**
```bash
lsof -ti:3000 | xargs kill -9
```

**CORS errors**
Check `crates/praxis-api/config/default.toml`:
```toml
[cors]
enabled = true
origins = ["http://localhost:3000"]
```

**SSE not streaming**
- Ensure `Accept: text/event-stream` header is sent
- Check browser Network tab for stream status
- Verify API is not buffering responses

## 🚧 Development Notes

This is a **prototype interface** focused on functionality over polish. Expect:
- Basic styling
- Minimal animations
- Limited edge case handling
- No comprehensive testing
- Potential performance issues with long conversations

## 📝 Future Improvements (Not Implemented)

- [ ] Markdown rendering
- [ ] Syntax highlighting
- [ ] File attachments
- [ ] Voice input
- [ ] Better mobile support
- [ ] Accessibility (ARIA labels, keyboard nav)
- [ ] Internationalization
- [ ] Theme customization
- [ ] Export conversations
- [ ] Search functionality

## 🤝 Contributing

This is a development/testing tool. For production use cases, consider building a proper frontend with:
- Comprehensive error handling
- Full accessibility support
- Performance optimizations
- Security hardening
- Comprehensive testing

## 📄 License

Same as parent project (Praxis).
