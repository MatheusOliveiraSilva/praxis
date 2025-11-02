# Pesquisa: Modelos OpenAI - Reasoning e Streaming

## 📊 **Estado Atual dos Modelos OpenAI (Nov 2024 - Jan 2025)**

---

## 🤖 **1. Família GPT-4 (Geração Atual)**

### **GPT-4 Turbo / GPT-4o / GPT-4o-mini**

| Característica | Suporte | Detalhes |
|---------------|---------|----------|
| **Streaming** | ✅ Sim | Via `stream: true`, SSE |
| **Reasoning Explícito** | ❌ Não | Não expõe "pensamento interno" separado |
| **Tool Calling** | ✅ Sim | `tool_calls` na resposta |
| **Multimodal** | ✅ Sim (gpt-4o) | Visão, imagens, texto |
| **Structured Outputs** | ✅ Sim | JSON mode, schemas |
| **Max Output Tokens** | ~4096 | gpt-4o: 4096, gpt-4-turbo: 4096 |

### **Como o "Reasoning" Funciona**

GPT-4 **NÃO tem campo `reasoning_content`** separado. 

**O que acontece:**
1. Modelo gera tokens **diretamente** como resposta final
2. Não há "pensamento interno" exposto pela API
3. Se você quer ver raciocínio, precisa **pedir no prompt**:

```json
// Prompt engineering para "forçar" reasoning
{
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant. Always think step-by-step before answering. Show your work."
    },
    {
      "role": "user",
      "content": "What's 157 * 23?"
    }
  ]
}
```

**Resposta (streaming):**
```
data: {"choices":[{"delta":{"content":"Let"}}]}
data: {"choices":[{"delta":{"content":" me"}}]}
data: {"choices":[{"delta":{"content":" calculate"}}]}
data: {"choices":[{"delta":{"content":" step"}}]}
data: {"choices":[{"delta":{"content":" by"}}]}
data: {"choices":[{"delta":{"content":" step"}}]}
...
data: {"choices":[{"delta":{"content":"The answer is 3611"}}]}
```

**Problema:** Reasoning e resposta **misturados** no mesmo `content`. Você não consegue separar programaticamente.

---

## 🧠 **2. Família o1 (Reasoning Models)**

### **o1-preview / o1-mini**

Lançados em **Setembro 2024**, são modelos **projetados para reasoning**.

| Característica | Suporte | Detalhes |
|---------------|---------|----------|
| **Streaming** | ❌ **NÃO** | Resposta completa de uma vez |
| **Reasoning Explícito** | ✅ **SIM** | Campo `reasoning_content` separado |
| **Tool Calling** | ❌ Não (por enquanto) | Limitação atual |
| **Multimodal** | ❌ Não | Só texto |
| **Max Output Tokens** | ~16384 (o1-preview) | Bem maior que GPT-4 |
| **Max Reasoning Tokens** | ~32000 | Pode "pensar" muito mais |

### **Como Funciona**

#### **Request:**
```json
POST /v1/chat/completions
{
  "model": "o1-preview",
  "messages": [
    {
      "role": "user",
      "content": "Solve this complex math problem: ..."
    }
  ]
  // NÃO pode usar "stream": true
}
```

#### **Response (após ~10-30 segundos):**
```json
{
  "id": "chatcmpl-o1-abc123",
  "object": "chat.completion",
  "created": 1699999999,
  "model": "o1-preview-2024-09-12",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "The answer is 42.",
        "reasoning_content": "Let me think through this step by step:\n\n1. First, I need to understand what the problem is asking...\n2. Breaking it down into smaller parts...\n3. Analyzing each component...\n...\n15. Therefore, the final answer is 42."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 50,
    "completion_tokens": 12,
    "reasoning_tokens": 450,
    "total_tokens": 512
  }
}
```

**Campos Importantes:**
- ✅ `reasoning_content`: Pensamento interno do modelo (separado!)
- ✅ `content`: Resposta final ao usuário
- ✅ `reasoning_tokens`: Quantos tokens foram usados para "pensar"

### **Limitações Atuais do o1:**

1. ❌ **Sem streaming** - Você espera a resposta completa
2. ❌ **Sem tool calling** - Não pode chamar funções (ainda)
3. ❌ **Sem system messages** - Só aceita `user` e `assistant`
4. ❌ **Não aceita `temperature`** - Parâmetros limitados
5. ⚠️ **Custo alto** - `reasoning_tokens` são cobrados

**Pricing (exemplo):**
- Input: $15 per 1M tokens
- Output: $60 per 1M tokens
- **Reasoning tokens contam como output!**

