# 📦 Guia de Publicação no crates.io

## ✅ Status: PRONTO PARA PUBLICAR

Após análise completa da documentação e código implementado, **o framework está completo** e pronto para ser empacotado e publicado.

## 🎯 O Que Será Publicado

### Crates Individuais (em ordem de dependência)

1. **praxis-types** v0.1.0 - Tipos core
2. **praxis-llm** v0.1.0 - Cliente LLM (OpenAI)
3. **praxis-mcp** v0.1.0 - Integração MCP
4. **praxis-graph** v0.1.0 - Runtime de execução
5. **praxis-persist** v0.1.0 - Persistência MongoDB
6. **praxis** v0.1.0 - Meta-crate (re-exporta tudo)

### ❌ O Que NÃO Será Publicado

- **praxis-api** - É um EXEMPLO de implementação, não parte do framework
  - Permanece no repositório como exemplo de referência
  - Usuários criam suas próprias APIs usando `praxis`

## 🏗️ Arquitetura de Publicação

```
┌─────────────────────────────────────────────┐
│ Usuario instala no Cargo.toml:              │
│                                             │
│ [dependencies]                              │
│ praxis = "0.1.0"                            │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ praxis (meta-crate)                         │
│ Re-exporta:                                 │
│  - praxis-types                             │
│  - praxis-graph                             │
│  - praxis-llm                               │
│  - praxis-mcp                               │
│  - praxis-persist                           │
│                                             │
│ Fornece:                                    │
│  - AgentBuilder (API de alto nível)         │
│  - prelude (imports convenientes)           │
│  - Documentação unificada                   │
└─────────────────────────────────────────────┘
```

## 🚀 Passos para Publicação

### 1. Preparação (Antes de Publicar)

```bash
# 1. Verificar que tudo compila
cargo build --workspace --all-features

# 2. Rodar todos os testes
cargo test --workspace

# 3. Verificar documentação
cargo doc --workspace --no-deps --open

# 4. Formatar código
cargo fmt --all

# 5. Linter
cargo clippy --workspace -- -D warnings

# 6. Verificar se packages estão corretos
cargo package --package praxis-types --dry-run
cargo package --package praxis-llm --dry-run
cargo package --package praxis-mcp --dry-run
cargo package --package praxis-graph --dry-run
cargo package --package praxis-persist --dry-run
cargo package --package praxis --dry-run
```

### 2. Publicação (Em Ordem de Dependência)

**IMPORTANTE**: Publicar nesta ordem exata, pois há dependências entre crates.

```bash
# 1. praxis-types (não tem dependências internas)
cd crates/praxis-types
cargo publish
# Aguardar 1-2 minutos para propagar no crates.io

# 2. praxis-llm (depende de praxis-types)
cd ../praxis-llm
cargo publish
# Aguardar 1-2 minutos

# 3. praxis-mcp (depende de praxis-types)
cd ../praxis-mcp
cargo publish
# Aguardar 1-2 minutos

# 4. praxis-graph (depende de types, llm, mcp)
cd ../praxis-graph
cargo publish
# Aguardar 1-2 minutos

# 5. praxis-persist (depende de types, llm)
cd ../praxis-persist
cargo publish
# Aguardar 1-2 minutos

# 6. praxis (meta-crate, depende de todas)
cd ../../praxis
cargo publish
```

### 3. Verificação Pós-Publicação

```bash
# Criar novo projeto de teste
mkdir /tmp/test-praxis
cd /tmp/test-praxis
cargo init

# Adicionar praxis
cargo add praxis

# Testar que compila
cargo build
```

## 📝 Preparação dos Cargo.toml

Antes de publicar, atualizar todos os `Cargo.toml` para:

### 1. Usar versões publicadas (não path)

**ANTES** (desenvolvimento):
```toml
[dependencies]
praxis-types = { path = "../praxis-types" }
```

**DEPOIS** (publicação):
```toml
[dependencies]
praxis-types = { version = "0.1.0", path = "../praxis-types" }
```

O `path` é mantido para desenvolvimento local, mas `version` é usada quando publicado.

