/// Test: GPT-5 Streaming Analysis
/// 
/// Este exemplo faz uma chamada real ao GPT-5 e analisa EXATAMENTE
/// como os chunks chegam, incluindo campos de reasoning.

use praxis_llm_openai::{OpenAIClient, ChatOptions, Message};
use futures::StreamExt;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = "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";
    
    println!("🔬 GPT-5 Streaming Analysis\n");
    println!("=" .repeat(80));
    
    let client = OpenAIClient::new(api_key)?;
    
    let messages = vec![
        Message::system("You are a helpful assistant that thinks step by step."),
        Message::human("Explain how to calculate 157 * 23 step by step."),
    ];
    
    let options = ChatOptions::new()
        .temperature(0.7)
        .max_tokens(500);
    
    println!("📤 Sending request to GPT-5 with streaming enabled...\n");
    
    let mut stream = client.chat_completion_stream(
        "gpt-5",  // Testando GPT-5
        messages,
        options
    ).await?;
    
    let mut chunk_count = 0;
    let start = std::time::Instant::now();
    
    println!("📊 Raw Chunks (first 20):\n");
    println!("{}", "-".repeat(80));
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                chunk_count += 1;
                
                // Print detalhes dos primeiros 20 chunks
                if chunk_count <= 20 {
                    println!("\n🔹 Chunk #{}", chunk_count);
                    println!("   Timestamp: {:?}", start.elapsed());
                    println!("   Raw JSON: {}", serde_json::to_string_pretty(&chunk)?);
                    
                    // Analisar campos específicos
                    if let Some(content) = chunk.content() {
                        println!("   ✅ Content: {:?}", content);
                    }
                    
                    // Verificar se tem reasoning (campo novo?)
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(ref tool_calls) = choice.delta.tool_calls {
                            println!("   🔧 Tool calls: {:?}", tool_calls);
                        }
                        
                        // Imprimir TODOS os campos do delta para descobrir novos
                        println!("   📋 Delta fields:");
                        println!("      - role: {:?}", choice.delta.role);
                        println!("      - content: {:?}", choice.delta.content);
                        println!("      - tool_calls: {:?}", choice.delta.tool_calls);
                    }
                    
                    if chunk.is_done() {
                        println!("   🏁 Stream finished!");
                        println!("   Finish reason: {:?}", chunk.choices.first().unwrap().finish_reason);
                    }
                }
                
                // Print content para ver resposta completa
                if let Some(content) = chunk.content() {
                    print!("{}", content);
                    std::io::Write::flush(&mut std::io::stdout()).ok();
                }
                
                if chunk.is_done() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("\n❌ Error: {}", e);
                break;
            }
        }
    }
    
    println!("\n\n{}", "=".repeat(80));
    println!("📊 Summary:");
    println!("   Total chunks: {}", chunk_count);
    println!("   Total time: {:?}", start.elapsed());
    println!("   Avg time per chunk: {:?}", start.elapsed() / chunk_count);
    
    Ok(())
}
