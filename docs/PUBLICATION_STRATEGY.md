# 📦 Estratégia de Publicação do Praxis

## 🎯 Resumo Executivo

**Status**: ✅ Framework completo e pronto para publicação

**O que é publicado**: Meta-crate `praxis` que re-exporta todas as crates individuais

**O que NÃO é publicado**: `praxis-api` (é um exemplo de implementação, não parte do framework)

---

## 🏗️ Arquitetura de Publicação

### Modelo Atual vs. Modelo de Publicação

#### ❌ ANTES (Confusão Arquitetural)

```
Usuario pensa: "Preciso instalar praxis-api para ter um agente"
                ↓
              ERRADO!
```

`praxis-api` implementa UMA forma de usar o framework (REST API).
Não é o framework em si.

#### ✅ DEPOIS (Arquitetura Correta)

```
┌─────────────────────────────────────────┐
│  Usuário instala:                       │
│  cargo add praxis                       │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  praxis (meta-crate)                    │
│  - Re-exporta todas as crates           │
│  - Fornece AgentBuilder                 │
│  - Documentação unificada               │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  Crates Individuais (publicadas):       │
│  - praxis-types                         │
│  - praxis-graph                         │
│  - praxis-llm                           │
│  - praxis-mcp                           │
│  - praxis-persist                       │
└─────────────────────────────────────────┘
```

**praxis-api permanece como EXEMPLO no repositório**

---

## 📦 O Que Cada Crate Faz

### 1. **praxis-types** (Core Types)

**O que faz**: Define tipos fundamentais usados por todas as outras crates

**Exports principais**:
- `StreamEvent` - Eventos de streaming (reasoning, message, tool_call, etc)
- `GraphState` - Estado compartilhado durante execução
- `GraphConfig` - Configuração do graph
- `LLMConfig` - Configuração do LLM

**Dependências**: Mínimas (serde, tokio)

**Publicar**: ✅ Sim (base de tudo)

---

### 2. **praxis-llm** (LLM Clients)

**O que faz**: Clientes para APIs de LLM (OpenAI, com streaming)

**Exports principais**:
- `LLMClient` trait
- `OpenAIClient` implementação
- `Message`, `Content` tipos para conversas
- `ChatRequest`, `ChatResponse`

**Dependências**: praxis-types, reqwest, async-stream

**Publicar**: ✅ Sim (core do framework)

---

### 3. **praxis-mcp** (MCP Integration)

**O que faz**: Integração com Model Context Protocol para ferramentas externas

**Exports principais**:
- `MCPClient` - Cliente para um servidor MCP
- `MCPToolExecutor` - Gerencia múltiplos servidores MCP
- Integração com rmcp SDK

**Dependências**: praxis-types

**Publicar**: ✅ Sim (features essenciais)

---

### 4. **praxis-graph** (Execution Runtime)

**O que faz**: Runtime de execução (orquestração de Nodes, Router, loop de execução)

**Exports principais**:
- `Graph` - Orquestrador principal
- `Node` trait - Unidade básica de computação
- `LLMNode`, `ToolNode` - Implementações
- `Router` - Lógica de roteamento

**Dependências**: praxis-types, praxis-llm, praxis-mcp

**Publicar**: ✅ Sim (core do runtime)

---

### 5. **praxis-persist** (MongoDB Persistence)

**O que faz**: Persistência em MongoDB com context management e auto-summarização

**Exports principais**:
- `PersistClient` - Cliente principal (via builder)
- `ThreadRepository`, `MessageRepository`
- `ContextManager` - Gerenciamento de janela de contexto com sumarização

**Dependências**: praxis-types, praxis-llm, mongodb, bson

**Publicar**: ✅ Sim (necessário para conversas longas)

---

### 6. **praxis** (Meta-crate) 🆕

**O que faz**: Re-exporta tudo e fornece API de alto nível

**Exports principais**:
- `AgentBuilder` - Builder de alto nível
- `Agent` - Agente configurado
- `prelude` - Imports convenientes
- Re-exports de todas as crates

**Dependências**: Todas as crates acima

**Publicar**: ✅ Sim (PONTO DE ENTRADA PRINCIPAL)

---

### 7. **praxis-api** (REST API Example) ❌

**O que faz**: Implementação de referência de uma API REST com SSE streaming

**Características**:
- Endpoints HTTP (POST /v1/chat, GET /v1/health, etc)
- Server-Sent Events para streaming
- Middleware (CORS, logging, timeout)
- Configuração via TOML

**Por que NÃO publicar?**:
- É uma IMPLEMENTAÇÃO específica, não framework
- Usuários diferentes terão necessidades diferentes
- Serve como exemplo/template no repositório

**Alternativa**: Usuário cria sua própria API usando `praxis`

---

## 🎨 Como o Usuário Final Vai Usar

### Cenário 1: Aplicação Simples (CLI, script, etc)

