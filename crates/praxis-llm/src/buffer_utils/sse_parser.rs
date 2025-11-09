use anyhow::Result;
use futures::{Stream, StreamExt};
use reqwest::Response;
use std::pin::Pin;

use super::buffering::CircularLineBuffer;
use crate::StreamEvent;

/// Strategy pattern for parsing different SSE response types
pub trait SseLineParser: Send {
    /// Parse a data line into stream events
    fn parse_data_line(&self, data: &str) -> Result<Vec<StreamEvent>>;
    
    /// Check if this line signals end of stream
    fn is_done_marker(&self, data: &str) -> bool {
        data == "[DONE]"
    }
}

/// Generic SSE stream parser using circular buffer
/// Applies strategy pattern for different response types
pub fn parse_sse_stream<P: SseLineParser + 'static>(
    response: Response,
    parser: P,
) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
    let stream = response.bytes_stream();
    
    Box::pin(async_stream::stream! {
        let mut byte_chunks = Box::pin(stream);
        let mut buffer = CircularLineBuffer::with_capacity(4096);
        
        while let Some(chunk_result) = byte_chunks.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.extend(&bytes);
                    
                    // Process all complete lines in buffer
                    while let Some(line_result) = buffer.next_line() {
                        match line_result {
                            Ok(line) => {
                                if line.is_empty() {
                                    continue;
                                }
                                
                                // Parse SSE data lines
                                if let Some(data) = line.strip_prefix("data: ") {
                                    // Check for done marker
                                    if parser.is_done_marker(data) {
                                        yield Ok(StreamEvent::Done { finish_reason: None });
                                        break;
                                    }
                                    
                                    // Parse data using strategy
                                    match parser.parse_data_line(data) {
                                        Ok(events) => {
                                            for event in events {
                                                yield Ok(event);
                                            }
                                        }
                                        Err(e) => yield Err(e),
                                    }
                                }
                            }
                            Err(e) => yield Err(e),
                        }
                    }
                }
                Err(e) => yield Err(anyhow::anyhow!("Stream error: {}", e)),
            }
        }
    })
}

