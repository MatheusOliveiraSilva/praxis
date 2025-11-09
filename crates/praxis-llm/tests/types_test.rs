use praxis_llm::{Content, Message, Tool, ToolCall, ToolChoice};
use serde_json::json;

#[test]
fn test_content_text_creation() {
    let content = Content::text("Hello, world!");
    assert_eq!(content.as_text(), Some("Hello, world!"));
}

#[test]
fn test_content_from_string() {
    let content: Content = "Test".into();
    assert_eq!(content.as_text(), Some("Test"));
}

#[test]
fn test_message_system() {
    let msg = Message::system("You are helpful");
    assert_eq!(msg.role(), "system");
}

#[test]
fn test_message_human() {
    let msg = Message::human("Hello");
    assert_eq!(msg.role(), "user");
}

#[test]
fn test_message_ai() {
    let msg = Message::ai("Hi there!");
    assert_eq!(msg.role(), "assistant");
}

#[test]
fn test_message_tool_result() {
    let msg = Message::tool_result("call_123", "42");
    assert_eq!(msg.role(), "tool");
}

#[test]
fn test_message_serialization_human() {
    let msg = Message::human("Hello");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"role\":\"user\""));
    assert!(json.contains("Hello"));
}

#[test]
fn test_message_serialization_ai() {
    let msg = Message::ai("Response");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"role\":\"assistant\""));
}

#[test]
fn test_message_deserialization() {
    let json = r#"{"role":"user","content":"Test"}"#;
    let msg: Message = serde_json::from_str(json).unwrap();
    assert_eq!(msg.role(), "user");
}

#[test]
fn test_tool_creation() {
    let tool = Tool::new(
        "get_weather",
        "Get weather for location",
        json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            }
        })
    );
    
    assert_eq!(tool.function.name, "get_weather");
    assert!(tool.function.description.is_some());
}

#[test]
fn test_tool_choice_auto() {
    let choice = ToolChoice::auto();
    let json = serde_json::to_value(&choice).unwrap();
    assert_eq!(json, "auto");
}

#[test]
fn test_tool_choice_none() {
    let choice = ToolChoice::none();
    let json = serde_json::to_value(&choice).unwrap();
    assert_eq!(json, "none");
}

#[test]
fn test_tool_choice_required() {
    let choice = ToolChoice::required();
    let json = serde_json::to_value(&choice).unwrap();
    assert_eq!(json, "required");
}

#[test]
fn test_tool_choice_force() {
    let choice = ToolChoice::force("get_weather");
    match choice {
        ToolChoice::Specific { tool_type, function } => {
            assert_eq!(tool_type, "function");
            assert_eq!(function.name, "get_weather");
        },
        _ => panic!("Expected Specific variant"),
    }
}

#[test]
fn test_tool_call_parse_arguments() {
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        tool_type: "function".to_string(),
        function: praxis_llm::types::FunctionCall {
            name: "get_weather".to_string(),
            arguments: r#"{"city":"NYC","units":"celsius"}"#.to_string(),
        },
    };
    
    #[derive(serde::Deserialize)]
    struct WeatherArgs {
        city: String,
        units: String,
    }
    
    let args: WeatherArgs = tool_call.parse_arguments().unwrap();
    assert_eq!(args.city, "NYC");
    assert_eq!(args.units, "celsius");
}

#[test]
fn test_tool_call_arguments_value() {
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        tool_type: "function".to_string(),
        function: praxis_llm::types::FunctionCall {
            name: "test".to_string(),
            arguments: r#"{"key":"value"}"#.to_string(),
        },
    };
    
    let value = tool_call.arguments_value().unwrap();
    assert_eq!(value["key"], "value");
}

#[test]
fn test_message_ai_with_tools() {
    let tool_calls = vec![
        ToolCall {
            id: "call_1".to_string(),
            tool_type: "function".to_string(),
            function: praxis_llm::types::FunctionCall {
                name: "test".to_string(),
                arguments: "{}".to_string(),
            },
        }
    ];
    
    let msg = Message::ai_with_tools(tool_calls);
    assert_eq!(msg.role(), "assistant");
}

#[test]
fn test_content_parts() {
    let parts = vec![
        praxis_llm::types::ContentPart::Text {
            text: "Hello".to_string(),
        }
    ];
    let content = Content::Parts(parts);
    
    // Single text part should return text
    assert_eq!(content.as_text(), Some("Hello"));
}

