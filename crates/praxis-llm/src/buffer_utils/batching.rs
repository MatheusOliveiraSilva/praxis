use tokio::time::{interval, Duration, Interval};

/// Event batcher with time-based debouncing
/// Accumulates events and flushes them periodically
pub struct EventBatcher<T> {
    batch: Vec<T>,
    ticker: Interval,
    window_ms: u64,
}

impl<T> EventBatcher<T> {
    /// Create a new batcher with specified time window (milliseconds)
    pub fn new(window_ms: u64) -> Self {
        Self {
            batch: Vec::new(),
            ticker: interval(Duration::from_millis(window_ms)),
            window_ms,
        }
    }

    /// Add an event to the current batch
    pub fn push(&mut self, event: T) {
        self.batch.push(event);
    }

    /// Check if it's time to flush (non-blocking)
    /// Note: This is a simplified check. For production use, integrate with tokio::select!
    pub fn should_flush_now(&self) -> bool {
        // Simplified: just check if batch is not empty
        // Real flush timing is handled by ticker in tokio::select!
        !self.batch.is_empty()
    }

    /// Take the current batch, leaving an empty one
    pub fn take(&mut self) -> Vec<T> {
        std::mem::take(&mut self.batch)
    }

    /// Current batch size
    pub fn len(&self) -> usize {
        self.batch.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    /// Get reference to ticker for use in tokio::select!
    pub fn ticker(&mut self) -> &mut Interval {
        &mut self.ticker
    }

    /// Get window duration
    pub fn window_ms(&self) -> u64 {
        self.window_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batcher_basic() {
        let mut batcher = EventBatcher::<i32>::new(50);
        
        batcher.push(1);
        batcher.push(2);
        batcher.push(3);
        
        assert_eq!(batcher.len(), 3);
        assert!(!batcher.is_empty());
        
        let batch = batcher.take();
        assert_eq!(batch, vec![1, 2, 3]);
        assert!(batcher.is_empty());
    }
}

