# Streaming vs Reasoning: Comparação Visual

## 📊 **Diferenças Fundamentais**

### **GPT-4 (Streaming Habilitado)**

```
Cliente faz request:
POST /v1/chat/completions
{
  "model": "gpt-4",
  "messages": [...],
  "stream": true
}

Servidor responde (SSE):
┌─────────────────────────────────────────┐
│ Chunk 1 (t=0ms)                         │
│ delta: { role: "assistant", content: ""}│
├─────────────────────────────────────────┤
│ Chunk 2 (t=20ms)                        │
│ delta: { content: "The" }               │
├─────────────────────────────────────────┤
│ Chunk 3 (t=40ms)                        │
│ delta: { content: " answer" }           │
├─────────────────────────────────────────┤
│ Chunk 4 (t=60ms)                        │
│ delta: { content: " is" }               │
├─────────────────────────────────────────┤
│ Chunk 5 (t=80ms)                        │
│ delta: { content: " 42" }               │
├─────────────────────────────────────────┤
│ Chunk 6 (t=100ms)                       │
│ delta: {}                               │
│ finish_reason: "stop"                   │
└─────────────────────────────────────────┘

Total: 100ms
UX: Usuário vê texto aparecer em tempo real
```

---

### **o1-preview (Reasoning, Sem Streaming)**

```
Cliente faz request:
POST /v1/chat/completions
{
  "model": "o1-preview",
  "messages": [...]
  // NÃO pode usar "stream": true
}

Servidor responde (JSON completo):
┌─────────────────────────────────────────┐
│                                         │
│ Aguardando... (pode demorar 10-30s)    │
│                                         │
│         [modelo está pensando]          │
│                                         │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│ Resposta (t=15000ms)                    │
│                                         │
│ {                                       │
│   "choices": [{                         │
│     "message": {                        │
│       "role": "assistant",              │
│       "reasoning_content": "Let me...  │
│         ...think step by step...",      │
│       "content": "The answer is 42."    │
│     }                                   │
│   }],                                   │
│   "usage": {                            │
│     "reasoning_tokens": 450             │
│   }                                     │
│ }                                       │
└─────────────────────────────────────────┘

Total: 15000ms (tudo de uma vez)
UX: Usuário espera, depois vê tudo junto
```

---

## 🔄 **Como Nossa Arquitetura Praxis Espera**

```
┌──────────────────────────────────────────┐
│ Praxis StreamEvent (independente do LLM)│
├──────────────────────────────────────────┤
│ Reasoning { "Let" }                      │
│ Reasoning { " me" }                      │
│ Reasoning { " think" }                   │
│ Reasoning { "..." }                      │
│ Message { "The" }                        │
│ Message { " answer" }                    │
│ Message { " is" }                        │
│ Message { " 42" }                        │
└──────────────────────────────────────────┘

Sempre token-by-token, independente do provider!
```

---

## 🎯 **Problema: Incompatibilidade**

```
┌─────────────┐              ┌─────────────┐
│   GPT-4     │              │  o1-preview │
│             │              │             │
│ ✅ Streaming │              │ ❌ Streaming │
│ ❌ Reasoning │              │ ✅ Reasoning │
│             │              │             │
│ delta.      │              │ message.    │
│  content    │              │  reasoning_ │
│   (chunks)  │              │  content    │
│             │              │  (completo) │
└─────────────┘              └─────────────┘
        │                            │
        └────────────┬───────────────┘
                     ↓
         ┌───────────────────────┐
         │   Praxis Adapter      │
         │   (precisa unificar)  │
         └───────────────────────┘
                     ↓
         ┌───────────────────────┐
         │ StreamEvent::Reasoning│
         │ StreamEvent::Message  │
         └───────────────────────┘
```

---

## 💡 **Solução: Adapter Pattern**

### **Para GPT-4 (já funciona)**

```rust
// Streaming nativo
let mut stream = client.chat_completion_stream(...).await?;

while let Some(chunk) = stream.next().await {
    if let Some(content) = chunk.content() {
        // Emitir como Message (não há reasoning)
        emit(StreamEvent::Message { content });
    }
}
```

