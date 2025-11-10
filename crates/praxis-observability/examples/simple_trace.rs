use std::sync::Arc;
use praxis_observability::{LangfuseObserver, NodeObservation, NodeObservationData, LangfuseMessage, ToolCallInfo, ToolResultInfo};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create Langfuse observer
    let observer = Arc::new(LangfuseObserver::new(
        std::env::var("LANGFUSE_PUBLIC_KEY").unwrap_or_else(|_| "pk-test".to_string()),
        std::env::var("LANGFUSE_SECRET_KEY").unwrap_or_else(|_| "sk-test".to_string()),
        std::env::var("LANGFUSE_HOST").unwrap_or_else(|_| "https://cloud.langfuse.com".to_string()),
    )?);

    let run_id = uuid::Uuid::new_v4().to_string();
    let conversation_id = "test-conversation".to_string();

    // Start trace
    println!("Starting trace for run: {}", run_id);
    observer.trace_start(run_id.clone(), conversation_id.clone()).await?;

    // Simulate LLM node execution
    println!("Tracing LLM node execution...");
    let llm_observation = NodeObservation {
        span_id: uuid::Uuid::new_v4().to_string(),
        run_id: run_id.clone(),
        conversation_id: conversation_id.clone(),
        node_type: "llm".to_string(),
        started_at: chrono::Utc::now(),
        duration_ms: 1500,
        data: NodeObservationData::Llm {
            input_messages: vec![
                LangfuseMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                },
                LangfuseMessage {
                    role: "user".to_string(),
                    content: "What's the weather like?".to_string(),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                },
            ],
            output_message: LangfuseMessage {
                role: "assistant".to_string(),
                content: String::new(),
                name: None,
                tool_call_id: None,
                tool_calls: Some(vec![ToolCallInfo {
                    id: "call_123".to_string(),
                    name: "get_weather".to_string(),
                    arguments: serde_json::json!({
                        "location": "San Francisco"
                    }),
                }]),
            },
            model: "gpt-4".to_string(),
            usage: None,
        },
        metadata: std::collections::HashMap::new(),
    };

    observer.trace_llm_node(llm_observation).await?;

    // Simulate tool node execution
    println!("Tracing tool node execution...");
    let tool_observation = NodeObservation {
        span_id: uuid::Uuid::new_v4().to_string(),
        run_id: run_id.clone(),
        conversation_id: conversation_id.clone(),
        node_type: "tool".to_string(),
        started_at: chrono::Utc::now(),
        duration_ms: 500,
        data: NodeObservationData::Tool {
            tool_calls: vec![ToolCallInfo {
                id: "call_123".to_string(),
                name: "get_weather".to_string(),
                arguments: serde_json::json!({
                    "location": "San Francisco"
                }),
            }],
            tool_results: vec![ToolResultInfo {
                tool_call_id: "call_123".to_string(),
                tool_name: "get_weather".to_string(),
                result: "Sunny, 72°F".to_string(),
                is_error: false,
                duration_ms: 450,
            }],
        },
        metadata: std::collections::HashMap::new(),
    };

    observer.trace_tool_node(tool_observation).await?;

    // End trace
    println!("Ending trace...");
    observer.trace_end(run_id.clone(), "success".to_string(), 2000).await?;

    println!("Trace completed successfully!");
    println!("Check your Langfuse dashboard for the trace: {}", run_id);

    Ok(())
}

