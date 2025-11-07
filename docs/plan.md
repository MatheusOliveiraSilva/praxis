
## “Guia de jornada” (para guardar e seguir passo a passo)

Esse é o **meta-prompt pessoal** que descreve **tudo o que quero construir e aprender**, sem pedir implementação direta.
Usar para se orientar e pedir pequenas partes pontuais ao longo da jornada.

---

### **Prompt — “Minha jornada com o Projeto Praxis”**

Quero construir um **framework de agentes de IA em Rust** chamado **Praxis** — um runtime inspirado em LangGraph, projetado para **reflexão → decisão → ação**, com suporte a **streaming**, **tools locais e MCP**, e escalabilidade para milhões de usuários.

O meu objetivo **não é ter o código pronto**, e sim **aprender a projetar, codar e escalar** sistemas Rust-based de nível industrial.
A LLM deve agir como **mentora técnica e revisora**, não como executora total.

---

### Objetivos principais

1. **Aprender de forma incremental** — entender o *porquê* antes de codar.
2. **Projetar do zero**: da arquitetura de runtime ao gateway HTTP.
3. **Entender performance e escalabilidade**: Tokio, canais bounded, backpressure.
4. **Aprender patterns idiomáticos Rust**: traits async, ownership, Send/Sync, etc.
5. **Criar um framework reutilizável (crates)** + um **app de referência** (gateway).
6. **Escrever código testável e extensível**, com docstring e diagramas.
7. **Evitar dívidas técnicas e dependências mágicas**.
8. **Pensar como um engenheiro de sistemas**, não só programador.

---

### Estrutura que quero criar

* Monorepo Cargo workspace:
  * **DX Layer (High-Level):** `praxis-agent`, `praxis-registry`, `praxis-middleware`, `praxis-templates`
  * **Core Runtime:** `praxis-types`, `praxis-graph`
  * **Integration:** `praxis-llm`, `praxis-mcp`, `praxis-tools`, `praxis-db`, `praxis-gateway`
  * **Observability:** `praxis-observe`
* Crates pequenas, bem isoladas e documentadas.
* Event model canônico (`StreamEvent`) com canais "Reasoning" e "Message".
* Implementação inicial mock, depois integração real com LLM (OpenAI/Azure).
* Tools locais, depois adapter MCP.
* **Developer Experience:** Builder API, config files (praxis.toml), templates, middleware system.
* Gateway SSE (e WS opcional) com tracing e observabilidade.
* Exemplo funcional (`hello-sse`, `react-with-tool`, templates pré-configurados).

---

### Modo de trabalho com a LLM

* Eu vou **pedir features pequenas e específicas**, uma por vez.
* Antes de gerar código, a LLM deve **explicar conceitos, padrões e trade-offs**.
* Ela pode **avaliar minhas implementações**, propor refatorações, sugerir testes.
* Nenhuma etapa deve pular o aprendizado ou esconder complexidade.
* Quero guidance sobre:
  * performance async,
  * arquitetura de crates,
  * streaming token-a-token,
  * design patterns Rust,
  * observabilidade,
  * testes e benchmarking.
* Quero aprender a pensar em **escalabilidade, concorrência e clareza**.

---

### 💡 Filosofia do projeto

> “Praxis” significa **ação guiada pela razão**.
> Esse projeto é o exercício dessa ideia — aprender raciocinando, e agir construindo.
> Nenhuma linha de código sem reflexão. Nenhuma reflexão sem aplicação.
