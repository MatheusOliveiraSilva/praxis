/// Adapter Pattern for Event Conversion
/// 
/// Converts between provider-specific event formats and the graph's internal event format.
/// This abstraction allows the graph to work with different LLM providers without
/// coupling to their specific event structures.

/// Stream adapter trait for converting between event formats
/// 
/// # Type Parameters
/// * `ProviderEvent` - The event type from the LLM provider
/// * `GraphEvent` - The internal graph event type
pub trait StreamAdapter {
    type ProviderEvent;
    type GraphEvent;
    
    /// Convert a provider event to a graph event
    /// 
    /// Returns None if the event should be filtered/ignored
    fn adapt(&self, event: Self::ProviderEvent) -> Option<Self::GraphEvent>;
}

/// OpenAI stream adapter
/// 
/// Converts OpenAI `StreamEvent` to graph `StreamEvent`.
/// Currently uses the From trait for direct conversion, but this adapter
/// provides a clear extension point for custom logic.
pub struct OpenAIStreamAdapter;

impl StreamAdapter for OpenAIStreamAdapter {
    type ProviderEvent = praxis_llm::StreamEvent;
    type GraphEvent = crate::types::StreamEvent;
    
    fn adapt(&self, event: Self::ProviderEvent) -> Option<Self::GraphEvent> {
        // Use the From trait implementation for conversion
        // In the future, we could add filtering, transformation, or enrichment logic here
        Some(event.into())
    }
}

/// Future: Azure OpenAI adapter
#[allow(dead_code)]
pub struct AzureStreamAdapter;

// impl StreamAdapter for AzureStreamAdapter {
//     type ProviderEvent = AzureStreamEvent;
//     type GraphEvent = crate::types::StreamEvent;
//     
//     fn adapt(&self, event: Self::ProviderEvent) -> Option<Self::GraphEvent> {
//         // Convert Azure-specific events to graph events
//         todo!("Azure adapter not yet implemented")
//     }
// }

/// Future: Anthropic adapter
#[allow(dead_code)]
pub struct AnthropicStreamAdapter;

// impl StreamAdapter for AnthropicStreamAdapter {
//     type ProviderEvent = AnthropicStreamEvent;
//     type GraphEvent = crate::types::StreamEvent;
//     
//     fn adapt(&self, event: Self::ProviderEvent) -> Option<Self::GraphEvent> {
//         // Convert Anthropic-specific events to graph events
//         todo!("Anthropic adapter not yet implemented")
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_llm::StreamEvent as LLMEvent;
    use crate::types::StreamEvent as GraphEvent;
    
    #[test]
    fn test_openai_adapter_message() {
        let adapter = OpenAIStreamAdapter;
        let llm_event = LLMEvent::Message {
            content: "Hello".to_string(),
        };
        
        let graph_event = adapter.adapt(llm_event);
        assert!(graph_event.is_some());
        
        match graph_event.unwrap() {
            GraphEvent::Message { content } => {
                assert_eq!(content, "Hello");
            }
            _ => panic!("Expected Message event"),
        }
    }
    
    #[test]
    fn test_openai_adapter_reasoning() {
        let adapter = OpenAIStreamAdapter;
        let llm_event = LLMEvent::Reasoning {
            content: "Thinking...".to_string(),
        };
        
        let graph_event = adapter.adapt(llm_event);
        assert!(graph_event.is_some());
        
        match graph_event.unwrap() {
            GraphEvent::Reasoning { content } => {
                assert_eq!(content, "Thinking...");
            }
            _ => panic!("Expected Reasoning event"),
        }
    }
}

