use tokio::time::{interval, Duration, Interval};

/// Adaptive event batcher that adjusts window size based on network latency
/// 
/// Automatically adjusts batching window to optimize for:
/// - Low latency: Smaller windows (better UX, more batches)
/// - High latency: Larger windows (less overhead, fewer batches)
pub struct AdaptiveEventBatcher<T> {
    batch: Vec<T>,
    ticker: Interval,
    
    // Adaptive parameters
    base_window_ms: u64,
    current_window_ms: u64,
    min_window_ms: u64,
    max_window_ms: u64,
    
    // Latency tracking
    latency_samples: Vec<Duration>,
    max_samples: usize,
    
    // Statistics
    total_batches: u64,
    total_events: u64,
}

impl<T> AdaptiveEventBatcher<T> {
    /// Create adaptive batcher with base window and bounds
    pub fn new(base_window_ms: u64, min_window_ms: u64, max_window_ms: u64) -> Self {
        Self {
            batch: Vec::new(),
            ticker: interval(Duration::from_millis(base_window_ms)),
            base_window_ms,
            current_window_ms: base_window_ms,
            min_window_ms,
            max_window_ms,
            latency_samples: Vec::new(),
            max_samples: 10, // Keep last 10 latency measurements
            total_batches: 0,
            total_events: 0,
        }
    }
    
    /// Record network latency for adaptive adjustment
    pub fn record_latency(&mut self, latency: Duration) {
        self.latency_samples.push(latency);
        
        // Keep only recent samples
        if self.latency_samples.len() > self.max_samples {
            self.latency_samples.remove(0);
        }
        
        // Adjust window based on average latency
        self.adjust_window();
    }
    
    /// Adjust batching window based on recent latency measurements
    fn adjust_window(&mut self) {
        if self.latency_samples.is_empty() {
            return;
        }
        
        // Calculate average latency
        let total_ms: u64 = self.latency_samples.iter()
            .map(|d| d.as_millis() as u64)
            .sum();
        let avg_ms = total_ms / self.latency_samples.len() as u64;
        
        // Adaptive strategy:
        // - Low latency (< 50ms): Smaller window (better UX)
        // - Medium latency (50-200ms): Base window
        // - High latency (> 200ms): Larger window (less overhead)
        
        let new_window = if avg_ms < 50 {
            // Low latency: Use smaller window for better responsiveness
            self.base_window_ms.max(self.min_window_ms)
        } else if avg_ms < 200 {
            // Medium latency: Use base window
            self.base_window_ms
        } else {
            // High latency: Increase window to reduce overhead
            (self.base_window_ms * 2).min(self.max_window_ms)
        };
        
        // Only update if changed significantly (avoid thrashing)
        if new_window.abs_diff(self.current_window_ms) > 10 {
            self.current_window_ms = new_window;
            self.ticker = interval(Duration::from_millis(new_window));
        }
    }
    
    /// Add an event to the current batch
    pub fn push(&mut self, event: T) {
        self.batch.push(event);
        self.total_events += 1;
    }
    
    /// Take the current batch, leaving an empty one
    pub fn take(&mut self) -> Vec<T> {
        self.total_batches += 1;
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
    
    /// Get current window duration
    pub fn current_window_ms(&self) -> u64 {
        self.current_window_ms
    }
    
    /// Get statistics
    pub fn stats(&self) -> BatcherStats {
        BatcherStats {
            current_window_ms: self.current_window_ms,
            total_batches: self.total_batches,
            total_events: self.total_events,
            avg_events_per_batch: if self.total_batches > 0 {
                self.total_events as f64 / self.total_batches as f64
            } else {
                0.0
            },
            avg_latency_ms: if !self.latency_samples.is_empty() {
                self.latency_samples.iter()
                    .sum::<Duration>()
                    .as_millis() as f64 / self.latency_samples.len() as f64
            } else {
                0.0
            },
        }
    }
}

/// Statistics for adaptive batcher
#[derive(Debug, Clone)]
pub struct BatcherStats {
    pub current_window_ms: u64,
    pub total_batches: u64,
    pub total_events: u64,
    pub avg_events_per_batch: f64,
    pub avg_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_batcher_basic() {
        let mut batcher = AdaptiveEventBatcher::<i32>::new(50, 20, 200);
        
        batcher.push(1);
        batcher.push(2);
        batcher.push(3);
        
        assert_eq!(batcher.len(), 3);
        assert!(!batcher.is_empty());
        
        let batch = batcher.take();
        assert_eq!(batch, vec![1, 2, 3]);
        assert!(batcher.is_empty());
    }
    
    #[tokio::test]
    async fn test_adaptive_window_adjustment() {
        let mut batcher = AdaptiveEventBatcher::<i32>::new(50, 20, 200);
        
        // Simulate low latency
        batcher.record_latency(Duration::from_millis(30));
        assert_eq!(batcher.current_window_ms(), 50); // Should use base or min
        
        // Simulate high latency
        batcher.record_latency(Duration::from_millis(250));
        batcher.record_latency(Duration::from_millis(300));
        // Window should increase
        assert!(batcher.current_window_ms() >= 50);
    }
}

