//! RUNTIME MODULE LOADING SYSTEM - FAZA 2C OPERACJI "FORGE"
//! 
//! Hot-swapping agent logic bez restartu monolitu
//! Integration z TensorZero metrics i FORGE artifacts

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::fs;
use tracing::{debug, info, warn};

use crate::forge::{CompiledArtifact, TheForge};
use crate::forge::hot_loader::{StrategyHotLoader, StrategyContainer};

/// Runtime Module Loader - zarządza hot-swapping strategii
#[derive(Debug)]
pub struct RuntimeModuleLoader {
    /// Strategy hot loader
    strategy_loader: Arc<RwLock<StrategyHotLoader>>,
    
    /// FORGE integration
    forge: Option<Arc<RwLock<TheForge>>>,
    
    /// Artifact cache
    artifact_cache: Arc<RwLock<HashMap<String, CachedArtifact>>>,
    
    /// Loading metrics
    metrics: Arc<RwLock<LoadingMetrics>>,
    
    /// Configuration
    config: LoaderConfig,
}

/// Cached artifact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedArtifact {
    pub artifact: CompiledArtifact,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub file_path: PathBuf,
    pub checksum_verified: bool,
}

/// Loading metrics
#[derive(Debug, Default, Clone)]
pub struct LoadingMetrics {
    pub total_loads: u64,
    pub successful_loads: u64,
    pub failed_loads: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hot_swaps: u64,
    pub successful_swaps: u64,
    pub failed_swaps: u64,
    pub average_load_time_ms: u64,
    pub average_swap_time_ms: u64,
}

/// Loader configuration
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// Artifact cache directory
    pub cache_dir: PathBuf,
    
    /// Maximum cache size (number of artifacts)
    pub max_cache_size: usize,
    
    /// Cache TTL
    pub cache_ttl: Duration,
    
    /// Download timeout
    pub download_timeout: Duration,
    
    /// Verification enabled
    pub verify_checksums: bool,
    
    /// Auto-cleanup enabled
    pub auto_cleanup: bool,
    
    /// Cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("./artifacts/cache"),
            max_cache_size: 100,
            cache_ttl: Duration::from_secs(24 * 3600),
            download_timeout: Duration::from_secs(300),
            verify_checksums: true,
            auto_cleanup: true,
            cleanup_interval: Duration::from_secs(6 * 3600),
        }
    }
}

impl RuntimeModuleLoader {
    /// Create new runtime module loader
    pub async fn new(
        strategy_loader: Arc<RwLock<StrategyHotLoader>>,
        forge: Option<Arc<RwLock<TheForge>>>,
        config: LoaderConfig,
    ) -> Result<Self> {
        info!("🔄 Initializing Runtime Module Loader");
        
        // Create cache directory
        fs::create_dir_all(&config.cache_dir).await?;
        
        let loader = Self {
            strategy_loader,
            forge,
            artifact_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(LoadingMetrics::default())),
            config,
        };
        
        // Start cleanup task if enabled
        if loader.config.auto_cleanup {
            loader.start_cleanup_task().await;
        }
        