---

## 🔮 **3. GPT-5 / Próxima Geração (Rumores e Expectativas)**

**Status:** Não oficialmente anunciado, mas há especulações.

### **O que Sabemos (Vazamentos e Rumores):**

1. **Codinome:** "Orion" (não confirmado)
2. **Timeline:** Possivelmente Q1-Q2 2025
3. **Treinamento:** Reportadamente em fase final

### **Expectativas Baseadas em Tendências:**

#### **Provável:**
- ✅ **Reasoning nativo com streaming** - Combinar o melhor de GPT-4o + o1
- ✅ **Multimodal avançado** - Vídeo, áudio nativo
- ✅ **Tool calling mais robusto** - Parallel execution, retry logic
- ✅ **Maior contexto** - 128k+ tokens
- ✅ **Structured outputs melhorados** - Schemas mais complexos

#### **Possível (Especulação):**
- 🤔 **Reasoning em duas fases:**
  - Stream 1: `reasoning` (pensamento interno, pode ser ocultado)
  - Stream 2: `message` (resposta final ao usuário)
- 🤔 **Controle de reasoning:**
  ```json
  {
    "reasoning_effort": "low" | "medium" | "high",
    "show_reasoning": true  // Expor ou ocultar do usuário
  }
  ```
- 🤔 **Tool calling com reasoning:**
  - Modelo explica POR QUE está chamando uma tool
  - `reasoning_content`: "I need to call calculator because..."
- 🤔 **Multi-step reasoning:**
  - Pode emitir múltiplos blocos de reasoning
  - Pode "revisar" seu raciocínio

### **Como Poderia Ser a API (Especulação):**

```json
// Request
POST /v1/chat/completions
{
  "model": "gpt-5",
  "messages": [...],
  "stream": true,
  "reasoning_effort": "high",
  "stream_reasoning": true  // Novo parâmetro?
}

// Response (streaming)
data: {"type":"reasoning_start"}

data: {"type":"reasoning","content":"Let me think..."}
data: {"type":"reasoning","content":" First, I should..."}

data: {"type":"reasoning_end"}

data: {"type":"message_start"}

data: {"type":"message","content":"The answer"}
data: {"type":"message","content":" is 42"}

data: {"type":"message_end"}

data: [DONE]
```

---

## 🎯 **4. Comparação: GPT-4 vs o1 vs GPT-5 (Projetado)**

| Feature | GPT-4o | o1-preview | GPT-5 (Esperado) |
|---------|--------|------------|------------------|
| **Streaming** | ✅ Sim | ❌ Não | ✅ Sim |
| **Reasoning Separado** | ❌ Não | ✅ Sim | ✅ Sim |
| **Reasoning Streaming** | ❌ Não | ❌ Não | 🤔 Possível |
| **Tool Calling** | ✅ Sim | ❌ Não | ✅ Sim (melhorado) |
| **Multimodal** | ✅ Sim | ❌ Não | ✅ Sim (avançado) |
| **Max Reasoning Tokens** | N/A | ~32k | 🤔 50k+? |
| **Latência (primeira resposta)** | ~200ms | ~10-30s | 🤔 ~1-5s? |
| **Use Case** | Geral | Problemas complexos | Ambos |

---

## 🏗️ **5. Impacto na Nossa Arquitetura Praxis**

### **Desafio Atual:**

```
┌────────────────────────────────────────────────┐
│        Praxis StreamEvent (expectativa)        │
├────────────────────────────────────────────────┤
│ Reasoning { content }  ← Token-by-token        │
│ Message { content }    ← Token-by-token        │
└────────────────────────────────────────────────┘
                    ↕️
┌─────────────┬───────────────┬─────────────────┐
│   GPT-4o    │  o1-preview   │   GPT-5 (?)     │
├─────────────┼───────────────┼─────────────────┤
│ ✅ Streaming │ ❌ Streaming  │ ✅ Streaming    │
│ ❌ Reasoning │ ✅ Reasoning  │ ✅ Ambos?       │
│ (misturado) │ (completo)    │ (separado?)     │
└─────────────┴───────────────┴─────────────────┘
```

### **Estratégias:**

#### **Opção 1: Adapter por Modelo**

```rust
trait LLMAdapter {
    async fn execute(&self) -> Stream<StreamEvent>;
}

struct GPT4Adapter {
    // Streaming nativo, mas tudo vai como Message
    // Não tem reasoning separado
}

struct O1Adapter {
    // Não-streaming, simula chunks
    // Tem reasoning separado
}

struct GPT5Adapter {
    // (Futuro) Streaming nativo de reasoning + message
    // Parsing de dois canais
}
```

