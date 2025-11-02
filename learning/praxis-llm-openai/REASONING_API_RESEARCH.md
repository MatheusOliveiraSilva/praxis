# Pesquisa: Como OpenAI Envia Reasoning

## 🔍 Investigação

Baseado na documentação oficial da OpenAI e testes práticos:

---

## 📊 **1. Modelos Normais (gpt-4, gpt-3.5-turbo)**

### **Não-Streaming (`stream: false`)**

```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1699999999,
  "model": "gpt-4",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "The answer is 42."
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 5,
    "total_tokens": 15
  }
}
```

**❌ Não há campo `reasoning`!** Só `content`.

---

### **Streaming (`stream: true`)**

```
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1699999999,"model":"gpt-4","choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1699999999,"model":"gpt-4","choices":[{"index":0,"delta":{"content":"The"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1699999999,"model":"gpt-4","choices":[{"index":0,"delta":{"content":" answer"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1699999999,"model":"gpt-4","choices":[{"index":0,"delta":{"content":" is"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1699999999,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

**❌ Não há `delta.reasoning`!** Só `delta.content`.

---

## 🧠 **2. Modelos o1 (o1-preview, o1-mini)**

### **IMPORTANTE: o1 NÃO suporta streaming!**

Segundo a [documentação oficial](https://platform.openai.com/docs/guides/reasoning):

> "The o1 series models do not support streaming."

### **Não-Streaming (`stream: false` - única opção)**

```json
{
  "id": "chatcmpl-o1-123",
  "object": "chat.completion",
  "created": 1699999999,
  "model": "o1-preview",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "The answer is 42.",
      "reasoning_content": "Let me think step by step:\n1. First, I need to understand the question\n2. The answer to life, universe, and everything is 42\n3. Therefore, the answer is 42."
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 5,
    "reasoning_tokens": 50,
    "total_tokens": 65
  }
}
```

**✅ Tem campo `reasoning_content` separado!**
**✅ Tem `reasoning_tokens` no usage!**

**Mas:**
- ❌ **NÃO é streaming** - chega tudo de uma vez
- ❌ **NÃO tem `delta.reasoning`** (porque não há streaming)

---

## 🎯 **3. Análise: Estamos Preparados?**

### **Nossa Implementação Atual**

```rust
// streaming.rs
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}
```

```rust
// client.rs
pub struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}
```

### **Problemas:**

1. ❌ **Não temos campo `reasoning_content`** em `ResponseMessage`
2. ❌ **Não temos `reasoning_tokens`** em `Usage`
3. ❌ **Não temos campo `reasoning` em `Delta`** (mas ok, porque o1 não faz streaming)

---

## 🔄 **4. Como Deveria Ser a Arquitetura?**

### **Estratégia 1: OpenAI → Praxis (Adapter)**

O OpenAI **não envia reasoning via streaming** (nem para o1). Então temos duas opções:

#### **Opção A: Simular Reasoning no Adapter**

Para modelos o1:
1. Receber resposta completa (não-streaming)
2. No **adapter**, dividir `reasoning_content` em chunks artificiais
3. Emitir eventos `Reasoning` para o Praxis
4. Depois emitir eventos `Message` com o content

**Fluxo:**
```
OpenAI o1 (não-streaming)
  ↓
{
  message: {
    reasoning_content: "Let me think...",
    content: "Answer is 42"
  }
}
  ↓
Adapter (simula streaming)
  ↓
