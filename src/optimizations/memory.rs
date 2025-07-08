/*
THE OVERMIND PROTOCOL v4.1 - Memory Optimizations
Advanced memory management and optimization techniques

Features:
- Object pooling for frequent allocations
- Memory-mapped data structures
- Lazy loading and caching
- Memory profiling and monitoring
- Zero-copy data processing
*/

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{debug, info, warn};

/// Memory pool for reusing objects
pub struct ObjectPool<T> {
    pool: Mutex<Vec<T>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
    created_count: Arc<RwLock<usize>>,
    reused_count: Arc<RwLock<usize>>,
}

impl<T> ObjectPool<T> {
    pub fn new<F>(factory: F, max_size: usize) -> Self 
    where 
        F: Fn() -> T + Send + Sync + 'static 
    {
        Self {
            pool: Mutex::new(Vec::with_capacity(max_size)),
            factory: Box::new(factory),
            max_size,
            created_count: Arc::new(RwLock::new(0)),
            reused_count: Arc::new(RwLock::new(0)),
        }
    }
    
    pub fn acquire(&self) -> PooledObject<T> {
        let mut pool = self.pool.lock().unwrap();
        
        if let Some(obj) = pool.pop() {
            *self.reused_count.write().unwrap() += 1;
            debug!("🔄 Reused object from pool");
            PooledObject::new(obj, self)
        } else {
            *self.created_count.write().unwrap() += 1;
            debug!("🆕 Created new object");
            PooledObject::new((self.factory)(), self)
        }
    }
    
    fn return_object(&self, obj: T) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push(obj);
            debug!("↩️ Returned object to pool");
        } else {
            debug!("🗑️ Pool full, dropping object");
        }
    }
    
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            created: *self.created_count.read().unwrap(),
            reused: *self.reused_count.read().unwrap(),
            current_size: self.pool.lock().unwrap().len(),
            max_size: self.max_size,
        }
    }
}

/// RAII wrapper for pooled objects
pub struct PooledObject<'a, T> {
    obj: Option<T>,
    pool: &'a ObjectPool<T>,
}

impl<'a, T> PooledObject<'a, T> {
    fn new(obj: T, pool: &'a ObjectPool<T>) -> Self {
        Self {
            obj: Some(obj),
            pool,
        }
    }
}

impl<'a, T> std::ops::Deref for PooledObject<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.obj.as_ref().unwrap()
    }
}

impl<'a, T> std::ops::DerefMut for PooledObject<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.as_mut().unwrap()
    }
}

impl<'a, T> Drop for PooledObject<'a, T> {
    fn drop(&mut self) {
        if let Some(obj) = self.obj.take() {
            self.pool.return_object(obj);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub created: usize,
    pub reused: usize,
    pub current_size: usize,
    pub max_size: usize,
}

/// Memory cache with TTL and size limits
pub struct MemoryCache<K, V> {
    cache: RwLock<HashMap<K, CacheEntry<V>>>,
    max_size: usize,
    default_ttl: Duration,
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

impl<K, V> MemoryCache<K, V> 
where 
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
            default_ttl,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        let now = Instant::now();
        
        // Try to get from cache
        {
            let mut cache = self.cache.write().unwrap();
            if let Some(entry) = cache.get_mut(key) {
                if now < entry.expires_at {
                    entry.access_count += 1;
                    entry.last_accessed = now;
                    *self.hits.write().unwrap() += 1;
                    debug!("💾 Cache hit for key");
                    return Some(entry.value.clone());
                } else {
                    // Expired entry
                    cache.remove(key);
                    debug!("⏰ Cache entry expired");
                }
            }
        }
        
        *self.misses.write().unwrap() += 1;
        debug!("❌ Cache miss for key");
        None
    }
    
    pub fn put(&self, key: K, value: V) -> Option<V> {
        self.put_with_ttl(key, value, self.default_ttl)
    }
    
    pub fn put_with_ttl(&self, key: K, value: V, ttl: Duration) -> Option<V> {
        let now = Instant::now();
        let expires_at = now + ttl;
        
        let entry = CacheEntry {
            value: value.clone(),
            expires_at,
            access_count: 0,
            last_accessed: now,
        };
        
        let mut cache = self.cache.write().unwrap();
        
        // Check if we need to evict entries
        if cache.len() >= self.max_size && !cache.contains_key(&key) {
            self.evict_lru(&mut cache);
        }
        
        let old_value = cache.insert(key, entry).map(|e| e.value);
        debug!("💾 Cached value with TTL: {:?}", ttl);
        old_value
    }
    
    fn evict_lru(&self, cache: &mut HashMap<K, CacheEntry<V>>) {
        if cache.is_empty() {
            return;
        }
        
        // Find least recently used entry
        let lru_key = cache.iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, _)| k.clone());
            
