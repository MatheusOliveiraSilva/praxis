<!-- c4aa9d43-6aba-4941-a6ee-cd42cd37186b 6cef2943-0aec-4069-a7cf-2bdad1d254c6 -->
# Micropasso 1: Exploração Conceitual das Abstrações Core

## Objetivo

Antes de escrever código, vamos **entender e desenhar** as 3 abstrações fundamentais do Praxis:

1. **Node** — unidade de execução (reflexão, decisão, ação)
2. **Graph** — orquestrador de fluxo entre Nodes
3. **StreamEvent** — modelo de evento para comunicação assíncrona

## Por que começar assim?

- Você está retomando Rust após 2 meses, então começar com conceitos vai refrescar a memória
- O projeto usa async/concurrency (Tokio), que você ainda não estudou — precisamos construir intuição primeiro
- Decisões de arquitetura agora evitam refatorações grandes depois
- Alinha com sua filosofia: "entender o porquê antes de codar"

## Etapas

### 1. Discussão: O que é um "Node"?

Vamos explorar:

- O que um Node precisa fazer? (entrada → processamento → saída)
- Como representar "reflexão → decisão → ação" em uma trait?
- Que dados um Node precisa para executar? (contexto, estado, configuração)
- Como um Node se comunica com o exterior? (canais, callbacks, eventos)

**Resultado esperado**: Um "contrato informal" do que um Node deve fazer, sem sintaxe Rust ainda.

### 2. Discussão: O que é um "Graph"?

Vamos explorar:

- Como conectar múltiplos Nodes? (grafo direcionado, edges condicionais)
- Como orquestrar execução? (sequencial vs paralelo, loops, condicionais)
- Como gerenciar estado compartilhado entre Nodes?
- Como o Graph emite eventos para o mundo externo?

**Resultado esperado**: Um desenho conceitual de como Nodes se conectam e executam.

### 3. Discussão: O que é um "StreamEvent"?

Vamos explorar:

- Que tipos de eventos o sistema deve emitir? (reasoning chunks, message chunks, tool calls, erros)
- Como estruturar eventos para facilitar consumo (SSE, WebSockets)?
- Como diferenciar canais "internos" (reasoning) de "externos" (message)?
- Como representar eventos de forma type-safe?

**Resultado esperado**: Um esquema informal dos tipos de eventos e seus campos.

### 4. Criar documento de design

Consolidar as discussões em `docs/architecture.md`:

- Definições das abstrações
- Diagramas ASCII de fluxo de execução
- Exemplos de uso (pseudocódigo)
- Decisões arquiteturais e trade-offs

### 5. Introdução gentil a async/concurrency

Antes do próximo micropasso (implementação), você precisará estudar:

- Capítulo 16 do Rust Book (Concurrency)
- Conceitos básicos de async/await
- Por que usar Tokio (vs threads tradicionais)

Vou recomendar materiais específicos e responder dúvidas.

## O que NÃO faremos neste micropasso

- Criar Cargo workspace ou código Rust
- Implementar traits ou structs
- Instalar dependências

## Próximo micropasso (futuro)

Após esta exploração, você terá clareza para:

- Criar o workspace e estrutura de crates
- Implementar uma versão mock/síncrona das abstrações
- Estudar async e migrar para Tokio

## Tempo estimado

2-3 sessões de trabalho (discussão + documentação)

### To-dos

- [ ] Explorar e definir o que é um Node (entrada, processamento, saída)
- [ ] Explorar e definir como Graph conecta e orquestra Nodes
- [ ] Explorar e definir modelo de StreamEvent para comunicação assíncrona
- [ ] Consolidar discussões em docs/architecture.md com diagramas e exemplos
- [ ] Criar roteiro de estudo para async/concurrency (Cap 16 + Tokio basics)