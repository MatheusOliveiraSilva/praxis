use anyhow::Result;
use std::collections::VecDeque;

/// Circular buffer for efficient line-based parsing
/// Uses VecDeque for zero-copy line extraction
pub struct CircularLineBuffer {
    buffer: VecDeque<u8>,
}

impl CircularLineBuffer {
    /// Create a new buffer with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    /// Add bytes to the buffer
    pub fn extend(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes);
    }

    /// Extract next line (up to \n) from buffer
    /// Returns None if no complete line is available
    pub fn next_line(&mut self) -> Option<Result<String>> {
        // Find newline position
        let newline_pos = self.buffer.iter().position(|&b| b == b'\n')?;

        // Drain bytes up to and including newline (zero-copy!)
        let line_bytes: Vec<u8> = self.buffer.drain(..=newline_pos).collect();

        // Convert to UTF-8 string
        match std::str::from_utf8(&line_bytes) {
            Ok(line_str) => Some(Ok(line_str.trim().to_string())),
            Err(e) => Some(Err(anyhow::anyhow!("Invalid UTF-8: {}", e))),
        }
    }

    /// Current buffer size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_buffer_basic() {
        let mut buffer = CircularLineBuffer::with_capacity(64);
        
        buffer.extend(b"line1\nline2\n");
        
        assert_eq!(buffer.next_line().unwrap().unwrap(), "line1");
        assert_eq!(buffer.next_line().unwrap().unwrap(), "line2");
        assert!(buffer.next_line().is_none());
    }

    #[test]
    fn test_partial_line() {
        let mut buffer = CircularLineBuffer::with_capacity(64);
        
        buffer.extend(b"partial");
        assert!(buffer.next_line().is_none());
        
        buffer.extend(b" line\n");
        assert_eq!(buffer.next_line().unwrap().unwrap(), "partial line");
    }
}