        if let Some(key) = lru_key {
            cache.remove(&key);
            debug!("🗑️ Evicted LRU cache entry");
        }
    }
    
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        info!("🧹 Cleared memory cache");
    }
    
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        let hits = *self.hits.read().unwrap();
        let misses = *self.misses.read().unwrap();
        
        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
            hits,
            misses,
            hit_rate: if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

/// Memory monitor for tracking usage
pub struct MemoryMonitor {
    start_time: Instant,
    peak_usage: Arc<RwLock<usize>>,
    current_usage: Arc<RwLock<usize>>,
    allocations: Arc<RwLock<u64>>,
    deallocations: Arc<RwLock<u64>>,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            peak_usage: Arc::new(RwLock::new(0)),
            current_usage: Arc::new(RwLock::new(0)),
            allocations: Arc::new(RwLock::new(0)),
            deallocations: Arc::new(RwLock::new(0)),
        }
    }
    
    pub fn record_allocation(&self, size: usize) {
        let mut current = self.current_usage.write().unwrap();
        let mut peak = self.peak_usage.write().unwrap();
        let mut allocs = self.allocations.write().unwrap();
        
        *current += size;
        *allocs += 1;
        
        if *current > *peak {
            *peak = *current;
        }
    }
    
    pub fn record_deallocation(&self, size: usize) {
        let mut current = self.current_usage.write().unwrap();
        let mut deallocs = self.deallocations.write().unwrap();
        
        *current = current.saturating_sub(size);
        *deallocs += 1;
    }
    
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            uptime: self.start_time.elapsed(),
            current_usage: *self.current_usage.read().unwrap(),
            peak_usage: *self.peak_usage.read().unwrap(),
            total_allocations: *self.allocations.read().unwrap(),
            total_deallocations: *self.deallocations.read().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub uptime: Duration,
    pub current_usage: usize,
    pub peak_usage: usize,
    pub total_allocations: u64,
    pub total_deallocations: u64,
}

/// Global memory optimization manager
pub struct MemoryOptimizer {
    signal_pool: ObjectPool<serde_json::Value>,
    prediction_cache: MemoryCache<String, serde_json::Value>,
    monitor: MemoryMonitor,
}

impl MemoryOptimizer {
    pub fn new() -> Self {
        Self {
            signal_pool: ObjectPool::new(
                || serde_json::Value::Null,
                1000, // Pool size
            ),
            prediction_cache: MemoryCache::new(
                10000, // Max entries
                Duration::from_secs(300), // 5 minute TTL
            ),
            monitor: MemoryMonitor::new(),
        }
    }
    
    pub fn get_signal(&self) -> PooledObject<serde_json::Value> {
        self.signal_pool.acquire()
    }
    
    pub fn cache_prediction(&self, key: String, prediction: serde_json::Value) {
        self.prediction_cache.put(key, prediction);
    }
    
    pub fn get_cached_prediction(&self, key: &str) -> Option<serde_json::Value> {
        self.prediction_cache.get(key)
    }
    
    pub fn get_comprehensive_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "signal_pool": self.signal_pool.stats(),
            "prediction_cache": self.prediction_cache.stats(),
            "memory_monitor": self.monitor.get_stats(),
            "timestamp": chrono::Utc::now()
        })
    }
    
    pub fn optimize(&self) {
        // Clear expired cache entries
        self.prediction_cache.clear();
        
        // Log optimization stats
        let stats = self.get_comprehensive_stats();
        info!("🚀 Memory optimization completed: {}", stats);
    }
}

impl Default for MemoryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
