use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::observer::Observer;
use crate::types::{NodeObservation, NodeObservationData, NodeOutput};
use super::client::LangfuseClient;
use super::types::{GenerationBody, IngestionBatch, IngestionEvent, SpanBody, TraceBody, UsageInfo};

/// Langfuse implementation of the Observer trait
/// 
/// Sends trace data to Langfuse for observability and monitoring.
/// Uses async fire-and-forget pattern to avoid blocking execution.
pub struct LangfuseObserver {
    client: Arc<LangfuseClient>,
    /// Stores trace IDs for active runs
    traces: Arc<Mutex<HashMap<String, String>>>,
}

impl LangfuseObserver {
    /// Create a new Langfuse observer
    /// 
    /// # Arguments
    /// * `public_key` - Langfuse public API key
    /// * `secret_key` - Langfuse secret API key
    /// * `host` - Langfuse host URL (e.g., "https://cloud.langfuse.com")
    pub fn new(public_key: String, secret_key: String, host: String) -> Result<Self> {
        let client = LangfuseClient::new(public_key, secret_key, host)
            .context("Failed to create Langfuse client")?;

        Ok(Self {
            client: Arc::new(client),
            traces: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get or create trace ID for a run
    fn get_or_create_trace_id(&self, run_id: &str) -> String {
        let traces = self.traces.lock().unwrap();
        traces.get(run_id).cloned().unwrap_or_else(|| {
            uuid::Uuid::new_v4().to_string()
        })
    }

    /// Store trace ID for a run
    fn store_trace_id(&self, run_id: String, trace_id: String) {
        let mut traces = self.traces.lock().unwrap();
        traces.insert(run_id, trace_id);
    }

    /// Remove trace ID after completion
    fn remove_trace_id(&self, run_id: &str) {
        let mut traces = self.traces.lock().unwrap();
        traces.remove(run_id);
    }

    /// Convert observation to Langfuse format for LLM nodes (Chain of Responsibility Pattern)
    /// 
    /// Creates separate generation traces for each output (reasoning, message, tool_calls)
    async fn trace_llm_generation(&self, observation: NodeObservation) -> Result<()> {
        let trace_id = self.get_or_create_trace_id(&observation.run_id);

        match observation.data {
            NodeObservationData::Llm {
                input_messages,
                outputs,
                model,
                usage,
            } => {
                tracing::info!(
                    "Preparing generation for Langfuse - input_messages: {}, outputs: {}",
                    input_messages.len(),
                    outputs.len()
                );
                
                let input_json = serde_json::to_value(&input_messages)
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to serialize input_messages: {}", e);
                        serde_json::json!([])
                    });
                
                // Create a generation for each output
                for (i, output) in outputs.iter().enumerate() {
                    let generation_id = if outputs.len() == 1 {
                        observation.span_id.clone()
                    } else {
                        format!("{}-gen-{}", observation.span_id, i)
                    };
                    
                    let (generation_name, output_json, metadata) = match output {
                        NodeOutput::Reasoning { id, content } => {
                            let mut metadata = observation.metadata.clone();
                            metadata.insert("openai_id".to_string(), serde_json::json!(id));
                            metadata.insert("output_type".to_string(), serde_json::json!("reasoning"));
                            
                            ("reasoning".to_string(), serde_json::json!({
                                "reasoning": content
                            }), metadata)
                        }
                        NodeOutput::Message { id, content } => {
                            let mut metadata = observation.metadata.clone();
                            metadata.insert("openai_id".to_string(), serde_json::json!(id));
                            metadata.insert("output_type".to_string(), serde_json::json!("message"));
                            
                            ("message".to_string(), serde_json::json!({
                                "content": content
                            }), metadata)
                        }
                        NodeOutput::ToolCalls { calls } => {
                            let mut metadata = observation.metadata.clone();
                            metadata.insert("output_type".to_string(), serde_json::json!("tool_calls"));
                            
                            ("tool_calls".to_string(), serde_json::json!({
                                "tool_calls": calls
                            }), metadata)
                        }
                    };
                    
                    tracing::info!(
                        "Creating generation {} for {}: input_len={}, output_json={}",
                        generation_id,
                        generation_name,
                        input_messages.len(),
                        serde_json::to_string(&output_json).unwrap_or_default()
                    );

                    let generation_body = GenerationBody {
                        id: generation_id.clone(),
                        trace_id: trace_id.clone(),
                        name: generation_name,
                        start_time: observation.started_at.to_rfc3339(),
                        end_time: Some(
                            (observation.started_at
                                + chrono::Duration::milliseconds(observation.duration_ms as i64))
                            .to_rfc3339(),
                        ),
                        model: model.clone(),
                        model_parameters: None,
                        input: Some(input_json.clone()),
                        output: Some(output_json),
                        metadata: if metadata.is_empty() {
                            None
                        } else {
                            Some(metadata)
                        },
                        level: Some("DEFAULT".to_string()),
                        status_message: None,
                        usage: if i == outputs.len() - 1 {
                            // Only attach usage to the last generation to avoid duplication
                            // Convert from praxis-llm TokenUsage format to Langfuse format
                            usage.clone().map(|u| UsageInfo {
                                prompt_tokens: Some(u.input_tokens),
                                completion_tokens: Some(u.output_tokens),
                                total_tokens: Some(u.total_tokens),
                            })
                        } else {
                            None
                        },
                    };

                    tracing::debug!(
                        "Sending generation to Langfuse - input_messages_count: {}, has_output: {}",
                        input_messages.len(),
                        !input_messages.is_empty()
                    );

                    // Create batch ingestion event for this generation
                    let now = chrono::Utc::now();
                    let event = IngestionEvent {
                        id: format!("{}-generation-event", generation_id),
                        timestamp: now.to_rfc3339(),
                        event_type: "generation-create".to_string(),
                        body: serde_json::to_value(&generation_body)
                            .context("Failed to serialize generation body")?,
                    };

                    let batch = IngestionBatch {
                        batch: vec![event],
                    };

                    self.client.ingest_batch(batch).await?;
                    
                    tracing::info!("Sent generation {} to Langfuse", generation_id);
                }
            }
            _ => {
                anyhow::bail!("Expected LLM observation data, got Tool data");
            }
        }

        Ok(())
    }

    /// Convert observation to Langfuse format for tool nodes
    async fn trace_tool_span(&self, observation: NodeObservation) -> Result<()> {
        let trace_id = self.get_or_create_trace_id(&observation.run_id);

        match observation.data {
            NodeObservationData::Tool {
                tool_calls,
                tool_results,
            } => {
                let span_body = SpanBody {
                    id: observation.span_id.clone(),
                    trace_id: trace_id.clone(),
                    name: "tool_node".to_string(),
                    start_time: observation.started_at.to_rfc3339(),
                    end_time: Some(
                        (observation.started_at
                            + chrono::Duration::milliseconds(observation.duration_ms as i64))
                        .to_rfc3339(),
                    ),
                    metadata: if observation.metadata.is_empty() {
                        None
                    } else {
                        Some(observation.metadata)
                    },
                    level: Some("DEFAULT".to_string()),
                    status_message: None,
                    input: Some(serde_json::json!({
                        "tool_calls": tool_calls,
                    })),
                    output: Some(serde_json::json!({
                        "tool_results": tool_results,
                    })),
                };

                // Create batch ingestion event
                let now = chrono::Utc::now();
                let event = IngestionEvent {
                    id: format!("{}-span-event", observation.span_id),
                    timestamp: now.to_rfc3339(),
                    event_type: "span-create".to_string(),
                    body: serde_json::to_value(&span_body)
                        .context("Failed to serialize span body")?,
                };

                let batch = IngestionBatch {
                    batch: vec![event],
                };

                self.client.ingest_batch(batch).await?;
            }
            _ => {
                anyhow::bail!("Expected Tool observation data, got LLM data");
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Observer for LangfuseObserver {
    async fn trace_start(&self, run_id: String, conversation_id: String) -> Result<()> {
        let trace_id = uuid::Uuid::new_v4().to_string();
        
        tracing::info!(
            "Starting Langfuse trace: trace_id={}, run_id={}, conversation_id={}",
            trace_id,
            run_id,
            conversation_id
        );
        
        let now = chrono::Utc::now();
        let trace_body = TraceBody {
            id: trace_id.clone(),
            name: Some(format!("agent_run_{}", &run_id[..8])),
            user_id: Some(conversation_id.clone()),
            metadata: Some({
                let mut map = HashMap::new();
                map.insert("run_id".to_string(), serde_json::json!(run_id));
                map.insert(
                    "conversation_id".to_string(),
                    serde_json::json!(conversation_id),
                );
                map
            }),
            tags: Some(vec!["praxis".to_string(), "agent".to_string()]),
            timestamp: Some(now.to_rfc3339()),
        };

        // Store trace ID for this run
        self.store_trace_id(run_id.clone(), trace_id.clone());

        // Create batch ingestion event
        let event = IngestionEvent {
            id: format!("{}-trace-event", trace_id),
            timestamp: now.to_rfc3339(),
            event_type: "trace-create".to_string(),
            body: serde_json::to_value(&trace_body)
                .context("Failed to serialize trace body")?,
        };

        let batch = IngestionBatch {
            batch: vec![event],
        };

        // Send trace creation request
        match self.client.ingest_batch(batch).await {
            Ok(_) => {
                tracing::info!("Langfuse trace created successfully: trace_id={}", trace_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to create Langfuse trace: {}", e);
                Err(e)
            }
        }
    }

    async fn trace_llm_node(&self, observation: NodeObservation) -> Result<()> {
        tracing::debug!(
            "Tracing LLM node: span_id={}, run_id={}",
            observation.span_id,
            observation.run_id
        );
        
        match self.trace_llm_generation(observation).await {
            Ok(_) => {
                tracing::info!("LLM node traced successfully in Langfuse");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to trace LLM node: {}", e);
                Err(e)
            }
        }
    }

    async fn trace_tool_node(&self, observation: NodeObservation) -> Result<()> {
        tracing::debug!(
            "Tracing tool node: span_id={}, run_id={}",
            observation.span_id,
            observation.run_id
        );
        
        match self.trace_tool_span(observation).await {
            Ok(_) => {
                tracing::info!("Tool node traced successfully in Langfuse");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to trace tool node: {}", e);
                Err(e)
            }
        }
    }

    async fn trace_end(&self, run_id: String, status: String, total_duration_ms: u64) -> Result<()> {
        let trace_id = self.get_or_create_trace_id(&run_id);

        tracing::info!(
            "Ending Langfuse trace: trace_id={}, run_id={}, status={}, duration_ms={}",
            trace_id,
            run_id,
            status,
            total_duration_ms
        );

        let now = chrono::Utc::now();
        let update_body = TraceBody {
            id: trace_id.clone(),
            name: Some(format!("agent_run_{}", &run_id[..8.min(run_id.len())])),
            user_id: None,
            metadata: Some({
                let mut map = HashMap::new();
                map.insert("status".to_string(), serde_json::json!(status));
                map.insert(
                    "total_duration_ms".to_string(),
                    serde_json::json!(total_duration_ms),
                );
                map
            }),
            tags: Some(vec!["praxis".to_string(), "completed".to_string()]),
            timestamp: None,
        };

        // Create batch ingestion event
        let event = IngestionEvent {
            id: format!("{}-trace-update-event", trace_id),
            timestamp: now.to_rfc3339(),
            event_type: "trace-create".to_string(), // Updates also use trace-create
            body: serde_json::to_value(&update_body)
                .context("Failed to serialize trace update body")?,
        };

        let batch = IngestionBatch {
            batch: vec![event],
        };

        match self.client.ingest_batch(batch).await {
            Ok(_) => {
                tracing::info!("Langfuse trace finalized successfully: trace_id={}", trace_id);
            }
            Err(e) => {
                tracing::error!("Failed to finalize Langfuse trace: {}", e);
                return Err(e);
            }
        }

        // Clean up stored trace ID
        self.remove_trace_id(&run_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observer_creation() {
        let observer = LangfuseObserver::new(
            "pk-test".to_string(),
            "sk-test".to_string(),
            "https://cloud.langfuse.com".to_string(),
        );
        
        assert!(observer.is_ok());
    }
}

