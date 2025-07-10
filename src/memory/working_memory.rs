// Working Memory - Redis L1 Cache for Ultra-Low Latency Access
// Target: <1ms response time, 16GB hot memory, 32 txs/batch

use super::{WorkingMemoryConfig, ComponentHealth, HealthStatus};
use anyhow::{anyhow, Result};
use redis::{Client, Commands, Connection, RedisResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Transaction context for working memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionContext {
    /// Transaction signature
    pub signature: String,
    
    /// Block slot
    pub slot: u64,
    
    /// Transaction timestamp
    pub timestamp: u64,
    
    /// Account keys involved
    pub account_keys: Vec<String>,
    
    /// Program IDs
    pub program_ids: Vec<String>,
    
    /// Transaction type
    pub tx_type: TransactionType,
    
    /// MEV score
    pub mev_score: f64,
    
    /// Processing metadata
    pub metadata: TransactionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// Regular transfer
    Transfer,
    
    /// DEX swap
    Swap,
    
    /// Liquidity provision
    LiquidityAdd,
    
    /// Liquidity removal
    LiquidityRemove,
    
    /// Arbitrage
    Arbitrage,
    
    /// MEV sandwich
    MEVSandwich,
    
    /// Unknown/Other
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetadata {
    /// Processing latency (ms)
    pub processing_latency_ms: f64,
    
    /// Memory access count
    pub memory_access_count: u32,
    
    /// Cache hit/miss
    pub cache_hit: bool,
    
    /// Priority score
    pub priority_score: f64,
    
    /// Tags for categorization
    pub tags: Vec<String>,
    
    /// Related transactions
    pub related_txs: Vec<String>,
}

/// Memory batch for efficient processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBatch {
    /// Batch ID
    pub id: String,
    
    /// Transactions in batch
    pub transactions: Vec<TransactionContext>,
    
    /// Batch timestamp
    pub timestamp: u64,
    
    /// Batch size
    pub size: usize,
    
    /// Processing status
    pub status: BatchStatus,
    
    /// Batch metadata
    pub metadata: BatchMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMetadata {
    /// Total processing time (ms)
    pub total_processing_ms: f64,
    
    /// Average MEV score
    pub avg_mev_score: f64,
    
    /// High priority count
    pub high_priority_count: u32,
    
    /// Error count
    pub error_count: u32,
    
    /// Compression ratio
    pub compression_ratio: f64,
}

impl MemoryBatch {
    pub fn new(transactions: Vec<TransactionContext>) -> Self {
        let size = transactions.len();
        let avg_mev_score = if size > 0 {
            transactions.iter().map(|tx| tx.mev_score).sum::<f64>() / size as f64
        } else {
            0.0
        };
        
        let high_priority_count = transactions
            .iter()
            .filter(|tx| tx.metadata.priority_score > 0.8)
            .count() as u32;
        
        Self {
            id: format!("batch_{}", Uuid::new_v4()),
            transactions,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            size,
            status: BatchStatus::Pending,
            metadata: BatchMetadata {
                total_processing_ms: 0.0,
                avg_mev_score,
                high_priority_count,
                error_count: 0,
                compression_ratio: 1.0,
            },
        }
    }
    
    pub fn from_transactions(transactions: Vec<TransactionContext>) -> Self {
        Self::new(transactions)
    }
}

/// Working Memory implementation with Redis L1
pub struct WorkingMemory {
    /// Configuration
    config: WorkingMemoryConfig,
    
    /// Redis client
    client: Option<Client>,
    
    /// Connection pool
    connections: Arc<RwLock<Vec<Connection>>>,
    
    /// Local cache for ultra-fast access
    local_cache: Arc<RwLock<HashMap<String, TransactionContext>>>,
    
    /// Batch queue
    batch_queue: Arc<RwLock<Vec<MemoryBatch>>>,
    
    /// Performance metrics
    metrics: Arc<RwLock<WorkingMemoryMetrics>>,
    
    /// Running status
    running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Default)]
pub struct WorkingMemoryMetrics {
    /// Total operations
    pub total_operations: u64,
    
    /// Cache hits
    pub cache_hits: u64,
    
    /// Cache misses
    pub cache_misses: u64,
    
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    
    /// Memory utilization (%)
    pub memory_utilization: f64,
    
    /// Batch processing rate (batches/s)
    pub batch_processing_rate: f64,
    
    /// Error rate
    pub error_rate: f64,
    
    /// Last update
    pub last_update: u64,
}

impl WorkingMemory {
    pub async fn new(config: WorkingMemoryConfig) -> Result<Self> {
        info!("🔄 Initializing Working Memory (Redis L1)");
        
        // Initialize Redis client
        let client = if let Some(endpoint) = config.redis_endpoints.first() {
            Some(Client::open(endpoint.as_str())?)
        } else {
            None
        };
        
        // Initialize connection pool
        let mut connections = Vec::new();
        if let Some(ref client) = client {
            for _ in 0..config.pool_size {
                connections.push(client.get_connection()?);
            }
        }
        
        info!("✅ Working Memory initialized with {} connections", connections.len());
        
        Ok(Self {
            config,
            client,
            connections: Arc::new(RwLock::new(connections)),
            local_cache: Arc::new(RwLock::new(HashMap::new())),
            batch_queue: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(WorkingMemoryMetrics::default())),
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Working Memory");
        
        *self.running.write().await = true;
        
        // Start batch processor
        self.start_batch_processor().await;
        
        // Start cache cleanup
        self.start_cache_cleanup().await;
        
        info!("✅ Working Memory started");
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Working Memory");
        
        *self.running.write().await = false;
        
        info!("✅ Working Memory stopped");
        Ok(())
    }
    
    /// Store transaction in working memory
    pub async fn store_transaction(&self, tx: TransactionContext) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Store in local cache first (ultra-fast)
        self.local_cache.write().await.insert(tx.signature.clone(), tx.clone());
        
        // Store in Redis with TTL
        if let Some(mut conn) = self.get_connection().await {
            let key = format!("tx:{}", tx.signature);
            let value = serde_json::to_string(&tx)?;
            
            let _: RedisResult<()> = conn.set_ex(&key, value, self.config.ttl_seconds);
        }
        
        // Update metrics
        let processing_time = start_time.elapsed().as_millis() as f64;
        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.avg_response_time_ms = 
            (metrics.avg_response_time_ms + processing_time) / 2.0;
        
        debug!("📝 Stored transaction {} in {:.2}ms", tx.signature, processing_time);
        Ok(())
    }
    
    /// Retrieve transaction from working memory
    pub async fn get_transaction(&self, signature: &str) -> Result<Option<TransactionContext>> {
        let start_time = std::time::Instant::now();
        
        // Check local cache first
        if let Some(tx) = self.local_cache.read().await.get(signature) {
            let mut metrics = self.metrics.write().await;
            metrics.cache_hits += 1;
            return Ok(Some(tx.clone()));
        }
        
        // Check Redis
        if let Some(mut conn) = self.get_connection().await {
            let key = format!("tx:{}", signature);
            let result: RedisResult<String> = conn.get(&key);
            
            if let Ok(value) = result {
                let tx: TransactionContext = serde_json::from_str(&value)?;
                
                // Update local cache
                self.local_cache.write().await.insert(signature.to_string(), tx.clone());
                
                let mut metrics = self.metrics.write().await;
                metrics.cache_hits += 1;
                
                let processing_time = start_time.elapsed().as_millis() as f64;
                debug!("🔍 Retrieved transaction {} in {:.2}ms", signature, processing_time);
                
                return Ok(Some(tx));
            }
        }
        
        // Cache miss
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
        
        Ok(None)
    }
    
    /// Push batch to processing queue
    pub async fn push(&self, batch: MemoryBatch) -> Result<()> {
        debug!("📦 Pushing batch {} with {} transactions", batch.id, batch.size);
        
        self.batch_queue.write().await.push(batch);
        Ok(())
    }
    
    /// Get expired entries for archival
    pub async fn get_expired_entries(&self) -> Result<Vec<TransactionContext>> {
        let mut expired = Vec::new();
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // Check local cache for expired entries
        let cache = self.local_cache.read().await;
        for (_, tx) in cache.iter() {
            if current_time - tx.timestamp > self.config.ttl_seconds {
                expired.push(tx.clone());
            }
        }
        
        Ok(expired)
    }
    
    /// Remove expired entries
    pub async fn remove_expired(&self) -> Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut removed_count = 0;
        
        // Remove from local cache
        {
            let mut cache = self.local_cache.write().await;
            cache.retain(|_, tx| {
                let expired = current_time - tx.timestamp > self.config.ttl_seconds;
                if expired {
                    removed_count += 1;
                }
                !expired
            });
        }
        
        debug!("🗑️ Removed {} expired entries from local cache", removed_count);
        Ok(())
    }
    
    async fn get_connection(&self) -> Option<Connection> {
        if let Some(ref client) = self.client {
            client.get_connection().ok()
        } else {
            None
        }
    }
    
    async fn start_batch_processor(&self) {
        let batch_queue = Arc::clone(&self.batch_queue);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            while *running.read().await {
                interval.tick().await;
                
                let mut queue = batch_queue.write().await;
                if !queue.is_empty() {
                    let batch = queue.remove(0);
                    drop(queue);
                    
                    if let Err(e) = Self::process_batch(batch, &config).await {
                        error!("Failed to process batch: {}", e);
                    }
                }
            }
        });
    }
    
    async fn start_cache_cleanup(&self) {
        let local_cache = Arc::clone(&self.local_cache);
        let running = Arc::clone(&self.running);
        let ttl = self.config.ttl_seconds;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            while *running.read().await {
                interval.tick().await;
                
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                let mut cache = local_cache.write().await;
                let initial_size = cache.len();
                
                cache.retain(|_, tx| current_time - tx.timestamp <= ttl);
                
                let removed = initial_size - cache.len();
                if removed > 0 {
                    debug!("🧹 Cleaned up {} expired entries from local cache", removed);
                }
            }
        });
    }
    
    async fn process_batch(mut batch: MemoryBatch, config: &WorkingMemoryConfig) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        batch.status = BatchStatus::Processing;
        
        // Process transactions in batch
        for tx in &mut batch.transactions {
            // Simulate processing
            tx.metadata.processing_latency_ms = 0.5; // Target: <1ms
            tx.metadata.cache_hit = true;
        }
        
        batch.status = BatchStatus::Completed;
        batch.metadata.total_processing_ms = start_time.elapsed().as_millis() as f64;
        
        debug!("✅ Processed batch {} in {:.2}ms", 
               batch.id, batch.metadata.total_processing_ms);
        
        Ok(())
    }
    
    pub async fn health_check(&self) -> Result<ComponentHealth> {
        let metrics = self.metrics.read().await;
        
        let status = if metrics.avg_response_time_ms < 1.0 && 
                        metrics.error_rate < 0.01 &&
                        metrics.memory_utilization < 0.9 {
            HealthStatus::Healthy
        } else if metrics.avg_response_time_ms < 5.0 && 
                   metrics.error_rate < 0.05 &&
                   metrics.memory_utilization < 0.95 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };
        
        Ok(ComponentHealth {
            status,
            latency_ms: metrics.avg_response_time_ms,
            error_rate: metrics.error_rate,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        })
    }
    
    pub async fn get_metrics(&self) -> WorkingMemoryMetrics {
        self.metrics.read().await.clone()
    }
}