        info!("✅ Runtime Module Loader initialized");
        Ok(loader)
    }
    
    /// Load strategy for agent (with caching and hot-swap)
    pub async fn load_strategy_for_agent(
        &self,
        agent_id: &str,
        artifact: CompiledArtifact,
    ) -> Result<StrategyContainer> {
        let start_time = Instant::now();
        info!("🔄 Loading strategy {} for agent {}", artifact.strategy_id, agent_id);
        
        // Check cache first
        let cached_path = self.get_cached_artifact_path(&artifact).await?;
        
        let strategy_container = {
            let mut loader = self.strategy_loader.write().unwrap();
            loader.deploy_strategy(agent_id, &artifact).await?
        };
        
        // Update metrics
        let load_time = start_time.elapsed().as_millis() as u64;
        self.update_load_metrics(load_time, true).await;
        
        info!("✅ Strategy loaded for agent {} in {}ms", agent_id, load_time);
        Ok(strategy_container.strategy_container)
    }
    
    /// Hot-swap strategy for agent
    pub async fn hot_swap_strategy(
        &self,
        agent_id: &str,
        old_strategy_id: &str,
        new_artifact: CompiledArtifact,
    ) -> Result<StrategyContainer> {
        let start_time = Instant::now();
        info!("🔥 Hot-swapping strategy for agent {}: {} → {}", 
              agent_id, old_strategy_id, new_artifact.strategy_id);
        
        // Ensure new artifact is cached and verified
        let _cached_path = self.get_cached_artifact_path(&new_artifact).await?;
        
        // Perform hot swap
        let strategy_container = {
            let mut loader = self.strategy_loader.write().unwrap();
            loader.deploy_strategy(agent_id, &new_artifact).await?
        };
        
        // Update metrics
        let swap_time = start_time.elapsed().as_millis() as u64;
        self.update_swap_metrics(swap_time, true).await;
        
        info!("✅ Hot-swap completed for agent {} in {}ms", agent_id, swap_time);
        Ok(strategy_container.strategy_container)
    }
    
    /// Get cached artifact path (download if not cached)
    async fn get_cached_artifact_path(&self, artifact: &CompiledArtifact) -> Result<PathBuf> {
        let cache_key = format!("{}_{}", artifact.strategy_id, artifact.checksum);
        
        // Check if already cached
        let cached_path = {
            let cache = self.artifact_cache.read().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                // Verify file still exists and checksum matches
                if cached.file_path.exists() && self.verify_artifact_checksum(&cached.file_path, &artifact.checksum).await? {
                    Some(cached.file_path.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(path) = cached_path {
            // Update access info
            self.update_cache_access(&cache_key).await;

            // Update metrics
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.cache_hits += 1;
            }

            debug!("💾 Cache hit for artifact: {}", cache_key);
            return Ok(path);
        }
        
        // Cache miss - download artifact
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.cache_misses += 1;
        }
        
        debug!("📥 Cache miss for artifact: {}, downloading...", cache_key);
        self.download_and_cache_artifact(artifact, &cache_key).await
    }
    
    /// Download and cache artifact
    async fn download_and_cache_artifact(
        &self,
        artifact: &CompiledArtifact,
        cache_key: &str,
    ) -> Result<PathBuf> {
        let file_name = format!("{}.so", cache_key);
        let file_path = self.config.cache_dir.join(&file_name);
        
        // Download artifact (simulate for now - in production would download from S3)
        if !Path::new(&artifact.binary_path).exists() {
            return Err(anyhow!("Artifact binary not found: {}", artifact.binary_path));
        }
        
        // Copy to cache
        fs::copy(&artifact.binary_path, &file_path).await?;
        
        // Verify checksum if enabled
        let checksum_verified = if self.config.verify_checksums {
            self.verify_artifact_checksum(&file_path, &artifact.checksum).await?
        } else {
            false
        };
        
        // Add to cache
        let cached_artifact = CachedArtifact {
            artifact: artifact.clone(),
            cached_at: chrono::Utc::now(),
            access_count: 1,
            last_accessed: chrono::Utc::now(),
            file_path: file_path.clone(),
            checksum_verified,
        };
        
        {
            let mut cache = self.artifact_cache.write().unwrap();
            cache.insert(cache_key.to_string(), cached_artifact);
            
            // Cleanup cache if too large
            if cache.len() > self.config.max_cache_size {
                self.cleanup_cache_internal(&mut cache);
            }
        }
        
        info!("📦 Artifact cached: {} → {}", cache_key, file_path.display());
        Ok(file_path)
    }
    
    /// Verify artifact checksum
    async fn verify_artifact_checksum(&self, file_path: &Path, expected_checksum: &str) -> Result<bool> {
        use sha2::{Sha256, Digest};
        
        let content = fs::read(file_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let checksum = format!("{:x}", hasher.finalize());
        
        Ok(checksum == expected_checksum)
    }
    
    /// Update cache access information
    async fn update_cache_access(&self, cache_key: &str) {
        let mut cache = self.artifact_cache.write().unwrap();
        if let Some(cached) = cache.get_mut(cache_key) {
            cached.access_count += 1;
            cached.last_accessed = chrono::Utc::now();
        }
    }
    
    /// Start cleanup task
    async fn start_cleanup_task(&self) {
        let cache = self.artifact_cache.clone();
        let cleanup_interval = self.config.cleanup_interval;
        let cache_ttl = self.config.cache_ttl;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                let mut cache_guard = cache.write().unwrap();
                let now = chrono::Utc::now();
                
                // Remove expired entries
                cache_guard.retain(|key, cached| {
                    let age = now.signed_duration_since(cached.cached_at);
                    let expired = age > chrono::Duration::from_std(cache_ttl).unwrap();
                    
                    if expired {
                        debug!("🗑️ Removing expired cache entry: {}", key);
                        // Delete file
                        if cached.file_path.exists() {
                            if let Err(e) = std::fs::remove_file(&cached.file_path) {
                                warn!("Failed to delete cached file {}: {}", cached.file_path.display(), e);
                            }
                        }
                    }
                    
                    !expired
                });
                
                debug!("🧹 Cache cleanup completed, {} entries remaining", cache_guard.len());
            }
        });
    }
    
    /// Internal cache cleanup (LRU eviction)
    fn cleanup_cache_internal(&self, cache: &mut HashMap<String, CachedArtifact>) {
        // Sort by last accessed time and remove oldest entries
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, cached)| cached.last_accessed);
        
        let remove_count = cache.len() - self.config.max_cache_size + 10; // Remove extra for buffer
        
        let keys_to_remove: Vec<String> = entries.iter()
            .take(remove_count)
            .map(|(key, _)| key.to_string())
            .collect();

        for key in keys_to_remove {
            if let Some(cached) = cache.remove(&key) {
                debug!("🗑️ Evicting cache entry: {}", key);

                // Delete file
                if cached.file_path.exists() {
                    if let Err(e) = std::fs::remove_file(&cached.file_path) {
                        warn!("Failed to delete cached file {}: {}", cached.file_path.display(), e);
                    }
                }
            }
        }
    }
    
    /// Update load metrics
    async fn update_load_metrics(&self, load_time_ms: u64, success: bool) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.total_loads += 1;
        
        if success {
            metrics.successful_loads += 1;
            
            // Update average load time
            let total = metrics.successful_loads;
            metrics.average_load_time_ms = 
                (metrics.average_load_time_ms * (total - 1) + load_time_ms) / total;
        } else {
            metrics.failed_loads += 1;
        }
    }
    
    /// Update swap metrics
    async fn update_swap_metrics(&self, swap_time_ms: u64, success: bool) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.hot_swaps += 1;
        
        if success {
            metrics.successful_swaps += 1;
            
            // Update average swap time
            let total = metrics.successful_swaps;
            metrics.average_swap_time_ms = 
                (metrics.average_swap_time_ms * (total - 1) + swap_time_ms) / total;
        } else {
            metrics.failed_swaps += 1;
        }
    }
    
    /// Get loading metrics
    pub async fn get_metrics(&self) -> LoadingMetrics {
        self.metrics.read().unwrap().clone()
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.artifact_cache.read().unwrap();
        
        let total_size: u64 = cache.values()
            .filter_map(|cached| cached.file_path.metadata().ok())
            .map(|metadata| metadata.len())
            .sum();
        
        CacheStats {
            total_entries: cache.len(),
            total_size_bytes: total_size,
            oldest_entry: cache.values()
                .map(|cached| cached.cached_at)
                .min(),
            newest_entry: cache.values()
                .map(|cached| cached.cached_at)
                .max(),
        }
    }
    
    /// Clear cache
    pub async fn clear_cache(&self) -> Result<()> {
        info!("🗑️ Clearing artifact cache");
        
        let mut cache = self.artifact_cache.write().unwrap();
        
        // Delete all cached files
        for cached in cache.values() {
            if cached.file_path.exists() {
                if let Err(e) = std::fs::remove_file(&cached.file_path) {
                    warn!("Failed to delete cached file {}: {}", cached.file_path.display(), e);
                }
            }
        }
        
        cache.clear();
        info!("✅ Cache cleared");
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub oldest_entry: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_entry: Option<chrono::DateTime<chrono::Utc>>,
}

