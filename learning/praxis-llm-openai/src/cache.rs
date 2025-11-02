// HTTP Response Cache (in-memory, TTL-based)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cached response with expiration
#[derive(Debug, Clone)]
struct CachedResponse {
    data: Vec<u8>,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedResponse {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// Simple in-memory cache with TTL
pub struct ResponseCache {
    store: Arc<RwLock<HashMap<String, CachedResponse>>>,
    default_ttl: Duration,
}

impl ResponseCache {
    /// Create new cache with default TTL
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }
    
    /// Get cached response (if not expired)
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let store = self.store.read().ok()?;
        let cached = store.get(key)?;
        
        if cached.is_expired() {
            // Don't return expired entries
            drop(store); // Release read lock
            self.invalidate(key); // Cleanup
            return None;
        }
        
        Some(cached.data.clone())
    }
    
    /// Store response with default TTL
    pub fn set(&self, key: String, data: Vec<u8>) {
        self.set_with_ttl(key, data, self.default_ttl);
    }
    
    /// Store response with custom TTL
    pub fn set_with_ttl(&self, key: String, data: Vec<u8>, ttl: Duration) {
        if let Ok(mut store) = self.store.write() {
            store.insert(key, CachedResponse {
                data,
                cached_at: Instant::now(),
                ttl,
            });
        }
    }
    
    /// Remove specific entry
    pub fn invalidate(&self, key: &str) {
        if let Ok(mut store) = self.store.write() {
            store.remove(key);
        }
    }
    
    /// Clear all entries
    pub fn clear(&self) {
        if let Ok(mut store) = self.store.write() {
            store.clear();
        }
    }
    
    /// Remove expired entries (periodic cleanup)
    pub fn cleanup_expired(&self) {
        if let Ok(mut store) = self.store.write() {
            store.retain(|_, v| !v.is_expired());
        }
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        if let Ok(store) = self.store.read() {
            let total = store.len();
            let expired = store.values().filter(|v| v.is_expired()).count();
            
            CacheStats {
                total_entries: total,
                expired_entries: expired,
                active_entries: total - expired,
            }
        } else {
            CacheStats::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}

/// Generate cache key from request parameters
pub fn cache_key(model: &str, messages: &[u8], options: &[u8]) -> String {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    model.hash(&mut hasher);
    messages.hash(&mut hasher);
    options.hash(&mut hasher);
    
    format!("chat:{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    
    #[test]
    fn test_cache_basic() {
        let cache = ResponseCache::new(Duration::from_secs(1));
        
        // Set
        cache.set("key1".to_string(), vec![1, 2, 3]);
        
        // Get
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
        
        // Non-existent key
        assert_eq!(cache.get("key2"), None);
    }
    
    #[test]
    fn test_cache_expiration() {
        let cache = ResponseCache::new(Duration::from_millis(100));
        
        cache.set("key1".to_string(), vec![1, 2, 3]);
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
        
        // Wait for expiration
        sleep(Duration::from_millis(150));
        
        // Should be expired
        assert_eq!(cache.get("key1"), None);
    }
    
    #[test]
    fn test_cache_invalidate() {
        let cache = ResponseCache::new(Duration::from_secs(10));
        
        cache.set("key1".to_string(), vec![1, 2, 3]);
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
        
        cache.invalidate("key1");
        assert_eq!(cache.get("key1"), None);
    }
    
    #[test]
    fn test_cache_cleanup() {
        let cache = ResponseCache::new(Duration::from_millis(50));
        
        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        
        sleep(Duration::from_millis(100));
        
        // Add fresh entry
        cache.set("key3".to_string(), vec![3]);
        
        // Before cleanup
        let stats = cache.stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.expired_entries, 2);
        
        // After cleanup
        cache.cleanup_expired();
        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.expired_entries, 0);
    }
}