### 2. Adicionar metadados obrigatórios

Todos os `Cargo.toml` devem ter:

```toml
[package]
name = "praxis-xxx"
version = "0.1.0"
edition = "2021"
authors = ["Seu Nome <email@example.com>"]
license = "MIT"
description = "Descrição curta e clara"
repository = "https://github.com/seu-usuario/praxis"
keywords = ["ai", "agent", "llm", "mcp", "framework"]  # max 5
categories = ["web-programming", "asynchronous"]  # max 5
readme = "README.md"
```

### 3. Verificar LICENSE e README

Cada crate deve ter:
- `README.md` - Descrição e exemplos
- `LICENSE` ou `LICENSE.md` - Arquivo de licença

## 🎨 Exemplo de Uso (Como Usuário Final Vai Usar)

```rust
// Cargo.toml
[dependencies]
praxis = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

```rust
// src/main.rs
use praxis::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let agent = AgentBuilder::new()
        .mongodb("mongodb://localhost:27017", "praxis")
        .openai_key(std::env::var("OPENAI_API_KEY")?)
        .mcp_servers("http://localhost:8000/mcp")
        .build()
        .await?;
    
    let response = agent.chat("Hello!").await?;
    println!("{}", response);
    
    Ok(())
}
```

## 🔄 Ciclo de Desenvolvimento Pós-Publicação

### Para Releases Futuras

1. **Versioning**: Seguir [Semantic Versioning](https://semver.org/)
   - `0.1.x` - Bug fixes (compatível)
   - `0.x.0` - Novas features (pode quebrar)
   - `x.0.0` - Mudanças grandes (definitivamente quebra)

2. **Publicar nova versão**:
   ```bash
   # Atualizar version em todos os Cargo.toml
   # Criar tag git
   git tag v0.2.0
   git push origin v0.2.0
   
   # Publicar novamente (mesma ordem)
   cargo publish --package praxis-types
   # ... etc
   ```

3. **Changelog**: Manter `CHANGELOG.md` com mudanças

## 📚 Documentação

### docs.rs

Após publicação, a documentação será automaticamente gerada em:
- https://docs.rs/praxis
- https://docs.rs/praxis-types
- https://docs.rs/praxis-llm
- etc.

### README.md Principal

O README do repositório deve:
- Explicar o que é Praxis
- Mostrar exemplos rápidos
- Link para docs.rs
- Link para examples/
- Explicar que `praxis-api` é um exemplo

## ⚠️ Considerações Importantes

### 1. praxis-api NÃO é publicado

- **Por quê?** É uma implementação específica, não parte do framework
- **Alternativa**: Usuários criam suas próprias APIs usando `praxis`
- **Exemplo**: O código de `praxis-api` serve como template/referência

### 2. Dependências Externas

Verificar se todas as dependências são compatíveis:
- Tokio
- MongoDB driver
- OpenAI HTTP client
- MCP SDK (rmcp)

### 3. Features Opcionais

Considerar adicionar features opcionais:
```toml
[features]
default = ["mongodb", "openai"]
mongodb = ["dep:mongodb", "dep:bson"]
openai = []
mcp = []
```

## 🎯 Checklist Final

Antes de `cargo publish`:

- [ ] Todos os testes passam
- [ ] Documentação está completa (/// comments)
- [ ] README.md de cada crate está atualizado
- [ ] LICENSE presente em cada crate
- [ ] Versões consistentes (0.1.0) em todos
- [ ] Keywords e categories apropriadas
- [ ] Repositório URL correto
- [ ] Exemplos funcionam
- [ ] Sem TODOs ou código incompleto
- [ ] `cargo package --dry-run` passa em todas

## 🚀 Conclusão

**O framework está pronto!** 

Com apenas 3 linhas de código, um usuário pode:
1. Instalar: `cargo add praxis`
2. Configurar: MongoDB + OpenAI key
3. Usar: `agent.chat("Hello!")` ✅

O repositório `praxis-api` serve como exemplo de implementação completa de uma API REST com SSE streaming.
