# Praxis AI Agent - Next.js UI

Interface moderna construída com Next.js, TypeScript e Tailwind CSS.

## 🚀 Stack

- **Next.js 14** (App Router)
- **TypeScript**
- **Tailwind CSS**
- **Server-Sent Events (SSE)** para streaming

## 📦 Instalação

```bash
cd agent_ui
npm install
```

## 🏃 Como Rodar

### Desenvolvimento
```bash
npm run dev
```

Acesse: http://localhost:3000

### Produção
```bash
npm run build
npm start
```

## 🛠️ Pré-requisitos

Certifique-se que estão rodando:

1. **Praxis API** (porta 8000)
```bash
cd crates/praxis-api
cargo run --release --bin praxis-api
```

2. **MCP Weather Server** (porta 8005)
```bash
cd mcp_servers/weather
uv run python weather.py
```

3. **MongoDB** (porta 27017)
```bash
cd praxis_example
./scripts/setup-mongo.sh
```

## 🎨 Features

- ✅ **Real-time Streaming** via SSE
- ✅ **Tool Call Visualization** com status
- ✅ **Sidebar com histórico** de conversas
- ✅ **CRUD completo** de threads
- ✅ **Typing indicators**
- ✅ **Design moderno** estilo Cursor
- ✅ **Totalmente tipado** com TypeScript

## 📁 Estrutura

```
agent_ui/
├── app/
│   ├── layout.tsx         # Layout root
│   ├── page.tsx           # Página principal
│   └── globals.css        # Estilos globais
├── components/
│   ├── Sidebar.tsx        # Lista de threads
│   ├── ChatArea.tsx       # Área de mensagens
│   ├── MessageBubble.tsx  # Mensagem individual
│   ├── ToolCallCard.tsx   # Card de tool call
│   ├── TypingIndicator.tsx# Indicador de digitação
│   └── icons/             # Ícones SVG
├── types/
│   └── index.ts           # Tipos TypeScript
└── package.json
```

## 🎯 Como Usar

1. Abra http://localhost:3000
2. Clique no **+** para criar uma nova conversa
3. Digite sua mensagem e veja o streaming em tempo real
4. Tool calls aparecem com status e resultados

## 🐛 Troubleshooting

### Porta 3000 ocupada
```bash
# Use outra porta
npm run dev -- -p 3001
```

### Erro de CORS
Verifique se o CORS está configurado no `crates/praxis-api/config/default.toml`:
```toml
[cors]
enabled = true
origins = ["http://localhost:3000", "http://127.0.0.1:3000"]
```

### API não responde
```bash
# Verifique se a API está rodando
curl http://localhost:8000/health
```

## 🔧 Configuração

O frontend se conecta com:
- **API Base**: `http://localhost:8000`
- **User ID**: `test_user` (hardcoded para desenvolvimento)

Para mudar, edite as constantes em `app/page.tsx` e `components/ChatArea.tsx`.
