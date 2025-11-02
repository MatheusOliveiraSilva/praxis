# 🚨 DESCOBERTA CRÍTICA: GPT-5 Responses API

## ⚠️ **MUDANÇA FUNDAMENTAL NA ARQUITETURA**

GPT-5 com reasoning **NÃO usa Chat Completions API tradicional**!

Existe uma **NOVA API** chamada **"Responses API"** especificamente para reasoning models.

---

## 📊 **Duas APIs Diferentes**

### **1. Chat Completions API (Antiga/Tradicional)**

```bash
POST /v1/chat/completions

# Para GPT-4, GPT-3.5, etc
{
  "model": "gpt-4",
  "messages": [...],
  "temperature": 0.7,
  "max_tokens": 500
}
```

**Funciona com:**
- ✅ GPT-4, GPT-4o
- ✅ GPT-3.5
- ⚠️ GPT-5 (suportado, mas não recomendado para reasoning)

---

### **2. Responses API (Nova!)** 🆕

```bash
POST /v1/responses

# Para GPT-5, o1, modelos de reasoning
{
  "model": "gpt-5",
  "input": [...],              # ← "input" ao invés de "messages"
  "reasoning": {
    "effort": "medium",        # ← Parâmetro de reasoning
    "summary": "auto"          # ← Pode obter summary!
  },
  "max_output_tokens": 5000    # ← Novo parâmetro
}
```

**Funciona com:**
- ✅ GPT-5, GPT-5-mini, GPT-5-nano
- ✅ o1 (provavelmente o4-mini também)
- ❌ GPT-4 (não suporta)

---

## 🎯 **Diferenças Principais**

| Aspecto | Chat Completions API | Responses API |
|---------|---------------------|---------------|
| **Endpoint** | `/v1/chat/completions` | `/v1/responses` |
| **Parâmetro de Input** | `messages` | `input` |
| **Reasoning** | ❌ Não exposto | ✅ Com summary! |
| **Max Tokens** | `max_tokens` / `max_completion_tokens` | `max_output_tokens` |
| **Output** | `choices[].message` | `output` array |
| **Reasoning Tokens** | Não visível | `output_tokens_details.reasoning_tokens` |
| **Summary** | ❌ Não disponível | ✅ `reasoning.summary` |

---

## 🔍 **Responses API: Estrutura Completa**

### **Request:**

```python
from openai import OpenAI
client = OpenAI()

response = client.responses.create(
    model="gpt-5",
    
    # Input (similar a messages)
    input=[
        {"role": "user", "content": "What is 47 * 89?"}
    ],
    
    # ✅ NOVO: Configuração de reasoning
    reasoning={
        "effort": "high",     # low | medium | high
        "summary": "auto"     # auto | detailed | concise
    },
    
    # ✅ NOVO: max_output_tokens (inclui reasoning)
    max_output_tokens=5000,
    
    # Opcional: incluir conteúdo encriptado
    include=["reasoning.encrypted_content"]
)
```

### **Response:**

```json
{
  "id": "resp_abc123",
  "object": "response",
  "created": 1699999999,
  "model": "gpt-5-2025-08-07",
  
  // ✅ NOVO: Output é array (não choices)
  "output": [
    {
      "id": "rs_xyz789",
      "type": "reasoning",  // ← Item de reasoning!
      "summary": [
        {
          "type": "summary_text",
          "text": "I need to multiply 47 by 89. Let me break this down:\n1. 47 * 90 = 4230\n2. 47 * 1 = 47\n3. 4230 - 47 = 4183"
        }
      ]
    },
    {
      "id": "msg_def456",
      "type": "message",  // ← Mensagem final
      "status": "completed",
      "content": [
        {
          "type": "output_text",
          "text": "The answer is 4183."
        }
      ],
      "role": "assistant"
    }
  ],
  
  // ✅ NOVO: Usage com detalhes
  "usage": {
    "input_tokens": 15,
    "output_tokens": 1186,
    "output_tokens_details": {
      "reasoning_tokens": 1024  // ← Aqui!
    },
    "total_tokens": 1201
  },
  
  "status": "completed"  // ou "incomplete"
}
```

---

## 🆕 **Recursos da Responses API**

### **1. Reasoning Summary** ✅

Você **PODE ver o raciocínio** do modelo (summary, não tokens brutos):

