use crate::node::{EventSender, Node, NodeType};
use crate::types::GraphOutput;
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use praxis_llm::{ChatClient, ReasoningClient, ChatOptions, ChatRequest, ResponseRequest, ReasoningConfig, Message, ToolChoice};
use praxis_mcp::MCPToolExecutor;
use crate::types::GraphState;
use std::pin::Pin;
use std::sync::Arc;

pub struct LLMNode {
    client: Arc<dyn ChatClient>,
    reasoning_client: Option<Arc<dyn ReasoningClient>>,
    mcp_executor: Arc<MCPToolExecutor>,
}

impl LLMNode {
    pub fn new(client: Arc<dyn ChatClient>, mcp_executor: Arc<MCPToolExecutor>) -> Self {
        let reasoning_client = None; // We'll set this from client if it implements both traits
        Self { 
            client,
            reasoning_client,
            mcp_executor,
        }
    }

    pub fn with_reasoning_client(mut self, reasoning_client: Arc<dyn ReasoningClient>) -> Self {
        self.reasoning_client = Some(reasoning_client);
        self
    }

    /// Convert praxis_llm::StreamEvent to Graph StreamEvent
    /// Uses automatic From trait conversion
    fn convert_event(event: praxis_llm::StreamEvent) -> crate::types::StreamEvent {
        event.into()
    }

    /// Check if model should use Reasoning API
    fn is_reasoning_model(model: &str) -> bool {
        model.starts_with("gpt-5") || model.starts_with("o")
    }
    
