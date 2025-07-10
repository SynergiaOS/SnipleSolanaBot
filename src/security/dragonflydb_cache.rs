//! DRAGONFLYDB CACHE LAYER
//! 
//! High-performance cache layer for THE OVERMIND PROTOCOL using DragonflyDB Cloud
//! VPC: vpc-05f61f843ed60555e, Account: 962364259018, CIDR: 192.168.0.0/16

use anyhow::{Result, anyhow};
use redis::{Client, Connection, Commands};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, info, warn};

/// DragonflyDB configuration
#[derive(Debug, Clone)]
pub struct DragonflyConfig {
    /// DragonflyDB connection URL
    pub url: String,
    
    /// Database password
    pub password: Option<String>,
    
    /// Database number
    pub database: u8,
    
    /// VPC ID for network isolation
    pub vpc_id: String,
    
    /// CIDR block for network security
    pub cidr: String,
    
    /// AWS Account ID
    pub account_id: String,
    
    /// Connection timeout
    pub connection_timeout: Duration,
    
    /// Command timeout
    pub command_timeout: Duration,
    
    /// Cache TTL for secrets
    pub cache_ttl: Duration,
    
    /// Max cache size (number of entries)
    pub max_cache_size: usize,
    
    /// Enable compression
    pub compression: bool,
}

/// Cached secret with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSecret {
    pub key: String,
    pub value: String,
    pub cached_at: u64, // Unix timestamp
    pub ttl_seconds: u64,
    pub access_count: u64,
    pub last_accessed: u64,
}

/// DragonflyDB cache client
pub struct DragonflyCache {
    config: DragonflyConfig,
    client: Client,
    connection: Arc<RwLock<Option<Connection>>>,
    local_cache: Arc<RwLock<HashMap<String, CachedSecret>>>,
    stats: Arc<RwLock<CacheStats>>,
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub deletes: u64,
    pub errors: u64,
    pub total_requests: u64,
}