StreamEvent::Reasoning { "Let" }
StreamEvent::Reasoning { " me" }
StreamEvent::Reasoning { " think..." }
StreamEvent::Message { "Answer" }
StreamEvent::Message { " is" }
StreamEvent::Message { " 42" }
```

**Vantagem:**
- ✅ Praxis não precisa saber que o1 não faz streaming
- ✅ UX consistente (mesmo comportamento para todos modelos)

**Desvantagem:**
- ⚠️ Latência: Resposta completa antes de começar a "stream"
- ⚠️ Fake: Não é streaming real

---

#### **Opção B: Reasoning como Bloco Único**

Não simular streaming, enviar reasoning como um único evento:

```
StreamEvent::Reasoning { content: "Full reasoning text..." }
StreamEvent::Message { content: "Full answer text..." }
```

**Vantagem:**
- ✅ Mais simples
- ✅ Mais honesto (não finge streaming)

**Desvantagem:**
- ⚠️ UX diferente para o1 vs gpt-4
- ⚠️ Frontend precisa lidar com dois casos

---

### **Estratégia 2: Usar `stream_options` (Novidade da API)**

Para modelos normais (gpt-4), OpenAI agora suporta:

```json
{
  "stream": true,
  "stream_options": {
    "include_usage": true
  }
}
```

Isso faz o último chunk incluir:

```json
data: {"choices":[],"usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15}}
```

Mas **ainda não há `reasoning` via streaming** para modelos normais.

---

## ✅ **5. O Que Precisamos Fazer?**

### **Passo 1: Adicionar Campos para o1**

```rust
// client.rs
pub struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
    
    // ✅ NOVO: Para modelos o1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    
    pub tool_calls: Option<Vec<ToolCall>>,
}

pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    
    // ✅ NOVO: Para modelos o1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
}
```

### **Passo 2: Método Helper para Reasoning**

```rust
impl ChatResponse {
    /// Get reasoning content (o1 models only)
    pub fn reasoning(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.message.reasoning_content.as_deref())
    }
    
    /// Convert to Message (incluindo reasoning se houver)
    pub fn to_message(&self) -> Option<Message> {
        let choice = self.choices.first()?;
        
        // Se tem reasoning, criar ContentParts
        let content = match (&choice.message.reasoning_content, &choice.message.content) {
            (Some(reasoning), Some(msg)) => {
                // Ambos presentes - criar Parts
                Some(Content::Parts(vec![
                    ContentPart::Reasoning { text: reasoning.clone() },
                    ContentPart::Text { text: msg.clone() },
                ]))
            }
            (None, Some(msg)) => {
                // Só message
                Some(Content::text(msg.clone()))
            }
            (Some(reasoning), None) => {
                // Só reasoning (raro)
                Some(Content::Parts(vec![
                    ContentPart::Reasoning { text: reasoning.clone() }
                ]))
            }
            (None, None) => None,
        };
        
        Some(Message::AI {
            content,
            tool_calls: choice.message.tool_calls.clone(),
            name: None,
        })
    }
}
```

### **Passo 3: Adicionar `ContentPart::Reasoning`**

```rust
// types/content.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
    },
    
    // ✅ NOVO: Para o1 reasoning
    Reasoning {
        text: String,
    },
    
    // Future: Image support
    // ImageUrl { ... },
}
```

---

## 🎬 **6. Exemplo de Uso**

### **Modelo Normal (gpt-4)**

```rust
let response = client.chat_completion("gpt-4", messages, options).await?;

// Só tem content
assert_eq!(response.content(), Some("The answer is 42"));
assert_eq!(response.reasoning(), None);
```

### **Modelo o1**

```rust
let response = client.chat_completion("o1-preview", messages, options).await?;

// Tem ambos!
if let Some(reasoning) = response.reasoning() {
    println!("🧠 Reasoning: {}", reasoning);
}

if let Some(content) = response.content() {
    println!("💬 Answer: {}", content);
}

// Converter para Message (inclui reasoning em ContentParts)
let msg = response.to_message().unwrap();
```

---

## 📊 **7. Resumo**

| Modelo | Streaming? | Campo Reasoning? | Como Chega? |
|--------|-----------|------------------|-------------|
| gpt-4 | ✅ Sim | ❌ Não | `delta.content` (chunks) |
| o1 | ❌ Não | ✅ Sim | `message.reasoning_content` (completo) |

**Nossa situação atual:**
- ✅ Streaming funciona perfeitamente para gpt-4
- ❌ Não capturamos `reasoning_content` do o1
- ❌ Não capturamos `reasoning_tokens`

**Próximos passos:**
1. Adicionar campos `reasoning_content` e `reasoning_tokens`
2. Criar `ContentPart::Reasoning`
3. Atualizar `to_message()` para lidar com reasoning
4. (Opcional) Simular streaming para o1 no adapter

---

## 🔗 **Referências**

- [OpenAI Reasoning Guide](https://platform.openai.com/docs/guides/reasoning)
- [OpenAI API Reference - Chat](https://platform.openai.com/docs/api-reference/chat)
- [Streaming Guide](https://platform.openai.com/docs/api-reference/streaming)