```python
response = client.responses.create(
    model="gpt-5",
    input="What is the capital of France?",
    reasoning={
        "effort": "low",
        "summary": "auto"  # ← Pede summary!
    }
)

# Output contém reasoning item
for item in response.output:
    if item['type'] == 'reasoning':
        print("🧠 Reasoning:", item['summary'][0]['text'])
    elif item['type'] == 'message':
        print("💬 Response:", item['content'][0]['text'])
```

**Output:**
```
🧠 Reasoning: I'm looking at a straightforward question: the capital 
of France is Paris. It's a well-known fact...

💬 Response: The capital of France is Paris.
```

---

### **2. Reasoning Effort Levels**

```python
reasoning = {
    "effort": "low"      # Rápido, barato, menos preciso
    # ou
    "effort": "medium"   # Balanceado (padrão)
    # ou
    "effort": "high"     # Lento, caro, mais preciso
}
```

---

### **3. Summary Levels**

```python
reasoning = {
    "summary": "auto"      # Melhor disponível para o modelo
    # ou
    "summary": "detailed"  # Mais detalhado
    # ou
    "summary": "concise"   # Mais conciso
}
```

---

### **4. Incomplete Responses**

Se o modelo fica sem tokens:

```json
{
  "status": "incomplete",
  "incomplete_details": {
    "reason": "max_output_tokens"
  },
  "output": [...]  // Pode estar vazio ou parcial
}
```

**Como lidar:**

```python
response = client.responses.create(
    model="gpt-5",
    reasoning={"effort": "medium"},
    input=[...],
    max_output_tokens=300  # Limite baixo
)

if response.status == "incomplete":
    if response.incomplete_details.reason == "max_output_tokens":
        if response.output_text:
            print("Partial:", response.output_text)
        else:
            print("Ran out during reasoning")
```

---

## 🔄 **Chat Completions vs Responses**

### **Chat Completions (GPT-4 style):**

```python
# Request
response = client.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "user", "content": "Hello"}
    ]
)

# Response
print(response.choices[0].message.content)
```

### **Responses (GPT-5 style):**

```python
# Request
response = client.responses.create(
    model="gpt-5",
    input=[
        {"role": "user", "content": "Hello"}
    ],
    reasoning={"effort": "medium", "summary": "auto"}
)

# Response
for item in response.output:
    if item.type == "reasoning":
        print("Reasoning:", item.summary[0].text)
    elif item.type == "message":
        print("Message:", item.content[0].text)
```

---

## 🚨 **IMPACTO NA NOSSA IMPLEMENTAÇÃO**

### **Situação Atual:**

```rust
// Nossa implementação (Chat Completions API)
let response = client.chat_completion(
    "gpt-5",
    messages,
    options
).await?;
```

**Problemas:**
1. ❌ Estamos usando a **API errada** para GPT-5 com reasoning
2. ❌ Não conseguimos acessar **reasoning summary**
3. ❌ Não temos os parâmetros corretos (`reasoning.effort`, `reasoning.summary`)
4. ❌ Response structure é diferente (`output` vs `choices`)

---

### **O Que Precisamos:**

#### **Opção 1: Implementar Responses API** (Ideal)

```rust
// Nova trait para Responses API
pub trait ResponsesClient {
    async fn create_response(
        &self,
        model: &str,
        input: Vec<Message>,
        options: ResponseOptions,
    ) -> Result<Response>;
}

pub struct ResponseOptions {
    pub reasoning: Option<ReasoningConfig>,
    pub max_output_tokens: Option<u32>,
}

pub struct ReasoningConfig {
    pub effort: ReasoningEffort,  // Low, Medium, High
    pub summary: Option<SummaryLevel>,  // Auto, Detailed, Concise
}

pub struct Response {
    pub id: String,
    pub output: Vec<OutputItem>,  // Reasoning + Message items
    pub usage: Usage,
    pub status: ResponseStatus,  // Completed, Incomplete
}

pub enum OutputItem {
    Reasoning {
        id: String,
        summary: Vec<SummaryText>,
    },
    Message {
        id: String,
        content: Vec<ContentItem>,
        role: String,
    },
}
```

#### **Opção 2: Manter Chat Completions** (Temporário)

Continuar usando Chat Completions API para GPT-5, mas:
- ❌ Sem acesso a reasoning summary
- ❌ Menos performance (não recomendado pela OpenAI)
- ✅ Mais simples (código existente funciona)

---

## 📊 **Comparação: Mesma Request, Duas APIs**

