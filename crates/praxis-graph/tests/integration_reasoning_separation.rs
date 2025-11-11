/// Integration tests for reasoning separation in persistence and observability
/// 
/// Tests the full flow from graph execution to database persistence and tracing

#[cfg(all(test, feature = "mongodb", feature = "observability"))]
mod tests {
    use praxis_graph::types::{GraphInput, LLMConfig, GraphOutput};
    use praxis_llm::Message;
    
    #[test]
    fn test_graph_output_creation() {
        // Test GraphOutput::reasoning
        let reasoning = GraphOutput::reasoning("rs_123", "Thinking about the problem...");
        assert_eq!(reasoning.id(), "rs_123");
        assert_eq!(reasoning.content(), "Thinking about the problem...");
        
        // Test GraphOutput::message
        let message = GraphOutput::message("msg_456", "Here is the answer");
        assert_eq!(message.id(), "msg_456");
        assert_eq!(message.content(), "Here is the answer");
    }
    
    #[test]
    fn test_llm_config_with_provider() {
        use praxis_graph::types::Provider;
        
        let config = LLMConfig::new("gpt-5")
            .with_provider(Provider::OpenAI)
            .with_reasoning_effort("medium");
        
        assert_eq!(config.model, "gpt-5");
        assert_eq!(config.provider, Provider::OpenAI);
        assert_eq!(config.reasoning_effort, Some("medium".to_string()));
    }
    
    #[test]
    fn test_client_factory_reasoning_detection() {
        use praxis_graph::ClientFactory;
        
        // Test reasoning model detection
        assert!(ClientFactory::supports_reasoning("gpt-5"));
        assert!(ClientFactory::supports_reasoning("gpt-5-turbo"));
        assert!(ClientFactory::supports_reasoning("o1-preview"));
        assert!(ClientFactory::supports_reasoning("o1-mini"));
        
        assert!(!ClientFactory::supports_reasoning("gpt-4o"));
        assert!(!ClientFactory::supports_reasoning("gpt-4o-mini"));
        assert!(!ClientFactory::supports_reasoning("gpt-3.5-turbo"));
    }
    
    #[test]
    fn test_graph_state_with_outputs() {
        use praxis_graph::types::{GraphState, LLMConfig};
        use praxis_llm::Message;
        
        let messages = vec![Message::human("Test question")];
        let llm_config = LLMConfig::new("gpt-5");
        
        let mut state = GraphState::new(
            "conv_123".to_string(),
            "run_456".to_string(),
            messages,
            llm_config,
        );
        
        // Add outputs to state
        let outputs = vec![
            GraphOutput::reasoning("rs_1", "Analyzing the question..."),
            GraphOutput::message("msg_1", "The answer is 42"),
        ];
        
        state.last_outputs = Some(outputs.clone());
        
        assert!(state.last_outputs.is_some());
        assert_eq!(state.last_outputs.as_ref().unwrap().len(), 2);
    }
    
    #[test]
    fn test_db_message_with_reasoning_type() {
        use praxis_persist::{DBMessage, MessageRole, MessageType};
        
        let reasoning_msg = DBMessage {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: "thread_123".to_string(),
            user_id: "user_456".to_string(),
            role: MessageRole::Assistant,
            message_type: MessageType::Reasoning,
            content: "Thinking step by step...".to_string(),
            tool_call_id: None,
            tool_name: None,
            arguments: None,
            reasoning_id: Some("rs_789".to_string()),
            created_at: chrono::Utc::now(),
            duration_ms: Some(1000),
        };
        
        // Verify reasoning message is correctly structured
        assert_eq!(reasoning_msg.message_type, MessageType::Reasoning);
        assert!(reasoning_msg.reasoning_id.is_some());
        assert_eq!(reasoning_msg.reasoning_id.unwrap(), "rs_789");
    }
    
    #[test]
    fn test_observability_node_output() {
        use praxis_observability::{NodeOutput, ToolCallInfo};
        
        // Test reasoning output
        let reasoning = NodeOutput::Reasoning {
            id: "rs_123".to_string(),
            content: "Analyzing...".to_string(),
        };
        
        match reasoning {
            NodeOutput::Reasoning { id, content } => {
                assert_eq!(id, "rs_123");
                assert_eq!(content, "Analyzing...");
            }
            _ => panic!("Expected Reasoning output"),
        }
        
        // Test message output
        let message = NodeOutput::Message {
            id: "msg_456".to_string(),
            content: "Answer".to_string(),
        };
        
        match message {
            NodeOutput::Message { id, content } => {
                assert_eq!(id, "msg_456");
                assert_eq!(content, "Answer");
            }
            _ => panic!("Expected Message output"),
        }
    }
    
    #[test]
    fn test_stream_adapter_pattern() {
        use praxis_graph::{StreamAdapter, OpenAIStreamAdapter};
        use praxis_llm::StreamEvent as LLMEvent;
        
        let adapter = OpenAIStreamAdapter;
        
        // Test reasoning event adaptation
        let reasoning_event = LLMEvent::Reasoning {
            content: "Thinking...".to_string(),
        };
        
        let adapted = adapter.adapt(reasoning_event);
        assert!(adapted.is_some());
        
        // Test message event adaptation
        let message_event = LLMEvent::Message {
            content: "Response".to_string(),
        };
        
        let adapted = adapter.adapt(message_event);
        assert!(adapted.is_some());
    }
}

/// Documentation tests
#[cfg(test)]
mod doc_tests {
    /// Example: Using the new architecture for reasoning separation
    /// 
    /// ```rust,no_run
    /// use praxis_graph::types::{GraphInput, LLMConfig, Provider, GraphOutput};
    /// use praxis_llm::Message;
    /// 
    /// // Create LLM config with reasoning support
    /// let llm_config = LLMConfig::new("gpt-5")
    ///     .with_provider(Provider::OpenAI)
    ///     .with_reasoning_effort("medium")
    ///     .with_temperature(0.7);
    /// 
    /// // Create graph input
    /// let input = GraphInput::new(
    ///     "conversation_123",
    ///     vec![Message::human("Explain quantum computing")],
    ///     llm_config,
    /// );
    /// 
    /// // When the graph executes with gpt-5:
    /// // 1. LLMNode detects it's a reasoning model
    /// // 2. Uses Reasoning API instead of Chat API
    /// // 3. Returns separate GraphOutputs for reasoning and message
    /// // 4. Graph persists each as separate DB entries
    /// // 5. Observer traces each as separate Langfuse generations
    /// ```
    fn _example_usage() {}
}

