use praxis_llm::StreamEvent;

#[test]
fn test_stream_event_message() {
    let event = StreamEvent::Message {
        content: "Hello".to_string(),
    };
    
    match event {
        StreamEvent::Message { content } => assert_eq!(content, "Hello"),
        _ => panic!("Expected Message variant"),
    }
}

#[test]
fn test_stream_event_reasoning() {
    let event = StreamEvent::Reasoning {
        content: "Thinking...".to_string(),
    };
    
    match event {
        StreamEvent::Reasoning { content } => assert_eq!(content, "Thinking..."),
        _ => panic!("Expected Reasoning variant"),
    }
}

#[test]
fn test_stream_event_tool_call() {
    let event = StreamEvent::ToolCall {
        index: 0,
        id: Some("call_123".to_string()),
        name: Some("get_weather".to_string()),
        arguments: Some(r#"{"city":"NYC"}"#.to_string()),
    };
    
    match event {
        StreamEvent::ToolCall { index, id, name, arguments } => {
            assert_eq!(index, 0);
            assert_eq!(id, Some("call_123".to_string()));
            assert_eq!(name, Some("get_weather".to_string()));
            assert!(arguments.is_some());
        },
        _ => panic!("Expected ToolCall variant"),
    }
}

#[test]
fn test_stream_event_done() {
    let event = StreamEvent::Done {
        finish_reason: Some("stop".to_string()),
    };
    
    match event {
        StreamEvent::Done { finish_reason } => {
            assert_eq!(finish_reason, Some("stop".to_string()));
        },
        _ => panic!("Expected Done variant"),
    }
}

#[test]
fn test_stream_event_serialization_message() {
    let event = StreamEvent::Message {
        content: "Test".to_string(),
    };
    
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"message\""));
    assert!(json.contains("Test"));
}

#[test]
fn test_stream_event_serialization_reasoning() {
    let event = StreamEvent::Reasoning {
        content: "Analyze".to_string(),
    };
    
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"reasoning\""));
}

#[test]
fn test_stream_event_deserialization_message() {
    let json = r#"{"type":"message","content":"Hello"}"#;
    let event: StreamEvent = serde_json::from_str(json).unwrap();
    
    match event {
        StreamEvent::Message { content } => assert_eq!(content, "Hello"),
        _ => panic!("Expected Message variant"),
    }
}

#[test]
fn test_stream_event_deserialization_tool_call() {
    let json = r#"{"type":"tool_call","index":0,"id":"call_1","name":"test","arguments":"{}"}"#;
    let event: StreamEvent = serde_json::from_str(json).unwrap();
    
    match event {
        StreamEvent::ToolCall { index, .. } => assert_eq!(index, 0),
        _ => panic!("Expected ToolCall variant"),
    }
}

#[test]
fn test_stream_event_clone() {
    let event = StreamEvent::Message {
        content: "Original".to_string(),
    };
    
    let cloned = event.clone();
    
    match (event, cloned) {
        (StreamEvent::Message { content: c1 }, StreamEvent::Message { content: c2 }) => {
            assert_eq!(c1, c2);
        },
        _ => panic!("Clone should preserve variant"),
    }
}

#[test]
fn test_stream_event_debug() {
    let event = StreamEvent::Message {
        content: "Debug test".to_string(),
    };
    
    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("Message"));
}

#[test]
fn test_stream_event_tool_call_partial() {
    // Tool calls can come in chunks with partial data
    let event = StreamEvent::ToolCall {
        index: 0,
        id: None,
        name: Some("get_weather".to_string()),
        arguments: None,
    };
    
    match event {
        StreamEvent::ToolCall { id, name, arguments, .. } => {
            assert_eq!(id, None);
            assert!(name.is_some());
            assert_eq!(arguments, None);
        },
        _ => panic!("Expected ToolCall variant"),
    }
}

#[test]
fn test_stream_event_done_no_reason() {
    let event = StreamEvent::Done {
        finish_reason: None,
    };
    
    match event {
        StreamEvent::Done { finish_reason } => {
            assert_eq!(finish_reason, None);
        },
        _ => panic!("Expected Done variant"),
    }
}