### **Via Chat Completions (Atual):**

```bash
POST /v1/chat/completions
{
  "model": "gpt-5",
  "messages": [{"role": "user", "content": "47*89"}],
  "reasoning_effort": "high"  # ← Parâmetro informal
}

Response:
{
  "choices": [{
    "message": {
      "content": "4183"  # ← Só a resposta
    }
  }],
  "usage": {
    "completion_tokens_details": {
      "reasoning_tokens": 128  # ← Tokens gastos, mas reasoning não visível
    }
  }
}
```

### **Via Responses (Novo):**

```bash
POST /v1/responses
{
  "model": "gpt-5",
  "input": [{"role": "user", "content": "47*89"}],
  "reasoning": {
    "effort": "high",
    "summary": "auto"  # ← Parâmetro oficial
  }
}

Response:
{
  "output": [
    {
      "type": "reasoning",
      "summary": [{
        "text": "Let me break this down: 47*90=4230, 47*1=47, 4230-47=4183"
      }]
    },
    {
      "type": "message",
      "content": [{"text": "4183"}]
    }
  ],
  "usage": {
    "output_tokens_details": {
      "reasoning_tokens": 128
    }
  }
}
```

**Diferença crítica:** Com Responses API, você **VÊ o summary do reasoning**!

---

## 🎯 **Prioridades de Implementação**

### **Fase 1: Suporte Básico a Responses API**

1. [ ] Criar client para `/v1/responses`
2. [ ] Structs: `ResponseOptions`, `ReasoningConfig`, `Response`
3. [ ] Parse do `output` array (reasoning + message items)
4. [ ] Suporte a `reasoning.effort` e `reasoning.summary`

### **Fase 2: Reasoning Summary**

1. [ ] Extrair reasoning summary do response
2. [ ] Converter para nossos `StreamEvent::Reasoning`
3. [ ] Compatibilidade com arquitetura Praxis

### **Fase 3: Streaming (se disponível)**

1. [ ] Investigar se Responses API suporta streaming
2. [ ] Se sim, implementar SSE parsing adaptado

---

## 📝 **Exemplo de Uso Futuro**

```rust
// Novo código (Responses API)
let response = client.create_response(
    "gpt-5",
    vec![Message::user("What is 47*89?")],
    ResponseOptions {
        reasoning: Some(ReasoningConfig {
            effort: ReasoningEffort::High,
            summary: Some(SummaryLevel::Auto),
        }),
        max_output_tokens: Some(5000),
    }
).await?;

// Processar output
for item in response.output {
    match item {
        OutputItem::Reasoning { summary, .. } => {
            for text in summary {
                println!("🧠 {}", text.text);
                // Pode emitir como StreamEvent::Reasoning
            }
        }
        OutputItem::Message { content, .. } => {
            for c in content {
                println!("💬 {}", c.text);
                // Emitir como StreamEvent::Message
            }
        }
    }
}

println!("📊 Reasoning tokens: {}", 
    response.usage.output_tokens_details.reasoning_tokens);
```

---

## ✅ **Conclusão**

### **Descobertas:**

1. ✅ GPT-5 tem uma **API nova** (`/v1/responses`)
2. ✅ Com essa API, você **PODE ver reasoning** (via summary)
3. ✅ Suporta `reasoning.effort` e `reasoning.summary`
4. ✅ Estrutura de response é diferente (`output` array)
5. ⚠️ Nossa implementação atual usa a API errada

### **Próximos Passos:**

1. **Curto prazo:** Testar Responses API com curl
2. **Médio prazo:** Implementar client Rust para Responses API
3. **Longo prazo:** Unificar ambas APIs (Chat Completions + Responses)

### **Trade-offs:**

| Decisão | Prós | Contras |
|---------|------|---------|
| **Implementar Responses API** | ✅ Acesso a reasoning<br>✅ Melhor performance<br>✅ Futuro-proof | ⚠️ Mais código<br>⚠️ Nova API para aprender |
| **Manter Chat Completions** | ✅ Simples<br>✅ Código existente | ❌ Sem reasoning summary<br>❌ Não recomendado OpenAI |

**Recomendação:** Implementar Responses API para GPT-5/o1.

---

**Status:** 🚨 Descoberta crítica  
**Impacto:** Alto - Arquitetura precisa adaptar  
**Urgência:** Média - Chat Completions ainda funciona (sem reasoning summary)