#### **Opção 2: Unified Response Type**

```rust
enum LLMResponse {
    // GPT-4: streaming simples
    StreamingContent(Stream<String>),
    
    // o1: completo com reasoning
    CompleteWithReasoning {
        reasoning: String,
        content: String,
    },
    
    // GPT-5: streaming duplo
    StreamingDual {
        reasoning_stream: Stream<String>,
        content_stream: Stream<String>,
    },
}
```

---

## 📝 **6. O Que Implementar AGORA**

### **Prioridade 1: Suporte ao o1 (Atual)**

✅ **Adicionar campos:**
```rust
pub struct ResponseMessage {
    pub content: Option<String>,
    pub reasoning_content: Option<String>,  // ← NOVO
    pub tool_calls: Option<Vec<ToolCall>>,
}

pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub reasoning_tokens: Option<u32>,  // ← NOVO
    pub total_tokens: u32,
}
```

✅ **Adapter para o1:**
```rust
// Detectar modelo e escolher estratégia
match model {
    "o1-preview" | "o1-mini" => {
        // Não-streaming
        let response = client.chat_completion(...).await?;
        
        // Converter para StreamEvents (simular streaming)
        if let Some(reasoning) = response.reasoning() {
            emit(StreamEvent::Reasoning { reasoning });
        }
        emit(StreamEvent::Message { content });
    }
    _ => {
        // Streaming normal (GPT-4)
        let stream = client.chat_completion_stream(...).await?;
        // ...
    }
}
```

### **Prioridade 2: Preparar para GPT-5**

🔮 **Design extensível:**
```rust
// Nossa trait pode evoluir
trait LLMClient {
    async fn stream(
        &self,
        model: &str,
        messages: Vec<Message>,
        options: LLMOptions,
    ) -> Result<LLMResponseStream>;
}

// Stream flexível
enum LLMResponseStream {
    Single(Stream<StreamEvent>),  // GPT-4, o1 simulado
    Dual {                        // GPT-5 futuro
        reasoning: Stream<StreamEvent::Reasoning>,
        message: Stream<StreamEvent::Message>,
    },
}
```

---

## 🔬 **7. Referências e Fontes**

### **Oficial (OpenAI):**
- [o1 System Card](https://openai.com/index/openai-o1-system-card/)
- [Reasoning Models Guide](https://platform.openai.com/docs/guides/reasoning)
- [API Reference - Chat Completions](https://platform.openai.com/docs/api-reference/chat)

### **Análises e Discussões:**
- [o1 vs GPT-4: What Changed](https://community.openai.com/t/o1-reasoning-models/...)
- [Speculation on GPT-5 Features](https://www.reddit.com/r/OpenAI/...)
- [Reasoning Models Deep Dive](https://simonwillison.net/2024/Sep/12/openai-o1/)

---

## ✅ **8. Conclusão: Nossa Situação**

### **O que está funcionando:**
- ✅ GPT-4 streaming (perfeito)
- ✅ Tool calling (GPT-4)
- ✅ Multimodal preparado (estrutura existe)

### **O que falta:**
- ❌ Campos `reasoning_content` e `reasoning_tokens` (o1)
- ❌ Lógica para detectar modelo e escolher estratégia
- ❌ Adapter para simular streaming do o1

### **O que considerar para futuro:**
- 🔮 GPT-5 pode trazer streaming dual (reasoning + message)
- 🔮 Nossa arquitetura precisa ser flexível
- 🔮 StreamEvent já está bem desenhado, mas precisamos de adapters

---

## 🎯 **Recomendação Final**

### **Curto Prazo (Agora):**
1. Adicionar campos `reasoning_content` e `reasoning_tokens`
2. Método `response.reasoning()` para acessar
3. **NÃO simular streaming do o1** (enviar como bloco único)
4. Documentar limitações do o1 no README

### **Médio Prazo (Quando GPT-5 lançar):**
1. Monitorar API changes
2. Implementar adapter específico se necessário
3. Testar streaming dual se disponível

### **Longo Prazo (Arquitetura Praxis):**
1. Abstrair via trait `LLMClient`
2. Cada provider (OpenAI, Anthropic, etc) implementa
3. Praxis Graph não sabe detalhes do provider
4. Adapter layer cuida das diferenças

---

**Última Atualização:** Janeiro 2025  
**Status do GPT-5:** Não anunciado oficialmente  
**Próxima Revisão:** Quando houver anúncios oficiais