```toml
[dependencies]
praxis = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

```rust
use praxis::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = AgentBuilder::new()
        .mongodb("mongodb://localhost:27017", "praxis")
        .openai_key(std::env::var("OPENAI_API_KEY")?)
        .build()
        .await?;
    
    let response = agent.chat("Hello!").await?;
    println!("{}", response);
    
    Ok(())
}
```

**3 linhas de código** para ter um agente funcional! ✅

---

### Cenário 2: API REST Customizada

Usuário cria seu próprio servidor HTTP:

```toml
[dependencies]
praxis = "0.1.0"
axum = "0.7"
tokio = { version = "1", features = ["full"] }
```

```rust
use praxis::prelude::*;
use axum::{Router, routing::post};

#[tokio::main]
async fn main() -> Result<()> {
    // Criar agente
    let agent = AgentBuilder::new()
        .mongodb("mongodb://localhost:27017", "praxis")
        .openai_key(std::env::var("OPENAI_API_KEY")?)
        .build()
        .await?;
    
    // Criar API customizada
    let app = Router::new()
        .route("/chat", post(handle_chat))
        .with_state(agent);
    
    // Rodar servidor
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn handle_chat(/* ... */) {
    // Implementação customizada
}
```

**Referência**: Olhar código de `praxis-api` no repositório para inspiração

---

### Cenário 3: Uso Avançado (Controle Total)

Usuário pode usar as crates individuais diretamente:

```toml
[dependencies]
praxis-graph = "0.1.0"
praxis-llm = "0.1.0"
praxis-mcp = "0.1.0"
# Não precisa de praxis-persist se não quiser MongoDB
```

```rust
use praxis_graph::{Graph, GraphConfig};
use praxis_llm::OpenAIClient;
use praxis_mcp::MCPToolExecutor;

// Construção manual para controle total
let llm_client = Arc::new(OpenAIClient::new(api_key, model));
let mcp_executor = Arc::new(MCPToolExecutor::new());
let graph = Graph::new(llm_client, mcp_executor, GraphConfig::default());

// Usar Graph diretamente
let event_rx = graph.spawn_run(input);
```

---

## 🚀 Benefícios da Meta-Crate `praxis`

### ✅ Para o Usuário

1. **Simplicidade**: `cargo add praxis` → tudo instalado
2. **Descoberta**: Tudo no `prelude`, fácil de achar
3. **Documentação**: docs.rs unificada
4. **Versionamento**: Uma versão para tudo
5. **Exemplos**: Fáceis de copiar/colar

### ✅ Para o Desenvolvedor (você)

1. **Separação clara**: Framework vs Implementação
2. **Reutilização**: Usuários criam suas próprias implementações
3. **Manutenção**: Atualizar crates individuais independentemente
4. **Flexibilidade**: Usuários podem escolher quais crates usar

---

## 📊 Comparação com Outros Frameworks

### LangGraph (Python)

```python
from langgraph import Agent

agent = Agent.from_config("config.yaml")
response = agent.chat("Hello")
```

### Praxis (Rust)

```rust
use praxis::prelude::*;

let agent = AgentBuilder::new()
    .mongodb("...", "db")
    .openai_key("...")
    .build()
    .await?;

let response = agent.chat("Hello").await?;
```

**Mesma simplicidade, mas com:**
- ⚡ Performance de Rust
- 🔒 Type safety
- 🚀 Async/await nativo
- 📊 Escalabilidade horizontal

---

## 🎯 Mensagem Final

### Para Publicação no crates.io:

```bash
# Publicar (nesta ordem)
cargo publish --package praxis-types
cargo publish --package praxis-llm
cargo publish --package praxis-mcp
cargo publish --package praxis-graph
cargo publish --package praxis-persist
cargo publish --package praxis  # Meta-crate

# praxis-api NÃO é publicado (é exemplo)
```

### Para o README do Repositório:

```markdown
# Praxis - AI Agent Framework for Rust

## Installation

```bash
cargo add praxis
```

## Quick Start

[... exemplo de 10 linhas ...]

## Examples

- **Simple CLI**: See `examples/simple_cli.rs`
- **REST API**: See `praxis-api/` directory (reference implementation)
- **Advanced**: See `examples/advanced/`
```

### Para os Usuários:

> "Instale `praxis`, configure MongoDB e OpenAI, e você tem um agente de IA funcional em 3 linhas de código. Para uma API REST completa, veja o código de `praxis-api` como referência."

---

## ✅ Conclusão

**O framework está COMPLETO e PRONTO para publicação!**

- ✅ Todas as peças implementadas
- ✅ Arquitetura clara (framework vs implementação)
- ✅ API de alto nível (AgentBuilder)
- ✅ Documentação completa
- ✅ Exemplos funcionando

**Próximo passo**: Preparar metadados e publicar no crates.io 🚀