impl DragonflyConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(DragonflyConfig {
            url: env::var("DRAGONFLYDB_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            password: env::var("DRAGONFLYDB_PASSWORD").ok(),
            database: env::var("DRAGONFLYDB_DATABASE")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
            vpc_id: env::var("DRAGONFLYDB_VPC_ID")
                .unwrap_or_else(|_| "vpc-05f61f843ed60555e".to_string()),
            cidr: env::var("DRAGONFLYDB_CIDR")
                .unwrap_or_else(|_| "192.168.0.0/16".to_string()),
            account_id: env::var("DRAGONFLYDB_ACCOUNT_ID")
                .unwrap_or_else(|_| "962364259018".to_string()),
            connection_timeout: Duration::from_secs(10),
            command_timeout: Duration::from_secs(5),
            cache_ttl: Duration::from_secs(
                env::var("CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .unwrap_or(300)
            ),
            max_cache_size: env::var("CACHE_MAX_SIZE")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            compression: env::var("CACHE_COMPRESSION")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}

impl DragonflyCache {
    /// Create new DragonflyDB cache client
    pub fn new(config: DragonflyConfig) -> Result<Self> {
        info!("🐉 Initializing DragonflyDB cache client");
        info!("🌐 VPC: {}, CIDR: {}", config.vpc_id, config.cidr);
        
        let client = Client::open(config.url.as_str())
            .map_err(|e| anyhow!("Failed to create DragonflyDB client: {}", e))?;
        
        Ok(DragonflyCache {
            config,
            client,
            connection: Arc::new(RwLock::new(None)),
            local_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }
    
    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let config = DragonflyConfig::from_env()?;
        Self::new(config)
    }
    
    /// Connect to DragonflyDB
    pub async fn connect(&self) -> Result<()> {
        info!("🔌 Connecting to DragonflyDB Cloud...");
        
        let mut conn = self.client.get_connection()
            .map_err(|e| anyhow!("Failed to connect to DragonflyDB: {}", e))?;
        
        // Test connection
        let _: String = redis::cmd("PING").query(&mut conn)
            .map_err(|e| anyhow!("DragonflyDB ping failed: {}", e))?;
        
        // Set database if specified
        if self.config.database > 0 {
            let _: () = redis::cmd("SELECT").arg(self.config.database).query(&mut conn)
                .map_err(|e| anyhow!("Failed to select database {}: {}", self.config.database, e))?;
        }
        
        {
            let mut connection = self.connection.write().unwrap();
            *connection = Some(conn);
        }
        
        info!("✅ Connected to DragonflyDB Cloud successfully");
        info!("🏦 Database: {}, VPC: {}", self.config.database, self.config.vpc_id);
        
        Ok(())
    }
    
    /// Get secret from cache (checks local cache first, then DragonflyDB)
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        self.increment_stat("total_requests");
        
        // Check local cache first
        {
            let local_cache = self.local_cache.read().unwrap();
            if let Some(cached) = local_cache.get(key) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                if now - cached.cached_at < cached.ttl_seconds {
                    debug!("🎯 Local cache hit for key: {}", key);
                    self.increment_stat("hits");
                    return Ok(Some(cached.value.clone()));
                }
            }
        }
        
        // Check DragonflyDB
        match self.get_from_dragonflydb(key).await {
            Ok(Some(value)) => {
                debug!("🐉 DragonflyDB cache hit for key: {}", key);
                self.increment_stat("hits");
                
                // Update local cache
                self.update_local_cache(key, &value);
                
                Ok(Some(value))
            }
            Ok(None) => {
                debug!("❌ Cache miss for key: {}", key);
                self.increment_stat("misses");
                Ok(None)
            }
            Err(e) => {
                warn!("⚠️ DragonflyDB error for key {}: {}", key, e);
                self.increment_stat("errors");
                Ok(None)
            }
        }
    }
    
    /// Set secret in cache (both local and DragonflyDB)
    pub async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()> {
        self.increment_stat("total_requests");
        self.increment_stat("sets");
        
        let ttl_seconds = ttl.unwrap_or(self.config.cache_ttl).as_secs();
        
        // Set in DragonflyDB
        if let Err(e) = self.set_in_dragonflydb(key, value, ttl_seconds).await {
            warn!("⚠️ Failed to set in DragonflyDB: {}", e);
            self.increment_stat("errors");
        }
        
        // Update local cache
        self.update_local_cache(key, value);
        
        debug!("💾 Cached secret: {} (TTL: {}s)", key, ttl_seconds);
        Ok(())
    }
    
    /// Delete secret from cache
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.increment_stat("total_requests");
        self.increment_stat("deletes");
        
        // Delete from DragonflyDB
        if let Err(e) = self.delete_from_dragonflydb(key).await {
            warn!("⚠️ Failed to delete from DragonflyDB: {}", e);
            self.increment_stat("errors");
        }
        
        // Delete from local cache
        {
            let mut local_cache = self.local_cache.write().unwrap();
            local_cache.remove(key);
        }
        
        debug!("🗑️ Deleted secret: {}", key);
        Ok(())
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().unwrap();
        CacheStats {
            hits: stats.hits,
            misses: stats.misses,
            sets: stats.sets,
            deletes: stats.deletes,
            errors: stats.errors,
            total_requests: stats.total_requests,
        }
    }
    
    /// Get cache hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let stats = self.stats.read().unwrap();
        if stats.hits + stats.misses == 0 {
            0.0
        } else {
            stats.hits as f64 / (stats.hits + stats.misses) as f64
        }
    }
    
    // Private helper methods
    
    async fn get_from_dragonflydb(&self, key: &str) -> Result<Option<String>> {
        let mut connection = self.connection.write().unwrap();
        if let Some(ref mut conn) = connection.as_mut() {
            // Note: This is a simplified version. In real implementation,
            // we'd need to handle connection pooling and async properly
            match redis::cmd("GET").arg(key).query::<Option<String>>(conn) {
                Ok(value) => Ok(value),
                Err(e) => Err(anyhow!("DragonflyDB get error: {}", e)),
            }
        } else {
            Err(anyhow!("No DragonflyDB connection available"))
        }
    }
    
    async fn set_in_dragonflydb(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        let mut connection = self.connection.write().unwrap();
        if let Some(ref mut conn) = connection.as_mut() {
            let _: () = redis::cmd("SETEX").arg(key).arg(ttl_seconds).arg(value).query(conn)
                .map_err(|e| anyhow!("DragonflyDB set error: {}", e))?;
            Ok(())
        } else {
            Err(anyhow!("No DragonflyDB connection available"))
        }
    }
    
    async fn delete_from_dragonflydb(&self, key: &str) -> Result<()> {
        let mut connection = self.connection.write().unwrap();
        if let Some(ref mut conn) = connection.as_mut() {
            let _: () = redis::cmd("DEL").arg(key).query(conn)
                .map_err(|e| anyhow!("DragonflyDB delete error: {}", e))?;
            Ok(())
        } else {
            Err(anyhow!("No DragonflyDB connection available"))
        }
    }
    
    fn update_local_cache(&self, key: &str, value: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let cached_secret = CachedSecret {
            key: key.to_string(),
            value: value.to_string(),
            cached_at: now,
            ttl_seconds: self.config.cache_ttl.as_secs(),
            access_count: 1,
            last_accessed: now,
        };
        
        let mut local_cache = self.local_cache.write().unwrap();
        
        // Evict old entries if cache is full
        if local_cache.len() >= self.config.max_cache_size {
            // Simple LRU eviction - remove oldest entry
            if let Some(oldest_key) = local_cache.keys()
                .min_by_key(|k| local_cache.get(*k).map(|v| v.last_accessed).unwrap_or(0))
                .cloned() {
                local_cache.remove(&oldest_key);
            }
        }
        
        local_cache.insert(key.to_string(), cached_secret);
    }
    
    fn increment_stat(&self, stat_name: &str) {
        let mut stats = self.stats.write().unwrap();
        match stat_name {
            "hits" => stats.hits += 1,
            "misses" => stats.misses += 1,
            "sets" => stats.sets += 1,
            "deletes" => stats.deletes += 1,
            "errors" => stats.errors += 1,
            "total_requests" => stats.total_requests += 1,
            _ => {}
        }
    }
}

/// Create DragonflyDB cache client from environment
pub async fn create_dragonflydb_cache() -> Result<DragonflyCache> {
    let cache = DragonflyCache::from_env()?;
    cache.connect().await?;
    Ok(cache)
}