/// Create runtime module loader with default configuration
pub async fn create_runtime_loader(
    strategy_loader: Arc<RwLock<StrategyHotLoader>>,
    forge: Option<Arc<RwLock<TheForge>>>,
) -> Result<RuntimeModuleLoader> {
    RuntimeModuleLoader::new(strategy_loader, forge, LoaderConfig::default()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forge::hot_loader::StrategyHotLoader;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_runtime_loader_creation() {
        let strategy_loader = Arc::new(RwLock::new(StrategyHotLoader::new().unwrap()));
        
        let temp_dir = TempDir::new().unwrap();
        let config = LoaderConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..LoaderConfig::default()
        };
        
        let loader = RuntimeModuleLoader::new(strategy_loader, None, config).await.unwrap();
        
        let metrics = loader.get_metrics().await;
        assert_eq!(metrics.total_loads, 0);
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let strategy_loader = Arc::new(RwLock::new(StrategyHotLoader::new().unwrap()));
        
        let temp_dir = TempDir::new().unwrap();
        let config = LoaderConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..LoaderConfig::default()
        };
        
        let loader = RuntimeModuleLoader::new(strategy_loader, None, config).await.unwrap();
        
        let stats = loader.get_cache_stats().await;
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_size_bytes, 0);
    }
}
