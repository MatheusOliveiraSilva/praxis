use anyhow::Result;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Azure OpenAI Raw Streaming Debug ===\n");

    // Load Azure OpenAI configuration
    let api_key = std::env::var("AZURE_OPENAI_API_KEY")?;
    let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")?;
    let api_version = std::env::var("AZURE_OPENAI_API_VERSION")
        .unwrap_or_else(|_| "2024-02-15-preview".to_string());
    let deployment_name = "gpt-5";

    println!("Endpoint: {}", endpoint);
    println!("Deployment: {}", deployment_name);
    println!("API Version: {}\n", api_version);

    // Create raw output files
    let mut raw_bytes_file = File::create("azure_raw_bytes.txt")?;
    let mut raw_lines_file = File::create("azure_raw_lines.txt")?;
    let mut parsed_json_file = File::create("azure_parsed_json.txt")?;

    writeln!(raw_bytes_file, "=== Raw Bytes Stream ===\n")?;
    writeln!(raw_lines_file, "=== Raw Lines (SSE Format) ===\n")?;
    writeln!(parsed_json_file, "=== Parsed JSON Objects ===\n")?;

    // Build headers
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("api-key", HeaderValue::from_str(&api_key)?);

    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    // Build request
    let url = format!(
        "{}/openai/deployments/{}/chat/completions?api-version={}",
        endpoint, deployment_name, api_version
    );

    let payload = serde_json::json!({
        "messages": [
            {
                "role": "user",
                "content": "Explain quantum computing in 3 sentences."
            }
        ],
        "stream": true,
        "reasoning_effort": "medium",
        "max_completion_tokens": 500
    });

    println!("Request payload:");
    println!("{}\n", serde_json::to_string_pretty(&payload)?);
    println!("Sending request...\n");
    println!("---\n");

    // Send request
    let response = http_client
        .post(&url)
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        anyhow::bail!("API error ({}): {}", status, error_text);
    }

    // Process stream
    let mut stream = response.bytes_stream();
    let mut chunk_number = 0;
    let mut line_buffer = String::new();
    let mut event_number = 0;

    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        chunk_number += 1;

        // Save raw bytes
        writeln!(raw_bytes_file, "--- Chunk #{} ({} bytes) ---", chunk_number, bytes.len())?;
        writeln!(raw_bytes_file, "{:?}", bytes)?;
        writeln!(raw_bytes_file, "")?;

        // Convert to string and process lines
        let text = String::from_utf8_lossy(&bytes);
        line_buffer.push_str(&text);

        // Process complete lines
        while let Some(line_end) = line_buffer.find('\n') {
            let line = line_buffer[..line_end].trim_end_matches('\r').to_string();
            line_buffer = line_buffer[line_end + 1..].to_string();

            if !line.is_empty() {
                writeln!(raw_lines_file, "Line: {}", line)?;

                // Try to parse SSE format
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    
                    if data == "[DONE]" {
                        writeln!(raw_lines_file, "  -> [DONE] signal\n")?;
                        writeln!(parsed_json_file, "--- [DONE] signal ---\n")?;
                    } else {
                        // Try to parse as JSON
                        match serde_json::from_str::<serde_json::Value>(data) {
                            Ok(json) => {
                                event_number += 1;
                                writeln!(raw_lines_file, "  -> Valid JSON event #{}\n", event_number)?;
                                writeln!(parsed_json_file, "--- Event #{} ---", event_number)?;
                                writeln!(parsed_json_file, "{}\n", serde_json::to_string_pretty(&json)?)?;
                                
                                // Print to console
                                if let Some(choices) = json["choices"].as_array() {
                                    if let Some(choice) = choices.first() {
                                        if let Some(delta) = choice["delta"].as_object() {
                                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                                print!("{}", content);
                                                std::io::stdout().flush()?;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                writeln!(raw_lines_file, "  -> Parse error: {}\n", e)?;
                            }
                        }
                    }
                } else {
                    writeln!(raw_lines_file, "  -> Not a data line\n")?;
                }
            }
        }

        raw_bytes_file.flush()?;
        raw_lines_file.flush()?;
        parsed_json_file.flush()?;
    }

    println!("\n\n---\n");
    println!("✓ Debug complete!");
    println!("\n📁 Files created:");
    println!("  - azure_raw_bytes.txt     (raw byte chunks)");
    println!("  - azure_raw_lines.txt     (SSE formatted lines)");
    println!("  - azure_parsed_json.txt   (parsed JSON events)");
    println!("\nTotal chunks received: {}", chunk_number);
    println!("Total events parsed: {}", event_number);

    Ok(())
}

