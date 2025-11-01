
### **Global Engineering Standards**

Você é uma LLM colaborando em um projeto Rust de alta escala.
Siga **estas diretrizes obrigatórias** em todas as ações, sugestões, e geração de código.

### **1. Princípio geral**

* Sempre **explique o raciocínio e o design** antes de escrever código.

  * Nunca apenas “gere o código”.
  * Diga o *porquê* de cada decisão, trade-off, e implicação de escala.
  * Quando possível, apresente **alternativas de arquitetura** com prós e contras.
  * Evite criar .md explicando o que fez, me pergunte antes de fazer isso, sempre!!!

---

### **2. Escalabilidade**

* Cada feature deve escalar para **milhões de usuários** sem reescrita fundamental.
* Evite soluções “rápidas” ou locais; prefira **designs desacoplados e horizontais**.
* Use **backpressure**, **bounded queues**, e **async I/O** para controlar carga.
* Prefira **stateless services** com persistência pluggable.
* Sempre considere **horizontal scale-out** antes de adotar qualquer cache global, mutex, ou estado compartilhado.

---

### **3. Arquitetura e Design Patterns**

* Sempre proponha **design patterns formais** antes de escrever implementações.

  * Exemplos: *Strategy*, *Builder*, *Observer*, *Factory*, *Command*, *Visitor*, *Reactor*, *Event Sourcing*, *Pipeline*.
* Prefira **composition over inheritance**.
* Evite “God objects”; cada módulo deve ter uma **responsabilidade única** (SRP).
* Camadas bem definidas:

  * **Core (domain)** → **Interface (traits)** → **Infra (adapters)** → **App (entrypoints)**.
* Mantenha dependências **unidirecionais** e **limpas** (Clean Architecture).
* Todas as interfaces críticas devem ter **contratos de versionamento**.

---

### **4. Testabilidade**

* Cada feature implementada **deve incluir testes unitários e integração mínima**.

  * Use mocks/fakes para external deps.
  * Nenhum módulo deve depender de I/O real para validar comportamento.
* Explique o raciocínio de testes: *o que validar, por que, e o limite do escopo*.
* Incluir **benchmarks ou stress tests** quando performance for fator crítico.
* Adotar convenção:

  * `tests/unit/` → testes isolados;
  * `tests/integration/` → e2e/módulos combinados;
  * `benches/` → microbenchmarks.

---

### **5. Dívida técnica**

* **Nunca** introduza *debt intencional* sem anotar explicitamente:

  * explique o motivo, o trade-off, e o prazo de quitação.
* Use comentários formais:

  ```rust
  // TECHDEBT[2025-11]: Refatorar para trait streaming unificada.
  ```
* Em cada pull request gerado, a LLM deve listar:

  * “Pontos de possível dívida técnica”
  * “Planos de mitigação futura”

---

### **6. Documentação**

* Cada módulo, trait e função pública deve ter docstring `///` com:

  * propósito, parâmetros, comportamento esperado, invariantes, complexidade e exemplos.
* Adicionar **README** local em cada crate explicando:

  * papel, dependências, e diagramas de fluxo.
* Sempre incluir **diagrama de componentes** (em ASCII ou PlantUML) antes do código.

---

### **7. Engenharia orientada a aprendizado**

* A cada nova feature, inclua explicações didáticas sobre:

  * conceitos Rust usados (lifetimes, Send/Sync, ownership),
  * trade-offs entre abordagens,
  * como medir performance e reduzir alocações.
* A prioridade é **clareza e manutenção** antes de concisão.
* Nunca deixar código "mágico" sem explicar como e por que funciona.

---

### **8. Robustez e segurança**

* Sempre valide input externo.
* Use `Result`/`anyhow` p/ erros previsíveis e `thiserror` p/ domínio.
* Evite panics; erros devem ser propagados graciosamente.
* Logging com `tracing` deve conter:

  * `run_id`, `request_id`, `span` hierárquico.
* Implementar **timeouts**, **cancellation tokens**, e **retry policies** configuráveis.

---

### **9. Performance**

* Priorize **eficiência de memória e throughput**.
* Evite clones desnecessários; use `Arc`, `Bytes`, `Cow`, ou referências.
* Analise O(n) e O(log n) em loops críticos.
* Sempre testar sob carga; explicar como escalar e otimizar.
* Para streaming, medir latência média e p95/p99.

---

### **10. Mentalidade de sistema**

* Cada parte deve poder ser **substituída isoladamente**.
* As crates devem formar um **ecossistema modular e coeso**.
* Toda decisão deve ser **auditoriável e explicável** (razão, impacto, reversão possível).
* Todo comentario no codigo deve ser feito em ingles.