### **Para o1 (precisa implementar)**

```rust
// Não-streaming, simular chunks
let response = client.chat_completion(...).await?;

// 1. Processar reasoning
if let Some(reasoning) = response.reasoning() {
    // Opção A: Enviar tudo de uma vez
    emit(StreamEvent::Reasoning { content: reasoning });
    
    // Opção B: Simular streaming (dividir em palavras)
    for word in reasoning.split_whitespace() {
        emit(StreamEvent::Reasoning { content: word });
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

// 2. Processar message
if let Some(content) = response.content() {
    // Mesmo processo
    emit(StreamEvent::Message { content });
}
```

---

## 🏗️ **Arquitetura Proposta**

```
┌─────────────────────────────────────────────────┐
│              LLMClient Trait                    │
│  (abstração Praxis, provider-agnostic)          │
├─────────────────────────────────────────────────┤
│  async fn stream(...)                           │
│    -> Stream<StreamEvent>                       │
│                                                 │
│  // Sempre retorna StreamEvent!                │
│  // Adapter faz a conversão                    │
└─────────────────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
┌───────▼────────┐       ┌────────▼────────┐
│  GPT4Adapter   │       │   O1Adapter     │
├────────────────┤       ├─────────────────┤
│ - Usa streaming│       │ - Não usa stream│
│   nativo       │       │ - Simula chunks │
│ - delta.content│       │ - reasoning_    │
│   → Message    │       │   content       │
│                │       │   → Reasoning   │
└────────────────┘       └─────────────────┘
```

---

## 📝 **Trade-offs**

### **Opção 1: Simular Streaming para o1**

**Prós:**
- ✅ UX consistente
- ✅ Praxis não precisa saber qual modelo está usando
- ✅ Frontend único

**Contras:**
- ❌ Latência inicial (espera resposta completa)
- ❌ "Fake" streaming (não é real)
- ❌ Mais complexo

### **Opção 2: Enviar Blocos Únicos para o1**

**Prós:**
- ✅ Simples
- ✅ Honesto (não finge streaming)
- ✅ Menos código

**Contras:**
- ❌ UX diferente (frontend precisa lidar com 2 casos)
- ❌ Praxis sabe detalhes do provider

---

## 🎬 **Exemplo de Implementação**

### **Estrutura Unificada**

```rust
pub enum LLMResponse {
    Streaming(Pin<Box<dyn Stream<Item = StreamEvent>>>),
    Complete {
        reasoning: Option<String>,
        content: String,
        tool_calls: Option<Vec<ToolCall>>,
    },
}

impl OpenAIClient {
    pub async fn execute(&self, model: &str, ...) -> Result<LLMResponse> {
        match model {
            "gpt-4" | "gpt-3.5-turbo" => {
                // Streaming real
                let stream = self.chat_completion_stream(...).await?;
                Ok(LLMResponse::Streaming(stream))
            }
            "o1-preview" | "o1-mini" => {
                // Não-streaming
                let response = self.chat_completion(...).await?;
                Ok(LLMResponse::Complete {
                    reasoning: response.reasoning().map(String::from),
                    content: response.content().unwrap().to_string(),
                    tool_calls: response.tool_calls().map(|t| t.to_vec()),
                })
            }
            _ => Err(anyhow!("Unsupported model"))
        }
    }
}
```

---

## ✅ **Conclusão**

**Nossa implementação atual:**
- ✅ Streaming funciona perfeitamente para GPT-4
- ❌ **NÃO captura reasoning do o1**
- ❌ **NÃO tem campos necessários**

**Precisamos:**
1. Adicionar `reasoning_content` e `reasoning_tokens`
2. Criar adapter para o1 (simular streaming ou enviar blocos)
3. Decidir estratégia de UX (streaming fake vs blocos completos)

**Recomendação:**
- **Curto prazo:** Adicionar campos, enviar blocos completos
- **Longo prazo:** Implementar streaming simulado para UX consistente
