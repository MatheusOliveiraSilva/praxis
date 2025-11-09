use praxis_llm::{ChatRequest, ChatOptions, ResponseRequest, ResponseOptions, Message, Tool, ToolChoice, ReasoningConfig};
use serde_json::json;

#[test]
fn test_chat_request_creation() {
    let messages = vec![Message::human("Hello")];
    let request = ChatRequest::new("gpt-4o", messages.clone());
    
    assert_eq!(request.model, "gpt-4o");
    assert_eq!(request.messages.len(), 1);
}

#[test]
fn test_chat_request_with_options() {
    let messages = vec![Message::human("Hello")];
    let options = ChatOptions::new()
        .temperature(0.7)
        .max_tokens(100);
    
    let request = ChatRequest::new("gpt-4o", messages)
        .with_options(options);
    
    assert_eq!(request.options.temperature, Some(0.7));
    assert_eq!(request.options.max_tokens, Some(100));
}

#[test]
fn test_chat_options_builder() {
    let tools = vec![
        Tool::new("test", "Test tool", json!({"type": "object"}))
    ];
    
    let options = ChatOptions::new()
        .temperature(0.5)
        .max_tokens(200)
        .tools(tools.clone())
        .tool_choice(ToolChoice::auto());
    
    assert_eq!(options.temperature, Some(0.5));
    assert_eq!(options.max_tokens, Some(200));
    assert!(options.tools.is_some());
    assert!(options.tool_choice.is_some());
}

#[test]
fn test_chat_options_default() {
    let options = ChatOptions::default();
    
    assert_eq!(options.temperature, None);
    assert_eq!(options.max_tokens, None);
    assert_eq!(options.tools, None);
    assert_eq!(options.tool_choice, None);
}

#[test]
fn test_response_request_creation() {
    let messages = vec![Message::human("Solve this")];
    let request = ResponseRequest::new("gpt-5", messages.clone());
    
    assert_eq!(request.model, "gpt-5");
    assert_eq!(request.input.len(), 1);
    assert_eq!(request.reasoning, None);
}

#[test]
fn test_response_request_with_reasoning() {
    let messages = vec![Message::human("Solve this")];
    let request = ResponseRequest::new("gpt-5", messages)
        .with_reasoning(ReasoningConfig::high());
    
    assert!(request.reasoning.is_some());
}

#[test]
fn test_response_request_with_options() {
    let messages = vec![Message::human("Test")];
    let options = ResponseOptions::new()
        .temperature(0.8)
        .max_output_tokens(500);
    
    let request = ResponseRequest::new("gpt-5", messages)
        .with_options(options);
    
    assert_eq!(request.options.temperature, Some(0.8));
    assert_eq!(request.options.max_output_tokens, Some(500));
}

#[test]
fn test_response_options_builder() {
    let options = ResponseOptions::new()
        .temperature(1.0)
        .max_output_tokens(1000);
    
    assert_eq!(options.temperature, Some(1.0));
    assert_eq!(options.max_output_tokens, Some(1000));
}

#[test]
fn test_response_options_default() {
    let options = ResponseOptions::default();
    
    assert_eq!(options.temperature, None);
    assert_eq!(options.max_output_tokens, None);
}

#[test]
fn test_reasoning_config_low() {
    let config = ReasoningConfig::low();
    // Just verify it creates without panic
    assert!(matches!(config.effort, praxis_llm::ReasoningEffort::Low));
}

#[test]
fn test_reasoning_config_medium() {
    let config = ReasoningConfig::medium();
    assert!(matches!(config.effort, praxis_llm::ReasoningEffort::Medium));
}

#[test]
fn test_reasoning_config_high() {
    let config = ReasoningConfig::high();
    assert!(matches!(config.effort, praxis_llm::ReasoningEffort::High));
}

#[test]
fn test_chat_request_clone() {
    let request = ChatRequest::new("gpt-4o", vec![Message::human("Hi")]);
    let cloned = request.clone();
    
    assert_eq!(request.model, cloned.model);
    assert_eq!(request.messages.len(), cloned.messages.len());
}

#[test]
fn test_response_request_clone() {
    let request = ResponseRequest::new("gpt-5", vec![Message::human("Hi")]);
    let cloned = request.clone();
    
    assert_eq!(request.model, cloned.model);
    assert_eq!(request.input.len(), cloned.input.len());
}