    /// Template Method: Create stream based on model configuration
    async fn create_stream(
        &self,
        state: &GraphState,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<praxis_llm::StreamEvent>> + Send>>> {
        let model = &state.llm_config.model;
        let use_reasoning_api = Self::is_reasoning_model(model) && self.reasoning_client.is_some();
        
        tracing::info!(
            "LLM_NODE: Creating stream with model={}, use_reasoning_api={}",
            model,
            use_reasoning_api
        );
        
        if use_reasoning_api {
            self.create_reasoning_stream(state).await
        } else {
            self.create_chat_stream(state).await
        }
    }
    
    async fn create_reasoning_stream(
        &self,
        state: &GraphState,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<praxis_llm::StreamEvent>> + Send>>> {
        let reasoning_config = state.llm_config.reasoning_effort
            .as_ref()
            .map(|effort| match effort.as_str() {
                "low" => ReasoningConfig::low(),
                "high" => ReasoningConfig::high(),
                _ => ReasoningConfig::medium(),
            });

        let request = ResponseRequest::new(
            state.llm_config.model.clone(),
            state.messages.clone()
        );
        let request = if let Some(config) = reasoning_config {
            request.with_reasoning(config)
        } else {
            request
        };

        self.reasoning_client
            .as_ref()
            .unwrap()
            .reason_stream(request)
            .await
    }
    
    async fn create_chat_stream(
        &self,
        state: &GraphState,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<praxis_llm::StreamEvent>> + Send>>> {
        let tools = self.mcp_executor.get_llm_tools().await?;
        
        let mut options = ChatOptions::new()
            .tools(tools)
            .tool_choice(ToolChoice::auto());

        if let Some(temp) = state.llm_config.temperature {
            options = options.temperature(temp);
        }
        if let Some(max_tokens) = state.llm_config.max_tokens {
            options = options.max_tokens(max_tokens);
        }

        let request = ChatRequest::new(
            state.llm_config.model.clone(),
            state.messages.clone()
        ).with_options(options);

        self.client.chat_stream(request).await
    }
    
    /// Template Method: Process stream and return structured outputs
    async fn process_stream(
        &self,
        mut stream: Pin<Box<dyn futures::Stream<Item = Result<praxis_llm::StreamEvent>> + Send>>,
        event_tx: EventSender,
    ) -> Result<Vec<GraphOutput>> {
        let mut reasoning_content = String::new();
        let mut message_content = String::new();
        let mut tool_call_buffers: std::collections::HashMap<u32, (Option<String>, Option<String>, String)> = std::collections::HashMap::new();

        // Forward events and accumulate content separately
        while let Some(event_result) = stream.next().await {
            let llm_event = event_result?;

            // Convert and forward to client
            let graph_event = Self::convert_event(llm_event.clone());
            event_tx.send(graph_event).await?;

            // Accumulate based on event type (keep reasoning and message separate)
            match llm_event {
                praxis_llm::StreamEvent::Reasoning { content } => {
                    reasoning_content.push_str(&content);
                }
                praxis_llm::StreamEvent::Message { content } => {
                    message_content.push_str(&content);
                }
                praxis_llm::StreamEvent::ToolCall { index, id, name, arguments } => {
                let entry = tool_call_buffers.entry(index).or_insert((None, None, String::new()));
                
                if let Some(id) = id {
                    entry.0 = Some(id);
                }
                if let Some(name) = name {
                    entry.1 = Some(name);
                }
                if let Some(args) = arguments {
                    entry.2.push_str(&args);
                }
            }
                _ => {}
            }
        }

        // Build output items
        let mut outputs = Vec::new();
        
        // Add reasoning output if present
        if !reasoning_content.is_empty() {
            outputs.push(GraphOutput::reasoning(
                format!("rs_{}", uuid::Uuid::new_v4()),
                reasoning_content,
            ));
        }
        
        // Build tool calls
        let tool_calls: Vec<praxis_llm::ToolCall> = tool_call_buffers
            .into_iter()
            .filter_map(|(_, (id, name, arguments))| {
            if let (Some(id), Some(name)) = (id, name) {
                    Some(praxis_llm::ToolCall {
                    id,
                    tool_type: "function".to_string(),
                    function: praxis_llm::types::FunctionCall {
                        name,
                        arguments,
                    },
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // Add message output if present
        if !message_content.is_empty() || !tool_calls.is_empty() {
            if tool_calls.is_empty() {
                outputs.push(GraphOutput::message(
                    format!("msg_{}", uuid::Uuid::new_v4()),
                    message_content,
                ));
            } else {
                outputs.push(GraphOutput::message_with_tools(
                    format!("msg_{}", uuid::Uuid::new_v4()),
                    message_content,
                    tool_calls,
                ));
            }
        }
        
        Ok(outputs)
    }
    
    /// Template Method: Save outputs to state
    fn save_outputs(&self, state: &mut GraphState, outputs: &[GraphOutput]) -> Result<()> {
        // Concatenate all content for backward compatibility
        let mut combined_content = String::new();
        let mut combined_tool_calls = Vec::new();
        
        for output in outputs {
            match output {
                GraphOutput::Reasoning { content, .. } => {
                    combined_content.push_str(content);
                }
                GraphOutput::Message { content, tool_calls, .. } => {
                    combined_content.push_str(content);
                    if let Some(calls) = tool_calls {
                        combined_tool_calls.extend(calls.clone());
                    }
                }
            }
        }

        // Add assistant message to state
        let content = if !combined_content.is_empty() {
            Some(praxis_llm::Content::Text(combined_content))
        } else {
            None
        };
        
        let tool_calls = if !combined_tool_calls.is_empty() {
            Some(combined_tool_calls)
        } else {
            None
        };
        
        let assistant_message = Message::AI {
            content,
            tool_calls,
                name: None,
        };

        state.add_message(assistant_message);

        Ok(())
    }
}

#[async_trait]
impl Node for LLMNode {
    /// Template Method Pattern: Execute node with structured steps
    async fn execute(&self, state: &mut GraphState, event_tx: EventSender) -> Result<()> {
        // Step 1: Create stream (Chat or Reasoning API)
        let stream = self.create_stream(state).await?;
        
        // Step 2: Process stream and get structured outputs
        let outputs = self.process_stream(stream, event_tx).await?;
        
        // Step 3: Save outputs to state
        self.save_outputs(state, &outputs)?;
        
        // Store outputs in state for later use by graph
        state.last_outputs = Some(outputs);
        
        Ok(())
    }

    fn node_type(&self) -> NodeType {
        NodeType::LLM
    }
}


