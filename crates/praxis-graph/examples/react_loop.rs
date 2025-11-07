use anyhow::Result;
use praxis_graph::{Graph, GraphConfig, GraphInput, LLMConfig, MockToolExecutor, StreamEvent};
use praxis_llm::{Content, Message, OpenAIClient};
use std::io::{self, Write};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         Praxis React Agent - Interactive Demo             ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("This demo shows a React agent that can:");
    println!("  • Think through problems (reasoning)");
    println!("  • Use tools (calculator, weather, search)");
    println!("  • Respond to your questions");
    println!();
    println!("Type 'exit' to quit");
    println!();

    // Get API key from environment
    let api_key = std::env::var("OPENAI_API_KEY").expect(
        "OPENAI_API_KEY must be set in environment. Run: export OPENAI_API_KEY=your_key_here"
    );

    // Create LLM client
    let llm_client = Arc::new(OpenAIClient::new(api_key)?);

    // Create tool executor (mock for now)
    let tool_executor = Arc::new(MockToolExecutor);

    // Create graph config
    let config = GraphConfig::default();

    // Create graph
    let graph = Graph::new(llm_client, tool_executor, config);

    // Conversation loop
    let conversation_id = uuid::Uuid::new_v4().to_string();
    
    loop {
        // Get user input
        print!("\n\x1b[1;36m You: \x1b[0m");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("exit") {
            println!("\n👋 Goodbye!");
            break;
        }

        // Create message
        let user_message = Message::Human {
            content: Content::text(input),
            name: None,
        };

        // Create graph input
        let llm_config = LLMConfig::new("gpt-4")
            .with_temperature(0.7)
            .with_max_tokens(4096);

        let graph_input = GraphInput::new(conversation_id.clone(), user_message, llm_config);

        // Spawn execution
        let mut event_rx = graph.spawn_run(graph_input);

        // Print assistant label
        print!("\n\x1b[1;32mAssistant:\x1b[0m ");
        io::stdout().flush()?;

        let mut in_reasoning = false;
        let mut in_message = false;

        // Process events
        while let Some(event) = event_rx.recv().await {
            match event {
                StreamEvent::InitStream { .. } => {
                    // Silent - just track
                }

                StreamEvent::Reasoning { content } => {
                    if !in_reasoning {
                        print!("\n\x1b[2;3m💭 Reasoning: ");
                        in_reasoning = true;
                        in_message = false;
                    }
                    print!("{}", content);
                    io::stdout().flush()?;
                }

                StreamEvent::Message { content } => {
                    if !in_message {
                        if in_reasoning {
                            print!("\x1b[0m\n\n");
                        }
                        print!("\x1b[0m");
                        in_message = true;
                        in_reasoning = false;
                    }
                    print!("{}", content);
                    io::stdout().flush()?;
                }

                StreamEvent::ToolCall {
                    index: _,
                    id: _,
                    name,
                    arguments,
                } => {
                    if in_reasoning {
                        print!("\x1b[0m\n");
                    }
                    if let Some(name) = name {
                        if let Some(args) = arguments {
                            print!("\n\x1b[1;33m🔧 Calling tool: {} ({})\x1b[0m", name, args);
                        } else {
                            print!("\n\x1b[1;33m🔧 Calling tool: {}\x1b[0m", name);
                        }
                        io::stdout().flush()?;
                    }
                    in_reasoning = false;
                    in_message = false;
                }

                StreamEvent::ToolResult {
                    tool_call_id: _,
                    result,
                    is_error,
                    duration_ms,
                } => {
                    if is_error {
                        print!(
                            "\n\x1b[1;31m❌ Tool error ({}ms): {}\x1b[0m",
                            duration_ms, result
                        );
                    } else {
                        // Truncate long results
                        let display_result = if result.len() > 100 {
                            format!("{}...", &result[..100])
                        } else {
                            result
                        };
                        print!(
                            "\n\x1b[1;32m✓ Tool result ({}ms): {}\x1b[0m",
                            duration_ms, display_result
                        );
                    }
                    io::stdout().flush()?;
                    in_reasoning = false;
                    in_message = false;
                }

                StreamEvent::Done { finish_reason: _ } => {
                    // LLM stream done, continue to next node
                }

                StreamEvent::Error { message, .. } => {
                    print!("\n\n\x1b[1;31m❌ Error: {}\x1b[0m", message);
                    io::stdout().flush()?;
                    break;
                }

                StreamEvent::EndStream {
                    status: _,
                    total_duration_ms,
                } => {
                    print!("\n\n\x1b[2m[Completed in {}ms]\x1b[0m", total_duration_ms);
                    io::stdout().flush()?;
                    break;
                }
            }
        }

        println!(); // Final newline
    }

    Ok(())
}

